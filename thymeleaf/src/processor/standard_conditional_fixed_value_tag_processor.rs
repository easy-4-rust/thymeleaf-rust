use crate::TemplateMode;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::TemplateValue;
use crate::util::{EvaluationUtils, JavaEvaluationValue, Utf16String};

use super::{
    AbstractStandardExpressionAttributeTagProcessor, delegate_standard_element_tag_processor,
};

/// 按布尔表达式添加或删除 HTML 固定值条件属性的 Processor。
///
/// 真值输出 `name="name"`，假值删除属性。对应 Java:
/// `org.thymeleaf.standard.processor.StandardConditionalFixedValueTagProcessor`。
pub struct StandardConditionalFixedValueTagProcessor {
    processor: AbstractStandardExpressionAttributeTagProcessor,
}

impl StandardConditionalFixedValueTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;
    /// HTML 固定值条件属性全集。
    pub const ATTR_NAMES: &'static [&'static str] = &[
        "async",
        "autofocus",
        "autoplay",
        "checked",
        "controls",
        "declare",
        "default",
        "defer",
        "disabled",
        "formnovalidate",
        "hidden",
        "ismap",
        "loop",
        "multiple",
        "novalidate",
        "nowrap",
        "open",
        "pubdate",
        "readonly",
        "required",
        "reversed",
        "selected",
        "scoped",
        "seamless",
    ];

    /// 创建指定条件属性 Processor。
    /// 对应 Java 语义：`StandardConditionalFixedValueTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        dialect_prefix: Option<Utf16String>,
        attr_name: Utf16String,
    ) -> Result<Self, TemplateProcessingException> {
        let target_name = attr_name.clone();
        Ok(Self {
            processor: AbstractStandardExpressionAttributeTagProcessor::with_restricted_execution(
                TemplateMode::HTML,
                dialect_prefix,
                attr_name,
                Self::PRECEDENCE,
                true,
                false,
                move |_context,
                      _tag,
                      _attribute_name,
                      _attribute_value,
                      expression_result,
                      structure_handler| {
                    let value = expression_result.as_deref().map_or(
                        JavaEvaluationValue::Null,
                        TemplateValue::to_evaluation_value,
                    );
                    if EvaluationUtils::evaluate_as_boolean(&value).map_err(|error| {
                        Box::new(TemplateProcessingException::with_cause(
                            Some("Could not evaluate fixed-value conditional attribute".to_owned()),
                            error,
                        )) as Box<dyn TemplateEngineException>
                    })? {
                        structure_handler.set_attribute(
                            target_name.clone(),
                            Some(target_name.clone()),
                            None,
                        );
                    } else {
                        structure_handler.remove_attribute(target_name.clone());
                    }
                    Ok(())
                },
                "org.thymeleaf.standard.processor.StandardConditionalFixedValueTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardConditionalFixedValueTagProcessor, processor);
