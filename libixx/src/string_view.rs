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

    write!(f, "{}", unsafe { std::str::from_utf8_unchecked(part) })?;

    for part in &self.parts[1..] {
      let part = match &part {
        Label::InPlace(items) => items,
        Label::Reference(reference) => self.index.resolve_reference(reference).unwrap(),
      };

      write!(f, ".{}", unsafe { std::str::from_utf8_unchecked(part) })?;
    }

    Ok(())
  }
}

impl StringView<'_, '_> {
  pub fn matches(&self, search: &[&[u8]]) -> Result<bool, IxxError> {
    let mut self_parts_start = 0;
    let mut self_parts_start_str_idx = 0;

    for segment in search {
      for part in segment.split(|char| *char == b'.') {
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

fn ascii_ignore_case_find(a: &[u8], b: &[u8]) -> Option<usize> {
  if a.len() < b.len() || b.is_empty() {
    return None;
  }

  let end = a.len() - (b.len() - 1);

  for start in 0..end {
    'outer: {
      for i in 0..b.len() {
        if !eq_ignore_ascii_case(a[start + i], b[i]) {
          break 'outer;
        }
      }
      return Some(start);
    }
  }

  None
}

/// This is not correct, as the supplied bytes could be part of a multi-byte utf8 character.
/// Therefore it can never be correct to motify the case before comparing.
fn eq_ignore_ascii_case(a: u8, b: u8) -> bool {
  // set ascii-lowercase bit always to true
  let a = a | 0b100000;
  let b = b | 0b100000;

  a == b
}
