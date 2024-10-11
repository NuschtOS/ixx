use std::{collections::HashMap, fs::File};

use args::{Action, Args};
use clap::Parser;
use libixx::Index;

mod args;

fn main() -> anyhow::Result<()> {
  let args = Args::parse();

  match args.action {
    Action::Index(module) => {
      let mut index = Index::new();

      for path in module.files {
        println!("Parsing {}", path.to_string_lossy());
        let file = File::open(path)?;
        let options: HashMap<String, libixx::option::Option> = serde_json::from_reader(file)?;
        options.keys().for_each(|value| index.push(value));
        println!("indexed {} options", options.len());
      }

      println!("Writing index to {}", module.output.to_string_lossy());

      let mut output = File::create(module.output)?;
      index.write_into(&mut output)?;
    }
    Action::Search(module) => {
      let mut file = File::open(module.index)?;
      let index = Index::read_from(&mut file)?;

      let result = index.search(&module.query, module.max_results as usize)?;
      for option in result {
        println!("{}", option);
      }
    }
  }

  Ok(())
}
