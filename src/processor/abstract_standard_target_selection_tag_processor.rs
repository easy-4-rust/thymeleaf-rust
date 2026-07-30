use std::sync::Arc;

use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use crate::engine::AttributeName;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::{IStandardExpression, StandardExpressions, TemplateValue};
use crate::model::IProcessableElementTag;
use crate::util::JavaString;

use super::{IProcessor, StandardAttributeCallback, expression_processing_error};

/// Standard selection target 属性 Processor 的组合式抽象实现。
///
/// 对应 Java: `org.thymeleaf.standard.processor.AbstractStandardTargetSelectionTagProcessor`。
pub struct AbstractStandardTargetSelectionTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl AbstractStandardTargetSelectionTagProcessor {
    /// 创建 selection target Processor，并保留校验和附加局部变量两个扩展钩子。
    #[allow(clippy::type_complexity)]
    pub fn new<V, L>(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
        attr_name: JavaString,
        precedence: i32,
        validate_selection_value: V,
        compute_additional_local_variables: L,
        processor_class_name: &'static str,
    ) -> Result<Self, TemplateProcessingException>
    where
        V: Fn(
                &dyn ITemplateContext,
                &dyn IProcessableElementTag,
                &AttributeName,
                Option<&JavaString>,
                &dyn IStandardExpression,
            ) -> Result<(), Box<dyn TemplateEngineException>>
            + Send
            + Sync
            + 'static,
        L: Fn(
                &dyn ITemplateContext,
                &dyn IProcessableElementTag,
                &AttributeName,
                Option<&JavaString>,
                &dyn IStandardExpression,
            ) -> Result<
                Option<Vec<(JavaString, Option<Arc<TemplateValue>>)>>,
                Box<dyn TemplateEngineException>,
            > + Send
            + Sync
            + 'static,
    {
        let validate_selection_value = Arc::new(validate_selection_value);
        let compute_additional_local_variables = Arc::new(compute_additional_local_variables);
        let callback: StandardAttributeCallback = Box::new(
            move |context, tag, attribute_name, attribute_value, structure_handler| {
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
                        expression_processing_error("Could not parse selection expression", error)
                    })?;
                (validate_selection_value)(
                    context,
                    tag,
                    attribute_name,
                    attribute_value.as_ref(),
                    expression.as_ref(),
                )?;
                let new_selection_target = expression.execute(context).map_err(|error| {
                    expression_processing_error("Could not execute selection expression", error)
                })?;
                if let Some(variables) = (compute_additional_local_variables)(
                    context,
                    tag,
                    attribute_name,
                    attribute_value.as_ref(),
                    expression.as_ref(),
                )? {
                    for (name, value) in variables {
                        structure_handler.set_local_variable(name, value);
                    }
                }
                structure_handler.set_selection_target(new_selection_target);
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

impl IProcessor for AbstractStandardTargetSelectionTagProcessor {
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

impl IElementProcessor for AbstractStandardTargetSelectionTagProcessor {
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

impl IElementTagProcessor for AbstractStandardTargetSelectionTagProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}
