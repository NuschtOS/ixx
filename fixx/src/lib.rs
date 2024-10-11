use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hash(option: String) -> u8 {
  libixx::hash(&option)
}

#[wasm_bindgen]
pub struct Index(libixx::Index);

#[wasm_bindgen]
impl Index {
  pub fn read(buf: Vec<u8>) -> Result<Self, String> {
    libixx::Index::read(&buf)
      .map(Self)
      .map_err(|err| format!("{:?}", err))
  }

  pub fn search(&self, query: String, max_results: usize) -> Result<Vec<String>, String> {
    self
      .0
      .search(&query, max_results)
      .map_err(|err| format!("{:?}", err))
  }
}
