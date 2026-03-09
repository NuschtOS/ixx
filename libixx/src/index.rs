use std::{
  collections::HashMap,
  io::{Cursor, Read, Seek, Write},
  string::FromUtf8Error,
};

use binrw::{BinRead, BinWrite, Endian, binrw};

use levenshtein::levenshtein;

use crate::{IxxError, string_view::StringView};

#[binrw]
#[brw(magic = b"ixx02")]
#[derive(Debug, Clone, PartialEq)]
pub struct Index {
  #[bw(calc = labels.len() as u32)]
  label_count: u32,
  #[br(count = label_count)]
  pub(crate) labels: Vec<PascalString>,
  #[bw(calc = entries.len() as u32)]
  entry_count: u32,
  #[br(count = entry_count)]
  pub(crate) entries: Vec<Entry>,
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
  label_count: u8,
  #[br(count = label_count)]
  pub(crate) labels: Vec<LabelReference>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LabelReference(pub u64);

impl BinRead for LabelReference {
  type Args<'a> = ();

  fn read_options<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    _args: Self::Args<'_>,
  ) -> binrw::BinResult<Self> {
    let first = u8::read_options(reader, endian, ())?;

    match first {
      0 => Ok(Self(u64::from(u8::read_options(reader, endian, ())?))),
      1 => Ok(Self(u64::from(u16::read_options(reader, endian, ())?))),
      2 => Ok(Self(u64::from(u32::read_options(reader, endian, ())?))),
      3 => Ok(Self(u64::read_options(reader, endian, ())?)),
      _ => Err(binrw::Error::AssertFail {
        pos: reader.stream_position()?,
        message: "Invalid label integer size".into(),
      }),
    }
  }
}

impl BinWrite for LabelReference {
  type Args<'a> = ();

  fn write_options<W: Write + Seek>(
    &self,
    writer: &mut W,
    endian: Endian,
    _args: Self::Args<'_>,
  ) -> binrw::BinResult<()> {
    if self.0 <= u64::from(u8::MAX) {
      0u8.write_options(writer, endian, ())?;
      (self.0 as u8).write_options(writer, endian, ())?;
    } else if self.0 <= u64::from(u16::MAX) {
      1u8.write_options(writer, endian, ())?;
      (self.0 as u16).write_options(writer, endian, ())?;
    } else if self.0 <= u64::from(u32::MAX) {
      2u8.write_options(writer, endian, ())?;
      (self.0 as u32).write_options(writer, endian, ())?;
    } else {
      3u8.write_options(writer, endian, ())?;
      self.0.write_options(writer, endian, ())?;
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

impl From<&str> for PascalString {
  fn from(value: &str) -> Self {
    value.to_string().into()
  }
}

impl TryFrom<PascalString> for String {
  type Error = FromUtf8Error;

  fn try_from(value: PascalString) -> Result<Self, Self::Error> {
    String::from_utf8(value.data)
  }
}

impl PartialEq<str> for &PascalString {
  fn eq(&self, other: &str) -> bool {
    self.data == other.as_bytes()
  }
}

impl Index {
  pub fn build(entries: &[(&str, u8)]) -> Self {
    let mut labels = HashMap::new();

    for (entry, _) in entries {
      for label in entry.split('.') {
        labels
          .entry(label)
          .and_modify(|count| *count += 1u64)
          .or_insert(1u64);
      }
    }

    let mut labels = labels.into_iter().collect::<Vec<_>>();

    labels.sort_by(|(_, a), (_, b)| a.cmp(b).reverse());

    let labels_lookup = labels
      .iter()
      .enumerate()
      .map(|(idx, (label, _))| (label, idx))
      .collect::<HashMap<_, _>>();

    let labels = labels.iter().map(|(label, _)| label.to_string().into()).collect();

    let entries = entries
      .iter()
      .map(|(entry, scope_id)| Entry {
        scope_id: *scope_id,
        labels: entry
          .split('.')
          .map(|label| {
            LabelReference(
              *labels_lookup
                .get(&label)
                .expect("this can not happen, the hashmap was build based on the same data")
                as u64,
            )
          })
          .collect(),
      })
      .collect();

    Index { labels, entries }
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

  pub fn resolve_reference(&self, reference: LabelReference) -> Option<&PascalString> {
    self.labels.get(reference.0 as usize)
  }

  pub fn get_idx_by_name(&self, scope_id: u8, name: &str) -> Option<usize> {
    let labels = name
      .split('.')
      .map(|segment| {
        self
          .labels
          .iter()
          .enumerate()
          .find(|(_, label)| label == segment)
          .map(|(idx, _)| LabelReference(idx as u64))
      })
      .collect::<Option<Vec<_>>>()?;

    self
      .entries
      .iter()
      .enumerate()
      .find(|(_, entry)| entry.scope_id == scope_id && entry.labels == labels)
      .map(|(idx, _)| idx)
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
      .map(|segment| {
        segment
          .split(|char| *char == b'.')
          .filter(|x| !x.is_empty())
          .collect()
      })
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
  pub fn size(&self) -> usize {
    self.entries.len()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Cursor;

  #[test]
  fn build_single_entry() {
    let index = Index::build(&[("foo.bar", 0)]);

    assert_eq!(index.entries.len(), 1);
    assert_eq!(index.labels.len(), 2);

    let entry = &index.entries[0];
    assert_eq!(entry.scope_id, 0);
    assert_eq!(entry.labels.len(), 2);

    let l0 = index.resolve_reference(entry.labels[0]).unwrap();
    let l1 = index.resolve_reference(entry.labels[1]).unwrap();

    assert_eq!(l0.data, b"foo");
    assert_eq!(l1.data, b"bar");
  }

  #[test]
  fn build_two_entries_shared_label() {
    let index = Index::build(&[("foo.bar", 0), ("foo.baz", 0)]);

    assert_eq!(index.entries.len(), 2);

    let e0 = &index.entries[0];
    let e1 = &index.entries[1];

    let foo0 = index.resolve_reference(e0.labels[0]).unwrap();
    let foo1 = index.resolve_reference(e1.labels[0]).unwrap();

    assert_eq!(foo0.data, b"foo");
    assert_eq!(foo1.data, b"foo");

    assert_eq!(foo0, foo1);
  }

  #[test]
  fn resolve_reference() {
    let index = Index::build(&[("foo.bar", 0)]);

    let entry = &index.entries[0];

    let label = index.resolve_reference(entry.labels[1]).unwrap();
    assert_eq!(label.data, b"bar");
  }

  #[test]
  fn get_idx_by_name() {
    let index = Index::build(&[("foo.bar", 0), ("foo.baz", 1)]);

    let idx = index.get_idx_by_name(0, "foo.bar");
    assert_eq!(idx, Some(0));

    let idx = index.get_idx_by_name(1, "foo.baz");
    assert_eq!(idx, Some(1));

    let idx = index.get_idx_by_name(0, "foo.baz");
    assert_eq!(idx, None);
  }

  #[test]
  fn write_read_roundtrip() {
    let index = Index::build(&[("foo.bar", 0), ("foo.baz", 1)]);

    let mut buf = Cursor::new(Vec::new());
    index.write_into(&mut buf).unwrap();

    // dump raw bytes for debugging
    let data = buf.get_ref();
    eprintln!("written {} bytes: {:02x?}", data.len(), data);

    buf.set_position(0);

    let decoded = Index::read_from(&mut buf).unwrap();

    eprintln!("decoded index: {:#?}", decoded);

    assert_eq!(index, decoded);
  }

  #[test]
  fn search_exact_match() {
    let index = Index::build(&[("foo.bar", 0), ("foo.baz", 0), ("alpha.beta", 1)]);

    let results = index.search(None, "foo.bar", 10).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].2.as_str(), "foo.bar");
  }

  #[test]
  fn search_wildcard() {
    let index = Index::build(&[("foo.bar", 0), ("foo.baz", 0), ("alpha.beta", 1)]);

    let results = index.search(None, "foo.*", 10).unwrap();

    assert_eq!(results.len(), 2);
  }
}
