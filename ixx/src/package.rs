use serde::Deserialize;
use url::Url;

use crate::Declaration;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Package {
  pub attr_name: String,
  pub broken: Option<bool>,
  pub declaration: Option<Declaration>,
  pub description: Option<String>,
  pub eval_error: Option<bool>,
  pub homepage: Option<OneOrMany<Url>>,
  pub known_vulnerabilities: Option<Vec<String>>,
  pub licenses: Option<Vec<String>>,
  pub maintainers: Option<Vec<String>>,
  pub name: Option<String>,
  pub outputs: Option<Vec<String>>,
  pub pname: Option<String>,
  pub teams: Option<Vec<String>>,
  pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[serde(untagged)]
pub enum OneOrMany<T> {
  One(T),
  Many(Vec<T>),
}
