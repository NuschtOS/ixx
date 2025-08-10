use std::{collections::HashMap, fs::File, path::PathBuf};

use anyhow::anyhow;
use libixx::Index;
use serde::Deserialize;
use url::Url;

use crate::{
  args::IndexModule,
  option::{self, Declaration},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Config {
  scopes: Vec<Scope>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Scope {
  name: Option<String>,
  options_json: PathBuf,
  url_prefix: Url,
  options_prefix: Option<String>,
}

struct Entry {
  name: String,
  scope: u8,
  option: libixx::Option,
}

pub(crate) fn index(module: IndexModule) -> anyhow::Result<()> {
  let mut raw_options: Vec<Entry> = vec![];

  let config_file = File::open(module.config)?;
  let config: Config = serde_json::from_reader(config_file)?;

  let mut index = Index::new(module.chunk_size);

  for scope in config.scopes {
    println!("Parsing {}", scope.options_json.to_string_lossy());
    let file = File::open(scope.options_json)?;
    let options: HashMap<String, option::Option> = serde_json::from_reader(file)?;

    let scope_idx = index.push_scope(
      scope
        .name
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
      raw_options.push(Entry {
        name,
        scope: scope_idx,
        option,
      });
    }
  }

  println!("Read {} options", raw_options.len());

  raw_options.sort_by(|a, b| a.name.cmp(&b.name));

  println!("Sorted options");

  for entry in &raw_options {
    index.push(entry.scope, &entry.name);
  }

  println!("Writing index to {}", module.index_output.to_string_lossy());

  let mut output = File::create(module.index_output)?;
  index.write_into(&mut output)?;

  println!("Writing meta to {}", module.meta_output.to_string_lossy());

  if !module.meta_output.exists() {
    std::fs::create_dir(&module.meta_output)?;
  }

  let options = raw_options
    .into_iter()
    .map(|entry| entry.option)
    .collect::<Vec<_>>();

  for (idx, chunk) in options.chunks(module.chunk_size as usize).enumerate() {
    let mut file = File::create(module.meta_output.join(format!("{}.json", idx)))?;
    serde_json::to_writer(&mut file, chunk)?;
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
          .map(|segments| segments.last().unwrap_or(""))
          .unwrap_or(""),
      ))?;
    }
  }

  Ok(url)
}

#[cfg(test)]
mod test {
  use url::Url;

  use crate::{action::index::update_declaration, option::Declaration};

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
