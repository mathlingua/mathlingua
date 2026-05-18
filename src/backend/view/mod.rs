//! Backend support for building the JSON payload consumed by the web viewer.
//!
//! The public surface re-exports the serialized view model and the collection
//! builder.  Rendering internals stay private to keep CLI/web integration
//! decoupled from MathLingua's parsed AST details.

/// Builds a serialized view model from parsed MathLingua collection files.
pub use builder::build_collection_view;
/// Serialized view-model types emitted to the web viewer.
pub use model::{ArgumentView, CollectionView, FileView, GroupView, SectionView};

mod builder;
mod model;
mod render;
