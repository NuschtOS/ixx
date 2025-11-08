use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Package {
  pub attr_name: String,
  pub broken: Option<bool>,
  pub declaration: Option<Url>,
  pub description: Option<String>,
  pub eval_error: Option<bool>,
  pub homepages: Vec<Url>,
  pub known_vulnerabilities: Vec<String>,
  pub licenses: Vec<String>,
  pub maintainers: Vec<String>,
  pub name: Option<String>,
  pub outputs: Vec<String>,
  pub pname: Option<String>,
  pub teams: Vec<String>,
  pub version: Option<String>,
}
