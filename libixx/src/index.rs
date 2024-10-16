use std::io::{Cursor, Read, Seek, Write};

use binrw::{binrw, BinRead, BinWrite, Endian, NullString};

use crate::IxxError;

#[binrw]
#[brw(magic = b"ixx01")]
#[derive(Debug, Clone, PartialEq)]
pub struct Index {
  meta: Meta,
  #[bw(calc = options.len() as u32)]
  count: u32,
  #[br(count = count)]
  options: Vec<OptionEntry>,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
  pub chunk_size: u32,
  #[bw(calc = scopes.len() as u8)]
  scope_count: u8,
  #[br(count = scope_count)]
  pub scopes: Vec<NullString>,
}

#[binrw]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct OptionEntry {
  /// index in the scopes Vec
  scope_id: u8,
  #[bw(calc = labels.len() as u16)]
  count: u16,
  #[br(count = count)]
  labels: Vec<Label>,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
struct Reference {
  option_idx: u16,
  label_idx: u8,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
enum Label {
  #[brw(magic = b"0")]
  InPlace(NullString),
  #[brw(magic = b"1")]
  Reference(Reference),
}

impl Index {
  pub fn new(chunk_size: u32) -> Self {
    Self {
      meta: Meta {
        chunk_size,
        scopes: vec![],
      },
      options: vec![],
    }
  }

  pub fn read(buf: &[u8]) -> Result<Self, IxxError> {
    Self::read_from(&mut Cursor::new(buf))
  }

  pub fn read_from<R: Read + Seek>(read: &mut R) -> Result<Self, IxxError> {
    Ok(BinRead::read_options(read, Endian::Little, ())?)
  }

  pub fn write_into<W: Write + Seek>(&self, write: &mut W) -> Result<(), IxxError> {
    Ok(BinWrite::write_options(self, write, Endian::Little, ())?)
  }

  pub fn push(&mut self, scope_id: u8, option: &str) {
    let labels = option
      .split('.')
      .map(|segment| {
        let segment = segment.into();

        for (option_idx, OptionEntry { labels: option, .. }) in self.options.iter().enumerate() {
          for (label_idx, label) in option.iter().enumerate() {
            if let Label::InPlace(inplace) = label {
              if inplace != &segment {
                continue;
              }

              return Label::Reference(Reference {
                option_idx: option_idx as u16,
                label_idx: label_idx as u8,
              });
            }
          }
        }

        Label::InPlace(segment)
      })
      .collect();

    self.options.push(OptionEntry { scope_id, labels });
  }

  fn resolve_reference(&self, reference: &Reference) -> Result<&NullString, IxxError> {
    let option_idx = reference.option_idx as usize;

    if self.options.len() <= option_idx {
      return Err(IxxError::ReferenceOutOfBounds);
    }

    let entry = &self.options[option_idx].labels;

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
      let segment = segment.into();

      'outer: {
        for (option_idx, OptionEntry { labels: option, .. }) in self.options.iter().enumerate() {
          for (label_idx, label) in option.iter().enumerate() {
            if let Label::InPlace(inplace) = label {
              if inplace != &segment {
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
        .options
        .iter()
        .enumerate()
        .find(|(idx, OptionEntry { labels: option, .. })| do_labels_match(*idx, option, &labels))
        .map(|(idx, _)| idx),
    )
  }

  pub fn search(
    &self,
    scope_id: Option<u8>,
    query: &str,
    max_results: usize,
  ) -> Result<Vec<(usize, String)>, IxxError> {
    let search = query
      .split('*')
      .map(|segment| segment.to_lowercase())
      .collect::<Vec<_>>();

    let mut results = Vec::new();

    for (
      idx,
      OptionEntry {
        scope_id: option_scope_id,
        labels: option,
      },
    ) in self.options.iter().enumerate()
    {
      if let Some(scope_id) = scope_id {
        if *option_scope_id != scope_id {
          continue;
        }
      }

      let mut option_name = String::new();
      for label in option {
        option_name.push_str(&String::try_from(
          match label {
            Label::InPlace(data) => data,
            Label::Reference(reference) => self.resolve_reference(reference)?,
          }
          .clone(),
        )?);
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

  pub fn meta(&self) -> &Meta {
    &self.meta
  }

  pub fn push_scope(&mut self, scope: String) -> u8 {
    if self.meta.scopes.len() == u8::MAX.into() {
      panic!("You reached the limit of 256 scopes. Please contact the developers for further assistance.");
    }

    let idx = self.meta.scopes.len();
    self.meta.scopes.push(scope.into());
    idx as u8
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
