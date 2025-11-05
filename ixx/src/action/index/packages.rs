use std::io::Cursor;

use anyhow::Context;
use libixx::Index;
use tokio::{fs::File, io::AsyncWriteExt, task::JoinSet};

use crate::{
  action::index::{Config, PackageEntry},
  args::IndexModule,
  package::{self, OneOrMany},
};

pub(crate) async fn index_packages(module: &IndexModule, config: &Config) -> anyhow::Result<()> {
  let mut raw_packages: Vec<PackageEntry> = vec![];

  let mut index = Index::new(module.chunk_size);

  for scope in &config.scopes {
    let packages_jsons = match &scope.packages_jsons {
      Some(packages_jsons) => packages_jsons,
      None => { continue; }
    };

    let scope_idx = index.push_scope(
      scope
        .name
        .as_ref()
        .map(|x| x.to_string())
        .unwrap_or_else(|| scope.url_prefix.to_string()),
    );

    let mut join_set = JoinSet::new();

    for packages_json in packages_jsons {
      let packages_json = packages_json.clone();
      join_set.spawn(async move {
        println!("Parsing {}", packages_json.to_string_lossy());
        let packages: Vec<package::Package> = {
          let raw_packages = tokio::fs::read_to_string(&packages_json)
            .await
            .with_context(|| {
              format!(
                "Failed to read options json: {}",
                packages_json.to_string_lossy()
              )
            })?;
          serde_json::from_str(&raw_packages)?
        };

        let packages = packages
          .into_iter()
          .map(|package| {
            Ok::<_, anyhow::Error>(PackageEntry {
              name: package.attr_name.clone(),
              scope: scope_idx,
              option: into_package(package)?,
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

  raw_packages.sort_by(|a, b| a.name.cmp(&b.name));

  println!("Sorted packages");

  for entry in &raw_packages {
    index.push(entry.scope, &entry.name);
  }

  println!(
    "Writing packages index to {}",
    module.packages_index_output.to_string_lossy()
  );

  {
    let index_buf = {
      let mut buf = Vec::new();
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
    "write packages meta to {}",
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

fn into_package(package: package::Package) -> anyhow::Result<libixx::Package> {
  Ok(libixx::Package {
    attr_name: package.attr_name,
    eval_error: package.eval_error,
    broken: package.broken,
    description: package.description,
    homepages: match package.homepage {
      None => vec![],
      Some(OneOrMany::One(homepage)) => vec![homepage],
      Some(OneOrMany::Many(homepages)) => homepages,
    },
    outputs: package.outputs.unwrap_or_default(),
    insecure: package.insecure,
    name: package.name,
    pname: package.pname,
    unfree: package.unfree,
    version: package.version,
  })
}
