pub use builder::{build_collection_view, build_single_group_view};
pub(crate) use builder::build_collection_view_with_cached_proto_groups;
pub use markdown::render_group_markdown;
pub use model::{
    ArgumentView, CollectionView, DirectoryView, FileView, GroupView, PageView, SectionView,
    TypeEntryView,
};
pub use render::{RenderRegistry, build_render_registry};

mod builder;
mod markdown;
mod model;
mod render;

