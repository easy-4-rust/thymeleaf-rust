use std::sync::Arc;

use crate::TemplateMode;
use crate::cdatasection::{ICDATASectionProcessor, ICDATASectionStructureHandler};
use crate::comment::{ICommentProcessor, ICommentStructureHandler};
use crate::context::ITemplateContext;
use crate::doctype::{IDocTypeProcessor, IDocTypeStructureHandler};
use crate::element::{
    IElementModelProcessor, IElementModelStructureHandler, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use crate::exceptions::{ConfigurationException, TemplateEngineException};
use crate::model::{
    ICDATASection, IComment, IDocType, IModel, IProcessableElementTag, IProcessingInstruction,
    ITemplateEnd, ITemplateStart, IText, IXMLDeclaration,
};
use crate::postprocessor::IPostProcessor;
use crate::preprocessor::IPreProcessor;
use crate::processinginstruction::{
    IProcessingInstructionProcessor, IProcessingInstructionStructureHandler,
};
use crate::processor::IProcessor;
use crate::templateboundaries::{
    ITemplateBoundariesProcessor, ITemplateBoundariesStructureHandler,
};
use crate::text::{ITextProcessor, ITextStructureHandler};
use crate::xmldeclaration::{IXMLDeclarationProcessor, IXMLDeclarationStructureHandler};

/// 为配置中的 Processor 附加方言 precedence，并保留其具体处理能力。
///
/// 对应 Java: `org.thymeleaf.util.ProcessorConfigurationUtils`。Java 的内部 wrapper
/// 层次在本文件中保持一一可追踪；Rust 使用共享 `Arc` 保存被包装对象身份。
pub struct ProcessorConfigurationUtils;

impl ProcessorConfigurationUtils {
    /// 去除元素 Processor 的方言包装器。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils#unwrap(IElementProcessor)`。
    #[must_use]
    pub fn unwrap_element(processor: &dyn IElementProcessor) -> &dyn IElementProcessor {
        processor
            .get_wrapped_processor()
            .and_then(IProcessor::as_element_processor)
            .unwrap_or(processor)
    }

    /// 去除 CDATA Processor 的方言包装器。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils#unwrap(ICDATASectionProcessor)`。
    #[must_use]
    pub fn unwrap_cdata_section(
        processor: &dyn ICDATASectionProcessor,
    ) -> &dyn ICDATASectionProcessor {
        processor
            .get_wrapped_processor()
            .and_then(IProcessor::as_cdata_section_processor)
            .unwrap_or(processor)
    }

    /// 去除 Comment Processor 的方言包装器。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils#unwrap(ICommentProcessor)`。
    #[must_use]
    pub fn unwrap_comment(processor: &dyn ICommentProcessor) -> &dyn ICommentProcessor {
        processor
            .get_wrapped_processor()
            .and_then(IProcessor::as_comment_processor)
            .unwrap_or(processor)
    }

    /// 去除 DOCTYPE Processor 的方言包装器。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils#unwrap(IDocTypeProcessor)`。
    #[must_use]
    pub fn unwrap_doc_type(processor: &dyn IDocTypeProcessor) -> &dyn IDocTypeProcessor {
        processor
            .get_wrapped_processor()
            .and_then(IProcessor::as_doc_type_processor)
            .unwrap_or(processor)
    }

    /// 去除 ProcessingInstruction Processor 的方言包装器。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils#unwrap(IProcessingInstructionProcessor)`。
    #[must_use]
    pub fn unwrap_processing_instruction(
        processor: &dyn IProcessingInstructionProcessor,
    ) -> &dyn IProcessingInstructionProcessor {
        processor
            .get_wrapped_processor()
            .and_then(IProcessor::as_processing_instruction_processor)
            .unwrap_or(processor)
    }

    /// 去除 TemplateBoundaries Processor 的方言包装器。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils#unwrap(ITemplateBoundariesProcessor)`。
    #[must_use]
    pub fn unwrap_template_boundaries(
        processor: &dyn ITemplateBoundariesProcessor,
    ) -> &dyn ITemplateBoundariesProcessor {
        processor
            .get_wrapped_processor()
            .and_then(IProcessor::as_template_boundaries_processor)
            .unwrap_or(processor)
    }

    /// 去除 Text Processor 的方言包装器。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils#unwrap(ITextProcessor)`。
    #[must_use]
    pub fn unwrap_text(processor: &dyn ITextProcessor) -> &dyn ITextProcessor {
        processor
            .get_wrapped_processor()
            .and_then(IProcessor::as_text_processor)
            .unwrap_or(processor)
    }

    /// 去除 XMLDeclaration Processor 的方言包装器。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils#unwrap(IXMLDeclarationProcessor)`。
    #[must_use]
    pub fn unwrap_xml_declaration(
        processor: &dyn IXMLDeclarationProcessor,
    ) -> &dyn IXMLDeclarationProcessor {
        processor
            .get_wrapped_processor()
            .and_then(IProcessor::as_xml_declaration_processor)
            .unwrap_or(processor)
    }

    /// 去除 PreProcessor 的方言包装器。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils#unwrap(IPreProcessor)`。
    #[must_use]
    pub fn unwrap_pre_processor(processor: &dyn IPreProcessor) -> &dyn IPreProcessor {
        processor.get_wrapped_pre_processor().unwrap_or(processor)
    }

    /// 去除 PostProcessor 的方言包装器。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils#unwrap(IPostProcessor)`。
    #[must_use]
    pub fn unwrap_post_processor(processor: &dyn IPostProcessor) -> &dyn IPostProcessor {
        processor.get_wrapped_post_processor().unwrap_or(processor)
    }

    /// 包装元素 Processor，并按 Tag/Model 能力创建对应 Java wrapper。
    pub fn wrap_element(
        processor: Arc<dyn IProcessor>,
        dialect_precedence: i32,
    ) -> Result<Arc<dyn IElementProcessor>, ConfigurationException> {
        let element = processor.as_element_processor().ok_or_else(|| {
            configuration_error(format!(
                "Unknown element processor interface implemented by {}",
                processor.java_class_name()
            ))
        })?;
        if element.as_element_tag_processor().is_some() {
            return Ok(Arc::new(ElementTagProcessorWrapper::new(
                processor,
                dialect_precedence,
            )));
        }
        if element.as_element_model_processor().is_some() {
            return Ok(Arc::new(ElementModelProcessorWrapper::new(
                processor,
                dialect_precedence,
            )));
        }
        Err(configuration_error(format!(
            "Unknown element processor interface implemented by {}",
            processor.java_class_name()
        )))
    }

    /// 包装 CDATA Processor。
    pub fn wrap_cdata_section(
        processor: Arc<dyn IProcessor>,
        dialect_precedence: i32,
    ) -> Result<Arc<dyn ICDATASectionProcessor>, ConfigurationException> {
        require_capability(
            processor.as_cdata_section_processor().is_some(),
            &processor,
            "CDATA section",
        )?;
        Ok(Arc::new(CDATASectionProcessorWrapper::new(
            processor,
            dialect_precedence,
        )))
    }

    /// 包装 Comment Processor。
    pub fn wrap_comment(
        processor: Arc<dyn IProcessor>,
        dialect_precedence: i32,
    ) -> Result<Arc<dyn ICommentProcessor>, ConfigurationException> {
        require_capability(
            processor.as_comment_processor().is_some(),
            &processor,
            "comment",
        )?;
        Ok(Arc::new(CommentProcessorWrapper::new(
            processor,
            dialect_precedence,
        )))
    }

    /// 包装 DOCTYPE Processor。
    pub fn wrap_doc_type(
        processor: Arc<dyn IProcessor>,
        dialect_precedence: i32,
    ) -> Result<Arc<dyn IDocTypeProcessor>, ConfigurationException> {
        require_capability(
            processor.as_doc_type_processor().is_some(),
            &processor,
            "DOCTYPE",
        )?;
        Ok(Arc::new(DocTypeProcessorWrapper::new(
            processor,
            dialect_precedence,
        )))
    }

    /// 包装 ProcessingInstruction Processor。
    pub fn wrap_processing_instruction(
        processor: Arc<dyn IProcessor>,
        dialect_precedence: i32,
    ) -> Result<Arc<dyn IProcessingInstructionProcessor>, ConfigurationException> {
        require_capability(
            processor.as_processing_instruction_processor().is_some(),
            &processor,
            "processing instruction",
        )?;
        Ok(Arc::new(ProcessingInstructionProcessorWrapper::new(
            processor,
            dialect_precedence,
        )))
    }

    /// 包装 TemplateBoundaries Processor。
    pub fn wrap_template_boundaries(
        processor: Arc<dyn IProcessor>,
        dialect_precedence: i32,
    ) -> Result<Arc<dyn ITemplateBoundariesProcessor>, ConfigurationException> {
        require_capability(
            processor.as_template_boundaries_processor().is_some(),
            &processor,
            "template boundaries",
        )?;
        Ok(Arc::new(TemplateBoundariesProcessorWrapper::new(
            processor,
            dialect_precedence,
        )))
    }

    /// 包装 Text Processor。
    pub fn wrap_text(
        processor: Arc<dyn IProcessor>,
        dialect_precedence: i32,
    ) -> Result<Arc<dyn ITextProcessor>, ConfigurationException> {
        require_capability(processor.as_text_processor().is_some(), &processor, "text")?;
        Ok(Arc::new(TextProcessorWrapper::new(
            processor,
            dialect_precedence,
        )))
    }

    /// 包装 XMLDeclaration Processor。
    pub fn wrap_xml_declaration(
        processor: Arc<dyn IProcessor>,
        dialect_precedence: i32,
    ) -> Result<Arc<dyn IXMLDeclarationProcessor>, ConfigurationException> {
        require_capability(
            processor.as_xml_declaration_processor().is_some(),
            &processor,
            "XML declaration",
        )?;
        Ok(Arc::new(XMLDeclarationProcessorWrapper::new(
            processor,
            dialect_precedence,
        )))
    }

    /// 为 PreProcessor 附加方言级 precedence 并保留原对象身份。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils#wrap(IPreProcessor,IProcessorDialect)`。
    pub fn wrap_pre_processor(
        pre_processor: Arc<dyn IPreProcessor>,
        dialect_precedence: i32,
    ) -> Arc<dyn IPreProcessor> {
        Arc::new(PreProcessorWrapper {
            pre_processor,
            dialect_precedence,
        })
    }

    /// 为 PostProcessor 附加方言级 precedence 并保留原对象身份。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils#wrap(IPostProcessor,IProcessorDialect)`。
    pub fn wrap_post_processor(
        post_processor: Arc<dyn IPostProcessor>,
        dialect_precedence: i32,
    ) -> Arc<dyn IPostProcessor> {
        Arc::new(PostProcessorWrapper {
            post_processor,
            dialect_precedence,
        })
    }
}

struct AbstractProcessorWrapper {
    processor: Arc<dyn IProcessor>,
    dialect_precedence: i32,
    processor_precedence: i32,
}

impl AbstractProcessorWrapper {
    fn new(processor: Arc<dyn IProcessor>, dialect_precedence: i32) -> Self {
        let processor_precedence = processor.get_precedence();
        Self {
            processor,
            dialect_precedence,
            processor_precedence,
        }
    }

    fn template_mode(&self) -> Option<TemplateMode> {
        self.processor.get_template_mode()
    }

    /// 返回包装前的 Processor 动态实例。
    ///
    /// 对应 Java: `AbstractProcessorWrapper#unwrap()`。
    fn unwrap(&self) -> &dyn IProcessor {
        self.processor.as_ref()
    }
}

macro_rules! implement_base_processor {
    ($wrapper:ty) => {
        impl IProcessor for $wrapper {
            fn is_attribute_definitions_aware(&self) -> bool {
                self.base.processor.is_attribute_definitions_aware()
            }

            fn set_attribute_definitions(
                &self,
                definitions: Arc<crate::engine::AttributeDefinitions>,
            ) {
                self.base.processor.set_attribute_definitions(definitions);
            }

            fn is_element_definitions_aware(&self) -> bool {
                self.base.processor.is_element_definitions_aware()
            }

            fn set_element_definitions(&self, definitions: Arc<crate::engine::ElementDefinitions>) {
                self.base.processor.set_element_definitions(definitions);
            }

            fn java_class_name(&self) -> &'static str {
                self.base.processor.java_class_name()
            }

            fn get_dialect_precedence(&self) -> Option<i32> {
                Some(self.base.dialect_precedence)
            }

            fn get_wrapped_processor(&self) -> Option<&dyn IProcessor> {
                Some(self.base.unwrap())
            }

            fn get_template_mode(&self) -> Option<TemplateMode> {
                self.base.template_mode()
            }

            fn get_precedence(&self) -> i32 {
                self.base.processor_precedence
            }
        }
    };
}

struct AbstractElementProcessorWrapper {
    base: AbstractProcessorWrapper,
}

impl AbstractElementProcessorWrapper {
    fn new(processor: Arc<dyn IProcessor>, dialect_precedence: i32) -> Self {
        Self {
            base: AbstractProcessorWrapper::new(processor, dialect_precedence),
        }
    }

    fn processor(&self) -> &dyn IElementProcessor {
        self.base
            .processor
            .as_element_processor()
            .expect("element wrapper capability was checked")
    }
}

struct ElementTagProcessorWrapper {
    base: AbstractElementProcessorWrapper,
}

impl ElementTagProcessorWrapper {
    fn new(processor: Arc<dyn IProcessor>, dialect_precedence: i32) -> Self {
        Self {
            base: AbstractElementProcessorWrapper::new(processor, dialect_precedence),
        }
    }
}

impl IProcessor for ElementTagProcessorWrapper {
    fn is_attribute_definitions_aware(&self) -> bool {
        self.base.base.processor.is_attribute_definitions_aware()
    }
    fn set_attribute_definitions(&self, definitions: Arc<crate::engine::AttributeDefinitions>) {
        self.base
            .base
            .processor
            .set_attribute_definitions(definitions);
    }
    fn is_element_definitions_aware(&self) -> bool {
        self.base.base.processor.is_element_definitions_aware()
    }
    fn set_element_definitions(&self, definitions: Arc<crate::engine::ElementDefinitions>) {
        self.base
            .base
            .processor
            .set_element_definitions(definitions);
    }
    fn as_element_processor(&self) -> Option<&dyn IElementProcessor> {
        Some(self)
    }
    fn java_class_name(&self) -> &'static str {
        self.base.base.processor.java_class_name()
    }
    fn get_dialect_precedence(&self) -> Option<i32> {
        Some(self.base.base.dialect_precedence)
    }
    fn get_wrapped_processor(&self) -> Option<&dyn IProcessor> {
        Some(self.base.base.processor.as_ref())
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.base.base.template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.base.base.processor_precedence
    }
}

impl IElementProcessor for ElementTagProcessorWrapper {
    fn as_element_tag_processor(&self) -> Option<&dyn IElementTagProcessor> {
        Some(self)
    }
    fn get_matching_element_name(&self) -> Option<&MatchingElementName> {
        self.base.processor().get_matching_element_name()
    }
    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
        self.base.processor().get_matching_attribute_name()
    }
}

impl IElementTagProcessor for ElementTagProcessorWrapper {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base
            .processor()
            .as_element_tag_processor()
            .expect("tag wrapper capability was checked")
            .process(context, tag, structure_handler)
    }
}

struct ElementModelProcessorWrapper {
    base: AbstractElementProcessorWrapper,
}

impl ElementModelProcessorWrapper {
    fn new(processor: Arc<dyn IProcessor>, dialect_precedence: i32) -> Self {
        Self {
            base: AbstractElementProcessorWrapper::new(processor, dialect_precedence),
        }
    }
}

impl IProcessor for ElementModelProcessorWrapper {
    fn is_attribute_definitions_aware(&self) -> bool {
        self.base.base.processor.is_attribute_definitions_aware()
    }
    fn set_attribute_definitions(&self, definitions: Arc<crate::engine::AttributeDefinitions>) {
        self.base
            .base
            .processor
            .set_attribute_definitions(definitions);
    }
    fn is_element_definitions_aware(&self) -> bool {
        self.base.base.processor.is_element_definitions_aware()
    }
    fn set_element_definitions(&self, definitions: Arc<crate::engine::ElementDefinitions>) {
        self.base
            .base
            .processor
            .set_element_definitions(definitions);
    }
    fn as_element_processor(&self) -> Option<&dyn IElementProcessor> {
        Some(self)
    }
    fn java_class_name(&self) -> &'static str {
        self.base.base.processor.java_class_name()
    }
    fn get_dialect_precedence(&self) -> Option<i32> {
        Some(self.base.base.dialect_precedence)
    }
    fn get_wrapped_processor(&self) -> Option<&dyn IProcessor> {
        Some(self.base.base.processor.as_ref())
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.base.base.template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.base.base.processor_precedence
    }
}

impl IElementProcessor for ElementModelProcessorWrapper {
    fn as_element_model_processor(&self) -> Option<&dyn IElementModelProcessor> {
        Some(self)
    }
    fn get_matching_element_name(&self) -> Option<&MatchingElementName> {
        self.base.processor().get_matching_element_name()
    }
    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
        self.base.processor().get_matching_attribute_name()
    }
}

impl IElementModelProcessor for ElementModelProcessorWrapper {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base
            .processor()
            .as_element_model_processor()
            .expect("model wrapper capability was checked")
            .process(context, model, structure_handler)
    }
}

macro_rules! define_event_wrapper {
    (
        $name:ident,
        $capability:ident,
        $trait_name:ident,
        $event:ty,
        $handler:ty
    ) => {
        struct $name {
            base: AbstractProcessorWrapper,
        }

        impl $name {
            fn new(processor: Arc<dyn IProcessor>, dialect_precedence: i32) -> Self {
                Self {
                    base: AbstractProcessorWrapper::new(processor, dialect_precedence),
                }
            }
        }

        implement_base_processor!($name);

        impl $trait_name for $name {
            fn process(
                &self,
                context: &dyn ITemplateContext,
                event: &$event,
                structure_handler: &mut $handler,
            ) -> Result<(), Box<dyn TemplateEngineException>> {
                self.base
                    .processor
                    .$capability()
                    .expect("event wrapper capability was checked")
                    .process(context, event, structure_handler)
            }
        }
    };
}

define_event_wrapper!(
    CDATASectionProcessorWrapper,
    as_cdata_section_processor,
    ICDATASectionProcessor,
    dyn ICDATASection,
    dyn ICDATASectionStructureHandler
);
define_event_wrapper!(
    CommentProcessorWrapper,
    as_comment_processor,
    ICommentProcessor,
    dyn IComment,
    dyn ICommentStructureHandler
);
define_event_wrapper!(
    DocTypeProcessorWrapper,
    as_doc_type_processor,
    IDocTypeProcessor,
    dyn IDocType,
    dyn IDocTypeStructureHandler
);
define_event_wrapper!(
    ProcessingInstructionProcessorWrapper,
    as_processing_instruction_processor,
    IProcessingInstructionProcessor,
    dyn IProcessingInstruction,
    dyn IProcessingInstructionStructureHandler
);
define_event_wrapper!(
    TextProcessorWrapper,
    as_text_processor,
    ITextProcessor,
    dyn IText,
    dyn ITextStructureHandler
);
define_event_wrapper!(
    XMLDeclarationProcessorWrapper,
    as_xml_declaration_processor,
    IXMLDeclarationProcessor,
    dyn IXMLDeclaration,
    dyn IXMLDeclarationStructureHandler
);

struct TemplateBoundariesProcessorWrapper {
    base: AbstractProcessorWrapper,
}

impl TemplateBoundariesProcessorWrapper {
    fn new(processor: Arc<dyn IProcessor>, dialect_precedence: i32) -> Self {
        Self {
            base: AbstractProcessorWrapper::new(processor, dialect_precedence),
        }
    }
}

implement_base_processor!(TemplateBoundariesProcessorWrapper);

impl ITemplateBoundariesProcessor for TemplateBoundariesProcessorWrapper {
    fn process_template_start(
        &self,
        context: &dyn ITemplateContext,
        template_start: &dyn ITemplateStart,
        structure_handler: &mut dyn ITemplateBoundariesStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base
            .processor
            .as_template_boundaries_processor()
            .expect("template boundaries capability was checked")
            .process_template_start(context, template_start, structure_handler)
    }

    fn process_template_end(
        &self,
        context: &dyn ITemplateContext,
        template_end: &dyn ITemplateEnd,
        structure_handler: &mut dyn ITemplateBoundariesStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base
            .processor
            .as_template_boundaries_processor()
            .expect("template boundaries capability was checked")
            .process_template_end(context, template_end, structure_handler)
    }
}

/// 附加方言 precedence 的 PreProcessor 委托包装器。
///
/// 对应 Java: `ProcessorConfigurationUtils.PreProcessorWrapper`。
struct PreProcessorWrapper {
    pre_processor: Arc<dyn IPreProcessor>,
    dialect_precedence: i32,
}

impl IPreProcessor for PreProcessorWrapper {
    fn get_dialect_precedence(&self) -> Option<i32> {
        Some(self.dialect_precedence)
    }

    fn get_wrapped_pre_processor(&self) -> Option<&dyn IPreProcessor> {
        Some(self.unwrap())
    }

    fn is_attribute_definitions_aware(&self) -> bool {
        self.pre_processor.is_attribute_definitions_aware()
    }

    fn set_attribute_definitions(
        &self,
        attribute_definitions: Arc<crate::engine::AttributeDefinitions>,
    ) {
        self.pre_processor
            .set_attribute_definitions(attribute_definitions);
    }

    fn is_element_definitions_aware(&self) -> bool {
        self.pre_processor.is_element_definitions_aware()
    }

    fn set_element_definitions(&self, element_definitions: Arc<crate::engine::ElementDefinitions>) {
        self.pre_processor
            .set_element_definitions(element_definitions);
    }

    fn get_template_mode(&self) -> TemplateMode {
        self.pre_processor.get_template_mode()
    }

    fn get_precedence(&self) -> i32 {
        self.pre_processor.get_precedence()
    }

    fn get_handler_factory(&self) -> crate::preprocessor::PreProcessorHandlerFactory {
        self.pre_processor.get_handler_factory()
    }

    fn get_handler_class_name(&self) -> &'static str {
        self.pre_processor.get_handler_class_name()
    }
}

impl PreProcessorWrapper {
    /// 返回包装前的 PreProcessor。
    ///
    /// 对应 Java: `PreProcessorWrapper#unwrap()`。
    fn unwrap(&self) -> &dyn IPreProcessor {
        self.pre_processor.as_ref()
    }
}

/// 附加方言 precedence 的 PostProcessor 委托包装器。
///
/// 对应 Java: `ProcessorConfigurationUtils.PostProcessorWrapper`。
struct PostProcessorWrapper {
    post_processor: Arc<dyn IPostProcessor>,
    dialect_precedence: i32,
}

impl IPostProcessor for PostProcessorWrapper {
    fn get_dialect_precedence(&self) -> Option<i32> {
        Some(self.dialect_precedence)
    }

    fn get_wrapped_post_processor(&self) -> Option<&dyn IPostProcessor> {
        Some(self.unwrap())
    }

    fn is_attribute_definitions_aware(&self) -> bool {
        self.post_processor.is_attribute_definitions_aware()
    }

    fn set_attribute_definitions(
        &self,
        attribute_definitions: Arc<crate::engine::AttributeDefinitions>,
    ) {
        self.post_processor
            .set_attribute_definitions(attribute_definitions);
    }

    fn is_element_definitions_aware(&self) -> bool {
        self.post_processor.is_element_definitions_aware()
    }

    fn set_element_definitions(&self, element_definitions: Arc<crate::engine::ElementDefinitions>) {
        self.post_processor
            .set_element_definitions(element_definitions);
    }

    fn get_template_mode(&self) -> TemplateMode {
        self.post_processor.get_template_mode()
    }

    fn get_precedence(&self) -> i32 {
        self.post_processor.get_precedence()
    }

    fn get_handler_factory(&self) -> crate::postprocessor::PostProcessorHandlerFactory {
        self.post_processor.get_handler_factory()
    }

    fn get_handler_class_name(&self) -> &'static str {
        self.post_processor.get_handler_class_name()
    }
}

impl PostProcessorWrapper {
    /// 返回包装前的 PostProcessor。
    ///
    /// 对应 Java: `PostProcessorWrapper#unwrap()`。
    fn unwrap(&self) -> &dyn IPostProcessor {
        self.post_processor.as_ref()
    }
}

fn require_capability(
    present: bool,
    processor: &Arc<dyn IProcessor>,
    kind: &str,
) -> Result<(), ConfigurationException> {
    if present {
        Ok(())
    } else {
        Err(configuration_error(format!(
            "Processor {} does not implement the required {kind} processor interface",
            processor.java_class_name()
        )))
    }
}

fn configuration_error(message: String) -> ConfigurationException {
    ConfigurationException::new(Some(message))
}
