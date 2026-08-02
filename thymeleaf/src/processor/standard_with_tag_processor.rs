use crate::TemplateMode;
use crate::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::{AssignationUtils, TemplateValue};
use crate::model::IProcessableElementTag;
use crate::util::JavaString;

use super::{
    IProcessor, StandardAttributeCallback, expression_processing_error, is_empty_or_java_whitespace,
};

/// 顺序声明 `th:with` 局部变量的 Processor。
///
/// EngineContext 会立即写入每个变量，使同一赋值序列的后续表达式可见；自定义上下文
/// 退化为结构处理器延迟变量。对应 Java:
/// `org.thymeleaf.standard.processor.StandardWithTagProcessor`。
pub struct StandardWithTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl StandardWithTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 600;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "with";

    /// 创建 Processor。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, TemplateProcessingException> {
        let callback: StandardAttributeCallback = Box::new(
            |context, _tag, _attribute_name, attribute_value, structure_handler| {
                let assignations = AssignationUtils::parse_assignation_sequence(
                    context,
                    attribute_value.as_ref(),
                    false,
                )
                .map_err(|error| {
                    expression_processing_error(
                        "Could not parse value as attribute assignations",
                        error,
                    )
                })?;
                for assignation in assignations.get_assignations().iter() {
                    let assignation = assignation.as_ref().ok_or_else(|| {
                        Box::new(TemplateProcessingException::new(Some(
                            "Assignation list cannot contain any nulls".to_owned(),
                        ))) as Box<dyn TemplateEngineException>
                    })?;
                    let left_value = assignation.get_left().execute(context).map_err(|error| {
                        expression_processing_error(
                            "Could not execute variable name expression",
                            error,
                        )
                    })?;
                    let right_value = assignation
                        .get_right()
                        .ok_or_else(|| {
                            Box::new(TemplateProcessingException::new(Some(
                                "Variable assignation has no right-side expression".to_owned(),
                            ))) as Box<dyn TemplateEngineException>
                        })?
                        .execute(context)
                        .map_err(|error| {
                            expression_processing_error(
                                "Could not execute variable value expression",
                                error,
                            )
                        })?;
                    let variable_name = left_value
                        .as_deref()
                        .and_then(TemplateValue::to_java_string);
                    if is_empty_or_java_whitespace(variable_name.as_ref()) {
                        return Err(Box::new(TemplateProcessingException::new(Some(format!(
                            "Variable name expression evaluated as null or empty: \"{}\"",
                            assignation
                                .get_left()
                                .get_string_representation()
                                .map_or_else(|_| String::new(), |value| value.to_string_lossy())
                        )))));
                    }
                    let variable_name =
                        variable_name.expect("variable name was checked as non-empty");
                    if let Some(engine_context) = context.as_engine_context() {
                        engine_context.set_variable(Some(variable_name), right_value);
                    } else {
                        structure_handler.set_local_variable(variable_name, right_value);
                    }
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
                Some(JavaString::from_rust_str(Self::ATTR_NAME)),
                true,
                Self::PRECEDENCE,
                true,
                "org.thymeleaf.standard.processor.StandardWithTagProcessor",
                callback,
            )?,
        })
    }
}

impl IProcessor for StandardWithTagProcessor {
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

impl IElementProcessor for StandardWithTagProcessor {
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

impl IElementTagProcessor for StandardWithTagProcessor {
    fn process(
        &self,
        context: &dyn crate::context::ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}
