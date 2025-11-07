use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Package {
  pub attr_name: String,
  pub broken: Option<bool>,
  pub declaration: Option<String>,
  pub description: Option<String>,
  pub eval_error: Option<bool>,
  pub homepages: Vec<Url>,
  pub insecure: Option<bool>,
  pub licenses: Vec<String>,
  pub maintainers: Vec<String>,
  pub name: Option<String>,
  pub outputs: Vec<String>,
  pub pname: Option<String>,
  pub teams: Option<String>,
  pub unfree: Option<bool>,
  pub version: Option<String>,
}
