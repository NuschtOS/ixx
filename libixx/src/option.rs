use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Option {
  pub declarations: Vec<Url>,
  pub default: std::option::Option<String>,
  pub description: String,
  pub example: std::option::Option<String>,
  pub read_only: bool,
  pub r#type: String,
  pub name: String,
}
