use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Option {
  pub declarations: Vec<String>,
  pub default: std::option::Option<String>,
  pub description: String,
  pub example: std::option::Option<String>,
  pub read_only: bool,
  pub r#type: String,
  pub name: String,
}
