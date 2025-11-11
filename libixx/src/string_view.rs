use std::fmt::Display;

pub struct StringView<'a> {
  parts: Vec<&'a str>,
}

impl<'a> From<Vec<&'a str>> for StringView<'a> {
  fn from(parts: Vec<&'a str>) -> Self {
    Self { parts }
  }
}

impl Display for StringView<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.parts.is_empty() {
      return Ok(());
    }

    write!(f, "{}", self.parts[0])?;

    for part in &self.parts[1..] {
      write!(f, ".{}", part)?;
    }

    Ok(())
  }
}

impl StringView<'_> {
  pub fn matches(&self, search: &[&str]) -> bool {
    let mut self_parts_start = 0;
    let mut self_parts_start_str_idx = 0;

    for segment in search {
      for part in segment.split('.') {
        'outer: {
          for (self_part_idx, self_part) in self.parts[self_parts_start..].iter().enumerate() {
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
          return false;
        }
      }
    }

    true
  }
}

fn ascii_ignore_case_find(a: &str, b: &str) -> Option<usize> {
  if a.len() < b.len() || b.is_empty() {
    return None;
  }

  let end = a.len() - (b.len() - 1);

  (0..end).find(|&start| {
    std::iter::zip(a[start..].chars(), b.chars()).all(|(a, b)| a.eq_ignore_ascii_case(&b))
  })
}
