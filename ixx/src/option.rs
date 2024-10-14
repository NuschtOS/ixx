use serde::{Deserialize, Serialize};
use url::Url;

use crate::utils::highlight;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Declaration {
  /// Example Value: `/nix/store/vgvk6q3zsjgb66f8s5cm8djz6nmcag1i-source/modules/initrd.nix`
  StorePath(String),
  Url {
    name: String,
    url: Url,
  },
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

impl Content {
  pub(crate) fn render(self) -> String {
    match self {
      Self::LiteralExpression { text } => highlight(&text),
      Self::Markdown { text } => markdown::to_html(&text),
    }
  }
}
