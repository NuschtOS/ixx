use std::{
  collections::HashMap,
  io::{Cursor, Read, Seek, Write},
  string::FromUtf8Error,
};

use binrw::{BinRead, BinWrite, Endian, VecArgs, binrw};

use levenshtein::levenshtein;

use crate::{IxxError, string_view::StringView};

pub struct IndexBuilder {
  index: Index,
  label_cache: HashMap<Vec<u8>, (usize, u8)>,
}

#[binrw]
#[brw(magic = b"ixx02")]
#[derive(Debug, Clone, PartialEq)]
pub struct Index {
  pub(crate) meta: Meta,
  #[bw(calc = entries.len() as u32)]
  count: u32,
  #[br(count = count)]
  pub(crate) entries: Vec<Entry>,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
  pub chunk_size: u32,
  #[bw(calc = scopes.len() as u8)]
  scope_count: u8,
  #[br(count = scope_count)]
  pub scopes: Vec<PascalString>,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
pub struct PascalString {
  #[bw(calc = data.len() as u8)]
  len: u8,
  #[br(count = len)]
  pub(crate) data: Vec<u8>,
}

#[binrw]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Entry {
  /// index in the scopes Vec
  pub(crate) scope_id: u8,
  #[bw(calc = labels.len() as u8)]
  count: u8,
  #[br(count = count)]
  pub(crate) labels: Vec<Label>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Label {
  InPlace(Vec<u8>),
  Reference(Reference),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
  entry_idx: u64,
  label_idx: u8,
}

impl BinRead for Label {
  type Args<'a> = ();

  fn read_options<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    _args: Self::Args<'_>,
  ) -> binrw::BinResult<Self> {
    let first = u8::read_options(reader, endian, ())?;

    if first & (1 << 7) == 0 {
      let buf = Vec::<u8>::read_options(
        reader,
        endian,
        VecArgs {
          count: first as usize,
          inner: (),
        },
      )?;

      return Ok(Self::InPlace(buf));
    }

    let label_idx = first & (u8::MAX >> 3);

    match (first & 0b0110_0000) >> 5 {
      0 => Ok(Self::Reference(Reference {
        entry_idx: u64::from(u8::read_options(reader, endian, ())?),
        label_idx,
      })),
      1 => Ok(Self::Reference(Reference {
        entry_idx: u64::from(u16::read_options(reader, endian, ())?),
        label_idx,
      })),
      2 => Ok(Self::Reference(Reference {
        entry_idx: u64::from(u32::read_options(reader, endian, ())?),
        label_idx,
      })),
      3 => Ok(Self::Reference(Reference {
        entry_idx: u64::read_options(reader, endian, ())?,
        label_idx,
      })),
      _ => unreachable!(),
    }
  }
}

impl BinWrite for Label {
  type Args<'a> = ();

  fn write_options<W: Write + Seek>(
    &self,
    writer: &mut W,
    endian: Endian,
    _args: Self::Args<'_>,
  ) -> binrw::BinResult<()> {
    match self {
      Label::InPlace(buf) => {
        assert!(
          buf.len() <= (u8::MAX >> 1) as usize,
          "Label is too large: {} bytes (maximum is {} bytes)",
          buf.len(),
          (u8::MAX >> 1)
        );

        (buf.len() as u8).write_options(writer, endian, ())?;
        buf.write_options(writer, endian, ())?;
      }
      Label::Reference(Reference { entry_idx, label_idx }) => {
        assert!(
          *label_idx <= (u8::MAX >> 3),
          "Label index too big, contact developer!"
        );

        if *entry_idx < u64::from(u8::MAX) {
          ((1u8 << 7) | label_idx).write_options(writer, endian, ())?;
          (*entry_idx as u8).write_options(writer, endian, ())?;
        } else if *entry_idx < u64::from(u16::MAX) {
          ((1u8 << 7) | (1 << 5) | label_idx).write_options(writer, endian, ())?;
          (*entry_idx as u16).write_options(writer, endian, ())?;
        } else if *entry_idx < u64::from(u32::MAX) {
          ((1u8 << 7) | (2 << 5) | label_idx).write_options(writer, endian, ())?;
          (*entry_idx as u32).write_options(writer, endian, ())?;
        } else {
          ((1u8 << 7) | (3 << 5) | label_idx).write_options(writer, endian, ())?;
          entry_idx.write_options(writer, endian, ())?;
        }
      }
    }

    Ok(())
  }
}

impl From<String> for PascalString {
  fn from(value: String) -> Self {
    Self {
      data: value.into_bytes(),
    }
  }
}

impl TryFrom<PascalString> for String {
  type Error = FromUtf8Error;

  fn try_from(value: PascalString) -> Result<Self, Self::Error> {
    String::from_utf8(value.data)
  }
}

impl IndexBuilder {
  #[must_use]
  pub fn new(chunk_size: u32) -> Self {
    Self {
      index: Index {
        meta: Meta {
          chunk_size,
          scopes: vec![],
        },
        entries: vec![],
      },
      label_cache: HashMap::new(),
    }
  }

  pub fn push(&mut self, scope_id: u8, name: &str) {
    // optimize, if there is no dot in the name, compression does not make sense
    let labels = if name.contains('.') {
      name
        .split('.')
        .enumerate()
        .map(|(label_idx, segment)| {
          let segment = segment.as_bytes();

          if let Some((entry_idx, label_idx)) = self.label_cache.get(segment) {
            return Label::Reference(Reference {
              entry_idx: *entry_idx as u64,
              label_idx: *label_idx,
            });
          }

          self
            .label_cache
            .insert(segment.to_vec(), (self.index.entries.len(), label_idx as u8));

          Label::InPlace(segment.to_vec())
        })
        .collect()
    } else {
      self
        .label_cache
        .insert(name.as_bytes().to_vec(), (self.index.entries.len(), 0));
      vec![Label::InPlace(name.into())]
    };

    self.index.entries.push(Entry { scope_id, labels });
  }

  pub fn push_scope(&mut self, scope: String) -> u8 {
    assert!(
      self.index.meta.scopes.len() != u8::MAX as usize,
      "You reached the limit of 256 scopes. Please contact the developers for further assistance."
    );

    let idx = self.index.meta.scopes.len();
    self.index.meta.scopes.push(scope.into());
    idx as u8
  }
}

impl Index {
  pub fn read(buf: &[u8]) -> Result<Self, IxxError> {
    Self::read_from(&mut Cursor::new(buf))
  }

  pub fn read_from<R: Read + Seek>(read: &mut R) -> Result<Self, IxxError> {
    Ok(BinRead::read_options(read, Endian::Little, ())?)
  }

  pub fn write_into<W: Write + Seek>(&self, write: &mut W) -> Result<(), IxxError> {
    Ok(BinWrite::write_options(self, write, Endian::Little, ())?)
  }

  pub fn resolve_reference(&self, reference: &Reference) -> Result<&[u8], IxxError> {
    let entry_idx = reference.entry_idx as usize;

    if self.entries.len() <= entry_idx {
      return Err(IxxError::ReferenceOutOfBounds);
    }

    let entry = &self.entries[entry_idx].labels;

    let label_idx = reference.label_idx as usize;

    if entry.len() <= label_idx {
      return Err(IxxError::ReferenceOutOfBounds);
    }

    match &entry[label_idx] {
      Label::InPlace(string) => Ok(string),
      Label::Reference(_) => Err(IxxError::RecursiveReference),
    }
  }

  pub fn get_idx_by_name(&self, scope_id: u8, name: &str) -> Result<Option<usize>, IxxError> {
    let mut labels = Vec::new();

    for segment in name.split('.') {
      let segment = segment.as_bytes();

      'outer: {
        for (entry_idx, entry) in self.entries.iter().enumerate() {
          for (label_idx, label) in entry.labels.iter().enumerate() {
            let Label::InPlace(inplace) = label else {
              continue;
            };

            if inplace != segment {
              continue;
            }

            labels.push(Reference {
              entry_idx: entry_idx as u64,
              label_idx: label_idx as u8,
            });
            break 'outer;
          }
        }

        return Ok(None);
      }
    }

    Ok(
      self
        .entries
        .iter()
        .enumerate()
        .find(|(entry_idx, entry)| {
          entry.scope_id == scope_id && do_labels_match(*entry_idx, &entry.labels, &labels)
        })
        .map(|(entry_idx, _)| entry_idx),
    )
  }

  pub fn search(
    &self,
    scope_id: Option<u8>,
    query: &str,
    max_results: usize,
  ) -> Result<Vec<(usize, u8, String)>, IxxError> {
    let search = query
      .split('*')
      .map(str::as_bytes)
      // * at the start or end of a string
      .filter(|x| !x.is_empty())
      .map(|segment| segment.split(|char| *char == b'.').collect())
      .collect::<Vec<Vec<_>>>();

    let mut results = Vec::new();

    for (
      idx,
      Entry {
        scope_id: entry_scope_id,
        labels,
      },
    ) in self.entries.iter().enumerate()
    {
      if let Some(scope_id) = scope_id
        && *entry_scope_id != scope_id
      {
        continue;
      }

      let entry_name = StringView::from((self, labels.as_slice()));

      if entry_name.matches(&search)? {
        let entry_name = entry_name.to_string();
        let levenshtein = levenshtein(query, &entry_name);

        results.push((idx, *entry_scope_id, entry_name, levenshtein));
        if results.len() == max_results {
          break;
        }
      }
    }

    results.sort_by_key(|(_, _, _, levenshtein)| *levenshtein);

    let results = results
      .into_iter()
      .map(|(idx, entry_scope_id, entry_name, _)| (idx, entry_scope_id, entry_name))
      .collect();

    Ok(results)
  }

  #[must_use]
  pub fn meta(&self) -> &Meta {
    &self.meta
  }

  #[must_use]
  pub fn size(&self) -> usize {
    self.entries.len()
  }
}

impl From<IndexBuilder> for Index {
  fn from(value: IndexBuilder) -> Self {
    value.index
  }
}

fn do_labels_match(entry_idx: usize, labels: &[Label], search: &[Reference]) -> bool {
  if labels.len() != search.len() {
    return false;
  }

  labels
    .iter()
    .enumerate()
    .zip(search.iter())
    .all(|((label_idx, entry), search)| match entry {
      Label::InPlace(_) => entry_idx == search.entry_idx as usize && label_idx == search.label_idx as usize,
      Label::Reference(reference) => reference == search,
    })
}

#[cfg(test)]
mod tests {
  use crate::index::*;

  #[test]
  fn test_push_one_entry() {
    let mut builder = IndexBuilder::new(1);
    builder.push(0, "foo.bar");
    let index: Index = builder.into();

    assert_eq!(index.entries.len(), 1);
    assert_eq!(index.entries[0].labels.len(), 2);
    match &index.entries[0].labels[0] {
      Label::InPlace(label) => assert_eq!(label, b"foo"),
      _ => panic!("Expected InPlace label for 'foo'"),
    }
    match &index.entries[0].labels[1] {
      Label::InPlace(label) => assert_eq!(label, b"bar"),
      _ => panic!("Expected InPlace label for 'bar'"),
    }
  }

  #[test]
  fn test_push_two_entries_with_compression() {
    let mut builder = IndexBuilder::new(1);
    builder.push(0, "foo.bar");
    builder.push(0, "foo.buz");
    let index: Index = builder.into();

    assert_eq!(index.entries.len(), 2);

    assert_eq!(index.entries[0].labels.len(), 2);
    match &index.entries[0].labels[0] {
      Label::InPlace(label) => assert_eq!(label, b"foo"),
      _ => unreachable!("Expected no Reference label"),
    }
    match &index.entries[0].labels[1] {
      Label::InPlace(label) => assert_eq!(label, b"bar"),
      _ => unreachable!("Expected no Reference label"),
    }

    assert_eq!(index.entries[1].labels.len(), 2);
    match &index.entries[1].labels[0] {
      Label::Reference(reference) => {
        assert_eq!(reference.entry_idx, 0);
        assert_eq!(reference.label_idx, 0);
      }
      _ => unreachable!("Expected no InPlace label"),
    }
    match &index.entries[1].labels[1] {
      Label::InPlace(label) => assert_eq!(label, b"buz"),
      Label::Reference(reference) => {
        assert_eq!(reference.entry_idx, 0);
        assert_eq!(reference.label_idx, 1);
      }
    }
  }

  #[test]
  fn test_push_compression_inplace_different_position() {
    let mut builder = IndexBuilder::new(1);
    builder.push(0, "pretalx");
    builder.push(0, "nixosTests.pretalx");
    let index: Index = builder.into();

    assert_eq!(index.entries.len(), 2);

    assert_eq!(index.entries[0].labels.len(), 1);
    match &index.entries[0].labels[0] {
      Label::InPlace(label) => assert_eq!(label, b"pretalx"),
      _ => unreachable!("Expected no Reference label"),
    }

    assert_eq!(index.entries[1].labels.len(), 2);
    match &index.entries[1].labels[0] {
      Label::Reference(reference) => {
        assert_eq!(reference.entry_idx, 0);
        assert_eq!(reference.label_idx, 0);
      }
      Label::InPlace(label) => assert_eq!(label, b"nixosTests"),
    }
    match &index.entries[1].labels[1] {
      Label::Reference(reference) => {
        assert_eq!(reference.entry_idx, 0);
      }
      _ => unreachable!("Expected no InPlace label"),
    }
  }

  #[test]
  fn test_labels_match_inplace() {
    let labels = vec![Label::InPlace(b"foo".to_vec()), Label::InPlace(b"bar".to_vec())];
    let search = vec![
      Reference {
        entry_idx: 0,
        label_idx: 0,
      },
      Reference {
        entry_idx: 0,
        label_idx: 1,
      },
    ];
    assert!(do_labels_match(0, &labels, &search));
  }

  #[test]
  fn test_labels_match_reference() {
    let reference = Reference {
      entry_idx: 1,
      label_idx: 2,
    };
    let labels = vec![Label::Reference(reference.clone())];
    let search = vec![reference.clone()];
    assert!(do_labels_match(0, &labels, &search));
  }

  #[test]
  fn test_labels_mismatch_inplace() {
    let labels = vec![Label::InPlace(b"foo".to_vec())];
    let search = vec![Reference {
      entry_idx: 1,
      label_idx: 0,
    }];
    assert!(!do_labels_match(0, &labels, &search));
  }

  #[test]
  fn test_labels_mismatch_reference() {
    let labels = vec![Label::Reference(Reference {
      entry_idx: 1,
      label_idx: 1,
    })];
    let search = vec![Reference {
      entry_idx: 2,
      label_idx: 1,
    }];
    assert!(!do_labels_match(0, &labels, &search));

    let labels = vec![Label::Reference(Reference {
      entry_idx: 1,
      label_idx: 1,
    })];
    let search = vec![Reference {
      entry_idx: 1,
      label_idx: 2,
    }];
    assert!(!do_labels_match(0, &labels, &search));
  }

  #[test]
  fn test_labels_length_mismatch() {
    let labels = vec![Label::InPlace(b"foo".to_vec())];
    let search = vec![
      Reference {
        entry_idx: 0,
        label_idx: 0,
      },
      Reference {
        entry_idx: 0,
        label_idx: 1,
      },
    ];
    assert!(!do_labels_match(0, &labels, &search));

    let labels = vec![Label::InPlace(b"foo".to_vec()), Label::InPlace(b"bar".to_vec())];
    let search = vec![Reference {
      entry_idx: 0,
      label_idx: 0,
    }];
    assert!(!do_labels_match(0, &labels, &search));
  }

  #[test]
  fn test_resolve_reference_inplace() {
    let entry = Entry {
      scope_id: 0,
      labels: vec![Label::InPlace(b"foo".to_vec()), Label::InPlace(b"bar".to_vec())],
    };
    let index = Index {
      meta: Meta {
        chunk_size: 1,
        scopes: vec![PascalString {
          data: b"abc".to_vec(),
        }],
      },
      entries: vec![entry],
    };
    let reference = Reference {
      entry_idx: 0,
      label_idx: 1,
    };
    let result = index.resolve_reference(&reference);
    assert_eq!(result.unwrap(), b"bar");
  }

  #[test]
  fn test_resolve_reference_multiple_entries() {
    let entry1 = Entry {
      scope_id: 0,
      labels: vec![Label::InPlace(b"foo".to_vec())],
    };
    let entry2 = Entry {
      scope_id: 0,
      labels: vec![Label::InPlace(b"bar".to_vec()), Label::InPlace(b"baz".to_vec())],
    };
    let index = Index {
      meta: Meta {
        chunk_size: 1,
        scopes: vec![PascalString {
          data: b"abc".to_vec(),
        }],
      },
      entries: vec![entry1, entry2],
    };
    let reference = Reference {
      entry_idx: 1,
      label_idx: 1,
    };
    let result = index.resolve_reference(&reference);
    assert_eq!(result.unwrap(), b"baz");
  }

  #[test]
  fn test_resolve_reference_mixed_labels() {
    let entry = Entry {
      scope_id: 0,
      labels: vec![
        Label::Reference(Reference {
          entry_idx: 0,
          label_idx: 0,
        }),
        Label::InPlace(b"real".to_vec()),
      ],
    };
    let index = Index {
      meta: Meta {
        chunk_size: 1,
        scopes: vec![PascalString {
          data: b"abc".to_vec(),
        }],
      },
      entries: vec![entry],
    };
    let reference = Reference {
      entry_idx: 0,
      label_idx: 1,
    };
    let result = index.resolve_reference(&reference);
    assert_eq!(result.unwrap(), b"real");
  }

  #[test]
  fn test_resolve_reference_out_of_bounds_entry() {
    let entry = Entry {
      scope_id: 0,
      labels: vec![Label::InPlace(b"foo".to_vec())],
    };
    let index = Index {
      meta: Meta {
        chunk_size: 1,
        scopes: vec![PascalString {
          data: b"abc".to_vec(),
        }],
      },
      entries: vec![entry],
    };
    let reference = Reference {
      entry_idx: 1,
      label_idx: 0,
    };
    let result = index.resolve_reference(&reference);
    assert!(matches!(result, Err(IxxError::ReferenceOutOfBounds)));
  }

  #[test]
  fn test_resolve_reference_out_of_bounds_label() {
    let entry = Entry {
      scope_id: 0,
      labels: vec![Label::InPlace(b"foo".to_vec())],
    };
    let index = Index {
      meta: Meta {
        chunk_size: 1,
        scopes: vec![PascalString {
          data: b"abc".to_vec(),
        }],
      },
      entries: vec![entry],
    };
    let reference = Reference {
      entry_idx: 0,
      label_idx: 1,
    };
    let result = index.resolve_reference(&reference);
    assert!(matches!(result, Err(IxxError::ReferenceOutOfBounds)));
  }

  #[test]
  fn test_resolve_reference_recursive() {
    let entry = Entry {
      scope_id: 0,
      labels: vec![Label::Reference(Reference {
        entry_idx: 0,
        label_idx: 0,
      })],
    };
    let index = Index {
      meta: Meta {
        chunk_size: 1,
        scopes: vec![PascalString {
          data: b"abc".to_vec(),
        }],
      },
      entries: vec![entry],
    };
    let reference = Reference {
      entry_idx: 0,
      label_idx: 0,
    };
    let result = index.resolve_reference(&reference);
    assert!(matches!(result, Err(IxxError::RecursiveReference)));
  }
}
