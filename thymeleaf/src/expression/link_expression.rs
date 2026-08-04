use std::sync::Arc;

use indexmap::IndexMap;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::util::string_case_utils::to_lower_case_default;
use crate::util::{NumberValue, Utf16String, ValidateError};

use super::{
    AssignationSequence, IStandardExpression, SimpleExpression, StandardExpressionExecutionContext,
    StandardExpressionResult, TemplateValue,
};

/// `@{...}` Standard Link Expression。
///
/// 对应 Java: `org.thymeleaf.standard.expression.LinkExpression`。
pub struct LinkExpression {
    base: Arc<dyn IStandardExpression>,
    parameters: Option<Arc<AssignationSequence>>,
}

impl LinkExpression {
    /// 创建链接表达式。
    /// 对应 Java 语义：`LinkExpression` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        base: Option<Arc<dyn IStandardExpression>>,
        parameters: Option<Arc<AssignationSequence>>,
    ) -> Result<Self, ValidateError> {
        let base = base.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Base cannot be null".to_owned()),
        })?;
        Ok(Self { base, parameters })
    }

    /// 返回链接基地址表达式。
    /// 对应 Java: `LinkExpression#getBase()`。
    pub fn get_base(&self) -> &dyn IStandardExpression {
        self.base.as_ref()
    }

    /// 返回可选参数赋值序列。
    /// 对应 Java: `LinkExpression#getParameters()`。
    pub fn get_parameters(&self) -> Option<&AssignationSequence> {
        self.parameters.as_deref()
    }

    /// 判断当前是否具有至少一个参数。
    /// 对应 Java: `LinkExpression#hasParameters()`。
    pub fn has_parameters(&self) -> bool {
        self.parameters
            .as_deref()
            .is_some_and(|parameters| parameters.size() > 0)
    }
}

impl IStandardExpression for LinkExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<Utf16String> {
        let mut units = vec![b'@' as u16, b'{' as u16];
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
        Ok(Utf16String::from_utf16(units))
    }

    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        _execution_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let Some(template_context) = context.as_template_context() else {
            return Err(Box::new(TemplateProcessingException::new(Some(format!(
                "Cannot evaluate expression \"{}\". Link expressions can only be evaluated in a \
                 template-processing environment (as a part of an in-template expression) where \
                 processing context is an implementation of interface \
                 org.thymeleaf.context.ITemplateContext, which it isn't ({})",
                self.get_string_representation()?.to_string_lossy(),
                std::any::type_name_of_val(context.as_any())
            )))));
        };

        // Java 始终以 RESTRICTED 求值 base，防止请求参数直接控制目标 URL。
        let base_value = self
            .base
            .execute_with_context(context, StandardExpressionExecutionContext::RESTRICTED)?;
        let base = normalize_base(base_value.as_deref());
        let parameters = self.resolve_parameters(context)?;
        let link = template_context
            .build_link(Some(&base), parameters.as_ref())
            .map_err(|error| Box::new(error) as super::StandardExpressionError)?;
        Ok(Some(Arc::new(TemplateValue::string(link))))
    }
}

impl SimpleExpression for LinkExpression {}

impl LinkExpression {
    #[expect(
        clippy::type_complexity,
        reason = "返回类型逐项保留 Java LinkExpression 参数 Map 的可空键值"
    )]
    fn resolve_parameters(
        &self,
        context: &dyn IExpressionContext,
    ) -> StandardExpressionResult<Option<IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>>>
    {
        let Some(assignation_sequence) =
            self.parameters.as_deref().filter(|_| self.has_parameters())
        else {
            return Ok(None);
        };
        let assignations = assignation_sequence.get_assignations();
        let mut parameters = IndexMap::with_capacity(assignations.len());
        let mut normalized_names: IndexMap<Utf16String, Utf16String> =
            IndexMap::with_capacity(assignations.len() + 1);

        for assignation in assignations.iter() {
            let assignation = assignation
                .as_ref()
                .expect("AssignationSequence rejects nulls");
            let name_expression = assignation.get_left();
            let name_value = name_expression
                .execute_with_context(context, StandardExpressionExecutionContext::NORMAL)?;
            let Some(mut parameter_name) = name_value
                .as_deref()
                .and_then(TemplateValue::to_utf16_string)
            else {
                return Err(invalid_parameter_name(self, name_expression)?);
            };
            if is_empty_or_java_whitespace(&parameter_name) {
                return Err(invalid_parameter_name(self, name_expression)?);
            }

            let parameter_value = match assignation.get_right() {
                None => None,
                Some(value_expression) => {
                    let value = value_expression.execute_with_context(
                        context,
                        StandardExpressionExecutionContext::NORMAL,
                    )?;
                    Some(match value.as_deref() {
                        None | Some(TemplateValue::Null) => {
                            Arc::new(TemplateValue::string(Utf16String::from_rust_str("")))
                        }
                        Some(TemplateValue::Literal(literal)) => literal
                            .get_value()
                            .cloned()
                            .map(TemplateValue::string)
                            .map(Arc::new)
                            .unwrap_or_else(|| Arc::new(TemplateValue::Null)),
                        Some(_) => value.expect("matched Some"),
                    })
                }
            };

            let normalized = to_lower_case_default(&parameter_name);
            if let Some(first_name) = normalized_names.get(&normalized) {
                parameter_name = first_name.clone();
            } else {
                normalized_names.insert(normalized, parameter_name.clone());
            }
            add_parameter(&mut parameters, parameter_name, parameter_value);
        }
        Ok(Some(parameters))
    }
}

fn add_parameter(
    parameters: &mut IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>,
    parameter_name: Utf16String,
    parameter_value: Option<Arc<TemplateValue>>,
) {
    let key = Some(parameter_name);
    let normalized_value = normalize_parameter_value(parameter_value);
    if let Some(current) = parameters.get_mut(&key) {
        let mut values = match current.as_deref() {
            Some(TemplateValue::List(values)) => values.as_ref().clone(),
            Some(value) => vec![Arc::new(value.clone())],
            None => vec![Arc::new(TemplateValue::Null)],
        };
        match normalized_value.as_deref() {
            Some(TemplateValue::List(additional)) => {
                values.extend(additional.iter().cloned());
            }
            Some(value) => values.push(Arc::new(value.clone())),
            None => values.push(Arc::new(TemplateValue::Null)),
        }
        *current = Some(Arc::new(TemplateValue::List(Arc::new(values))));
    } else {
        parameters.insert(key, normalized_value);
    }
}

fn normalize_parameter_value(value: Option<Arc<TemplateValue>>) -> Option<Arc<TemplateValue>> {
    let value = value?;
    match value.as_ref() {
        TemplateValue::List(values) => Some(Arc::new(TemplateValue::List(Arc::new(
            values.as_ref().clone(),
        )))),
        TemplateValue::Bytes(values) => Some(Arc::new(TemplateValue::List(Arc::new(
            values
                .iter()
                .map(|value| Arc::new(TemplateValue::Number(NumberValue::Byte(*value))))
                .collect(),
        )))),
        TemplateValue::Object(object) => object
            .iterable_values()
            .map(|values| Arc::new(TemplateValue::List(Arc::new(values))))
            .or(Some(value)),
        _ => Some(value),
    }
}

fn normalize_base(value: Option<&TemplateValue>) -> Utf16String {
    let Some(value) = value else {
        return Utf16String::from_rust_str("");
    };
    let value = value
        .to_utf16_string()
        .unwrap_or_else(|| Utf16String::from_rust_str(""));
    if is_empty_or_java_whitespace(&value) {
        return Utf16String::from_rust_str("");
    }
    trim(&value)
}

fn invalid_parameter_name(
    expression: &LinkExpression,
    name_expression: &dyn IStandardExpression,
) -> StandardExpressionResult<crate::expression::StandardExpressionError> {
    Ok(Box::new(TemplateProcessingException::new(Some(format!(
        "Parameters in link expression \"{}\" are incorrect: parameter name expression \"{}\" \
         evaluated as null or empty string.",
        expression.get_string_representation()?.to_string_lossy(),
        name_expression
            .get_string_representation()?
            .to_string_lossy()
    )))))
}

fn trim(value: &Utf16String) -> Utf16String {
    let units = value.as_utf16();
    let mut start = 0;
    while start < units.len() && units[start] <= 0x20 {
        start += 1;
    }
    let mut end = units.len();
    while end > start && units[end - 1] <= 0x20 {
        end -= 1;
    }
    Utf16String::from_utf16(units[start..end].to_vec())
}

fn is_empty_or_java_whitespace(value: &Utf16String) -> bool {
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
