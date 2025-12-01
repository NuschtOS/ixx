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

            if let Some(idx) = ascii_ignore_case_find(&self_part[self_parts_start_str_idx..], part)
            {
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
  a.len() == b.len()
    && a
      .iter()
      .zip(b)
      .all(|(a, b)| eq_ignore_ascii_case_char(*a, *b))
}

#[inline(always)]
pub fn eq_ignore_ascii_case_char(a: u8, b: u8) -> bool {
  // Branchless check for ASCII alphabetic
  a == b || (a ^ b == 0b00100000 && a.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
  use crate::string_view::{ascii_ignore_case_find, eq_ignore_ascii_case_char};

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
