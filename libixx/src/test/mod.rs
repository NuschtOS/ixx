use std::{collections::HashMap, fs::File, io::Write};

use crate::Index;

#[test]
fn test() {
  let options: HashMap<String, crate::option::Option> =
    serde_json::from_str(include_str!("./options.json")).unwrap();

  let options = options.keys().collect::<Vec<_>>();

  let mut index = Index::new();
  for option in &options {
    index.push(option);
  }

  let mut file = File::create("index.nuscht").unwrap();
  index.write_into(&mut file).unwrap();
}
