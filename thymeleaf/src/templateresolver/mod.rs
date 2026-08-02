//! 模板解析器链及其解析结果合同。

mod abstract_configurable_template_resolver;
mod abstract_template_resolver;
mod class_loader_template_resolver;
mod default_template_resolver;
mod file_template_resolver;
mod i_template_resolver;
mod string_template_resolver;
mod template_resolution;
mod template_resolution_error;
mod template_resolver_error;
mod url_template_resolver;
mod web_application_template_resolver;

pub use abstract_configurable_template_resolver::AbstractConfigurableTemplateResolver;
pub use abstract_template_resolver::AbstractTemplateResolver;
pub use class_loader_template_resolver::ClassLoaderTemplateResolver as EmbeddedTemplateResolver;
pub use class_loader_template_resolver::ClassLoaderTemplateResolver;
pub use default_template_resolver::DefaultTemplateResolver;
pub use file_template_resolver::FileTemplateResolver;
pub use i_template_resolver::ITemplateResolver;
pub use string_template_resolver::StringTemplateResolver;
pub use template_resolution::TemplateResolution;
pub use template_resolution_error::TemplateResolutionError;
pub use template_resolver_error::TemplateResolverError;
pub use url_template_resolver::UrlTemplateResolver;
pub use web_application_template_resolver::WebApplicationTemplateResolver;
