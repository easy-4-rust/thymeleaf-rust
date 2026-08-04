use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::util::{Utf16String, ValidateError};

use super::{
    AssignationSequence, ExpressionCache, StandardExpressionPreprocessor, StandardExpressionResult,
    expression_parsing_util::ExpressionParsingUtil,
};

/// 解析 Standard Expression 赋值序列。
///
/// 对应 Java: `org.thymeleaf.standard.expression.AssignationUtils`。
pub struct AssignationUtils;

impl AssignationUtils {
    /// 预处理并解析逗号分隔的赋值序列。
    /// 对应 Java: `AssignationUtils#parseAssignationSequence()`。
    pub fn parse_assignation_sequence(
        context: &dyn IExpressionContext,
        input: Option<&Utf16String>,
        allow_parameters_without_value: bool,
    ) -> StandardExpressionResult<Arc<AssignationSequence>> {
        let input = input.ok_or_else(|| {
            Box::new(ValidateError::IllegalArgument {
                message: Some("Input cannot be null".to_owned()),
            }) as super::StandardExpressionError
        })?;
        let preprocessed = StandardExpressionPreprocessor::preprocess(context, input)?;
        let configuration = context.get_configuration();
        if let Some(cached) =
            ExpressionCache::get_assignation_sequence_from_cache(configuration, &preprocessed)
        {
            return Ok(cached);
        }
        let parsed = Self::internal_parse_assignation_sequence(
            &preprocessed,
            allow_parameters_without_value,
        )
        .ok_or_else(|| parse_error("assignation sequence", input))?;
        let parsed = Arc::new(parsed);
        ExpressionCache::put_assignation_sequence_into_cache(
            configuration,
            &preprocessed,
            Arc::clone(&parsed),
        );
        Ok(parsed)
    }

    /// 不执行预处理和缓存，直接解析赋值序列。
    /// 对应 Java: `AssignationUtils#internalParseAssignationSequence()`。
    pub(crate) fn internal_parse_assignation_sequence(
        input: &Utf16String,
        allow_parameters_without_value: bool,
    ) -> Option<AssignationSequence> {
        ExpressionParsingUtil::parse_assignation_sequence(
            &java_trim(input),
            allow_parameters_without_value,
        )
    }
}

fn parse_error(kind: &str, input: &Utf16String) -> super::StandardExpressionError {
    Box::new(TemplateProcessingException::new(Some(format!(
        "Could not parse as {kind}: \"{}\"",
        input.to_string_lossy()
    ))))
}

fn java_trim(input: &Utf16String) -> Utf16String {
    let units = input.as_utf16();
    let start = units
        .iter()
        .position(|unit| *unit > 0x20)
        .unwrap_or(units.len());
    let end = units
        .iter()
        .rposition(|unit| *unit > 0x20)
        .map_or(start, |position| position + 1);
    Utf16String::from_utf16(units[start..end].to_vec())
}
