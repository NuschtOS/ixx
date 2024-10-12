use std::{collections::BTreeMap, fs::File, path::Path, sync::LazyLock};

use libixx::Index;
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};

use crate::{args::IndexModule, option};

pub(crate) fn index(module: IndexModule) -> anyhow::Result<()> {
  let mut raw_options: BTreeMap<String, option::Option> = BTreeMap::new();

  for path in module.files {
    println!("Parsing {}", path.to_string_lossy());
    let file = File::open(path)?;
    raw_options.append(&mut serde_json::from_reader(file)?);
  }

  println!("Read {} options", raw_options.len());

  let mut index = Index::default();
  let mut options = Vec::new();

  for (name, option) in raw_options {
    index.push(&name);
    options.push(into_option(&name, option));
  }

  println!("Writing index to {}", module.output.to_string_lossy());

  let mut output = File::create(module.output)?;
  index.write_into(&mut output)?;

  println!("Writing meta");

  if !Path::new("meta").exists() {
    std::fs::create_dir("meta")?;
  }

  for (idx, chunk) in options.chunks(module.chunk_size).enumerate() {
    let mut file = File::create(format!("meta/{}.json", idx))?;
    serde_json::to_writer(&mut file, &chunk)?;
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
