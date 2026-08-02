use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::util::{JavaString, ValidateError};

use super::{
    ExpressionSequence, IStandardExpression, SimpleExpression, StandardExpressionExecutionContext,
    StandardExpressionResult, TemplateValue,
};

/// `#{...}` 外部化消息表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.MessageExpression`。
pub struct MessageExpression {
    base: Arc<dyn IStandardExpression>,
    parameters: Option<Arc<ExpressionSequence>>,
}

impl MessageExpression {
    /// 创建消息表达式。
    ///
    /// # 参数
    /// - `base`：消息键表达式，Java null 会被拒绝；
    /// - `parameters`：可选参数表达式序列。
    pub fn new(
        base: Option<Arc<dyn IStandardExpression>>,
        parameters: Option<Arc<ExpressionSequence>>,
    ) -> Result<Self, ValidateError> {
        let base = base.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Base cannot be null".to_owned()),
        })?;
        Ok(Self { base, parameters })
    }

    /// 返回消息键表达式。
    pub fn get_base(&self) -> &dyn IStandardExpression {
        self.base.as_ref()
    }

    /// 返回可选参数序列。
    pub fn get_parameters(&self) -> Option<&ExpressionSequence> {
        self.parameters.as_deref()
    }

    /// 判断当前是否具有至少一个参数。
    pub fn has_parameters(&self) -> bool {
        self.parameters
            .as_deref()
            .is_some_and(|parameters| parameters.size() > 0)
    }
}

impl IStandardExpression for MessageExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        let mut units = vec![b'#' as u16, b'{' as u16];
        units.extend_from_slice(self.base.get_string_representation()?.as_utf16());
        if self.has_parameters() {
            units.push(b'(' as u16);
            units.extend_from_slice(
                self.parameters
                    .as_deref()
                    .expect("has_parameters checked")
                    .get_string_representation()?
                    .as_utf16(),
            );
            units.push(b')' as u16);
        }
        units.push(b'}' as u16);
        Ok(JavaString::from_utf16(units))
    }

    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let Some(template_context) = context.as_template_context() else {
            return Err(Box::new(TemplateProcessingException::new(Some(format!(
                "Cannot evaluate expression \"{}\". Message externalization expressions can only \
                 be evaluated in a template-processing environment (as a part of an in-template \
                 expression) where processing context is an implementation of interface \
                 org.thymeleaf.context.ITemplateContext, which it isn't ({})",
                self.get_string_representation()?.to_string_lossy(),
                std::any::type_name_of_val(context.as_any())
            )))));
        };

        let key_value = self.base.execute_with_context(context, execution_context)?;
        let key = match key_value.as_deref() {
            None | Some(TemplateValue::Null) => None,
            Some(value) => value.to_java_string(),
        };
        if key.as_ref().is_none_or(is_empty_or_java_whitespace) {
            return Err(Box::new(TemplateProcessingException::new(Some(
                "Message key for message resolution must be a non-null and non-empty String"
                    .to_owned(),
            ))));
        }

        let mut parameter_values = Vec::new();
        if let Some(parameters) = self.parameters.as_deref().filter(|_| self.has_parameters()) {
            let expressions = parameters.get_expressions();
            parameter_values.reserve(expressions.len());
            for parameter in expressions.iter() {
                let result = parameter
                    .as_ref()
                    .expect("ExpressionSequence rejects nulls")
                    .execute_with_context(context, execution_context)?;
                parameter_values.push(unwrap_literal(result));
            }
        }
        Ok(template_context
            .get_message(
                None,
                key.as_ref().expect("validated key"),
                Some(&parameter_values),
                true,
            )?
            .map(TemplateValue::string)
            .map(Arc::new))
    }
}

impl SimpleExpression for MessageExpression {}

fn unwrap_literal(value: Option<Arc<TemplateValue>>) -> Option<Arc<TemplateValue>> {
    match value.as_deref() {
        Some(TemplateValue::Literal(literal)) => literal
            .get_value()
            .cloned()
            .map(TemplateValue::string)
            .map(Arc::new),
        Some(TemplateValue::Null) | None => None,
        _ => value,
    }
}

fn is_empty_or_java_whitespace(value: &JavaString) -> bool {
    value.is_empty()
        || value.as_utf16().iter().all(|unit| {
            matches!(
                unit,
                0x0009..=0x000D
                    | 0x001C..=0x0020
                    | 0x1680
                    | 0x2000..=0x2006
                    | 0x2008..=0x200A
                    | 0x2028
                    | 0x2029
                    | 0x205F
                    | 0x3000
            )
        })
}
