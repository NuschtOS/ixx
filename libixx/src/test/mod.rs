use crate::Index;

#[test]
fn test() {
  let mut index = Index::default();

  index.push("home.enableDebugInfo");
  index.push("home.enableNixpkgsReleaseCheck");
  index.push("home.file.<name>.enable");
  index.push("home.language.measurement");
  index.push("home.pointerCursor.gtk.enable");
  index.push("home.pointerCursor.x11.enable");
  index.push("programs.home-manager.enable");
  index.push("services.home-manager.autoUpgrade.enable");
  index.push("services.home-manager.autoUpgrade.frequency");

  assert_eq!(
    index.search("ho*auto", 10).unwrap(),
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
    index.search("ho*auto*ena", 10).unwrap(),
    vec![(
      7usize,
      "services.home-manager.autoUpgrade.enable".to_string()
    )]
  );

  assert_eq!(
    index.search("ho*en*Nix", 10).unwrap(),
    vec![(1usize, "home.enableNixpkgsReleaseCheck".to_string())]
  );
}
