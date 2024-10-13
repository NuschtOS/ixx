use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use crate::IxxError;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Index(Vec<Vec<Label>>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Reference {
  option_idx: u16,
  label_idx: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum Label {
  InPlace(String),
  Reference(Reference),
}

impl Index {
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

  pub fn get_idx_by_name(&self, option: &str) -> Result<Option<usize>, IxxError> {
    let mut labels = Vec::new();
    for segment in option.split('.') {
      'outer: {
        for (option_idx, option) in self.0.iter().enumerate() {
          for (label_idx, label) in option.iter().enumerate() {
            if let Label::InPlace(inplace) = label {
              if inplace != segment {
                continue;
              }

              labels.push(Reference {
                option_idx: option_idx as u16,
                label_idx: label_idx as u8,
              });
              break 'outer;
            }
          }
        }

        return Ok(None);
      }
    }

    Ok(
      self
        .0
        .iter()
        .enumerate()
        .find(|(idx, option)| do_labels_match(*idx, option, &labels))
        .map(|(idx, _)| idx),
    )
  }

  pub fn search(&self, query: &str, max_results: usize) -> Result<Vec<(usize, String)>, IxxError> {
    let search = query
      .split('*')
      .map(|segment| segment.to_lowercase())
      .collect::<Vec<_>>();

    if search.is_empty() {
      return Ok(vec![]);
    }

    let mut results = Vec::new();

    for (idx, option) in self.0.iter().enumerate() {
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

      let lower_option_name = option_name.to_lowercase();

      let mut start = 0;

      'outer: {
        for segment in &search {
          match lower_option_name[start..].find(segment) {
            Some(idx) => start = idx + segment.len(),
            None => break 'outer,
          }
        }

        results.push((idx, option_name));
        if results.len() >= max_results {
          return Ok(results);
        }
      }
    }

    Ok(results)
  }

  pub fn all(&self, max: usize) -> Result<Vec<String>, IxxError> {
    let mut options = Vec::new();

    for option in &self.0[..max] {
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

      options.push(option_name);
    }

    Ok(options)
  }
}

fn do_labels_match(option_idx: usize, option: &[Label], search: &[Reference]) -> bool {
  let matching = option
    .iter()
    .enumerate()
    .zip(search.iter())
    .filter(|&((label_idx, option), search)| match option {
      Label::InPlace(_) => {
        option_idx == search.option_idx as usize && label_idx == search.label_idx as usize
      }
      Label::Reference(reference) => reference == search,
    })
    .count();

  matching == option.len() && matching == search.len()
}
