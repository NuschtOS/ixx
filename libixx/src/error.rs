use std::io;
use std::str::Utf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IxxError {
  #[error("invalid label reference")]
  InvalidLabelReference,
  #[error("recursive reference")]
  RecursiveReference,

  #[error("(de)serialization failed")]
  Binrw(#[from] binrw::Error),
  #[error("invalid utf8")]
  FromUtf8Error(#[from] Utf8Error),
  #[error("io error")]
  IoError(#[from] io::Error),
}
