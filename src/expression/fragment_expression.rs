use std::sync::{Arc, RwLock};

use indexmap::IndexMap;

use crate::context::{IExpressionContext, ITemplateContext};
use crate::exceptions::{TemplateInputException, TemplateProcessingException};
use crate::model::IModel;
use crate::util::{JavaString, ValidateError};

use super::fragment::FragmentParameterMap;
use super::{
    Assignation, AssignationSequence, AssignationUtils, ExpressionSequenceUtils, Fragment,
    IStandardExpression, StandardExpressionExecutionContext, StandardExpressionResult,
    TemplateValue, TextLiteralExpression, expression_parsing_util::ExpressionParsingUtil,
};

/// `~{template :: selector(parameters)}` Fragment 表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.FragmentExpression`。
#[derive(Clone)]
pub struct FragmentExpression {
    template_name: Option<Arc<dyn IStandardExpression>>,
    fragment_selector: Option<Arc<dyn IStandardExpression>>,
    parameters: Option<Arc<AssignationSequence>>,
    synthetic_parameters: bool,
    empty: bool,
}

impl FragmentExpression {
    /// Fragment 表达式选择器。
    pub const SELECTOR: u16 = b'~' as u16;
    const UNNAMED_PARAMETERS_PREFIX: &'static str = "_arg";

    /// 创建非空 Fragment 表达式。
    pub fn new(
        template_name: Option<Arc<dyn IStandardExpression>>,
        fragment_selector: Option<Arc<dyn IStandardExpression>>,
        parameters: Option<Arc<AssignationSequence>>,
        synthetic_parameters: bool,
    ) -> Result<Self, ValidateError> {
        if template_name.is_none() && fragment_selector.is_none() {
            return Err(ValidateError::IllegalArgument {
                message: Some(
                    "Fragment Expression cannot have null template name and null fragment selector"
                        .to_owned(),
                ),
            });
        }
        let synthetic_parameters = parameters
            .as_ref()
            .is_some_and(|values| values.size() > 0 && synthetic_parameters);
        Ok(Self {
            template_name,
            fragment_selector,
            parameters,
            synthetic_parameters,
            empty: false,
        })
    }

    fn empty() -> Self {
        Self {
            template_name: None,
            fragment_selector: None,
            parameters: None,
            synthetic_parameters: false,
            empty: true,
        }
    }

    /// 返回模板名称表达式。
    pub fn get_template_name(&self) -> Option<&dyn IStandardExpression> {
        self.template_name.as_deref()
    }

    /// 返回 Fragment selector 表达式。
    pub fn get_fragment_selector(&self) -> Option<&dyn IStandardExpression> {
        self.fragment_selector.as_deref()
    }

    /// 判断是否具有 Fragment selector。
    pub fn has_fragment_selector(&self) -> bool {
        self.fragment_selector.is_some()
    }

    /// 返回参数赋值序列。
    pub fn get_parameters(&self) -> Option<&AssignationSequence> {
        self.parameters.as_deref()
    }

    /// 判断是否具有至少一个参数。
    pub fn has_parameters(&self) -> bool {
        self.parameters
            .as_ref()
            .is_some_and(|values| values.size() > 0)
    }

    /// 判断参数名是否由引擎按位置合成。
    pub fn has_synthetic_parameters(&self) -> bool {
        self.synthetic_parameters
    }

    /// 解析完整 Fragment 表达式。
    pub fn parse_fragment_expression(input: Option<&JavaString>) -> Option<Self> {
        let input = input?;
        let trimmed = java_trim(input.as_utf16());
        if trimmed.len() < 3
            || trimmed[0] != b'~' as u16
            || trimmed[1] != b'{' as u16
            || trimmed.last() != Some(&(b'}' as u16))
        {
            return None;
        }
        let content = java_trim(&trimmed[2..trimmed.len() - 1]);
        if content.is_empty() {
            return Some(Self::empty());
        }
        Self::parse_fragment_expression_content(&JavaString::from_utf16(content.to_vec()))
    }

    fn parse_fragment_expression_content(input: &JavaString) -> Option<Self> {
        let trimmed = java_trim(input.as_utf16());
        if trimmed.is_empty() {
            return Some(Self::empty());
        }
        let parameter_start = index_of_last_parentheses_group(trimmed);
        let (without_parameters, mut parameters) = match parameter_start {
            Some(position) => (
                java_trim(&trimmed[..position]),
                Some(java_trim(&trimmed[position + 1..trimmed.len() - 1])),
            ),
            None => (trimmed, None),
        };
        let separator = find_double_colon(without_parameters);
        let (mut template_name, mut fragment_spec) = match separator {
            None => (java_trim(without_parameters), None),
            Some(position) => (
                java_trim(&without_parameters[..position]),
                Some(java_trim(&without_parameters[position + 2..])),
            ),
        };
        if separator.is_none() && template_name.is_empty() {
            template_name = parameters.take()?;
        } else if separator.is_some() && fragment_spec.is_some_and(<[u16]>::is_empty) {
            fragment_spec = Some(parameters.take()?);
        }

        let template_name = if template_name.is_empty() {
            None
        } else {
            Some(parse_default_as_literal(template_name)?)
        };
        let fragment_selector = match fragment_spec.filter(|value| !value.is_empty()) {
            Some(value) => Some(parse_default_as_literal(value)?),
            None => None,
        };

        let mut synthetic = false;
        let parameter_sequence = match parameters.filter(|value| !java_trim(value).is_empty()) {
            None => None,
            Some(value) => {
                let value = JavaString::from_utf16(value.to_vec());
                if let Some(assignations) =
                    AssignationUtils::internal_parse_assignation_sequence(&value, false)
                {
                    Some(Arc::new(assignations))
                } else {
                    let expressions =
                        ExpressionSequenceUtils::internal_parse_expression_sequence(&value)?;
                    synthetic = true;
                    Some(Arc::new(create_synthetic_parameters(&expressions)?))
                }
            }
        };

        Self::new(
            template_name,
            fragment_selector,
            parameter_sequence,
            synthetic,
        )
        .ok()
    }

    /// 在 RESTRICTED 上下文中执行模板名、selector 和参数。
    pub fn create_executed_fragment_expression(
        context: &dyn IExpressionContext,
        expression: &FragmentExpression,
    ) -> StandardExpressionResult<ExecutedFragmentExpression> {
        Self::do_create_executed_fragment_expression(
            context,
            expression,
            StandardExpressionExecutionContext::RESTRICTED,
        )
    }

    fn do_create_executed_fragment_expression(
        context: &dyn IExpressionContext,
        expression: &FragmentExpression,
        expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<ExecutedFragmentExpression> {
        if expression.empty {
            return Ok(ExecutedFragmentExpression::empty());
        }
        let template_name_expression_result = match &expression.template_name {
            Some(value) => value
                .execute_with_context(context, StandardExpressionExecutionContext::RESTRICTED)?,
            None => None,
        };
        let fragment_parameters = create_executed_parameters(
            context,
            expression.parameters.as_deref(),
            expression_context,
        )?;
        let fragment_selector_expression_result = match &expression.fragment_selector {
            Some(value) => value.execute_with_context(context, expression_context)?,
            None => None,
        };
        Ok(ExecutedFragmentExpression {
            fragment_expression: expression.clone(),
            expression_representation: expression.get_string_representation()?,
            template_name_expression_result,
            fragment_selector_expression_result,
            fragment_parameters,
            synthetic_parameters: expression.synthetic_parameters,
            empty: false,
        })
    }

    /// 将已执行表达式解析为模板模型 Fragment。
    pub fn resolve_executed_fragment_expression(
        context: &dyn ITemplateContext,
        executed: &ExecutedFragmentExpression,
        fail_if_not_exists: bool,
    ) -> StandardExpressionResult<Option<Arc<Fragment>>> {
        if executed.empty {
            return Ok(Some(Arc::new(Fragment::EMPTY_FRAGMENT)));
        }
        let configuration = context.get_configuration();
        let fragments = Self::resolve_fragments(executed);
        let mut template_name = Self::resolve_template_name(executed);
        let mut template_name_stack = Vec::new();
        if template_name
            .as_ref()
            .is_none_or(|value| java_trim(value.as_utf16()).is_empty())
        {
            if fragments.as_ref().is_none_or(Vec::is_empty) {
                return Ok(None);
            }
            template_name_stack = context
                .get_template_stack()
                .into_iter()
                .rev()
                .filter_map(|data| data.get_template().cloned())
                .collect();
            template_name = template_name_stack.first().cloned();
        }
        let Some(mut current_template) = template_name else {
            return Ok(None);
        };
        let mut stack_index = 0;
        loop {
            let model = configuration
                .get_template_manager()
                .parse_standalone(
                    context,
                    &current_template,
                    fragments.as_deref(),
                    None,
                    true,
                    fail_if_not_exists,
                )
                .map_err(|error| Box::new(error) as super::StandardExpressionError)?;
            let Some(model) = model else {
                return Ok(None);
            };
            if model.size() > 2 {
                let model: Arc<dyn IModel> = Arc::from(model);
                let fragment = Fragment::new(
                    Some(model),
                    executed.fragment_parameters.clone(),
                    executed.synthetic_parameters,
                )
                .map_err(|error| Box::new(error) as super::StandardExpressionError)?;
                return Ok(Some(Arc::new(fragment)));
            }
            stack_index += 1;
            if stack_index >= template_name_stack.len() {
                if fail_if_not_exists {
                    return Err(Box::new(TemplateInputException::new(Some(format!(
                        "Error resolving fragment: \"{}\": template or fragment could not be resolved",
                        executed.expression_representation.to_string_lossy()
                    )))));
                }
                return Ok(None);
            }
            current_template = template_name_stack[stack_index].clone();
        }
    }

    /// 将模板名结果转换为名称；`this` 和 null 表示当前模板。
    pub fn resolve_template_name(executed: &ExecutedFragmentExpression) -> Option<JavaString> {
        let result = executed.template_name_expression_result.as_deref()?;
        let value = result.to_java_string()?;
        (value != JavaString::from_rust_str("this")).then_some(value)
    }

    /// 将 selector 结果规范化为单元素 selector 集合。
    pub fn resolve_fragments(executed: &ExecutedFragmentExpression) -> Option<Vec<JavaString>> {
        let value = executed
            .fragment_selector_expression_result
            .as_deref()?
            .to_java_string()?;
        let units = value.as_utf16();
        let normalized = if units.len() > 3
            && units.first() == Some(&(b'[' as u16))
            && units.last() == Some(&(b']' as u16))
            && units[units.len() - 2] != b'\'' as u16
        {
            JavaString::from_utf16(java_trim(&units[1..units.len() - 1]).to_vec())
        } else {
            value
        };
        (!java_trim(normalized.as_utf16()).is_empty()).then(|| vec![normalized])
    }
}

impl IStandardExpression for FragmentExpression {
    fn is_fragment_expression(&self) -> bool {
        true
    }

    fn as_fragment_expression(&self) -> Option<&FragmentExpression> {
        Some(self)
    }

    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        let mut units = vec![b'~' as u16, b'{' as u16];
        if let Some(template_name) = &self.template_name {
            units.extend_from_slice(template_name.get_string_representation()?.as_utf16());
        }
        if let Some(fragment_selector) = &self.fragment_selector {
            units.extend(" :: ".encode_utf16());
            units.extend_from_slice(fragment_selector.get_string_representation()?.as_utf16());
        }
        if let Some(parameters) = &self.parameters
            && parameters.size() > 0
        {
            units.extend_from_slice(&[b' ' as u16, b'(' as u16]);
            units.extend_from_slice(parameters.get_string_representation()?.as_utf16());
            units.push(b')' as u16);
        }
        units.push(b'}' as u16);
        Ok(JavaString::from_utf16(units))
    }

    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        _expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let template_context = context.as_template_context().ok_or_else(|| {
            Box::new(TemplateProcessingException::new(Some(format!(
                "Cannot evaluate expression \"{}\". Fragment expressions can only be evaluated in a template-processing environment",
                self.get_string_representation()
                    .map_or_else(|_| String::new(), |value| value.to_string_lossy())
            )))) as super::StandardExpressionError
        })?;
        if self.empty {
            return Ok(Some(Arc::new(TemplateValue::Object(Arc::new(
                Fragment::EMPTY_FRAGMENT,
            )))));
        }
        let executed = Self::create_executed_fragment_expression(context, self)?;
        Ok(
            Self::resolve_executed_fragment_expression(template_context, &executed, false)?
                .map(|fragment| Arc::new(TemplateValue::Object(fragment))),
        )
    }
}

impl super::SimpleExpression for FragmentExpression {}

/// Fragment 表达式各子表达式执行后的中间值。
///
/// 对应 Java: `FragmentExpression.ExecutedFragmentExpression`。
pub struct ExecutedFragmentExpression {
    fragment_expression: FragmentExpression,
    expression_representation: JavaString,
    template_name_expression_result: Option<Arc<TemplateValue>>,
    fragment_selector_expression_result: Option<Arc<TemplateValue>>,
    fragment_parameters: Option<Arc<RwLock<FragmentParameterMap>>>,
    synthetic_parameters: bool,
    empty: bool,
}

impl ExecutedFragmentExpression {
    fn empty() -> Self {
        Self {
            fragment_expression: FragmentExpression::empty(),
            expression_representation: JavaString::from_rust_str("~{}"),
            template_name_expression_result: None,
            fragment_selector_expression_result: None,
            fragment_parameters: None,
            synthetic_parameters: false,
            empty: true,
        }
    }

    /// 返回产生此中间值的原 Fragment 表达式。
    ///
    /// 对应 Java: `ExecutedFragmentExpression#getFragmentExpression()`。
    pub fn get_fragment_expression(&self) -> &FragmentExpression {
        &self.fragment_expression
    }

    /// 返回模板名表达式执行结果。
    pub fn get_template_name_expression_result(&self) -> Option<&TemplateValue> {
        self.template_name_expression_result.as_deref()
    }

    /// 返回模板名表达式执行结果的共享身份。
    pub fn get_template_name_expression_result_arc(&self) -> Option<Arc<TemplateValue>> {
        self.template_name_expression_result.clone()
    }

    /// 返回 selector 表达式执行结果。
    pub fn get_fragment_selector_expression_result(&self) -> Option<&TemplateValue> {
        self.fragment_selector_expression_result.as_deref()
    }

    /// 返回参数 Map 的共享视图。
    pub fn get_fragment_parameters(&self) -> Option<&Arc<RwLock<FragmentParameterMap>>> {
        self.fragment_parameters.as_ref()
    }

    /// 判断参数名是否为合成位置参数。
    pub fn has_synthetic_parameters(&self) -> bool {
        self.synthetic_parameters
    }
}

fn create_executed_parameters(
    context: &dyn IExpressionContext,
    parameters: Option<&AssignationSequence>,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<RwLock<FragmentParameterMap>>>> {
    let Some(parameters) = parameters.filter(|values| values.size() > 0) else {
        return Ok(None);
    };
    let mut values = IndexMap::with_capacity(parameters.size() as usize + 2);
    for assignation in parameters.get_assignations().iter().flatten() {
        let parameter_name = assignation
            .get_left()
            .execute_with_context(context, expression_context)?
            .as_deref()
            .and_then(TemplateValue::to_java_string);
        let parameter_value = assignation
            .get_right()
            .ok_or_else(|| {
                Box::new(TemplateProcessingException::new(Some(
                    "Fragment parameter value cannot be null".to_owned(),
                ))) as super::StandardExpressionError
            })?
            .execute_with_context(context, expression_context)?;
        values.insert(parameter_name, parameter_value);
    }
    Ok(Some(Arc::new(RwLock::new(values))))
}

fn create_synthetic_parameters(
    expressions: &super::ExpressionSequence,
) -> Option<AssignationSequence> {
    let mut assignations = Vec::with_capacity(expressions.size() as usize + 2);
    for (index, expression) in expressions.get_expressions().iter().flatten().enumerate() {
        let name = JavaString::from_rust_str(&format!(
            "{}{index}",
            FragmentExpression::UNNAMED_PARAMETERS_PREFIX
        ));
        let wrapped = TextLiteralExpression::wrap_string_into_literal(Some(&name))?;
        let left: Arc<dyn IStandardExpression> = Arc::new(
            TextLiteralExpression::parse_text_literal_expression(&wrapped),
        );
        assignations.push(Some(Arc::new(
            Assignation::new(Some(left), Some(Arc::clone(expression))).ok()?,
        )));
    }
    AssignationSequence::new(Some(Arc::new(RwLock::new(assignations)))).ok()
}

fn parse_default_as_literal(input: &[u16]) -> Option<Arc<dyn IStandardExpression>> {
    let input = JavaString::from_utf16(java_trim(input).to_vec());
    if let Ok(expression) = ExpressionParsingUtil::parse_expression(&input) {
        return Some(expression);
    }
    let wrapped = TextLiteralExpression::wrap_string_into_literal(Some(&input))?;
    Some(Arc::new(
        TextLiteralExpression::parse_text_literal_expression(&wrapped),
    ))
}

fn index_of_last_parentheses_group(input: &[u16]) -> Option<usize> {
    if input.last() != Some(&(b')' as u16)) {
        return None;
    }
    let mut in_literal = false;
    let mut level = 1_i32;
    for index in (0..input.len() - 1).rev() {
        match input[index] {
            value if value == b'\'' as u16 => in_literal = !in_literal,
            value if value == b')' as u16 && !in_literal => level += 1,
            value if value == b'(' as u16 && !in_literal => {
                level -= 1;
                if level == 0 {
                    return (index != input.len() - 2).then_some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_double_colon(input: &[u16]) -> Option<usize> {
    input
        .windows(2)
        .position(|window| window == [b':' as u16, b':' as u16])
}

fn java_trim(input: &[u16]) -> &[u16] {
    let start = input
        .iter()
        .position(|unit| *unit > 0x20)
        .unwrap_or(input.len());
    let end = input
        .iter()
        .rposition(|unit| *unit > 0x20)
        .map_or(start, |position| position + 1);
    &input[start..end]
}
