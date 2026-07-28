//! Thymeleaf 核心模板引擎。
//!
//! 本 crate 以 Thymeleaf 3.1.5 的可观察语义为迁移基线，并保持 Web 框架中立。

pub mod cache;
pub mod dialect;
mod dialect_configuration;
pub mod exceptions;
mod template_spec;
pub mod templatemode;
pub mod templateresolver;
pub mod templateresource;
mod thymeleaf;
pub mod util;

pub use dialect::{AbstractDialect, AbstractDialectError, IDialect};
pub use dialect_configuration::{DialectConfiguration, DialectConfigurationError};
pub use exceptions::{
    AlreadyInitializedException, CacheConfigurationException, ConfigurationException,
    ParserInitializationException, TemplateAssertionException, TemplateEngineException,
    TemplateInputException, TemplateOutputException, TemplateProcessingException,
};
pub use template_spec::{
    TemplateResolutionAttributeValue, TemplateResolutionAttributes, TemplateSelectorSet,
    TemplateSpec, TemplateSpecError,
};
pub use templatemode::{TemplateMode, TemplateModeParseError};
pub use templateresolver::{TemplateResolution, TemplateResolutionError};
pub use templateresource::{
    FileTemplateResource, ITemplateResource, StringTemplateResource, TemplateResourceError,
    UrlTemplateResource,
};
pub use thymeleaf::Thymeleaf;
