use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::util::{JavaString, ValidateError};

use super::{
    ExpressionCache, ExpressionSequence, StandardExpressionPreprocessor, StandardExpressionResult,
    expression_parsing_util::ExpressionParsingUtil,
};

/// 解析逗号分隔的 Standard Expression 序列。
///
/// 对应 Java: `org.thymeleaf.standard.expression.ExpressionSequenceUtils`。
pub struct ExpressionSequenceUtils;

impl ExpressionSequenceUtils {
    /// 预处理、缓存并解析表达式序列。
    /// 对应 Java: `ExpressionSequenceUtils#parseExpressionSequence()`。
    pub fn parse_expression_sequence(
        context: &dyn IExpressionContext,
        input: Option<&JavaString>,
    ) -> StandardExpressionResult<Arc<ExpressionSequence>> {
        let input = input.ok_or_else(|| {
            Box::new(ValidateError::IllegalArgument {
                message: Some("Input cannot be null".to_owned()),
            }) as super::StandardExpressionError
        })?;
        let preprocessed = StandardExpressionPreprocessor::preprocess(context, input)?;
        let configuration = context.get_configuration();
        if let Some(cached) =
            ExpressionCache::get_expression_sequence_from_cache(configuration, &preprocessed)
        {
            return Ok(cached);
        }
        let parsed = Self::internal_parse_expression_sequence(&java_trim(&preprocessed))
            .ok_or_else(|| {
                Box::new(TemplateProcessingException::new(Some(format!(
                    "Could not parse as expression sequence: \"{}\"",
                    input.to_string_lossy()
                )))) as super::StandardExpressionError
            })?;
        let parsed = Arc::new(parsed);
        ExpressionCache::put_expression_sequence_into_cache(
            configuration,
            &preprocessed,
            Arc::clone(&parsed),
        );
        Ok(parsed)
    }

    /// 不执行预处理和缓存，直接解析表达式序列。
    /// 对应 Java: `ExpressionSequenceUtils#internalParseExpressionSequence()`。
    pub(crate) fn internal_parse_expression_sequence(
        input: &JavaString,
    ) -> Option<ExpressionSequence> {
        ExpressionParsingUtil::parse_expression_sequence(input)
    }
}

fn java_trim(input: &JavaString) -> JavaString {
    let units = input.as_utf16();
    let start = units
        .iter()
        .position(|unit| *unit > 0x20)
        .unwrap_or(units.len());
    let end = units
        .iter()
        .rposition(|unit| *unit > 0x20)
        .map_or(start, |position| position + 1);
    JavaString::from_utf16(units[start..end].to_vec())
}
