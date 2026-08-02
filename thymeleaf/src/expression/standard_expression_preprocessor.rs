use crate::context::IExpressionContext;
use crate::util::JavaString;

use super::{
    LiteralSubstitutionUtil, StandardExpressionExecutionContext, StandardExpressionResult,
    StandardExpressions, expression_parsing_util::ExpressionParsingUtil,
};

/// 执行 Standard Expression 中 `__...__` 片段的预处理器。
///
/// 对应 Java: `org.thymeleaf.standard.expression.StandardExpressionPreprocessor`。
pub(crate) struct StandardExpressionPreprocessor;

impl StandardExpressionPreprocessor {
    /// 以 RESTRICTED 上下文执行预处理表达式并替换到原输入。
    pub(crate) fn preprocess(
        context: &dyn IExpressionContext,
        input: &JavaString,
    ) -> StandardExpressionResult<JavaString> {
        if !input.as_utf16().contains(&(b'_' as u16)) {
            return Ok(input.clone());
        }
        let parser = StandardExpressions::get_expression_parser(context.get_configuration())?;
        if !parser.supports_standard_preprocessing() {
            return Ok(input.clone());
        }

        let units = input.as_utf16();
        let mut output = Vec::with_capacity(units.len().saturating_add(24));
        let mut current = 0;
        let mut found = false;
        while let Some(start) = find_delimiter(units, current) {
            let expression_start = start + 2;
            let Some(end) = find_delimiter(units, expression_start) else {
                break;
            };
            found = true;
            output.extend_from_slice(&unescape_marks(&units[current..start]));
            let expression_text =
                JavaString::from_utf16(unescape_marks(&units[expression_start..end]));
            let substituted =
                LiteralSubstitutionUtil::perform_literal_substitution(Some(&expression_text))
                    .expect("non-null input remains non-null");
            let expression = ExpressionParsingUtil::parse_expression(&substituted)?;
            let result = expression
                .execute_with_context(context, StandardExpressionExecutionContext::RESTRICTED)?;
            let result_text = result
                .as_deref()
                .and_then(super::TemplateValue::to_java_string)
                .unwrap_or_else(|| JavaString::from_rust_str("null"));
            output.extend_from_slice(result_text.as_utf16());
            current = end + 2;
        }
        if !found {
            return Ok(JavaString::from_utf16(unescape_marks(units)));
        }
        output.extend_from_slice(&unescape_marks(&units[current..]));
        Ok(java_trim_owned(output))
    }
}

fn find_delimiter(input: &[u16], from: usize) -> Option<usize> {
    (from..input.len().saturating_sub(1))
        .find(|index| input[*index] == b'_' as u16 && input[*index + 1] == b'_' as u16)
}

fn unescape_marks(input: &[u16]) -> Vec<u16> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        if input.get(index..index + 4)
            == Some(&[b'\\' as u16, b'_' as u16, b'\\' as u16, b'_' as u16])
        {
            output.extend_from_slice(&[b'_' as u16, b'_' as u16]);
            index += 4;
        } else {
            output.push(input[index]);
            index += 1;
        }
    }
    output
}

fn java_trim_owned(input: Vec<u16>) -> JavaString {
    let start = input
        .iter()
        .position(|unit| *unit > 0x20)
        .unwrap_or(input.len());
    let end = input
        .iter()
        .rposition(|unit| *unit > 0x20)
        .map_or(start, |position| position + 1);
    JavaString::from_utf16(input[start..end].to_vec())
}
