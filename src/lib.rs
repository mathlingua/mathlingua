pub use mlg::{
    CheckResult, InitResult, LspResult, VersionResult, ViewResult, check, check_diagnostics_report,
    check_diagnostics_schema, init, lsp, version, view,
};
pub mod backend;
pub mod cli;
pub mod events;
pub mod frontend;
mod mlg;
