use args::{Action, Args};
use clap::Parser;

mod action;
mod args;
mod option;
pub(crate) mod utils;

fn main() -> anyhow::Result<()> {
  let args = Args::parse();

  match args.action {
    Action::Index(module) => action::index::index(module),
    Action::Search(module) => action::search::search(module),
  }?;

  Ok(())
}
