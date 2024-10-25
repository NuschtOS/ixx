use crate::Index;

#[test]
fn test() {
  // chunk_size does not really matter here currently
  let mut index = Index::new(100);

  index.push(0, "home.enableDebugInfo");
  index.push(0, "home.enableNixpkgsReleaseCheck");
  index.push(0, "home.file.<name>.enable");
  index.push(0, "home.language.measurement");
  index.push(0, "home.pointerCursor.gtk.enable");
  index.push(0, "home.pointerCursor.x11.enable");
  index.push(0, "programs.home-manager.enable");
  index.push(0, "services.home-manager.autoUpgrade.enable");
  index.push(0, "services.home-manager.autoUpgrade.frequency");
  index.push(1, "home.enableDebugInfo");

  assert_eq!(
    index.search(None, "ho*auto", 10).unwrap(),
    vec![
      (7, 0, "services.home-manager.autoUpgrade.enable".to_string()),
      (
        8,
        0,
        "services.home-manager.autoUpgrade.frequency".to_string()
      )
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
    index
      .search(Some(0), "enablenixpkgsreleasecheck", 10)
      .unwrap(),
    vec![(1, 0, "home.enableNixpkgsReleaseCheck".to_string())]
  );
  assert_eq!(
    index
      .search(Some(1), "enablenixpkgsreleasecheck", 10)
      .unwrap(),
    vec![]
  );

  // TEST options with same name in different scopes
  assert_eq!(
    index.search(None, "ho*debug", 10).unwrap(),
    vec![
      (0, 0, "home.enableDebugInfo".to_string()),
      (9, 1, "home.enableDebugInfo".to_string())
    ]
  );

  assert_eq!(
    index.get_idx_by_name(0, "home.enableDebugInfo").unwrap(),
    Some(0)
  );

  assert_eq!(
    index.get_idx_by_name(1, "home.enableDebugInfo").unwrap(),
    Some(9)
  );
}
