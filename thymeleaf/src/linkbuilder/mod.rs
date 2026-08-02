//! 模板链接构建器合同。

mod abstract_link_builder;
mod i_link_builder;
mod standard_link_builder;

pub use abstract_link_builder::AbstractLinkBuilder;
pub use i_link_builder::{ILinkBuilder, LinkBuilderResult};
pub use standard_link_builder::StandardLinkBuilder;
