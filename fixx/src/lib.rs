use std::string::FromUtf8Error;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Index(libixx::Index);

#[wasm_bindgen]
pub struct SearchedOption {
  idx: usize,
  scope_id: u8,
  name: String,
}

#[wasm_bindgen]
impl Index {
  pub fn read(buf: Vec<u8>) -> Result<Self, String> {
    libixx::Index::read(&buf)
      .map(Self)
      .map_err(|err| format!("{:?}", err))
  }

  pub fn chunk_size(&self) -> u32 {
    self.0.meta().chunk_size
  }

  pub fn scopes(&self) -> Result<Vec<String>, String> {
    self
      .0
      .meta()
      .scopes
      .iter()
      .map(|scope| String::try_from(scope.clone()))
      .collect::<Result<Vec<String>, FromUtf8Error>>()
      .map_err(|err| format!("{:?}", err))
  }

  pub fn search(
    &self,
    scope_id: Option<u8>,
    query: String,
    max_results: usize,
  ) -> Result<Vec<SearchedOption>, String> {
    match self.0.search(scope_id, &query, max_results) {
      Ok(options) => Ok(
        options
          .into_iter()
          .map(|(idx, scope_id, name)| SearchedOption {
            idx,
            scope_id,
            name,
          })
          .collect(),
      ),
      Err(err) => Err(format!("{:?}", err)),
    }
  }

  pub fn get_idx_by_name(&self, scope_id: u8, name: String) -> Result<Option<usize>, String> {
    self
      .0
      .get_idx_by_name(scope_id, &name)
      .map_err(|err| format!("{:?}", err))
  }
}

#[wasm_bindgen]
impl SearchedOption {
  pub fn idx(&self) -> usize {
    self.idx
  }

  pub fn scope_id(&self) -> u8 {
    self.scope_id
  }

  pub fn name(self) -> String {
    self.name
  }
}
