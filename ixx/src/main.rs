use std::{
  collections::{hash_map::Entry, BTreeMap, HashMap},
  fs::File,
  sync::LazyLock,
};

use args::{Action, Args};
use clap::Parser;
use libixx::Index;
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};

mod args;
mod option;

fn main() -> anyhow::Result<()> {
  let args = Args::parse();

  match args.action {
    Action::Index(module) => {
      let mut options: BTreeMap<String, option::Option> = BTreeMap::new();

      for path in module.files {
        println!("Parsing {}", path.to_string_lossy());
        let file = File::open(path)?;
        options.append(&mut serde_json::from_reader(file)?);
      }

      println!("Read {} options", options.len());

      let mut index = Index::new();

      let mut buckets = HashMap::new();

      for (name, option) in options {
        index.push(&name);

        let option = into_option(&name, option);

        match buckets.entry(libixx::hash(&name)) {
          Entry::Vacant(vac) => {
            vac.insert(vec![option]);
          }
          Entry::Occupied(mut occ) => {
            occ.get_mut().push(option);
          }
        }
      }

      println!("Writing index to {}", module.output.to_string_lossy());

      let mut output = File::create(module.output)?;
      index.write_into(&mut output)?;

      println!("Writing meta");

      if !std::fs::exists("meta")? {
        std::fs::create_dir("meta")?;
      }

      for (name, bucket) in buckets {
        let mut file = File::create(format!("meta/{}.json", name))?;
        serde_json::to_writer(&mut file, &bucket)?;
      }
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

impl From<option::Content> for String {
  fn from(value: option::Content) -> Self {
    match value {
      option::Content::LiteralExpression { text } => code_highlighter(&text, "nix"),
      option::Content::Markdown { text } => markdown::to_html(&text),
    }
  }
}

fn into_option(name: &str, option: option::Option) -> libixx::Option {
  libixx::Option {
    declarations: option
      .declarations
      .iter()
      .map(|declaration| declaration.url.to_string())
      .collect(),
    default: option.default.map(|option| option.into()),
    description: option.description,
    example: option.example.map(|example| example.into()),
    read_only: option.read_only,
    r#type: option.r#type,
    name: name.to_string(),
  }
}

static THEME_SET: LazyLock<syntect::highlighting::ThemeSet> =
  LazyLock::new(ThemeSet::load_defaults);
static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

fn code_highlighter(code: &str, lang: &str) -> String {
  let syntax = if let Some(syntax) = SYNTAX_SET.find_syntax_by_name(lang) {
    syntax
  } else {
    SYNTAX_SET.find_syntax_by_extension("html").unwrap()
  };

  highlighted_html_for_string(
    code,
    &SYNTAX_SET,
    syntax,
    &THEME_SET.themes["InspiredGitHub"],
  )
  .unwrap_or_else(|e| e.to_string())
}
