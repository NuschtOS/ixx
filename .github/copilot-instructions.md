# Copilot Instructions for ixx

## Project Overview
- **ixx** is a Rust-based search engine and index creation tool for NüschtOS Search.
- The workspace contains multiple Rust crates: `ixx`, `libixx`, and `fixx` (WASM target).
- Main CLI entrypoint: `ixx/src/main.rs`.
- Index logic and binary format: `libixx/src/index.rs`.
- WASM build: `fixx` crate, output in `fixx/pkg`.

## Build & Test Workflows
- **Build all crates:**
  - Standard: `cargo build --workspace`
  - WASM: `wasm-pack build --release fixx --target web` (output: `fixx/pkg`)
- **Run CLI:**
  - `cargo run --package ixx -- <subcommand> [args]`
- **Testing:**
  - `cargo test --workspace`
- **Benchmarks:**
  - `cargo bench --package libixx`

## CLI Usage Patterns
- Subcommands: `index`, `search`, `meta` (see `ixx/src/args.rs`)
- Example: `ixx index <config-path> [--options-index-output ...]`
- Output files: index and chunk files for both options and packages
- Search supports output formats: `--format json|text`

## Index Format & Data Flow
- Indexes are binary files with magic header `ixx02` (see `libixx/src/index.rs`).
- Entries are grouped by scopes; labels use in-place or reference encoding for compression.
- Search uses Levenshtein distance for fuzzy matching.
- Data flows: CLI parses args → calls action module → reads/writes index via `libixx`.

## Conventions & Patterns
- All index and chunk files are written/read via `libixx` APIs.
- Option and package indexes are separated (see default paths in `ixx/src/args.rs`).
- Use `anyhow` for error handling, `serde` for serialization.
- Prefer explicit subcommand modules for CLI logic (`ixx/src/action/`).
- WASM build is only for the `fixx` crate; main logic is in Rust.

## Integration Points
- External: `wasm-pack`, `tokio`, `clap`, `serde`, `binrw`, `levenshtein`.
- WASM output integrates with web frontends via `fixx/pkg/fixx.js`.

## Key Files & Directories
- `ixx/src/main.rs`, `ixx/src/args.rs`, `ixx/src/action/`
- `libixx/src/index.rs` (index format, search logic)
- `fixx/pkg/` (WASM output)
- `Cargo.toml` (workspace, dependencies)

## Example Workflow
1. Build index: `cargo run --package ixx -- index <config-path>`
2. Search: `cargo run --package ixx -- search <index-path> --query <term> --format json`
3. Build WASM: `wasm-pack build --release fixx --target web`

---

For questions, see the main [README.md](../README.md) or join Matrix chat.
