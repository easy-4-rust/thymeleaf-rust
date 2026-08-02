use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::util::{JavaString, ValidateError};

use super::{
    Each, ExpressionCache, StandardExpressionPreprocessor, StandardExpressionResult,
    expression_parsing_util::ExpressionParsingUtil,
};

/// 解析 `th:each` 迭代声明。
///
/// 对应 Java: `org.thymeleaf.standard.expression.EachUtils`。
pub struct EachUtils;

impl EachUtils {
    /// 预处理、缓存并解析迭代声明。
    pub fn parse_each(
        context: &dyn IExpressionContext,
        input: Option<&JavaString>,
    ) -> StandardExpressionResult<Arc<Each>> {
        let input = input.ok_or_else(|| {
            Box::new(ValidateError::IllegalArgument {
                message: Some("Input cannot be null".to_owned()),
            }) as super::StandardExpressionError
        })?;
        let preprocessed = StandardExpressionPreprocessor::preprocess(context, input)?;
        let configuration = context.get_configuration();
        if let Some(cached) = ExpressionCache::get_each_from_cache(configuration, &preprocessed) {
            return Ok(cached);
        }
        let parsed =
            ExpressionParsingUtil::parse_each(&java_trim(&preprocessed)).ok_or_else(|| {
                Box::new(TemplateProcessingException::new(Some(format!(
                    "Could not parse as each: \"{}\"",
                    input.to_string_lossy()
                )))) as super::StandardExpressionError
            })?;
        let parsed = Arc::new(parsed);
        ExpressionCache::put_each_into_cache(configuration, &preprocessed, Arc::clone(&parsed));
        Ok(parsed)
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
