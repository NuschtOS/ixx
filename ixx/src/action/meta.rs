use libixx::Index;
use serde::Serialize;
use std::fs::File;

use crate::args::{Format, MetaModule};

#[derive(Serialize)]
struct Scope {
  pub(crate) id: u8,
  pub(crate) name: String,
}

#[derive(Serialize)]
struct Meta {
  pub(crate) chunk_size: u32,
  pub(crate) scopes: Vec<Scope>,
}

pub(crate) fn meta(module: MetaModule) -> anyhow::Result<()> {
  let mut file = File::open(module.index)?;
  let index = Index::read_from(&mut file)?;

  let raw_meta = index.meta();
  let meta = Meta {
    chunk_size: raw_meta.chunk_size,
    scopes: raw_meta
      .scopes
      .iter()
      .enumerate()
      .map(|(i, scope)| Scope {
        id: i as u8,
        name: scope.to_string(),
      })
      .collect(),
  };

  match module.format {
    Format::Json => {
      let json_output = serde_json::to_string_pretty(&meta)?;
      println!("{}", json_output);
    }
    Format::Text => {
      println!("chunk_size: {}", meta.chunk_size);
      println!("scopes:");
      for scope in meta.scopes {
        println!("  - id: {}, name: {}", scope.id, scope.name);
      }
    }
  }

  Ok(())
}
