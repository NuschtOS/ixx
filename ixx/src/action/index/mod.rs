use std::path::PathBuf;

use anyhow::Context;
use serde::Deserialize;
use tokio::join;
use url::Url;

use crate::{
  action::index::{options::index_options, packages::index_packages},
  args::IndexModule,
};

mod options;
mod packages;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Config {
  scopes: Vec<Scope>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Scope {
  name: Option<String>,
  options_json: Option<PathBuf>,
  packages_jsons: Option<Vec<PathBuf>>,
  url_prefix: Url,
  options_prefix: Option<String>,
}

struct OptionEntry {
  name: String,
  scope: u8,
  option: libixx::Option,
}

struct PackageEntry {
  name: String,
  scope: u8,
  option: libixx::Package,
}

pub(crate) async fn index(module: IndexModule) -> anyhow::Result<()> {
  let config: Config = {
    let raw_config = tokio::fs::read_to_string(&module.config)
      .await
      .with_context(|| {
        format!(
          "Failed to read config file: {}",
          module.config.to_string_lossy()
        )
      })?;
    serde_json::from_str(&raw_config)?
  };

  let (options_result, packages_result) = join!(
    index_options(&module, &config),
    index_packages(&module, &config),
  );

  options_result?;
  packages_result?;

  Ok(())
}
