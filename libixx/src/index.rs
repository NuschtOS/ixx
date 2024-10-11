use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use crate::IxxError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Index(Vec<Vec<Label>>);

#[derive(Serialize, Deserialize, Debug)]
struct Reference {
  option_idx: u16,
  label_idx: u8,
}

#[derive(Serialize, Deserialize, Debug)]
enum Label {
  InPlace(String),
  Reference(Reference),
}
impl Index {
  pub fn new() -> Self {
    Self(Vec::new())
  }

  pub fn read(buf: &[u8]) -> Result<Self, IxxError> {
    Ok(bincode::deserialize(buf)?)
  }

  pub fn read_from<R: Read>(read: &mut R) -> Result<Self, IxxError> {
    Ok(bincode::deserialize_from(read)?)
  }

  pub fn write(&self) -> Result<Vec<u8>, IxxError> {
    Ok(bincode::serialize(self)?)
  }

  pub fn write_into<W: Write>(&self, write: &mut W) -> Result<(), IxxError> {
    Ok(bincode::serialize_into(write, self)?)
  }

  pub fn push(&mut self, option: &str) {
    let labels = option
      .split('.')
      .map(|segment| {
        for (option_idx, option) in self.0.iter().enumerate() {
          for (label_idx, label) in option.iter().enumerate() {
            if let Label::InPlace(inplace) = label {
              if inplace != segment {
                continue;
              }

              return Label::Reference(Reference {
                option_idx: option_idx as u16,
                label_idx: label_idx as u8,
              });
            }
          }
        }

        Label::InPlace(segment.to_string())
      })
      .collect();

    self.0.push(labels);
  }

  fn resolve_reference(&self, reference: &Reference) -> Result<&str, IxxError> {
    let option_idx = reference.option_idx as usize;

    if self.0.len() <= option_idx {
      return Err(IxxError::ReferenceOutOfBounds);
    }

    let entry = &self.0[option_idx];

    let label_idx = reference.label_idx as usize;

    if entry.len() <= label_idx {
      return Err(IxxError::ReferenceOutOfBounds);
    }

    let label = &entry[label_idx];

    match label {
      Label::InPlace(ref string) => Ok(string),
      Label::Reference(_) => Err(IxxError::RecursiveReference),
    }
  }

  pub fn search(&self, query: &str, max_results: usize) -> Result<Vec<String>, IxxError> {
    let search = query
      .split('*')
      .map(|segment| segment.to_lowercase())
      .collect::<Vec<_>>();

    let mut results = Vec::new();

    for option in &self.0 {
      let mut option_name = String::new();
      for label in option {
        match label {
          Label::InPlace(data) => option_name.push_str(data),
          Label::Reference(reference) => option_name.push_str(self.resolve_reference(reference)?),
        }
        option_name.push('.')
      }
      // remove last dot...
      option_name.pop();

      let mut start = 0;

      'outer: {
        for segment in &search {
          match option_name[start..].find(segment) {
            Some(idx) => start = idx + segment.len(),
            None => break 'outer,
          }
        }

        results.push(option_name);
        if results.len() >= max_results {
          return Ok(results);
        }
      }
    }

    Ok(results)
  }
}
