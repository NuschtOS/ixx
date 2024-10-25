use std::fs::File;

use libixx::Index;

use crate::args::SearchModule;

pub(crate) fn search(module: SearchModule) -> anyhow::Result<()> {
  let mut file = File::open(module.index)?;
  let index = Index::read_from(&mut file)?;

  let result = index.search(module.scope_id, &module.query, module.max_results as usize)?;
  for (idx, scope_id, name) in result {
    println!("idx: {}, scope_id: {}, name: {}", idx, scope_id, name);
  }

  Ok(())
}
