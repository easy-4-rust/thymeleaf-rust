//! 模板解析器使用的资源抽象与具体资源实现。

mod class_loader_template_resource;
mod file_template_resource;
mod i_template_resource;
mod java_charset_decoder;
mod string_template_resource;
mod template_resource_error;
mod template_resource_reader;
mod template_resource_utils;
mod transcoding_reader;
mod url_template_resource;
mod web_application_template_resource;

pub use ClassLoaderTemplateResource as EmbeddedTemplateResource;
pub use class_loader_template_resource::ClassLoaderTemplateResource;
pub use file_template_resource::FileTemplateResource;
pub use i_template_resource::ITemplateResource;
pub use string_template_resource::StringTemplateResource;
pub use template_resource_error::TemplateResourceError;
pub use url_template_resource::{UrlResourceConnectionHandler, UrlTemplateResource};
pub use web_application_template_resource::WebApplicationTemplateResource;
