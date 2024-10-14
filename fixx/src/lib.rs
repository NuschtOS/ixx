use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Index(libixx::Index);

#[wasm_bindgen]
pub struct SearchedOption {
  idx: usize,
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

  pub fn search(&self, query: String, max_results: usize) -> Result<Vec<SearchedOption>, String> {
    match self.0.search(&query, max_results) {
      Ok(options) => Ok(
        options
          .into_iter()
          .map(|(idx, name)| SearchedOption { idx, name })
          .collect(),
      ),
      Err(err) => Err(format!("{:?}", err)),
    }
  }

  pub fn all(&self, max: usize) -> Result<Vec<String>, String> {
    self.0.all(max).map_err(|err| format!("{:?}", err))
  }

  pub fn get_idx_by_name(&self, name: String) -> Result<Option<usize>, String> {
    self
      .0
      .get_idx_by_name(&name)
      .map_err(|err| format!("{:?}", err))
  }
}

#[wasm_bindgen]
impl SearchedOption {
  pub fn idx(&self) -> usize {
    self.idx
  }

  pub fn name(self) -> String {
    self.name
  }
}
