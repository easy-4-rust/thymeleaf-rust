use std::sync::Arc;

use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::inline::{IInliner, StandardInlineMode};
use crate::model::IProcessableElementTag;
use crate::util::JavaString;

use super::{IProcessor, StandardAttributeCallback};

/// `th:inline` 字面量模式切换的组合式抽象 Processor。
///
/// 属性值不会作为表达式执行，以保证解析期预处理仍可缓存。
/// 对应 Java: `org.thymeleaf.standard.processor.AbstractStandardTextInlineSettingTagProcessor`。
pub struct AbstractStandardTextInlineSettingTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl AbstractStandardTextInlineSettingTagProcessor {
    /// 创建使用 `get_inliner` 映射字面量内联模式的 Processor。
    /// 对应 Java 语义：`AbstractStandardTextInlineSettingTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new<F>(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
        attr_name: JavaString,
        precedence: i32,
        get_inliner: F,
        processor_class_name: &'static str,
    ) -> Result<Self, TemplateProcessingException>
    where
        F: Fn(
                &dyn ITemplateContext,
                StandardInlineMode,
            ) -> Result<Arc<dyn IInliner>, Box<dyn TemplateEngineException>>
            + Send
            + Sync
            + 'static,
    {
        let get_inliner = Arc::new(get_inliner);
        let callback: StandardAttributeCallback = Box::new(
            move |context, _tag, _attribute_name, attribute_value, structure_handler| {
                let inline_mode =
                    StandardInlineMode::parse(attribute_value.as_ref()).map_err(|error| {
                        Box::new(TemplateProcessingException::with_cause(
                            Some(error.to_string()),
                            error,
                        )) as Box<dyn TemplateEngineException>
                    })?;
                structure_handler.set_inliner(Some((get_inliner)(context, inline_mode)?));
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

impl IProcessor for AbstractStandardTextInlineSettingTagProcessor {
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

impl IElementProcessor for AbstractStandardTextInlineSettingTagProcessor {
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

impl IElementTagProcessor for AbstractStandardTextInlineSettingTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}
