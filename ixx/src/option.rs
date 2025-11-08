use serde::{Deserialize, Serialize};

use crate::{Declaration, utils::highlight};

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

impl Content {
  pub(crate) fn render(self) -> String {
    match self {
      Self::LiteralExpression { text } => highlight(text.trim()),
      Self::Markdown { text } => markdown::to_html(text.trim()),
    }
  }
}
