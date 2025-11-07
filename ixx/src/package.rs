use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Package {
  pub attr_name: String,
  pub eval_error: Option<bool>,
  pub broken: Option<bool>,
  pub declaration: Option<String>,
  pub description: Option<String>,
  pub homepage: Option<OneOrMany<Url>>,
  pub outputs: Option<Vec<String>>,
  pub insecure: Option<bool>,
  pub name: Option<String>,
  pub pname: Option<String>,
  pub unfree: Option<bool>,
  pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum OneOrMany<T> {
  One(T),
  Many(Vec<T>),
}
