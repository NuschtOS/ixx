use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncWriteExt, join};
use url::Url;

use crate::{
  Declaration,
  action::index::{options::index_options, packages::index_packages},
  args::IndexModule,
};

mod options;
mod packages;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct Config {
  scopes: Vec<Scope>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct Scope {
  name: Option<String>,
  license_mapping: HashMap<String, License>,
  maintainer_mapping: HashMap<u32, Maintainer>,
  team_mapping: HashMap<u32, Team>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct License {
  free: bool,
  full_name: String,
  redistributable: bool,
  spdx_id: Option<String>,
  url: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Maintainer {
  email: Option<String>,
  matrix: Option<String>,
  github: String,
  name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Team {
  members: Vec<String>,
  scope: String,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Meta {
  scopes: HashMap<u8, ScopeMeta>,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ScopeMeta {
  licenses: HashMap<String, License>,
  maintainers: HashMap<u32, Maintainer>,
  teams: HashMap<u32, Team>,
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
    serde_json::from_str(&raw_config).with_context(|| {
      format!(
        "Failed to parse config file: {}",
        module.config.to_string_lossy()
      )
    })?
  };

  let (options_result, packages_result, meta_result) = join!(
    index_options(&module, &config),
    index_packages(&module, &config),
    async {
      let meta = Meta {
        scopes: config
          .scopes
          .iter()
          .enumerate()
          .map(|(idx, scope)| {
            (
              idx as u8,
              ScopeMeta {
                licenses: scope.license_mapping.clone(),
                maintainers: scope.maintainer_mapping.clone(),
                teams: scope.team_mapping.clone(),
              },
            )
          })
          .collect(),
      };

      let raw_meta = serde_json::to_string(&meta)?;
      let mut meta_file = File::create(&module.meta_output).await?;

      meta_file.write_all(raw_meta.as_bytes()).await?;

      Ok::<_, anyhow::Error>(())
    }
  );

  options_result?;
  packages_result?;
  meta_result?;

  Ok(())
}

fn update_declaration(url_prefix: &Url, declaration: Declaration) -> anyhow::Result<Url> {
  let mut url = match declaration {
    Declaration::StorePath(path) => {
      let mut url_path;
      if path.starts_with('/') {
        let idx = path
        .match_indices('/')
        .nth(3)
        .ok_or_else(|| anyhow::anyhow!("Invalid store path: {path}"))?
        .0
        // +1 to also remove the / itself, when we join it with a url, the path in the url would
        // get removed if we won't remove it.
        + 1;
        url_path = path.split_at(idx).1.to_owned();
      } else {
        url_path = path;
      }

      if let Some((path, line)) = url_path.split_once(':') {
        url_path = format!("{path}#L{line}");
      }

      url_prefix.join(&url_path)?
    }
    Declaration::Url { name: _, url } => url,
  };

  if !url.path().ends_with(".nix") {
    if url.path().ends_with('/') {
      url = url.join("default.nix")?;
    } else {
      url = url.join(&format!(
        "{}/default.nix",
        url
          .path_segments()
          .map_or("", |mut segments| segments.next_back().unwrap_or("")),
      ))?;
    }
  }

  Ok(url)
}

#[cfg(test)]
mod test {
  use url::Url;

  use crate::{Declaration, action::index::update_declaration};

  #[test]
  fn test_update_declaration() {
    assert_eq!(
      update_declaration(
        &Url::parse("https://example.com/some/path").unwrap(),
        Declaration::StorePath(
          "/nix/store/vgvk6q3zsjgb66f8s5cm8djz6nmcag1i-source/modules/initrd.nix".to_string()
        )
      )
      .unwrap(),
      Url::parse("https://example.com/some/modules/initrd.nix").unwrap()
    );

    assert_eq!(
      update_declaration(
        &Url::parse("https://example.com/some/path/").unwrap(),
        Declaration::StorePath(
          "/nix/store/vgvk6q3zsjgb66f8s5cm8djz6nmcag1i-source/modules/initrd.nix".to_string()
        )
      )
      .unwrap(),
      Url::parse("https://example.com/some/path/modules/initrd.nix").unwrap()
    );

    assert_eq!(
      update_declaration(
        &Url::parse("https://example.com/some/path/").unwrap(),
        Declaration::StorePath(
          "/nix/store/vgvk6q3zsjgb66f8s5cm8djz6nmcag1i-source-idk/modules/initrd.nix".to_string()
        )
      )
      .unwrap(),
      Url::parse("https://example.com/some/path/modules/initrd.nix").unwrap()
    );

    // package position
    assert_eq!(
      update_declaration(
        &Url::parse("https://example.com/some/path/").unwrap(),
        Declaration::StorePath(
          "/nix/store/pb93n2bk2zpyn1sqpkm3gyhra26zy4ps-source/pkgs/by-name/he/hello/package.nix:47"
            .to_string()
        )
      )
      .unwrap(),
      Url::parse("https://example.com/some/path/pkgs/by-name/he/hello/package.nix#L47").unwrap()
    );

    // Suffix default.nix if url is referencing folder
    assert_eq!(
      update_declaration(
        &Url::parse("https://example.com/some/path").unwrap(),
        Declaration::Url {
          name: "idk".to_string(),
          url: Url::parse("https://example.com/some/path").unwrap(),
        }
      )
      .unwrap(),
      Url::parse("https://example.com/some/path/default.nix").unwrap()
    );

    assert_eq!(
      update_declaration(
        &Url::parse("https://example.com/some/path").unwrap(),
        Declaration::Url {
          name: "idk".to_string(),
          url: Url::parse("https://example.com/some/path/").unwrap(),
        }
      )
      .unwrap(),
      Url::parse("https://example.com/some/path/default.nix").unwrap()
    );

    // nixpkgs edge case
    assert_eq!(
      update_declaration(
        &Url::parse("https://example.com/some/path/").unwrap(),
        Declaration::StorePath("nixos/hello/world.nix".to_string()),
      )
      .unwrap(),
      Url::parse("https://example.com/some/path/nixos/hello/world.nix").unwrap()
    );
  }
}
