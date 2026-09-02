//! Thymeleaf 核心模板引擎。
//!
//! 本 crate 以 Thymeleaf 3.1.5 的可观察语义为迁移基线，并保持 Web 框架中立。

pub mod cache;
pub mod cdatasection;
pub mod comment;
mod configuration_printer_helper;
pub mod context;
pub mod decoupled;
pub mod dialect;
mod dialect_configuration;
mod dialect_set_configuration;
mod dialect_set_configuration_error;
pub mod doctype;
#[cfg(feature = "dtd-validation")]
pub mod dtd;
pub mod element;
pub mod engine;
mod engine_configuration;
pub mod exceptions;
mod execution_attribute;
pub mod expression;
mod i_engine_configuration;
mod i_template_engine;
mod i_throttled_template_processor;
pub mod inline;
pub mod linkbuilder;
/// HTML/XML 标记模板解析器。
pub mod markup;
pub mod messageresolver;
pub mod model;
pub mod postprocessor;
pub mod preprocessor;
pub mod processinginstruction;
pub mod processor;
pub mod raw;
pub mod reader;
pub mod serializer;
pub mod standard;
mod template_engine;
mod template_spec;
pub mod templateboundaries;
pub mod templatemode;
pub mod templateparser;
pub mod templateresolver;
pub mod templateresource;
/// Java 8 Time 对象创建、规范化与批量格式化设施。
pub mod temporal;
pub mod text;
mod thymeleaf;
pub mod util;
pub mod web;
pub mod xmldeclaration;

pub(crate) use configuration_printer_helper::ConfigurationPrinterHelper;
pub use context::{
    AbstractContext, AbstractEngineContext, Context, Contexts, EngineContext, IContext,
    IContextVariableNames, ILazyContextVariable, IdentifierSequences, IdentifierSequencesError,
    LazyContextVariable, RequestParameterValues, StandardEngineContextFactory, WebEngineContext,
};
pub use decoupled::{DecoupledInjectedAttribute, DecoupledInjectedAttributeError};
pub use dialect::{
    AbstractDialect, AbstractDialectError, AbstractProcessorDialect, ExecutionAttributeMap,
    IDialect, IExecutionAttributeDialect, IProcessorDialect,
};
pub use dialect_configuration::{DialectConfiguration, DialectConfigurationError};
pub use dialect_set_configuration::DialectSetConfiguration;
pub use dialect_set_configuration_error::DialectSetConfigurationError;
pub use element::{
    ElementProcessorSet, IElementProcessor, MatchingAttributeName, MatchingAttributeNameError,
    MatchingElementName, MatchingElementNameError, UnmodifiableElementProcessorSet,
};
pub use engine::{
    AbstractElementTag, AbstractProcessableElementTag, Attribute, AttributeDefinition,
    AttributeDefinitionError, AttributeDefinitionKind, AttributeDefinitionValue,
    AttributeDefinitions, AttributeDefinitionsError, AttributeName, AttributeNameError,
    AttributeNameKind, AttributeNameValue, AttributeNames, AttributeNamesError, Attributes,
    AttributesError, CDATASection, CloseElementTag, Comment, DataDrivenTemplateIterator,
    DataDrivenTemplateIteratorError, DataDrivenTemplateSignal, DocType, DocTypeError,
    ElementDefinition, ElementDefinitionError, ElementDefinitionKind, ElementDefinitionValue,
    ElementDefinitions, ElementDefinitionsError, ElementName, ElementNameError, ElementNameKind,
    ElementNameValue, ElementNames, ElementNamesError, HTMLAttributeDefinition, HTMLAttributeName,
    HTMLElementDefinition, HTMLElementName, IAttributeDefinitionsAware, IElementDefinitionsAware,
    ISSEThrottledTemplateWriterControl, IterationStatusVar, IterationStatusVarError,
    OpenElementTag, ProcessingInstruction, StandaloneElementTag, StandaloneElementTagError,
    TemplateData, TemplateEnd, TemplateManager, TemplateStart, Text, TextAttributeDefinition,
    TextAttributeName, TextElementDefinition, TextElementName, XMLAttributeDefinition,
    XMLAttributeName, XMLDeclaration, XMLElementDefinition, XMLElementName,
};
pub use engine_configuration::EngineConfiguration;
pub use exceptions::{
    AlreadyInitializedException, CacheConfigurationException, ConfigurationException,
    ParserInitializationException, TemplateAssertionException, TemplateEngineException,
    TemplateInputException, TemplateOutputException, TemplateProcessingException,
};
pub use execution_attribute::ExecutionAttributeValue;
pub use expression::{
    IExpressionObjects, NoOpOgnlRuntime, OgnlRuntime, OgnlRuntimeError, TemplateObject,
    TemplateValue,
};
pub use i_engine_configuration::IEngineConfiguration;
pub use i_template_engine::{ITemplateEngine, TemplateEngineResult};
pub use i_throttled_template_processor::{
    IThrottledTemplateProcessor, ThrottledTemplateResult, ThrottledTemplateStatus,
};
pub use model::{
    AbstractModelVisitor, IAttribute, ICDATASection, ICloseElementTag, IComment, IDocType,
    IElementTag, IModelVisitor, IOpenElementTag, IProcessableElementTag, IProcessingInstruction,
    IStandaloneElementTag, ITemplateEnd, ITemplateEvent, ITemplateStart, IText, IXMLDeclaration,
};
pub use processor::{AbstractProcessor, IProcessor, ProcessorSet};
pub use raw::{
    IRawHandler, RawParseCause, RawParseException, RawParser, RawParserError, RawReader,
    RawStringReader, RawTemplateParser,
};
pub use serializer::{IStandardCSSSerializer, IStandardJavaScriptSerializer};
pub use standard::StandardDialect;
pub use template_engine::TemplateEngine;
pub use template_spec::{
    TemplateResolutionAttributeValue, TemplateResolutionAttributes, TemplateSelectorSet,
    TemplateSpec, TemplateSpecError,
};
pub use templatemode::{TemplateMode, TemplateModeParseError};
pub use templateparser::{ITemplateParser, TemplateParserError};
pub use templateresolver::{
    AbstractConfigurableTemplateResolver, AbstractTemplateResolver, ClassLoaderTemplateResolver,
    DefaultTemplateResolver, EmbeddedTemplateResolver, FileTemplateResolver, ITemplateResolver,
    StringTemplateResolver, TemplateResolution, TemplateResolutionError, TemplateResolverError,
    UrlTemplateResolver, WebApplicationTemplateResolver,
};
pub use templateresource::{
    ClassLoaderTemplateResource, EmbeddedTemplateResource, FileTemplateResource, ITemplateResource,
    StringTemplateResource, TemplateResourceError, UrlResourceConnectionHandler,
    UrlTemplateResource, WebApplicationTemplateResource,
};
pub use thymeleaf::Thymeleaf;
pub use util::{
    AbstractLazyCharSequence, AggregateCharSequence, AggregateCharSequenceError,
    AggregateComponent, IWritableCharSequence, LazyCharSequenceResolver,
    LazyProcessingCharSequence, Locale, TemplateWriter,
};
