use serde::Deserialize;

use crate::Declaration;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Package {
  pub attr_name: String,
  pub broken: Option<bool>,
  pub changelog: Option<String>,
  pub cpe: Option<String>,
  pub declaration: Option<Declaration>,
  pub description: Option<String>,
  pub disabled: Option<bool>,
  pub download_page: Option<String>,
  pub eval_error: Option<bool>,
  pub homepage: Option<OneOrMany<String>>,
  pub known_vulnerabilities: Option<Vec<String>>,
  pub licenses: Option<Vec<String>>,
  pub long_description: Option<String>,
  pub maintainers: Option<Vec<u32>>,
  pub name: Option<String>,
  pub outputs: Option<Vec<String>>,
  pub pname: Option<String>,
  pub possible_cpes: Option<Vec<String>>,
  pub purl: Option<String>,
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
