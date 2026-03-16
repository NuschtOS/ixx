use args::{Action, Args};
use clap::Parser;
use serde::Deserialize;
use url::Url;

mod action;
mod args;
mod option;
mod package;
pub(crate) mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();

  match args.action {
    Action::Index(module) => action::index::index(module).await,
    Action::Search(module) => action::search::search(module),
  }?;

  Ok(())
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Declaration {
  /// Example Value: `/nix/store/vgvk6q3zsjgb66f8s5cm8djz6nmcag1i-source/modules/initrd.nix`
  StorePath(String),
  Url {
    name: String,
    url: Url,
  },
}
