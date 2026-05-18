use crate::frontend::formulation::ast::{
    AuthorHeader, CommandHeader, Expression, ExpressionAlias, FormOrDeclaration,
    IsOrRefinedStatementSpec, IsOrSpec, IsViaStatement, LabelHeader, ResourceHeader,
    SpecOperatorAlias, WritingAlias,
};

include!("ast/repeated.rs");
include!("ast/sections.rs");
include!("ast/items.rs");
include!("ast/definition_groups.rs");
include!("ast/support_groups.rs");
include!("ast/metadata_resource_groups.rs");
include!("ast/clause_groups.rs");
