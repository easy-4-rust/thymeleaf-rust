use crate::TemplateMode;
use crate::element::{
    AbstractAttributeTagProcessor, IElementProcessor, IElementTagProcessor,
    IElementTagStructureHandler, MatchingAttributeName, MatchingElementName,
};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::expression::{EachUtils, TemplateValue};
use crate::model::IProcessableElementTag;
use crate::util::JavaString;

use super::{
    IProcessor, StandardAttributeCallback, expression_processing_error, is_empty_or_java_whitespace,
};

/// 解析并建立 `th:each` 元素迭代的 Processor。
///
/// 对应 Java: `org.thymeleaf.standard.processor.StandardEachTagProcessor`。
pub struct StandardEachTagProcessor {
    processor: AbstractAttributeTagProcessor<StandardAttributeCallback>,
}

impl StandardEachTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 200;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "each";

    /// 创建 Processor。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, TemplateProcessingException> {
        let callback: StandardAttributeCallback = Box::new(
            |context, _tag, _attribute_name, attribute_value, structure_handler| {
                let each =
                    EachUtils::parse_each(context, attribute_value.as_ref()).map_err(|error| {
                        expression_processing_error("Could not parse each expression", error)
                    })?;
                let iter_var_value = each.get_iter_var().execute(context).map_err(|error| {
                    expression_processing_error(
                        "Could not execute iteration variable expression",
                        error,
                    )
                })?;
                let status_var_value = if let Some(status_expression) = each.get_status_var() {
                    status_expression.execute(context).map_err(|error| {
                        expression_processing_error(
                            "Could not execute status variable expression",
                            error,
                        )
                    })?
                } else {
                    None
                };
                let iterated_value = each.get_iterable().execute(context).map_err(|error| {
                    expression_processing_error("Could not execute iterable expression", error)
                })?;

                let iter_var_name = iter_var_value
                    .as_deref()
                    .and_then(TemplateValue::to_java_string);
                if is_empty_or_java_whitespace(iter_var_name.as_ref()) {
                    return Err(Box::new(TemplateProcessingException::new(Some(format!(
                        "Iteration variable name expression evaluated as null: \"{}\"",
                        each.get_iter_var()
                            .get_string_representation()
                            .map_or_else(|_| String::new(), |value| value.to_string_lossy())
                    )))));
                }
                let status_var_name = status_var_value
                    .as_deref()
                    .and_then(TemplateValue::to_java_string);
                if each.has_status_var() && is_empty_or_java_whitespace(status_var_name.as_ref()) {
                    return Err(Box::new(TemplateProcessingException::new(Some(format!(
                        "Status variable name expression evaluated as null or empty: \"{}\"",
                        each.get_status_var()
                            .and_then(|expression| { expression.get_string_representation().ok() })
                            .map_or_else(String::new, |value| value.to_string_lossy())
                    )))));
                }
                structure_handler
                    .iterate_element(
                        iter_var_name.expect("iteration variable was checked"),
                        status_var_name,
                        iterated_value,
                    )
                    .map_err(|error| {
                        Box::new(TemplateProcessingException::with_cause(
                            Some("Could not configure element iteration".to_owned()),
                            error,
                        )) as Box<dyn TemplateEngineException>
                    })
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
                "org.thymeleaf.standard.processor.StandardEachTagProcessor",
                callback,
            )?,
        })
    }
}

impl IProcessor for StandardEachTagProcessor {
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

impl IElementProcessor for StandardEachTagProcessor {
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

impl IElementTagProcessor for StandardEachTagProcessor {
    fn process(
        &self,
        context: &dyn crate::context::ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, tag, structure_handler)
    }
}
