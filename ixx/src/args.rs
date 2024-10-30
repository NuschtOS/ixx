use clap::{Parser, Subcommand, ValueEnum};
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

  #[clap(short, long, default_value = "index.ixx")]
  pub(super) index_output: PathBuf,

  #[clap(short, long, default_value = "meta")]
  pub(crate) meta_output: PathBuf,

  #[clap(short, long, default_value = "100")]
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
