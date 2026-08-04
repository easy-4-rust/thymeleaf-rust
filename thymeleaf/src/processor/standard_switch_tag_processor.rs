use std::sync::{Arc, Mutex};

use crate::TemplateMode;
use crate::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::{IStandardExpression, StandardExpressions, TemplateObject, TemplateValue};
use crate::model::IProcessableElementTag;
use crate::util::Utf16String;

use super::{IProcessor, StandardAttributeCallback, expression_processing_error};

/// 保存 `th:switch` 表达式及是否已有 case 命中的共享状态。
///
/// 对应 Java: `StandardSwitchTagProcessor.SwitchStructure`。
pub struct SwitchStructure {
    expression: Arc<dyn IStandardExpression>,
    executed: Mutex<bool>,
}

impl SwitchStructure {
    /// 创建尚未命中的 switch 状态。
    /// 对应 Java 语义：`StandardSwitchTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(expression: Arc<dyn IStandardExpression>) -> Self {
        Self {
            expression,
            executed: Mutex::new(false),
        }
    }
    /// 返回 switch 表达式共享身份。
    /// 对应 Java: `StandardSwitchTagProcessor#getExpression()`。
    pub fn get_expression(&self) -> Arc<dyn IStandardExpression> {
        Arc::clone(&self.expression)
    }
    /// 判断是否已有 case 命中。
    /// 对应 Java: `StandardSwitchTagProcessor#isExecuted()`。
    pub fn is_executed(&self) -> bool {
        *self
            .executed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    /// 设置 case 命中状态。
    /// 对应 Java: `StandardSwitchTagProcessor#setExecuted()`。
    pub fn set_executed(&self, executed: bool) {
        *self
            .executed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = executed;
    }
}

impl TemplateObject for SwitchStructure {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.standard.processor.StandardSwitchTagProcessor$SwitchStructure"
    }
    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str(self.java_class_name())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 解析 `th:switch` 并把 SwitchStructure 放入局部上下文的 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardSwitchTagProcessor`。
pub struct StandardSwitchTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl StandardSwitchTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 250;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "switch";
    /// 内部上下文变量名。
    pub const SWITCH_VARIABLE_NAME: &'static str = "%%SWITCH_EXPR%%";

    /// 创建 Processor。
    /// 对应 Java 语义：`StandardSwitchTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
    ) -> Result<Self, TemplateProcessingException> {
        let callback: StandardAttributeCallback = Box::new(
            |context, _tag, _attribute_name, attribute_value, structure_handler| {
                let parser =
                    StandardExpressions::get_expression_parser(context.get_configuration())
                        .map_err(|error| {
                            expression_processing_error(
                                "Could not obtain Standard Expression parser",
                                error,
                            )
                        })?;
                let expression = parser
                    .parse_expression(context, attribute_value.as_ref())
                    .map_err(|error| {
                        expression_processing_error("Could not parse switch expression", error)
                    })?;
                structure_handler.set_local_variable(
                    Utf16String::from_rust_str(Self::SWITCH_VARIABLE_NAME),
                    Some(Arc::new(TemplateValue::Object(Arc::new(
                        SwitchStructure::new(expression),
                    )))),
                );
                Ok(())
            },
        );
        Ok(Self {
            processor: AbstractAttributeTagProcessor::new(
                Some(template_mode),
                dialect_prefix,
                None,
                false,
                Some(Utf16String::from_rust_str(Self::ATTR_NAME)),
                true,
                Self::PRECEDENCE,
                true,
                "org.thymeleaf.standard.processor.StandardSwitchTagProcessor",
                callback,
            )?,
        })
    }
}

impl IProcessor for StandardSwitchTagProcessor {
    fn as_element_processor(&self) -> Option<&dyn IElementProcessor> {
        Some(self)
    }

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

impl IElementProcessor for StandardSwitchTagProcessor {
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

impl IElementTagProcessor for StandardSwitchTagProcessor {
    fn process(
        &self,
        context: &dyn crate::context::ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}
