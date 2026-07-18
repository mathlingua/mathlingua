//! Fails when the generated syntax reference no longer matches the code.
//!
//! This is what keeps `docs/generated/` honest: any change to the structural AST or
//! parser, the formulation AST, or the LALRPOP grammar that alters the accepted syntax
//! makes this test fail until the documents are regenerated.
//!
//! Run `cargo run --example gen_syntax_docs` to update them.

#[path = "../tools/syntax_docs/mod.rs"]
mod syntax_docs;

#[test]
fn generated_syntax_docs_are_up_to_date() {
    let files = match syntax_docs::generate() {
        Ok(files) => files,
        Err(error) => panic!(
            "the syntax-doc generator could not read the implementation sources: {error}\n\n\
             This usually means the AST or parser changed shape in a way the generator does \
             not understand yet. Update `tools/syntax_docs/` to match."
        ),
    };

    let root = syntax_docs::repo_root();
    let mut stale = Vec::new();
    for file in &files {
        let current = std::fs::read_to_string(root.join(&file.path)).unwrap_or_default();
        if current != file.contents {
            stale.push(file.path.clone());
        }
    }

    assert!(
        stale.is_empty(),
        "the generated syntax reference is out of date:\n{}\n\n\
         The code is the source of truth, so regenerate the documents:\n\
         \tcargo run --example gen_syntax_docs",
        stale
            .iter()
            .map(|path| format!("  {path}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
