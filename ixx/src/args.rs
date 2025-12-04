use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
pub(super) struct Args {
  #[clap(subcommand)]
  pub(super) action: Action,
}

#[derive(Subcommand)]
pub(super) enum Action {
  #[clap(about = "Build the index")]
  Index(IndexModule),
  #[clap(about = "Search the index for packages or options")]
  Search(SearchModule),
  #[clap(about = "Show index metadata")]
  Meta(MetaModule),
}

#[derive(ValueEnum, Clone)]
pub(super) enum Format {
  Text,
  Json,
}

#[derive(Parser)]
pub(super) struct IndexModule {
  pub(super) config: PathBuf,

  #[clap(long, default_value = "options/index.ixx")]
  pub(super) options_index_output: PathBuf,

  #[clap(long, default_value = "options/meta")]
  pub(crate) options_meta_output: PathBuf,

  #[clap(long, default_value = "packages/index.ixx")]
  pub(super) packages_index_output: PathBuf,

  #[clap(long, default_value = "packages/meta")]
  pub(crate) packages_meta_output: PathBuf,

  #[clap(long, default_value = "meta.json")]
  pub(crate) meta_output: PathBuf,

  #[clap(long, default_value = "100")]
  pub(super) chunk_size: u32,
}

#[derive(Parser)]
pub(super) struct SearchModule {
  pub(super) query: String,

  #[clap(short, long, default_value = "index.ixx")]
  pub(super) index: PathBuf,

  #[clap(short, long)]
  pub(super) scope_id: Option<u8>,

  #[clap(short, long, default_value = "10")]
  pub(super) max_results: u32,

  #[clap(short, long, default_value = "text")]
  pub(super) format: Format,
}

#[derive(Parser)]
pub(super) struct MetaModule {
  #[clap(short, long, default_value = "index.ixx")]
  pub(super) index: PathBuf,

  #[clap(short, long, default_value = "text")]
  pub(super) format: Format,
}
