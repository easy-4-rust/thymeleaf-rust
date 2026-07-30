//! 模板解析器使用的资源抽象与具体资源实现。

mod embedded_template_resource;
mod file_template_resource;
mod i_template_resource;
mod string_template_resource;
mod template_resource_reader;
mod template_resource_utils;
mod url_template_resource;
mod web_application_template_resource;

pub use embedded_template_resource::EmbeddedTemplateResource;
pub use file_template_resource::FileTemplateResource;
pub use i_template_resource::{ITemplateResource, TemplateResourceError};
pub use string_template_resource::StringTemplateResource;
pub use url_template_resource::{UrlResourceConnectionHandler, UrlTemplateResource};
pub use web_application_template_resource::WebApplicationTemplateResource;
