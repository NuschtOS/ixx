use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Package {
  pub attr_name: String,
  pub eval_error: Option<bool>,
  pub broken: Option<bool>,
  pub declaration: Option<String>,
  pub description: Option<String>,
  pub homepages: Vec<Url>,
  pub licenses: Vec<String>,
  pub maintainers: Vec<String>,
  pub outputs: Vec<String>,
  pub insecure: Option<bool>,
  pub name: Option<String>,
  pub pname: Option<String>,
  pub unfree: Option<bool>,
  pub version: Option<String>,
}
