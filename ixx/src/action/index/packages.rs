use std::{
  collections::HashMap,
  io::Cursor,
  sync::{Arc, LazyLock},
};

use anyhow::Context;
use libixx::{Index, IndexBuilder};
use regex::{Captures, Regex};
use tokio::{fs::File, io::AsyncWriteExt, task::JoinSet};
use url::Url;

use crate::{
  action::index::{Config, License, Meta, PackageEntry, ScopeMeta, update_declaration},
  args::IndexModule,
  package::{self, OneOrMany},
};

pub(crate) async fn index_packages(
  module: &IndexModule,
  meta: &Meta,
  config: &Config,
) -> anyhow::Result<HashMap<u8, HashMap<String, License>>> {
  let mut raw_packages = Vec::<PackageEntry>::new();
  let mut all_extra_licenses = HashMap::<u8, HashMap<String, License>>::new();
  let mut index_builder = IndexBuilder::default();

  for (scope_idx, scope) in config.scopes.iter().enumerate() {
    let packages_jsons = match &scope.packages_jsons {
      Some(packages_jsons) => packages_jsons,
      None => {
        continue;
      }
    };

    let mut join_set = JoinSet::new();

    let url_prefix = Arc::new(scope.url_prefix.clone());

    for packages_json in packages_jsons {
      let scope_meta = meta.scopes[&(scope_idx as u8)].clone();
      let packages_json = packages_json.clone();
      let url_prefix = url_prefix.clone();

      join_set.spawn(async move {
        println!("Parsing {}", packages_json.to_string_lossy());
        let packages: Vec<package::Package> = {
          let raw_packages = tokio::fs::read_to_string(&packages_json).await.with_context(|| {
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

        let mut extra_licenses = HashMap::<String, License>::new();
        let packages = packages
          .into_iter()
          .map(|package| {
            let (pkg, extras) = into_package(&url_prefix, &scope_meta, package)?;
            extra_licenses.extend(extras);
            Ok::<_, anyhow::Error>(PackageEntry {
              name: pkg.attr_name.clone(),
              scope: scope_idx as u8,
              package: pkg,
            })
          })
          .collect::<Result<Vec<_>, _>>()?;

        Ok::<_, anyhow::Error>((packages, extra_licenses))
      });

      while let Some(result) = join_set.join_next().await {
        let (pkgs, extras) = result??;
        raw_packages.extend(pkgs);
        if let Some(val) = all_extra_licenses.get_mut(&(scope_idx as u8)) {
          val.extend(extras);
        };
      }
    }
  }

  println!("Read {} packages", raw_packages.len());
  if raw_packages.is_empty() {
    return Ok(all_extra_licenses);
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
    "Writing packages chunks to {}",
    module.packages_chunks_output.to_string_lossy()
  );

  if !module.packages_chunks_output.exists() {
    std::fs::create_dir(&module.packages_chunks_output).with_context(|| {
      format!(
        "Failed to create dir {}",
        module.packages_chunks_output.to_string_lossy()
      )
    })?;
  }

  let packages = raw_packages
    .into_iter()
    .map(|entry| entry.package)
    .collect::<Vec<_>>();

  let mut join_set = JoinSet::new();

  for (idx, chunk) in packages.chunks(module.chunk_size as usize).enumerate() {
    let path = module.packages_chunks_output.join(format!("{idx}.json"));

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

  Ok(all_extra_licenses)
}

fn one_or_many_to_url(something: Option<OneOrMany<String>>) -> Vec<Url> {
  match something {
    None => vec![],
    Some(OneOrMany::One(homepage)) => Url::parse(&homepage)
      .with_context(|| format!("Failed to parse URL '{homepage}'"))
      .ok()
      .into_iter()
      .collect(),
    Some(OneOrMany::Many(homepages)) => homepages
      .into_iter()
      .filter_map(|homepage| {
        Url::parse(&homepage)
          .with_context(|| format!("Failed to parse URL '{homepage}'"))
          .ok()
      })
      .collect(),
  }
}
static CVE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"CVE-(\d{4})-(\d+)").unwrap());
static GHSA_REGEX: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"GHSA((?:-[23456789cfghjmpqrvwx]{4}){3})").unwrap());

fn into_package(
  url_prefix: &Url,
  scope_meta: &ScopeMeta,
  package: package::Package,
) -> anyhow::Result<(libixx::Package, HashMap<String, License>)> {
  Ok((
    libixx::Package {
      attr_name: package.attr_name,
      broken: package.broken,
      changelogs: one_or_many_to_url(package.changelog),
      cpe: package.cpe,
      declaration: package
        .declaration
        .map(|declaration| update_declaration(url_prefix, declaration))
        .transpose()?,
      description: package
        .description
        .map(|description| markdown::to_html(&description)),
      disabled: package.disabled,
      download_page: package.download_page,
      eval_error: package.eval_error,
      homepages: one_or_many_to_url(package.homepage),
      known_vulnerabilities: package
        .known_vulnerabilities
        .unwrap_or_default()
        .into_iter()
        .map(|vulnerability| {
          let vulnerability = markdown::to_html(&vulnerability);
          let vulnerability = CVE_REGEX.replace_all(&vulnerability, |caps: &Captures| {
            format!(
              "<a href=\"https://www.cve.org/CVERecord?id=CVE-{0}-{1}\" target=\"_blank\">CVE-{0}-{1}</a>",
              &caps[1], &caps[2]
            )
          });

          GHSA_REGEX
            .replace_all(&vulnerability, |caps: &Captures| {
              format!(
                "<a href=\"https://github.com/advisories/GHSA{0}\" target=\"_blank\">GHSA{0}</a>",
                &caps[1]
              )
            })
            .to_string()
        })
        .collect(),
      licenses: package.licenses.clone().unwrap_or_default(),
      long_description: package
        .long_description
        .map(|description| markdown::to_html(&description)),
      maintainers: package.maintainers.unwrap_or_default(),
      name: package.name,
      outputs: package.outputs.unwrap_or_default(),
      pname: package.pname,
      possible_cpes: package.possible_cpes.unwrap_or_default(),
      purl: package.purl,
      source_provenance: package.source_provenance.unwrap_or_default(),
      teams: package.teams.unwrap_or_default(),
      version: package.version,
    },
    package
      .licenses
      .unwrap_or_default()
      .into_iter()
      .filter_map(|license| {
        if scope_meta
          .licenses
          .contains_key(&license.short_name.clone().unwrap_or_default())
        {
          None
        } else {
          Some((
            license
              .short_name
              .clone()
              .unwrap_or(license.full_name.clone().unwrap_or_default()),
            License {
              free: license.free,
              full_name: license.full_name,
              redistributable: license.redistributable,
              short_name: license.short_name,
              url: license.url,
            },
          ))
        }
      })
      .collect::<HashMap<String, License>>(),
  ))
}
