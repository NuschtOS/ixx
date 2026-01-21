use crate::{Index, IndexBuilder};

#[test]
fn test_get_idx_by_name_one_level() {
  let mut index_builder = IndexBuilder::default();

  index_builder.push(0, "a");
  index_builder.push(0, "b");
  index_builder.push(0, "c");

  let index: Index = index_builder.into();

  assert_eq!(
    index.get_idx_by_name(0, "a").unwrap(),
    Some(0),
    "Should find 'a' in scope 0"
  );
  assert_eq!(
    index.get_idx_by_name(0, "b").unwrap(),
    Some(1),
    "Should find 'b' in scope 0"
  );
  assert_eq!(
    index.get_idx_by_name(0, "c").unwrap(),
    Some(2),
    "Should find 'c' in scope 0"
  );
}

#[test]
fn test_get_idx_by_name_complex() {
  let mut index_builder = IndexBuilder::default();

  index_builder.push(0, "home.enableDebugInfo");
  index_builder.push(0, "home.enableNixpkgsReleaseCheck");

  index_builder.push(0, "pretalx");
  index_builder.push(0, "nixosTests.pretalx");
  index_builder.push(0, "nixosTests.allDrivers.pretalx");

  index_builder.push(1, "home.enableDebugInfo");

  let index: Index = index_builder.into();

  assert_eq!(index.get_idx_by_name(0, "home.enableDebugInfo").unwrap(), Some(0));

  assert_eq!(
    index.get_idx_by_name(0, "pretalx").unwrap(),
    Some(2),
    "Should find 'pretalx' in scope 0"
  );
  assert_eq!(
    index.get_idx_by_name(0, "nixosTests.pretalx").unwrap(),
    Some(3),
    "Should find 'nixosTests.pretalx' in scope 0"
  );
  assert_eq!(
    index.get_idx_by_name(0, "nixosTests.allDrivers.pretalx").unwrap(),
    Some(4),
    "Should find 'nixosTests.allDrivers.pretalx' in scope 0"
  );
}
