use args::{Action, Args};
use clap::Parser;

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
    Action::Meta(module) => action::meta::meta(module),
  }?;

  Ok(())
}
