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

  assert_eq!(
    index.search(None, "ho*auto", 10).unwrap(),
    vec![
      (
        7usize,
        "services.home-manager.autoUpgrade.enable".to_string()
      ),
      (
        8usize,
        "services.home-manager.autoUpgrade.frequency".to_string()
      )
    ]
  );

  assert_eq!(
    index.search(None, "ho*auto*ena", 10).unwrap(),
    vec![(
      7usize,
      "services.home-manager.autoUpgrade.enable".to_string()
    )]
  );

  assert_eq!(
    index.search(None, "ho*en*Nix", 10).unwrap(),
    vec![(1usize, "home.enableNixpkgsReleaseCheck".to_string())]
  );

  assert_eq!(
    index.search(None, "ho*en*Nix*Rel*Che", 10).unwrap(),
    vec![(1usize, "home.enableNixpkgsReleaseCheck".to_string())]
  );

  assert_eq!(
    index.search(None, "enablenixpkgsreleasecheck", 10).unwrap(),
    vec![(1usize, "home.enableNixpkgsReleaseCheck".to_string())]
  );

  // TEST scopes
  assert_eq!(
    index
      .search(Some(0), "enablenixpkgsreleasecheck", 10)
      .unwrap(),
    vec![(1usize, "home.enableNixpkgsReleaseCheck".to_string())]
  );
  assert_eq!(
    index
      .search(Some(1), "enablenixpkgsreleasecheck", 10)
      .unwrap(),
    vec![]
  );
}
