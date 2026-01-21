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
      .map_err(|err| format!("{err:?}"))
  }

  pub fn search(
    &self,
    scope_id: Option<u8>,
    #[wasm_bindgen(unchecked_param_type = "string")] query: &JsValue,
    max_results: usize,
  ) -> Result<Vec<SearchedOption>, String> {
    let query_str = query
      .as_string()
      .ok_or_else(|| "Invalid query: expected a string".to_string())?;
    match self.0.search(scope_id, &query_str, max_results) {
      Ok(options) => Ok(
        options
          .into_iter()
          .map(|(idx, scope_id, name)| SearchedOption { idx, scope_id, name })
          .collect(),
      ),
      Err(err) => Err(format!("{err:?}")),
    }
  }

  pub fn get_idx_by_name(
    &self,
    scope_id: u8,
    #[wasm_bindgen(unchecked_param_type = "string")] name: &JsValue,
  ) -> Result<Option<usize>, String> {
    let name_str = name
      .as_string()
      .ok_or_else(|| "Invalid name: expected a string".to_string())?;
    self
      .0
      .get_idx_by_name(scope_id, &name_str)
      .map_err(|err| format!("{err:?}"))
  }

  #[must_use]
  pub fn size(&self) -> usize {
    self.0.size()
  }
}

#[wasm_bindgen]
impl SearchedOption {
  #[must_use]
  pub fn idx(&self) -> usize {
    self.idx
  }

  #[must_use]
  pub fn scope_id(&self) -> u8 {
    self.scope_id
  }

  #[must_use]
  pub fn name(self) -> String {
    self.name
  }
}
