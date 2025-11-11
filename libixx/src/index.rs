use std::{
  collections::HashMap,
  io::{Cursor, Read, Seek, Write},
  string::FromUtf8Error,
};

use binrw::{BinRead, BinWrite, Endian, VecArgs, binrw};

use crate::{IxxError, string_view::StringView};

pub struct IndexBuilder {
  index: Index,
  label_cache: HashMap<Vec<u8>, (usize, u8)>,
}

#[binrw]
#[brw(magic = b"ixx02")]
#[derive(Debug, Clone, PartialEq)]
pub struct Index {
  meta: Meta,
  #[bw(calc = entries.len() as u32)]
  count: u32,
  #[br(count = count)]
  entries: Vec<Entry>,
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
  data: Vec<u8>,
}

#[binrw]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Entry {
  /// index in the scopes Vec
  scope_id: u8,
  #[bw(calc = labels.len() as u8)]
  count: u8,
  #[br(count = count)]
  labels: Vec<Label>,
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
        entry_idx: u8::read_options(reader, endian, ())? as u64,
        label_idx,
      })),
      1 => Ok(Self::Reference(Reference {
        entry_idx: u16::read_options(reader, endian, ())? as u64,
        label_idx,
      })),
      2 => Ok(Self::Reference(Reference {
        entry_idx: u32::read_options(reader, endian, ())? as u64,
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
        if buf.len() > (u8::MAX >> 1) as usize {
          panic!("Label is to wide.");
        }

        (buf.len() as u8).write_options(writer, endian, ())?;
        buf.write_options(writer, endian, ())?;
      }
      Label::Reference(Reference {
        entry_idx,
        label_idx,
      }) => {
        if *label_idx > (u8::MAX >> 3) {
          panic!("Label index to big, contact developer!");
        }

        if *entry_idx < u8::MAX as u64 {
          ((1u8 << 7) | (0 << 5) | label_idx).write_options(writer, endian, ())?;
          (*entry_idx as u8).write_options(writer, endian, ())?;
        } else if *entry_idx < u16::MAX as u64 {
          ((1u8 << 7) | (1 << 5) | label_idx).write_options(writer, endian, ())?;
          (*entry_idx as u16).write_options(writer, endian, ())?;
        } else if *entry_idx < u32::MAX as u64 {
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
    let labels = if !name.contains('.') {
      vec![Label::InPlace(name.into())]
    } else {
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

          self.label_cache.insert(
            segment.to_vec(),
            (self.index.entries.len(), label_idx as u8),
          );

          Label::InPlace(segment.to_vec())
        })
        .collect()
    };

    self.index.entries.push(Entry { scope_id, labels });
  }

  pub fn push_scope(&mut self, scope: String) -> u8 {
    if self.index.meta.scopes.len() == u8::MAX.into() {
      panic!(
        "You reached the limit of 256 scopes. Please contact the developers for further assistance."
      );
    }

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

    let label = &entry[label_idx];

    match label {
      Label::InPlace(string) => Ok(string),
      Label::Reference(_) => Err(IxxError::RecursiveReference),
    }
  }

  pub fn get_idx_by_name(&self, scope_id: u8, name: &str) -> Result<Option<usize>, IxxError> {
    let mut labels = Vec::new();
    for segment in name.split('.') {
      let segment = segment.as_bytes();

      'outer: {
        for (
          entry_idx,
          Entry {
            labels: inner_labels,
            ..
          },
        ) in self.entries.iter().enumerate()
        {
          for (label_idx, label) in inner_labels.iter().enumerate() {
            if let Label::InPlace(inplace) = label {
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
        }

        return Ok(None);
      }
    }

    Ok(
      self
        .entries
        .iter()
        .enumerate()
        .find(
          |(
            idx,
            Entry {
              scope_id: entry_scope_id,
              labels: inner_labels,
            },
          )| *entry_scope_id == scope_id && do_labels_match(*idx, inner_labels, &labels),
        )
        .map(|(idx, _)| idx),
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
      .map(|x| x.as_bytes())
      // * at the start or end of a string
      .filter(|x| !x.is_empty())
      .collect::<Vec<_>>();

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
        results.push((idx, *entry_scope_id, entry_name.to_string()));
        if results.len() == max_results {
          return Ok(results);
        }
      }
    }

    Ok(results)
  }

  pub fn meta(&self) -> &Meta {
    &self.meta
  }

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
  let matching = labels
    .iter()
    .enumerate()
    .zip(search.iter())
    .filter(|&((label_idx, entry), search)| match entry {
      Label::InPlace(_) => {
        entry_idx == search.entry_idx as usize && label_idx == search.label_idx as usize
      }
      Label::Reference(reference) => reference == search,
    })
    .count();

  matching == labels.len() && matching == search.len()
}
