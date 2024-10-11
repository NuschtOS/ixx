use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
pub(super) struct Args {
  #[clap(subcommand)]
  pub(super) action: Action,
}

#[derive(Subcommand)]
pub(super) enum Action {
  Index(IndexModule),
  Search(SearchModule),
}

#[derive(Parser)]
pub(super) struct IndexModule {
  #[clap(required = true)]
  pub(super) files: Vec<PathBuf>,

  #[clap(short, long, default_value = "index.ixx")]
  pub(super) output: PathBuf,
}

#[derive(Parser)]
pub(super) struct SearchModule {
  pub(super) query: String,

  #[clap(short, long, default_value = "index.ixx")]
  pub(super) index: PathBuf,

  #[clap(short, long, default_value = "10")]
  pub(super) max_results: u32,
}
