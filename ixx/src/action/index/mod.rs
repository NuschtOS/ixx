use std::path::PathBuf;

use anyhow::{Context, anyhow};
use serde::Deserialize;
use tokio::join;
use url::Url;

use crate::{
  Declaration,
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
  license_mapping: Option<String>,
  maintainer_mapping: Option<String>,
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

fn update_declaration(url_prefix: &Url, declaration: Declaration) -> anyhow::Result<Url> {
  let mut url = match declaration {
    Declaration::StorePath(path) => {
      let mut url_path;
      if path.starts_with("/") {
        let idx = path
        .match_indices('/')
        .nth(3)
        .ok_or_else(|| anyhow!("Invalid store path: {}", path))?
        .0
        // +1 to also remove the / itself, when we join it with a url, the path in the url would
        // get removed if we won't remove it.
        + 1;
        url_path = path.split_at(idx).1.to_owned();
      } else {
        url_path = path
      }

      if let Some((path, line)) = url_path.split_once(':') {
        url_path = format!("{path}#L{line}");
      }

      url_prefix.join(&url_path)?
    }
    Declaration::Url { name: _, url } => url,
  };

  if !url.path().ends_with(".nix") {
    if url.path().ends_with("/") {
      url = url.join("default.nix")?;
    } else {
      url = url.join(&format!(
        "{}/default.nix",
        url
          .path_segments()
          .map(|mut segments| segments.next_back().unwrap_or(""))
          .unwrap_or(""),
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
