pub use builder::build_collection_view;
pub(crate) use builder::build_collection_view_with_type_info;
pub use model::{
    ArgumentView, CollectionView, DirectoryView, FileView, GroupView, PageView, SectionView,
    TypeEntryView,
};

mod builder;
mod model;
mod render;
