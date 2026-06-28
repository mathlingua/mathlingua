pub use builder::build_collection_view;
pub use model::{
    ArgumentView, CollectionView, DirectoryView, FileView, GroupView, PageView, SectionView,
};

mod builder;
mod model;
mod render;
