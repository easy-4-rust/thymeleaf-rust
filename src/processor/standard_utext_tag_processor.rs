use std::sync::Arc;

use crate::TemplateMode;
use crate::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use crate::engine::EngineEventUtils;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::{
    Fragment, FragmentExpression, StandardExpressionExecutionContext, TemplateValue,
};
use crate::model::IProcessableElementTag;
use crate::util::JavaString;

use super::{IProcessor, StandardAttributeCallback, expression_processing_error};

/// 执行 `th:utext` 并插入不转义正文的 Processor。
///
/// Fragment 直接插入模型；普通文本仅在存在后处理器且可能包含结构时重新解析，并始终
/// 标记为不可处理以阻止代码注入。对应 Java:
/// `org.thymeleaf.standard.processor.StandardUtextTagProcessor`。
pub struct StandardUtextTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl StandardUtextTagProcessor {
    /// Java Processor precedence。
    pub const PRECEDENCE: i32 = 1400;
    /// Standard 属性本地名称。
    pub const ATTR_NAME: &'static str = "utext";

    /// 创建指定模板模式和方言前缀的 `th:utext` Processor。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, TemplateProcessingException> {
        let callback: StandardAttributeCallback = Box::new(
            move |context, tag, attribute_name, attribute_value, structure_handler| {
                let expression = EngineEventUtils::compute_attribute_expression(
                    context,
                    tag,
                    attribute_name,
                    attribute_value.as_ref().ok_or_else(|| {
                        Box::new(TemplateProcessingException::new(Some(
                            "Attribute value cannot be null".to_owned(),
                        ))) as Box<dyn TemplateEngineException>
                    })?,
                )
                .map_err(|error| {
                    expression_processing_error(
                        "Could not compute Standard attribute expression",
                        error,
                    )
                })?;
                let expression_result = if let Some(fragment_expression) =
                    expression.as_fragment_expression()
                {
                    let executed = FragmentExpression::create_executed_fragment_expression(
                        context,
                        fragment_expression,
                    )
                    .map_err(|error| {
                        expression_processing_error("Could not execute Fragment expression", error)
                    })?;
                    FragmentExpression::resolve_executed_fragment_expression(
                        context, &executed, true,
                    )
                    .map_err(|error| {
                        expression_processing_error("Could not resolve Fragment expression", error)
                    })?
                    .map(|fragment| Arc::new(TemplateValue::Object(fragment)))
                } else {
                    expression
                        .execute_with_context(
                            context,
                            StandardExpressionExecutionContext::RESTRICTED,
                        )
                        .map_err(|error| {
                            expression_processing_error(
                                "Could not execute Standard attribute expression",
                                error,
                            )
                        })?
                };

                if expression_result
                    .as_deref()
                    .is_some_and(|value| matches!(value, TemplateValue::NoOp))
                {
                    return Ok(());
                }

                if let Some(TemplateValue::Object(object)) = expression_result.as_deref()
                    && let Some(fragment) = object.as_any().downcast_ref::<Fragment>()
                {
                    if fragment.get_template_model().is_none() {
                        structure_handler.remove_body();
                    } else {
                        structure_handler.set_body_model(
                            fragment
                                .get_template_model_arc()
                                .expect("non-empty Fragment requires a model"),
                            false,
                        );
                    }
                    return Ok(());
                }

                let unescaped_text = expression_result
                    .as_deref()
                    .filter(|value| !matches!(value, TemplateValue::Null))
                    .and_then(TemplateValue::to_java_string)
                    .unwrap_or_else(|| JavaString::from_rust_str(""));
                if context
                    .get_configuration()
                    .get_post_processors(template_mode)
                    .is_empty()
                    || !might_contain_structures(&unescaped_text)
                {
                    structure_handler.set_body_text(unescaped_text, false);
                    return Ok(());
                }

                let owner_template_data = context.get_template_data();
                let parsed_fragment = context
                    .get_configuration()
                    .get_template_manager()
                    .parse_string(
                        owner_template_data.as_ref(),
                        &unescaped_text,
                        0,
                        0,
                        None,
                        false,
                    )
                    .map_err(|error| {
                        Box::new(TemplateProcessingException::with_cause(
                            Some("Could not parse unescaped text fragment".to_owned()),
                            error,
                        )) as Box<dyn TemplateEngineException>
                    })?;
                structure_handler.set_body_model(Arc::from(parsed_fragment), false);
                Ok(())
            },
        );
        Ok(Self {
            processor: AbstractAttributeTagProcessor::new(
                Some(template_mode),
                dialect_prefix,
                None,
                false,
                Some(JavaString::from_rust_str(Self::ATTR_NAME)),
                true,
                Self::PRECEDENCE,
                true,
                "org.thymeleaf.standard.processor.StandardUtextTagProcessor",
                callback,
            )?,
        })
    }
}

impl IProcessor for StandardUtextTagProcessor {
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

impl IElementProcessor for StandardUtextTagProcessor {
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

impl IElementTagProcessor for StandardUtextTagProcessor {
    fn process(
        &self,
        context: &dyn crate::context::ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}

fn might_contain_structures(unescaped_text: &JavaString) -> bool {
    unescaped_text
        .as_utf16()
        .iter()
        .rev()
        .any(|unit| matches!(*unit, 0x3E | 0x5D))
}
