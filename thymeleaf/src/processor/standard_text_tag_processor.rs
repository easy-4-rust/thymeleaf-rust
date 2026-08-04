use std::sync::Arc;

use crate::TemplateMode;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::TemplateValue;
use crate::util::{
    CharSequenceValue, LazyEscapingCharSequence, Utf16String, escape_text_immediately,
};

use super::{
    AbstractStandardExpressionAttributeTagProcessor, delegate_standard_element_tag_processor,
};

/// 执行 `th:text` 表达式并按当前模板模式转义正文的 Processor。
///
/// 短 HTML/XML/TEXT 文本立即转义，长文本及 JavaScript/CSS 延迟写出，从而保留
/// Java 实现的内存与 Writer 行为。对应 Java:
/// `org.thymeleaf.standard.processor.StandardTextTagProcessor`。
pub struct StandardTextTagProcessor {
    processor: AbstractStandardExpressionAttributeTagProcessor,
}

impl StandardTextTagProcessor {
    /// Java Processor precedence。
    pub const PRECEDENCE: i32 = 1300;
    /// Standard 属性本地名称。
    pub const ATTR_NAME: &'static str = "text";

    /// 创建指定模板模式和方言前缀的 `th:text` Processor。
    /// 对应 Java 语义：`StandardTextTagProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<Utf16String>,
    ) -> Result<Self, TemplateProcessingException> {
        Ok(Self {
            processor: AbstractStandardExpressionAttributeTagProcessor::with_restricted_execution(
                template_mode,
                dialect_prefix,
                Utf16String::from_rust_str(Self::ATTR_NAME),
                Self::PRECEDENCE,
                true,
                template_mode == TemplateMode::TEXT,
                move |context,
                      _tag,
                      _attribute_name,
                      _attribute_value,
                      expression_result,
                      structure_handler| {
                    let expression_result = expression_result
                        .filter(|value| !matches!(value.as_ref(), TemplateValue::Null));
                    let text: Arc<dyn CharSequenceValue> = match template_mode {
                        TemplateMode::JAVASCRIPT | TemplateMode::CSS => Arc::new(
                            LazyEscapingCharSequence::new(
                                Some(context.get_configuration_arc()),
                                Some(template_mode),
                                expression_result,
                            )
                            .map_err(|error| {
                                Box::new(TemplateProcessingException::with_cause(
                                    Some("Could not create lazy escaping sequence".to_owned()),
                                    error,
                                ))
                                    as Box<dyn TemplateEngineException>
                            })?,
                        ),
                        _ => {
                            let input = expression_result
                                .as_deref()
                                .and_then(TemplateValue::to_utf16_string)
                                .unwrap_or_else(|| Utf16String::from_rust_str(""));
                            if template_mode == TemplateMode::RAW {
                                Arc::new(input)
                            } else if input.len() > 100 {
                                Arc::new(
                                    LazyEscapingCharSequence::new(
                                        Some(context.get_configuration_arc()),
                                        Some(template_mode),
                                        Some(Arc::new(TemplateValue::String(Arc::new(input)))),
                                    )
                                    .map_err(|error| {
                                        Box::new(TemplateProcessingException::with_cause(
                                            Some(
                                                "Could not create lazy escaping sequence"
                                                    .to_owned(),
                                            ),
                                            error,
                                        ))
                                            as Box<dyn TemplateEngineException>
                                    })?,
                                )
                            } else {
                                Arc::new(escape_text_immediately(template_mode, &input).map_err(
                                    |error| Box::new(error) as Box<dyn TemplateEngineException>,
                                )?)
                            }
                        }
                    };
                    structure_handler.set_body_sequence(text, false);
                    Ok(())
                },
                "org.thymeleaf.standard.processor.StandardTextTagProcessor",
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardTextTagProcessor, processor);
