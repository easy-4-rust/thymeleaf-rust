use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::Utf16String;

use super::{
    AssignationSequence, AssignationUtils, Each, EachUtils, ExpressionCache, ExpressionSequence,
    ExpressionSequenceUtils, FragmentSignature, FragmentSignatureUtils, IStandardExpression,
    IStandardExpressionParser, LiteralSubstitutionUtil, StandardExpressionPreprocessor,
    StandardExpressionResult, expression_parsing_util::ExpressionParsingUtil,
};

/// Standard Expression 默认解析器。
///
/// 对应 Java: `org.thymeleaf.standard.expression.StandardExpressionParser`。
#[derive(Debug, Default)]
pub struct StandardExpressionParser;

impl StandardExpressionParser {
    /// 创建无状态、线程安全的默认解析器。
    pub const fn new() -> Self {
        Self
    }

    /// 解析赋值序列。对应 Java: `StandardExpressionParser#parseAssignationSequence`。
    pub fn parse_assignation_sequence(
        &self,
        context: &dyn IExpressionContext,
        input: Option<&Utf16String>,
        allow_parameters_without_value: bool,
    ) -> StandardExpressionResult<Arc<AssignationSequence>> {
        AssignationUtils::parse_assignation_sequence(context, input, allow_parameters_without_value)
    }

    /// 解析表达式序列。对应 Java: `StandardExpressionParser#parseExpressionSequence`。
    pub fn parse_expression_sequence(
        &self,
        context: &dyn IExpressionContext,
        input: Option<&Utf16String>,
    ) -> StandardExpressionResult<Arc<ExpressionSequence>> {
        ExpressionSequenceUtils::parse_expression_sequence(context, input)
    }

    /// 解析 each 声明。对应 Java: `StandardExpressionParser#parseEach`。
    pub fn parse_each(
        &self,
        context: &dyn IExpressionContext,
        input: Option<&Utf16String>,
    ) -> StandardExpressionResult<Arc<Each>> {
        EachUtils::parse_each(context, input)
    }

    /// 解析 Fragment 签名。对应 Java: `StandardExpressionParser#parseFragmentSignature`。
    pub fn parse_fragment_signature(
        &self,
        configuration: Option<&dyn crate::IEngineConfiguration>,
        input: Option<&Utf16String>,
    ) -> StandardExpressionResult<Arc<FragmentSignature>> {
        FragmentSignatureUtils::parse_fragment_signature(configuration, input)
    }
}

impl IStandardExpressionParser for StandardExpressionParser {
    fn parse_expression(
        &self,
        _context: &dyn IExpressionContext,
        input: Option<&Utf16String>,
    ) -> StandardExpressionResult<Arc<dyn IStandardExpression>> {
        let input = input.ok_or_else(|| {
            Box::new(crate::util::ValidateError::IllegalArgument {
                message: Some("Input cannot be null".to_owned()),
            }) as crate::expression::StandardExpressionError
        })?;
        let preprocessed = StandardExpressionPreprocessor::preprocess(_context, input)?;
        if let Some(cached) =
            ExpressionCache::get_expression_from_cache(_context.get_configuration(), &preprocessed)
        {
            return Ok(cached);
        }
        let substituted =
            LiteralSubstitutionUtil::perform_literal_substitution(Some(&preprocessed))
                .expect("non-null input remains non-null");
        let expression = ExpressionParsingUtil::parse_expression(&substituted)?;
        ExpressionCache::put_expression_into_cache(
            _context.get_configuration(),
            &preprocessed,
            Arc::clone(&expression),
        );
        Ok(expression)
    }

    fn supports_standard_preprocessing(&self) -> bool {
        true
    }
}

impl std::fmt::Display for StandardExpressionParser {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Standard Expression Parser")
    }
}
