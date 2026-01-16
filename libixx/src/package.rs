use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Package {
  pub attr_name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub broken: Option<bool>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub changelogs: Vec<Url>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cpe: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub disabled: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub declaration: Option<Url>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub download_page: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub eval_error: Option<bool>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub homepages: Vec<Url>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub known_vulnerabilities: Vec<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub licenses: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub long_description: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub maintainers: Vec<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub outputs: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pname: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub possible_cpes: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub purl: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub source_provenance: Vec<SourceProvenance>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub teams: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<String>,
}

// https://github.com/NixOS/nixpkgs/blob/master/doc/stdenv/meta.chapter.md#source-provenance-sec-meta-sourceprovenance
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[serde(untagged)]
pub enum SourceProvenance {
  FromSource,
  BinaryNativeCode,
  BinaryFirmware,
  BinaryBytecode,
}
