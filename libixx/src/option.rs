use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Option {
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub declarations: Vec<Url>,
  #[serde(skip_serializing_if = "std::option::Option::is_none")]
  pub default: std::option::Option<String>,
  pub description: String,
  #[serde(skip_serializing_if = "std::option::Option::is_none")]
  pub example: std::option::Option<String>,
  pub read_only: bool,
  pub r#type: String,
  pub name: String,
}
