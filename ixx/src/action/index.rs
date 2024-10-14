use std::{
  collections::{BTreeMap, HashMap},
  fs::File,
  path::{Path, PathBuf},
};

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
  options_json: PathBuf,
  url_prefix: Option<Url>,
  options_prefix: Option<String>,
}

pub(crate) fn index(module: IndexModule) -> anyhow::Result<()> {
  let mut raw_options: BTreeMap<String, libixx::Option> = BTreeMap::new();

  let config_file = File::open(module.config)?;
  let config: Config = serde_json::from_reader(config_file)?;

  for scope in config.scopes {
    println!("Parsing {}", scope.options_json.to_string_lossy());
    let file = File::open(scope.options_json)?;
    let options: HashMap<String, option::Option> = serde_json::from_reader(file)?;

    for (name, option) in options {
      // internal options which cannot be hidden when importing existing options.json
      if name == "_module.args" {
        continue;
      }

      let name = match &scope.options_prefix {
        Some(prefix) => format!("{}.{}", prefix, name),
        None => name,
      };
      let option = into_option(&name, option);
      raw_options.insert(name, option);
    }
  }

  println!("Read {} options", raw_options.len());

  let mut index = Index::default();
  raw_options.keys().for_each(|name| index.push(name));

  println!("Writing index to {}", module.output.to_string_lossy());

  let mut output = File::create(module.output)?;
  index.write_into(&mut output)?;

  println!("Writing meta");

  if !Path::new("meta").exists() {
    std::fs::create_dir("meta")?;
  }

  let options: Vec<libixx::Option> = raw_options.into_values().collect();
  for (idx, chunk) in options.chunks(module.chunk_size).enumerate() {
    let mut file = File::create(format!("meta/{}.json", idx))?;
    serde_json::to_writer(&mut file, &chunk)?;
  }

  Ok(())
}

fn into_option(name: &str, option: option::Option) -> libixx::Option {
  libixx::Option {
    declarations: option
      .declarations
      .into_iter()
      .map(update_declaration)
      .collect(),
    default: option.default.map(|option| option.render()),
    description: option.description,
    example: option.example.map(|example| example.render()),
    read_only: option.read_only,
    r#type: option.r#type,
    name: name.to_string(),
  }
}

fn update_declaration(declaration: Declaration) -> String {
  // NOTE: Is the url actually optional? If its true, this can be ignored.
  //       Otherwise the fallback with building the url outself is required.
  //
  // if "url" in declaration:
  //   return declaration["url"]
  // if declaration.startswith("/nix/store/"):
  //   # strip prefix: /nix/store/0a0mxyfmad6kaknkkr0ysraifws856i7-source
  //   return f"{url}{declaration[51:]}"
  // return declaration

  declaration.url
}
