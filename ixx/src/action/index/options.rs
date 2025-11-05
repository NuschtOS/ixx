use std::{collections::HashMap, io::Cursor};

use anyhow::{Context, anyhow};
use libixx::Index;
use tokio::{fs::File, io::AsyncWriteExt, task::JoinSet};
use url::Url;

use crate::{
  action::index::{Config, OptionEntry},
  args::IndexModule,
  option::{self, Declaration},
};

pub(crate) async fn index_options(module: &IndexModule, config: &Config) -> anyhow::Result<()> {
  let mut raw_options: Vec<OptionEntry> = vec![];

  let mut index = Index::new(module.chunk_size);

  for scope in &config.scopes {
    let options_json = match &scope.options_json {
      Some(packages_jsons) => packages_jsons,
      None => { continue; }
    };

    println!("Parsing {}", options_json.to_string_lossy());
    let options: HashMap<String, option::Option> = {
      let raw_options = tokio::fs::read_to_string(&options_json)
        .await
        .with_context(|| {
          format!(
            "Failed to read options json: {}",
            options_json.to_string_lossy()
          )
        })?;
      serde_json::from_str(&raw_options)?
    };

    let scope_idx = index.push_scope(
      scope
        .name
        .as_ref()
        .map(|x| x.to_string())
        .unwrap_or_else(|| scope.url_prefix.to_string()),
    );

    for (name, option) in options {
      // internal options which cannot be hidden when importing existing options.json
      if name == "_module.args" {
        continue;
      }

      // skip modular services until upstream doc rendering is fixed
      // https://github.com/NixOS/nixpkgs/issues/432550
      if name.starts_with("<imports = [ pkgs.") {
        continue;
      }

      let name = match &scope.options_prefix {
        Some(prefix) => format!("{}.{}", prefix, name),
        None => name,
      };

      let option = into_option(&scope.url_prefix, &name, option)?;

      raw_options.push(OptionEntry {
        name,
        scope: scope_idx,
        option,
      });
    }
  }

  println!("Read {} options", raw_options.len());
  if raw_options.is_empty() {
    return Ok(());
  }

  raw_options.sort_by(|a, b| a.name.cmp(&b.name));

  println!("Sorted options");

  println!("Building options index...");
  for entry in &raw_options {
    index.push(entry.scope, &entry.name);
  }

  println!(
    "Writing options index to {}",
    module.options_index_output.to_string_lossy()
  );

  {
    let index_buf = {
      let mut buf = Vec::new();
      index.write_into(&mut Cursor::new(&mut buf))?;
      buf
    };

    let mut index_output = File::create(&module.options_index_output)
      .await
      .with_context(|| {
        format!(
          "Failed to create {}",
          module.options_index_output.to_string_lossy()
        )
      })?;

    index_output.write_all(index_buf.as_slice()).await?;
  }

  println!(
    "Writing options meta to {}",
    module.options_meta_output.to_string_lossy()
  );

  if !module.options_meta_output.exists() {
    std::fs::create_dir(&module.options_meta_output).with_context(|| {
      format!(
        "Failed to create dir {}",
        module.options_meta_output.to_string_lossy()
      )
    })?;
  }

  let options = raw_options
    .into_iter()
    .map(|entry| entry.option)
    .collect::<Vec<_>>();

  let mut join_set = JoinSet::new();

  for (idx, chunk) in options.chunks(module.chunk_size as usize).enumerate() {
    let path = module.options_meta_output.join(format!("{}.json", idx));

    let meta_string = serde_json::to_string(chunk)
      .with_context(|| format!("Failed to write to {}", path.to_string_lossy()))?;

    join_set.spawn(async move {
      let mut file = File::create(&path)
        .await
        .with_context(|| format!("Failed to create {}", path.to_string_lossy()))?;

      file.write_all(meta_string.as_bytes()).await?;

      Ok::<_, anyhow::Error>(())
    });
  }

  while let Some(result) = join_set.join_next().await {
    result??;
  }

  Ok(())
}

fn into_option(
  url_prefix: &Url,
  name: &str,
  option: option::Option,
) -> anyhow::Result<libixx::Option> {
  Ok(libixx::Option {
    declarations: option
      .declarations
      .into_iter()
      .map(|declaration| update_declaration(url_prefix, declaration))
      .collect::<anyhow::Result<_>>()?,
    default: option.default.map(|option| option.render()),
    description: markdown::to_html(&option.description),
    example: option.example.map(|example| example.render()),
    read_only: option.read_only,
    r#type: option.r#type,
    name: name.to_string(),
  })
}

fn update_declaration(url_prefix: &Url, declaration: Declaration) -> anyhow::Result<Url> {
  let mut url = match declaration {
    Declaration::StorePath(path) => {
      if path.starts_with("/") {
        let idx = path
        .match_indices('/')
        .nth(3)
        .ok_or_else(|| anyhow!("Invalid store path: {}", path))?
        .0
        // +1 to also remove the / itself, when we join it with a url, the path in the url would
        // get removed if we won't remove it.
        + 1;
        url_prefix.join(path.split_at(idx).1)?
      } else {
        url_prefix.join(&path)?
      }
    }
    Declaration::Url { name: _, url } => url,
  };

  if !url.path().ends_with(".nix") {
    if url.path().ends_with("/") {
      url = url.join("default.nix")?;
    } else {
      url = url.join(&format!(
        "{}/default.nix",
        url
          .path_segments()
          .map(|mut segments| segments.next_back().unwrap_or(""))
          .unwrap_or(""),
      ))?;
    }
  }

  Ok(url)
}

#[cfg(test)]
mod test {
  use url::Url;

  use crate::{action::index::options::update_declaration, option::Declaration};

  #[test]
  fn test_update_declaration() {
    assert_eq!(
      update_declaration(
        &Url::parse("https://example.com/some/path").unwrap(),
        Declaration::StorePath(
          "/nix/store/vgvk6q3zsjgb66f8s5cm8djz6nmcag1i-source/modules/initrd.nix".to_string()
        )
      )
      .unwrap(),
      Url::parse("https://example.com/some/modules/initrd.nix").unwrap()
    );

    assert_eq!(
      update_declaration(
        &Url::parse("https://example.com/some/path/").unwrap(),
        Declaration::StorePath(
          "/nix/store/vgvk6q3zsjgb66f8s5cm8djz6nmcag1i-source/modules/initrd.nix".to_string()
        )
      )
      .unwrap(),
      Url::parse("https://example.com/some/path/modules/initrd.nix").unwrap()
    );

    assert_eq!(
      update_declaration(
        &Url::parse("https://example.com/some/path/").unwrap(),
        Declaration::StorePath(
          "/nix/store/vgvk6q3zsjgb66f8s5cm8djz6nmcag1i-source-idk/modules/initrd.nix".to_string()
        )
      )
      .unwrap(),
      Url::parse("https://example.com/some/path/modules/initrd.nix").unwrap()
    );

    // Suffix default.nix if url is referencing folder
    assert_eq!(
      update_declaration(
        &Url::parse("https://example.com/some/path").unwrap(),
        Declaration::Url {
          name: "idk".to_string(),
          url: Url::parse("https://example.com/some/path").unwrap(),
        }
      )
      .unwrap(),
      Url::parse("https://example.com/some/path/default.nix").unwrap()
    );

    assert_eq!(
      update_declaration(
        &Url::parse("https://example.com/some/path").unwrap(),
        Declaration::Url {
          name: "idk".to_string(),
          url: Url::parse("https://example.com/some/path/").unwrap(),
        }
      )
      .unwrap(),
      Url::parse("https://example.com/some/path/default.nix").unwrap()
    );

    // nixpkgs edge case
    assert_eq!(
      update_declaration(
        &Url::parse("https://example.com/some/path/").unwrap(),
        Declaration::StorePath("nixos/hello/world.nix".to_string()),
      )
      .unwrap(),
      Url::parse("https://example.com/some/path/nixos/hello/world.nix").unwrap()
    );
  }
}
