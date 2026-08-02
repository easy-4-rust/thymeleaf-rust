use std::sync::Arc;

use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use crate::engine::AttributeName;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::IProcessableElementTag;
use crate::util::JavaString;

use super::{IProcessor, StandardAttributeCallback};

/// Standard 条件可见性 Processor 的组合式抽象实现。
///
/// 它刻意不预先执行表达式，使 `th:case` 在已有分支命中后能够完全跳过后续表达式。
/// 对应 Java: `org.thymeleaf.standard.processor.AbstractStandardConditionalVisibilityTagProcessor`。
pub struct AbstractStandardConditionalVisibilityTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl AbstractStandardConditionalVisibilityTagProcessor {
    /// 创建由 `is_visible` 决定元素是否保留的条件 Processor。
    ///
    /// 对应 Java 受保护构造器与抽象 `isVisible` 方法。
    pub fn new<F>(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
        attr_name: JavaString,
        precedence: i32,
        is_visible: F,
        processor_class_name: &'static str,
    ) -> Result<Self, TemplateProcessingException>
    where
        F: Fn(
                &dyn ITemplateContext,
                &dyn IProcessableElementTag,
                &AttributeName,
                Option<&JavaString>,
            ) -> Result<bool, Box<dyn TemplateEngineException>>
            + Send
            + Sync
            + 'static,
    {
        let is_visible = Arc::new(is_visible);
        let callback: StandardAttributeCallback = Box::new(
            move |context, tag, attribute_name, attribute_value, structure_handler| {
                if !(is_visible)(context, tag, attribute_name, attribute_value.as_ref())? {
                    structure_handler.remove_element();
                }
                Ok(())
            },
        );
        Ok(Self {
            processor: AbstractAttributeTagProcessor::new(
                Some(template_mode),
                dialect_prefix,
                None,
                false,
                Some(attr_name),
                true,
                precedence,
                true,
                processor_class_name,
                callback,
            )?,
        })
    }
}

impl IProcessor for AbstractStandardConditionalVisibilityTagProcessor {
    fn java_class_name(&self) -> &'static str {
        self.processor.java_class_name()
    }

    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.processor.get_template_mode()
    }

    fn get_precedence(&self) -> i32 {
        self.processor.get_precedence()
    }
}

impl IElementProcessor for AbstractStandardConditionalVisibilityTagProcessor {
    fn as_element_tag_processor(&self) -> Option<&dyn IElementTagProcessor> {
        Some(self)
    }

    fn get_matching_element_name(&self) -> Option<&MatchingElementName> {
        self.processor.get_matching_element_name()
    }

    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
        self.processor.get_matching_attribute_name()
    }
}

impl IElementTagProcessor for AbstractStandardConditionalVisibilityTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}
