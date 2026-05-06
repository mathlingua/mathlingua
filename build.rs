// Cargo runs build scripts from the crate root. We need this one so LALRPOP
// generates the formulation parser from `src/frontend/formulation/grammar.lalrpop`
// before the crate compiles.
fn main() {
    lalrpop::process_root().expect("failed to generate parser");
}
