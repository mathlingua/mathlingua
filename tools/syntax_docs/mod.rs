//! Internal syntax-documentation generator.
//!
//! This tool reads the implementation sources and emits markdown describing the
//! exact syntax the language accepts, so the code stays the single source of truth
//! and the generated documents cannot silently drift from it.
//!
//! It is dev-only: it lives outside `src/` and depends on `syn`, a dev-dependency,
//! so nothing here is compiled into the shipped `mlg` binary. It is reached from two
//! places, both of which share this module:
//!
//! * `cargo run --example gen_syntax_docs` rewrites the generated documents.
//! * `cargo test --test syntax_docs` fails when they are stale.
//!
//! Extraction is deliberately strict. Anything that cannot be derived from the code
//! is an error, never a guess, so a generated document is either faithful or absent.

pub mod formulation;
pub mod lalrpop;
pub mod model;
pub mod render;
pub mod rust_util;
pub mod structural;

use std::path::{Path, PathBuf};

use model::GeneratedFile;

/// The repository root, derived from this crate's manifest directory.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read(relative: &str) -> Result<String, String> {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Reads every implementation source and renders the generated documents.
pub fn generate() -> Result<Vec<GeneratedFile>, String> {
    let structural = structural::extract(
        &read("src/frontend/structural/ast.rs")?,
        &read("src/frontend/structural/parser.rs")?,
    )?;
    let formulation = formulation::extract(
        &read("src/frontend/formulation/ast.rs")?,
        &read("src/frontend/formulation/parser.rs")?,
    )?;
    let grammar = lalrpop::extract(&read("src/frontend/formulation/grammar.lalrpop")?)?;

    Ok(vec![
        GeneratedFile {
            path: "docs/generated/README.md".to_owned(),
            contents: render::readme(),
        },
        GeneratedFile {
            path: "docs/generated/structural_syntax.md".to_owned(),
            contents: render::structural(&structural),
        },
        GeneratedFile {
            path: "docs/generated/formulation_syntax.md".to_owned(),
            contents: render::formulation(&formulation, &grammar),
        },
    ])
}
