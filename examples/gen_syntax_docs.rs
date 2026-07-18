//! Internal tool: regenerates the syntax reference under `docs/generated/`.
//!
//! ```sh
//! cargo run --example gen_syntax_docs            # rewrite the generated documents
//! cargo run --example gen_syntax_docs -- --check # exit non-zero if they are stale
//! ```
//!
//! This is an example rather than a `[[bin]]` so that it may use `syn`, a
//! dev-dependency, and therefore never enters the shipped `mlg` binary's dependency
//! graph. The same generator backs `tests/syntax_docs.rs`.

use std::process::ExitCode;

#[path = "../tools/syntax_docs/mod.rs"]
mod syntax_docs;

fn main() -> ExitCode {
    let check = std::env::args().skip(1).any(|arg| arg == "--check");

    let files = match syntax_docs::generate() {
        Ok(files) => files,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    let root = syntax_docs::repo_root();
    let mut stale = Vec::new();
    let mut wrote = Vec::new();

    for file in &files {
        let path = root.join(&file.path);
        let current = std::fs::read_to_string(&path).ok();

        if current.as_deref() == Some(file.contents.as_str()) {
            continue;
        }

        if check {
            stale.push(file.path.clone());
            continue;
        }

        if let Some(parent) = path.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            eprintln!("error: {}: {error}", parent.display());
            return ExitCode::FAILURE;
        }
        if let Err(error) = std::fs::write(&path, &file.contents) {
            eprintln!("error: {}: {error}", path.display());
            return ExitCode::FAILURE;
        }
        wrote.push(file.path.clone());
    }

    if check {
        if stale.is_empty() {
            println!("{} generated documents are up to date", files.len());
            return ExitCode::SUCCESS;
        }
        eprintln!("These generated documents are stale:");
        for path in &stale {
            eprintln!("  {path}");
        }
        eprintln!("\nRun `cargo run --example gen_syntax_docs` to update them.");
        return ExitCode::FAILURE;
    }

    if wrote.is_empty() {
        println!("{} generated documents already up to date", files.len());
    } else {
        for path in &wrote {
            println!("wrote {path}");
        }
    }
    ExitCode::SUCCESS
}
