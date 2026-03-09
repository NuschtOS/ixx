use crate::Index;

#[test]
fn test_get_idx_by_name_one_level() {
  let index = Index::build(vec![("a", 0), ("b", 0), ("c", 0)].as_slice());

  assert_eq!(
    index.get_idx_by_name(0, "a"),
    Some(0),
    "Should find 'a' in scope 0"
  );
  assert_eq!(
    index.get_idx_by_name(0, "b"),
    Some(1),
    "Should find 'b' in scope 0"
  );
  assert_eq!(
    index.get_idx_by_name(0, "c"),
    Some(2),
    "Should find 'c' in scope 0"
  );
}

#[test]
fn test_get_idx_by_name_complex() {
  let index = Index::build(
    vec![
      ("home.enableDebugInfo", 0),
      ("home.enableNixpkgsReleaseCheck", 0),
      ("pretalx", 0),
      ("nixosTests.pretalx", 0),
      ("nixosTests.allDrivers.pretalx", 0),
      ("home.enableDebugInfo", 1),
    ]
    .as_slice(),
  );

  assert_eq!(index.get_idx_by_name(0, "home.enableDebugInfo"), Some(0));

  assert_eq!(
    index.get_idx_by_name(0, "pretalx"),
    Some(2),
    "Should find 'pretalx' in scope 0"
  );
  assert_eq!(
    index.get_idx_by_name(0, "nixosTests.pretalx"),
    Some(3),
    "Should find 'nixosTests.pretalx' in scope 0"
  );
  assert_eq!(
    index.get_idx_by_name(0, "nixosTests.allDrivers.pretalx"),
    Some(4),
    "Should find 'nixosTests.allDrivers.pretalx' in scope 0"
  );
}
