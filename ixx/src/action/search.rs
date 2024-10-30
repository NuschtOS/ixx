use std::fs::File;

use libixx::Index;
use serde::{Deserialize, Serialize};

use crate::args::SearchModule;

#[derive(Serialize, Deserialize, Debug)]
struct Entry {
    idx: usize,
    scope_id: u8,
    name: String,
}

pub(crate) fn search(module: SearchModule) -> anyhow::Result<()> {
  let mut file = File::open(module.index)?;
  let index = Index::read_from(&mut file)?;

  let result = index.search(module.scope_id, &module.query, module.max_results as usize)?;

  if module.json {
    let entries: Vec<Entry> = result.into_iter()
        .map(|(idx, scope_id, name)| Entry { idx, scope_id, name })
        .collect();

    let json_output = serde_json::to_string_pretty(&entries)?;

    println!("{}", json_output);
  } else {
    for (idx, scope_id, name) in result {
      println!("idx: {}, scope_id: {}, name: {}", idx, scope_id, name);
    }
  }

  Ok(())
}
