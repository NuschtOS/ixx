use std::fmt::Display;

use crate::{Index, IxxError, index::Label};

pub struct StringView<'a, 'b> {
  index: &'a Index,
  parts: &'b [Label],
}

impl<'a, 'b> From<(&'a Index, &'b [Label])> for StringView<'a, 'b> {
  fn from((index, parts): (&'a Index, &'b [Label])) -> Self {
    Self { index, parts }
  }
}

impl Display for StringView<'_, '_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.parts.is_empty() {
      return Ok(());
    }

    let part = match &self.parts[0] {
      Label::InPlace(items) => items,
      Label::Reference(reference) => self.index.resolve_reference(reference).unwrap(),
    };

    write!(f, "{}", std::str::from_utf8(part).unwrap())?;

    for part in &self.parts[1..] {
      let part = match &part {
        Label::InPlace(items) => items,
        Label::Reference(reference) => self.index.resolve_reference(reference).unwrap(),
      };

      write!(f, ".{}", std::str::from_utf8(part).unwrap())?;
    }

    Ok(())
  }
}

impl StringView<'_, '_> {
  pub fn matches(&self, search: &[Vec<&[u8]>]) -> Result<bool, IxxError> {
    let mut self_parts_start = 0;
    let mut self_parts_start_str_idx = 0;

    for segment in search {
      for part in segment {
        'outer: {
          for (self_part_idx, self_part) in self.parts[self_parts_start..].iter().enumerate() {
            let self_part = match &self_part {
              Label::InPlace(items) => items,
              Label::Reference(reference) => self.index.resolve_reference(reference)?,
            };

            if let Some(idx) = ascii_ignore_case_find(&self_part[self_parts_start_str_idx..], part) {
              self_parts_start += self_part_idx;
              if self_part_idx == 0 {
                self_parts_start_str_idx += idx;
              } else {
                self_parts_start_str_idx = 0;
              }
              break 'outer;
            }
            self_parts_start_str_idx = 0;
          }
          return Ok(false);
        }
      }
    }

    Ok(true)
  }
}

#[inline(always)]
pub fn ascii_ignore_case_find(a: &[u8], needle: &[u8]) -> Option<usize> {
  let n = needle.len();
  if n == 0 || a.len() < n {
    return None;
  }

  for (i, window) in a.windows(n).enumerate() {
    if eq_ignore_ascii_case(window, needle) {
      return Some(i);
    }
  }

  None
}

#[inline(always)]
pub fn eq_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
  // the additional bounds check improved LLVM auto vectorization?
  a.len() == b.len() && a.iter().zip(b).all(|(a, b)| eq_ignore_ascii_case_char(*a, *b))
}

#[inline(always)]
pub fn eq_ignore_ascii_case_char(a: u8, b: u8) -> bool {
  // Branchless check for ASCII alphabetic
  a == b
    || (a ^ b == 0b0010_0000 &&
    // optimized a.is_ascii_alphabetic()
    (( b'A' <= a && a <= b'Z') || (b'a' <= a && a <= b'z')))
}

#[cfg(test)]
mod tests {
  use crate::index::*;
  use crate::string_view::*;

  fn make_index_with_labels(labels: Vec<Label>) -> Index {
    Index {
      meta: Meta {
        chunk_size: 1,
        scopes: vec![PascalString {
          data: b"scope".to_vec(),
        }],
      },
      entries: vec![Entry { scope_id: 0, labels }],
    }
  }

  #[test]
  fn test_string_view_matches_simple() {
    let index = make_index_with_labels(vec![
      Label::InPlace(b"foo".to_vec()),
      Label::InPlace(b"bar".to_vec()),
    ]);
    let view = StringView::from((&index, index.entries[0].labels.as_slice()));
    // Match both segments
    let pattern = vec![vec![b"foo".as_ref()], vec![b"bar".as_ref()]];
    assert!(view.matches(&pattern).unwrap());
    // Match only first segment
    let pattern = vec![vec![b"foo".as_ref()]];
    assert!(view.matches(&pattern).unwrap());
    // No match
    let pattern = vec![vec![b"baz".as_ref()]];
    assert!(!view.matches(&pattern).unwrap());
  }

  #[test]
  fn test_string_view_matches_case_insensitive() {
    let index = make_index_with_labels(vec![
      Label::InPlace(b"Foo".to_vec()),
      Label::InPlace(b"Bar".to_vec()),
    ]);
    let view = StringView::from((&index, index.entries[0].labels.as_slice()));
    let pattern = vec![vec![b"foo".as_ref()], vec![b"bar".as_ref()]];
    assert!(view.matches(&pattern).unwrap());
  }

  #[test]
  fn test_string_view_matches_partial_and_wildcard() {
    let index = make_index_with_labels(vec![
      Label::InPlace(b"foobar".to_vec()),
      Label::InPlace(b"baz".to_vec()),
    ]);
    let view = StringView::from((&index, index.entries[0].labels.as_slice()));
    // Partial match
    let pattern = vec![vec![b"foo".as_ref()]];
    assert!(view.matches(&pattern).unwrap());
    // Wildcard-like: match any segment
    let pattern = vec![vec![b"ba".as_ref()]];
    assert!(view.matches(&pattern).unwrap());
    // No match
    let pattern = vec![vec![b"qux".as_ref()]];
    assert!(!view.matches(&pattern).unwrap());
  }

  #[test]
  fn test_string_view_matches_empty_pattern() {
    let index = make_index_with_labels(vec![Label::InPlace(b"foo".to_vec())]);
    let view = StringView::from((&index, index.entries[0].labels.as_slice()));
    let pattern: Vec<Vec<&[u8]>> = vec![];
    assert!(view.matches(&pattern).unwrap());
  }

  #[test]
  fn test_string_view_matches_empty_labels() {
    let index = make_index_with_labels(vec![]);
    let view = StringView::from((&index, index.entries[0].labels.as_slice()));
    let pattern = vec![vec![b"foo".as_ref()]];
    assert!(!view.matches(&pattern).unwrap());
  }

  #[test]
  fn test_ascii_ignore_case_find() {
    assert_eq!(
      ascii_ignore_case_find("abcdefg".as_bytes(), "cde".as_bytes()),
      Some(2)
    );
    assert_eq!(
      ascii_ignore_case_find("abcdefg".as_bytes(), "cdefg".as_bytes()),
      Some(2)
    );
    assert_eq!(
      ascii_ignore_case_find("abcdefg".as_bytes(), "abc".as_bytes()),
      Some(0)
    );
    assert_eq!(
      ascii_ignore_case_find("abcdefg".as_bytes(), "abcdefg".as_bytes()),
      Some(0)
    );
    assert_eq!(
      ascii_ignore_case_find("abcdefg".as_bytes(), "xyz".as_bytes()),
      None
    );
  }

  #[test]
  fn test_eq_ignore_ascii_case() {
    for range in [b'a'..=b'z', b'A'..=b'Z'] {
      for x in range {
        let y = x.to_ascii_uppercase();
        println!("Testing {} and {}", x as char, y as char);
        assert!(eq_ignore_ascii_case_char(x, y));
      }
    }

    assert!(!eq_ignore_ascii_case_char(b'a', b'b'));
    assert!(!eq_ignore_ascii_case_char(b'!', b'?'));
  }
}
