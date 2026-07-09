use crate::backend::definition::load_collection_files;
use crate::backend::semantic::{plan_rename, prepare_rename as prepare_rename_in_files};
use std::path::Path;

pub use crate::backend::semantic::{
    RenameEditPlan as RenameEdit, RenameError, RenamePreparation, RenameSpan,
};

/// If `offset` lies on the command signature of a top-level item's heading in
/// `target` (whose current buffer is `target_source`), report the signature's
/// span and text so the editor can seed a rename. The collection rooted at (or
/// above) `root` supplies the target's headings; other files are not needed.
///
/// The lookup is read-only and reflects unsaved edits to `target`.
pub fn prepare_rename(
    root: &Path,
    target: &Path,
    target_source: &str,
    offset: usize,
) -> Option<RenamePreparation> {
    let (files, target_index) = load_collection_files(root, target, target_source);
    prepare_rename_in_files(&files[target_index], offset)
}

/// Compute the edits that rename the command whose heading is under `offset` in
/// `target` to `new_name`, touching the heading and every use of the command in
/// every `.mlg` file of the collection rooted at (or above) `root`.
///
/// `target_source` is the target's current buffer, so the plan reflects unsaved
/// edits. An `Err` explains why the rename was refused (see [`RenameError`]).
pub fn resolve_rename(
    root: &Path,
    target: &Path,
    target_source: &str,
    offset: usize,
    new_name: &str,
) -> Result<Vec<RenameEdit>, RenameError> {
    let (files, target_index) = load_collection_files(root, target, target_source);
    plan_rename(&files, &files[target_index], offset, new_name)
}
