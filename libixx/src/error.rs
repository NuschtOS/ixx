use thiserror::Error;

#[derive(Error, Debug)]
pub enum IxxError {
  #[error("referenced out of bounds data")]
  ReferenceOutOfBounds,
  #[error("recursive reference")]
  RecursiveReference,

  #[error("(de)serialization failed")]
  Bincode(#[from] bincode::Error),
}
