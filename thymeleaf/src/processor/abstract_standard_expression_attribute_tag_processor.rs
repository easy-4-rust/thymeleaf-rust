use std::sync::Arc;

use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use crate::engine::{AttributeName, EngineEventUtils};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::{FragmentExpression, StandardExpressionExecutionContext, TemplateValue};
use crate::model::IProcessableElementTag;
use crate::util::JavaString;

use super::{IProcessor, StandardAttributeCallback, expression_processing_error};

/// 解析、缓存并执行属性 Standard Expression 的组合式抽象 Processor。
///
/// 保留 FragmentExpression 快捷路径、受限执行上下文、NO-OP 短路和属性删除规则。
/// 对应 Java: `org.thymeleaf.standard.processor.AbstractStandardExpressionAttributeTagProcessor`。
pub struct AbstractStandardExpressionAttributeTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl AbstractStandardExpressionAttributeTagProcessor {
    /// 创建使用指定执行上下文的表达式属性 Processor。
    #[allow(clippy::too_many_arguments)]
    pub fn new<F>(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
        attr_name: JavaString,
        precedence: i32,
        remove_attribute: bool,
        expression_execution_context: &'static StandardExpressionExecutionContext,
        do_process: F,
        processor_class_name: &'static str,
    ) -> Result<Self, TemplateProcessingException>
    where
        F: Fn(
                &dyn ITemplateContext,
                &dyn IProcessableElementTag,
                &AttributeName,
                Option<&JavaString>,
                Option<Arc<TemplateValue>>,
                &mut dyn IElementTagStructureHandler,
            ) -> Result<(), Box<dyn TemplateEngineException>>
            + Send
            + Sync
            + 'static,
    {
        let do_process = Arc::new(do_process);
        let remove_if_noop = !remove_attribute;
        let callback: StandardAttributeCallback = Box::new(
            move |context, tag, attribute_name, attribute_value, structure_handler| {
                let expression_result = if let Some(attribute_value) = attribute_value.as_ref() {
                    let expression = EngineEventUtils::compute_attribute_expression(
                        context,
                        tag,
                        attribute_name,
                        attribute_value,
                    )
                    .map_err(|error| {
                        expression_processing_error(
                            "Could not compute Standard attribute expression",
                            error,
                        )
                    })?;
                    if let Some(fragment_expression) = expression.as_fragment_expression() {
                        let executed = FragmentExpression::create_executed_fragment_expression(
                            context,
                            fragment_expression,
                        )
                        .map_err(|error| {
                            expression_processing_error(
                                "Could not execute Fragment expression",
                                error,
                            )
                        })?;
                        FragmentExpression::resolve_executed_fragment_expression(
                            context, &executed, true,
                        )
                        .map_err(|error| {
                            expression_processing_error(
                                "Could not resolve Fragment expression",
                                error,
                            )
                        })?
                        .map(|fragment| {
                            Arc::new(TemplateValue::Object(fragment)) as Arc<TemplateValue>
                        })
                    } else {
                        expression
                            .execute_with_context(context, expression_execution_context)
                            .map_err(|error| {
                                expression_processing_error(
                                    "Could not execute Standard attribute expression",
                                    error,
                                )
                            })?
                    }
                } else {
                    None
                };

                if expression_result
                    .as_deref()
                    .is_some_and(|value| matches!(value, TemplateValue::NoOp))
                {
                    if remove_if_noop {
                        let complete_name = attribute_name.to_java_string().map_err(|error| {
                            Box::new(TemplateProcessingException::with_cause(
                                Some("Could not render attribute name".to_owned()),
                                error,
                            )) as Box<dyn TemplateEngineException>
                        })?;
                        structure_handler.remove_attribute(complete_name);
                    }
                    return Ok(());
                }

                (do_process)(
                    context,
                    tag,
                    attribute_name,
                    attribute_value.as_ref(),
                    expression_result,
                    structure_handler,
                )
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
                remove_attribute,
                processor_class_name,
                callback,
            )?,
        })
    }

    /// 创建 NORMAL 或 RESTRICTED 执行上下文的表达式属性 Processor。
    #[allow(clippy::too_many_arguments)]
    pub fn with_restricted_execution<F>(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
        attr_name: JavaString,
        precedence: i32,
        remove_attribute: bool,
        restricted_expression_execution: bool,
        do_process: F,
        processor_class_name: &'static str,
    ) -> Result<Self, TemplateProcessingException>
    where
        F: Fn(
                &dyn ITemplateContext,
                &dyn IProcessableElementTag,
                &AttributeName,
                Option<&JavaString>,
                Option<Arc<TemplateValue>>,
                &mut dyn IElementTagStructureHandler,
            ) -> Result<(), Box<dyn TemplateEngineException>>
            + Send
            + Sync
            + 'static,
    {
        Self::new(
            template_mode,
            dialect_prefix,
            attr_name,
            precedence,
            remove_attribute,
            if restricted_expression_execution {
                StandardExpressionExecutionContext::RESTRICTED
            } else {
                StandardExpressionExecutionContext::NORMAL
            },
            do_process,
            processor_class_name,
        )
    }
}

impl IProcessor for AbstractStandardExpressionAttributeTagProcessor {
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

impl IElementProcessor for AbstractStandardExpressionAttributeTagProcessor {
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

impl IElementTagProcessor for AbstractStandardExpressionAttributeTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}
