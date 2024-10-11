use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Option {
  pub declarations: Vec<Declaration>,
  pub description: String,
  pub loc: Vec<String>,
  pub read_only: bool,
  pub r#type: String,
  pub default: std::option::Option<Content>,
  pub example: std::option::Option<Content>,
  pub related_packages: std::option::Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Declaration {
  pub name: String,
  pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "_type")]
pub enum Content {
  LiteralExpression {
    text: String,
  },
  #[serde(rename = "literalMD")]
  Markdown {
    text: String,
  },
}
