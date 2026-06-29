pub use mlg::{
    CheckResult, DebugResult, InitResult, LspResult, VersionResult, ViewResult, WhteRbtObjResult,
    check, check_diagnostics_report, check_diagnostics_schema, debug, init, lsp, version, view,
    whte_rbt_obj,
};
pub mod backend;
pub mod cli;
pub mod events;
pub mod frontend;
mod mlg;
