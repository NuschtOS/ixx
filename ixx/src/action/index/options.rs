use std::{collections::HashMap, io::Cursor};

use anyhow::Context;
use libixx::{Index, IndexBuilder};
use tokio::{fs::File, io::AsyncWriteExt, task::JoinSet};
use url::Url;

use crate::{
  action::index::{Config, OptionEntry, update_declaration},
  args::IndexModule,
  option::{self, Content},
};

pub(crate) async fn index_options(module: &IndexModule, config: &Config) -> anyhow::Result<()> {
  let mut raw_options: Vec<OptionEntry> = vec![];

  let mut index_builder = IndexBuilder::new(module.chunk_size);

  for scope in &config.scopes {
    let options_json = match &scope.options_json {
      Some(options_jsons) => options_jsons,
      None => {
        continue;
      }
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
      serde_json::from_str(&raw_options).with_context(|| {
        format!(
          "Failed to parse options json: {}",
          options_json.to_string_lossy()
        )
      })?
    };

    let scope_idx = index_builder.push_scope(
      scope
        .name
        .as_ref()
        .map_or_else(|| scope.url_prefix.to_string(), ToString::to_string),
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
        Some(prefix) => format!("{prefix}.{name}"),
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

  println!("Sorting options");
  raw_options.sort_by(|a, b| a.name.cmp(&b.name));

  println!("Building options index");
  for entry in &raw_options {
    index_builder.push(entry.scope, &entry.name);
  }

  println!(
    "Writing options index to {}",
    module.options_index_output.to_string_lossy()
  );

  {
    let index_buf = {
      let mut buf = Vec::new();
      let index: Index = index_builder.into();
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
    let path = module.options_meta_output.join(format!("{idx}.json"));

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
    default: option.default.map(Content::render),
    description: markdown::to_html(&option.description),
    example: option.example.map(Content::render),
    read_only: option.read_only,
    r#type: option.r#type,
    name: name.to_string(),
  })
}
