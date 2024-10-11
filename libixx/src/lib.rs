pub use error::IxxError;
pub use index::Index;

mod error;
mod index;
pub mod option;

#[cfg(test)]
mod test;
