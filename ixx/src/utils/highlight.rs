use std::sync::LazyLock;

use tree_sitter_highlight::{Highlight, HighlightConfiguration, Highlighter, HtmlRenderer};

// last queried from commit in Cargo.toml
// curl https://raw.githubusercontent.com/nix-community/tree-sitter-nix/refs/heads/master/queries/highlights.scm | rg '@([^\s)]+)' -or '$1' | sort | uniq
// NOTE: This are all types of syntax that tree-sitter-nix is able to unserstand,
//       for every entry, there needs to be a class in the css.
const HIGHLIGHT_NAMES: [&str; 18] = [
  "comment",
  "embedded",
  "escape",
  "function",
  "function.builtin",
  "keyword",
  "number",
  "operator",
  "property",
  "punctuation.bracket",
  "punctuation.delimiter",
  "punctuation.special",
  "string",
  "string.special.path",
  "string.special.uri",
  "variable",
  "variable.builtin",
  "variable.parameter",
];

static HIGHLIGHT_NAME_CLASSES: LazyLock<Vec<String>> = LazyLock::new(|| {
  HIGHLIGHT_NAMES
    .iter()
    .map(|name| format!("class=\"{}\"", name.replace('.', "-")))
    .collect()
});

static CONFIG: LazyLock<HighlightConfiguration> = LazyLock::new(|| {
  let mut config = HighlightConfiguration::new(
    tree_sitter_nix::LANGUAGE.into(),
    "nix",
    tree_sitter_nix::HIGHLIGHTS_QUERY,
    "",
    "",
  )
  .unwrap();

  config.configure(&HIGHLIGHT_NAMES);

  config
});

pub(crate) fn highlight(code: &str) -> String {
  let mut highlighter = Highlighter::new();

  let highlight_event = highlighter
    .highlight(&CONFIG, code.as_bytes(), None, |_| None)
    .expect("Failed to highlight");

  let mut renderer = HtmlRenderer::new();
  renderer
    .render(highlight_event, code.as_bytes(), &|Highlight(idx)| {
      HIGHLIGHT_NAME_CLASSES[idx].as_bytes()
    })
    .expect("Failed to render HTML");

  String::from_utf8(renderer.html).expect("Failed to parse rendered code as utf8")
}
