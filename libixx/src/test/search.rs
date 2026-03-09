use crate::Index;

#[test]
fn test() {
  let index = Index::build(
    vec![
      ("home.enableDebugInfo", 0),
      ("home.enableNixpkgsReleaseCheck", 0),
      ("home.file.<name>.enable", 0),
      ("home.language.measurement", 0),
      ("home.pointerCursor.gtk.enable", 0),
      ("home.pointerCursor.x11.enable", 0),
      ("programs.home-manager.enable", 0),
      ("services.home-manager.autoUpgrade.enable", 0),
      ("services.home-manager.autoUpgrade.frequency", 0),
      ("pretalx", 0),
      ("nixosTests.pretalx", 0),
      ("nixosTests.allDrivers.pretalx", 0),
      ("home.enableDebugInfo", 1),
    ]
    .as_slice(),
  );

  assert_eq!(
    index.search(None, "ho*auto", 10).unwrap(),
    vec![
      (7, 0, "services.home-manager.autoUpgrade.enable".to_string()),
      (8, 0, "services.home-manager.autoUpgrade.frequency".to_string())
    ]
  );

  assert_eq!(
    index.search(None, "ho*auto*ena", 10).unwrap(),
    vec![(7, 0, "services.home-manager.autoUpgrade.enable".to_string())]
  );

  assert_eq!(
    index.search(None, "ho*en*Nix", 10).unwrap(),
    vec![(1, 0, "home.enableNixpkgsReleaseCheck".to_string())]
  );

  assert_eq!(
    index.search(None, "ho*en*Nix*Rel*Che", 10).unwrap(),
    vec![(1, 0, "home.enableNixpkgsReleaseCheck".to_string())]
  );

  assert_eq!(
    index.search(None, "enablenixpkgsreleasecheck", 10).unwrap(),
    vec![(1, 0, "home.enableNixpkgsReleaseCheck".to_string())]
  );

  // TEST scopes
  assert_eq!(
    index.search(Some(0), "enablenixpkgsreleasecheck", 10).unwrap(),
    vec![(1, 0, "home.enableNixpkgsReleaseCheck".to_string())]
  );
  assert_eq!(
    index.search(Some(1), "enablenixpkgsreleasecheck", 10).unwrap(),
    vec![]
  );

  // query with no matches
  assert_eq!(index.search(None, "nonexistent.option", 10).unwrap(), vec![]);

  // TEST options with same name in different scopes
  assert_eq!(
    index.search(None, "ho*debug", 10).unwrap(),
    vec![
      (0, 0, "home.enableDebugInfo".to_string()),
      (12, 1, "home.enableDebugInfo".to_string())
    ]
  );

  assert_eq!(index.get_idx_by_name(1, "home.enableDebugInfo"), Some(12));
}
