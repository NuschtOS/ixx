use crate::{Index, IndexBuilder};

#[test]
fn test_get_idx_by_name() {
  let mut index_builder = IndexBuilder::new(100);

  index_builder.push(0, "home.enableDebugInfo");
  index_builder.push(0, "home.enableNixpkgsReleaseCheck");
  index_builder.push(0, "programs.home-manager.enable");
  index_builder.push(0, "services.home-manager.autoUpgrade.enable");
  index_builder.push(0, "services.home-manager.autoUpgrade.frequency");

  index_builder.push(0, "pretalx");
  index_builder.push(0, "nixosTests.pretalx");
  index_builder.push(0, "nixosTests.allDrivers.pretalx");

  index_builder.push(1, "home.enableDebugInfo");

  let index: Index = index_builder.into();

  assert_eq!(index.get_idx_by_name(0, "home.enableDebugInfo").unwrap(), Some(0));

  assert_eq!(
    index.get_idx_by_name(0, "pretalx").unwrap(),
    Some(5),
    "Should find 'pretalx' in scope 0"
  );
  assert_eq!(
    index.get_idx_by_name(0, "nixosTests.pretalx").unwrap(),
    Some(6),
    "Should find 'nixosTests.pretalx' in scope 0"
  );
  assert_eq!(
    index.get_idx_by_name(0, "nixosTests.allDrivers.pretalx").unwrap(),
    Some(7),
    "Should find 'nixosTests.allDrivers.pretalx' in scope 0"
  );
}
