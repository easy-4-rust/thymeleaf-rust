//! 模板解析器使用的资源抽象与具体资源实现。

mod i_template_resource;
mod string_template_resource;

pub use i_template_resource::{ITemplateResource, TemplateResourceError};
pub use string_template_resource::StringTemplateResource;
