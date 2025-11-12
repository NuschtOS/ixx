use std::{
  io::Cursor,
  sync::{Arc, LazyLock},
};

use anyhow::Context;
use libixx::{Index, IndexBuilder};
use regex::{Captures, Regex};
use tokio::{fs::File, io::AsyncWriteExt, task::JoinSet};
use url::Url;

use crate::{
  action::index::{Config, PackageEntry, update_declaration},
  args::IndexModule,
  package::{self, OneOrMany},
};

pub(crate) async fn index_packages(module: &IndexModule, config: &Config) -> anyhow::Result<()> {
  let mut raw_packages: Vec<PackageEntry> = vec![];

  let mut index_builder = IndexBuilder::new(module.chunk_size);

  for scope in &config.scopes {
    let packages_jsons = match &scope.packages_jsons {
      Some(packages_jsons) => packages_jsons,
      None => {
        continue;
      }
    };

    let scope_idx = index_builder.push_scope(
      scope
        .name
        .as_ref()
        .map(|x| x.to_string())
        .unwrap_or_else(|| scope.url_prefix.to_string()),
    );

    let mut join_set = JoinSet::new();

    let url_prefix = Arc::new(scope.url_prefix.clone());

    for packages_json in packages_jsons {
      let packages_json = packages_json.clone();
      let url_prefix = url_prefix.clone();
      join_set.spawn(async move {
        println!("Parsing {}", packages_json.to_string_lossy());
        let packages: Vec<package::Package> = {
          let raw_packages = tokio::fs::read_to_string(&packages_json)
            .await
            .with_context(|| {
              format!(
                "Failed to read packages json: {}",
                packages_json.to_string_lossy()
              )
            })?;
          serde_json::from_str(&raw_packages).with_context(|| {
            format!(
              "Failed to parse packages json: {}",
              packages_json.to_string_lossy()
            )
          })?
        };

        let packages = packages
          .into_iter()
          .map(|package| {
            Ok::<_, anyhow::Error>(PackageEntry {
              name: package.attr_name.clone(),
              scope: scope_idx,
              option: into_package(&url_prefix, package)?,
            })
          })
          .collect::<Result<Vec<_>, _>>()?;

        Ok::<_, anyhow::Error>(packages)
      });

      while let Some(result) = join_set.join_next().await {
        raw_packages.extend(result??);
      }
    }
  }

  println!("Read {} packages", raw_packages.len());
  if raw_packages.is_empty() {
    return Ok(());
  }

  println!("Sorting packages");
  raw_packages.sort_by(|a, b| a.name.cmp(&b.name));

  for entry in &raw_packages {
    index_builder.push(entry.scope, &entry.name);
  }

  println!(
    "Writing packages index to {}",
    module.packages_index_output.to_string_lossy()
  );

  {
    let index_buf = {
      let mut buf = Vec::new();
      let index: Index = index_builder.into();
      index.write_into(&mut Cursor::new(&mut buf))?;
      buf
    };

    let mut index_output = File::create(&module.packages_index_output)
      .await
      .with_context(|| {
        format!(
          "Failed to create {}",
          module.packages_index_output.to_string_lossy()
        )
      })?;

    index_output.write_all(index_buf.as_slice()).await?;
  }

  println!(
    "Writing packages meta to {}",
    module.packages_meta_output.to_string_lossy()
  );

  if !module.packages_meta_output.exists() {
    std::fs::create_dir(&module.packages_meta_output).with_context(|| {
      format!(
        "Failed to create dir {}",
        module.packages_meta_output.to_string_lossy()
      )
    })?;
  }

  let packages = raw_packages
    .into_iter()
    .map(|entry| entry.option)
    .collect::<Vec<_>>();

  let mut join_set = JoinSet::new();

  for (idx, chunk) in packages.chunks(module.chunk_size as usize).enumerate() {
    let path = module.packages_meta_output.join(format!("{}.json", idx));

    let meta_string = serde_json::to_string(chunk)
      .with_context(|| format!("Failed to write to {}", path.to_string_lossy()))?;

    join_set.spawn(async move {
      let mut file = File::create(&path)
        .await
        .with_context(|| format!("Failed to create {}", path.to_string_lossy()))?;

      file.write_all(meta_string.as_bytes()).await?;

      Ok::<_, anyhow::Error>(())
    });
  }

  while let Some(result) = join_set.join_next().await {
    result??;
  }

  Ok(())
}

fn into_package(url_prefix: &Url, package: package::Package) -> anyhow::Result<libixx::Package> {
  static CVE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"CVE-(\d{4})-(\d+)").unwrap());
  static GHSA_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"GHSA((?:-[23456789cfghjmpqrvwx]{4}){3})").unwrap());

  Ok(libixx::Package {
    attr_name: package.attr_name,
    broken: package.broken,
    cpe: package.cpe,
    disabled: package.disabled,
    possible_cpes: package.possible_cpes.unwrap_or_default(),
    declaration: package.declaration.map(|declaration | update_declaration(url_prefix, declaration)).transpose()?,
    description: package.description.map(|description| markdown::to_html(&description)),
    eval_error: package.eval_error,
    homepages: match package.homepage {
      None => vec![],
      Some(OneOrMany::One(homepage)) => vec![homepage],
      Some(OneOrMany::Many(homepages)) => homepages,
    },
    known_vulnerabilities: package
      .known_vulnerabilities
      .unwrap_or_default()
      .into_iter()
      .map(|vulnerability| {
        let vulnerability = markdown::to_html(&vulnerability);
      let vulnerability =         CVE_REGEX
          .replace_all(&vulnerability, |caps: &Captures| {
            format!(
              "<a href=\"https://www.cve.org/CVERecord?id=CVE-{0}-{1}\" target=\"_blank\">CVE-{0}-{1}</a>",
              &caps[1], &caps[2]
            )
          });

      GHSA_REGEX.replace_all(&vulnerability, |caps: &Captures|{
          format!(
              "<a href=\"https://github.com/advisories/GHSA{0}\" target=\"_blank\">GHSA{0}</a>",
              &caps[1]
            )

      }).to_string()
      })
      .collect(),
    licenses: package.licenses.unwrap_or_default(),
    maintainers: package.maintainers.unwrap_or_default(),
    name: package.name,
    outputs: package.outputs.unwrap_or_default(),
    pname: package.pname,
    teams: package.teams.unwrap_or_default(),
    version: package.version,
  })
}
