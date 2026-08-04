use std::sync::{Arc, RwLock};

use crate::exceptions::TemplateProcessingException;
use crate::util::Utf16String;

use super::{
    AdditionExpression, AndExpression, Assignation, AssignationSequence, BooleanTokenExpression,
    ConditionalExpression, DefaultExpression, DivisionExpression, Each, EqualsExpression,
    ExpressionSequence, FragmentExpression, FragmentSignature, GenericTokenExpression,
    GreaterOrEqualToExpression, GreaterThanExpression, IStandardExpression,
    LessOrEqualToExpression, LessThanExpression, LinkExpression, MessageExpression,
    MinusExpression, MultiplicationExpression, NegationExpression, NoOpTokenExpression,
    NotEqualsExpression, NullTokenExpression, NumberTokenExpression, OrExpression,
    RemainderExpression, SelectionVariableExpression, StandardExpressionResult,
    SubtractionExpression, TextLiteralExpression, Token, VariableExpression,
};

/// Standard Expression 的分解与组合算法。
///
/// 对应 Java: `org.thymeleaf.standard.expression.ExpressionParsingUtil`。
///
/// Java 实现先构建占位符节点再按优先级组合；Rust 在同一 UTF-16 输入上直接执行
/// 等价的递归组合，仍保持右侧最后操作符切分、左结合、括号、字面量和 simple
/// expression 隔离语义。
pub(crate) struct ExpressionParsingUtil;

impl ExpressionParsingUtil {
    /// 解析单个完整 Standard Expression。
    ///
    /// # 参数
    /// - `input`：已经过预处理的 UTF-16 表达式。
    ///
    /// # 返回
    /// 语法完整时返回表达式树，否则返回模板处理错误。
    /// 对应 Java 语义：Java 接口/超类方法 `parseExpression()` 的 Rust 移植（`ExpressionParsingUtil` 继承路径）。
    pub(crate) fn parse_expression(
        input: &Utf16String,
    ) -> StandardExpressionResult<Arc<dyn IStandardExpression>> {
        let trimmed = java_trim(input.as_utf16());
        parse_range(trimmed).ok_or_else(|| {
            Box::new(TemplateProcessingException::new(Some(format!(
                "Could not parse as expression: \"{}\"",
                input.to_string_lossy()
            )))) as crate::expression::StandardExpressionError
        })
    }

    /// 解析赋值序列，供 `AssignationUtils` 与 Fragment/Link 表达式共享。
    /// 对应 Java 语义：Java 接口/超类方法 `parseAssignationSequence()` 的 Rust 移植（`ExpressionParsingUtil` 继承路径）。
    pub(crate) fn parse_assignation_sequence(
        input: &Utf16String,
        allow_parameters_without_value: bool,
    ) -> Option<AssignationSequence> {
        parse_assignation_sequence(input.as_utf16(), allow_parameters_without_value)
    }

    /// 解析逗号分隔的表达式序列。
    /// 对应 Java 语义：Java 接口/超类方法 `parseExpressionSequence()` 的 Rust 移植（`ExpressionParsingUtil` 继承路径）。
    pub(crate) fn parse_expression_sequence(input: &Utf16String) -> Option<ExpressionSequence> {
        parse_expression_sequence(input.as_utf16())
    }

    /// 解析 `iter[,status] : iterable` 声明。
    /// 对应 Java 语义：Java 接口/超类方法 `parseEach()` 的 Rust 移植（`ExpressionParsingUtil` 继承路径）。
    pub(crate) fn parse_each(input: &Utf16String) -> Option<Each> {
        let input = java_trim(input.as_utf16());
        let operator = find_top_level_character(input, b':' as u16)?;
        if operator == 0 || operator + 1 >= input.len() {
            return None;
        }
        let left = java_trim(&input[..operator]);
        let iterable = parse_range(&input[operator + 1..])?;
        let status_separator = find_top_level_character(left, b',' as u16);
        let (iter_var, status_var) = match status_separator {
            Some(position) if position > 0 && position + 1 < left.len() => (
                parse_range(&left[..position])?,
                Some(parse_range(&left[position + 1..])?),
            ),
            Some(_) => return None,
            None => (parse_range(left)?, None),
        };
        Each::new(Some(iter_var), status_var, Some(iterable)).ok()
    }

    /// 解析只允许 token 名称的 Fragment 签名。
    /// 对应 Java 语义：Java 接口/超类方法 `parseFragmentSignature()` 的 Rust 移植（`ExpressionParsingUtil` 继承路径）。
    pub(crate) fn parse_fragment_signature(input: &Utf16String) -> Option<FragmentSignature> {
        let input = java_trim(input.as_utf16());
        if input.is_empty() {
            return None;
        }
        let parameter_start = input.iter().rposition(|unit| *unit == b'(' as u16);
        let parameter_end = input.iter().rposition(|unit| *unit == b')' as u16);
        if parameter_start.is_some_and(|start| parameter_end.is_none_or(|end| start >= end)) {
            return None;
        }
        let fragment_name_end = parameter_start.unwrap_or(input.len());
        let fragment_name =
            Utf16String::from_utf16(java_trim(&input[..fragment_name_end]).to_vec());
        let parameter_names = parameter_start.and_then(|start| {
            let parameters = &input[start + 1..input.len().saturating_sub(1)];
            let values = parameters
                .split(|unit| *unit == b',' as u16)
                .map(|value| Some(Utf16String::from_utf16(java_trim(value).to_vec())))
                .filter(|value| value.as_ref().is_some_and(|value| !value.is_empty()))
                .collect::<Vec<_>>();
            (!values.is_empty()).then(|| Arc::new(RwLock::new(values)))
        });
        FragmentSignature::new(Some(fragment_name), parameter_names).ok()
    }
}

fn parse_range(input: &[u16]) -> Option<Arc<dyn IStandardExpression>> {
    let input = java_trim(input);
    if input.is_empty() {
        return None;
    }
    if is_outer_parenthesized(input) {
        return parse_range(java_trim(&input[1..input.len() - 1]));
    }

    if let Some((question, colon)) = find_conditional(input) {
        let condition = parse_range(&input[..question])?;
        let then_end = colon.unwrap_or(input.len());
        let then_expression = parse_range(&input[question + 1..then_end])?;
        let else_expression = match colon {
            Some(position) => parse_range(&input[position + 1..])?,
            None => NullTokenExpression::parse_null_token_expression(Some(
                &Utf16String::from_rust_str("null"),
            ))?,
        };
        return ConditionalExpression::new(
            Some(condition),
            Some(then_expression),
            Some(else_expression),
        )
        .ok()
        .map(|value| Arc::new(value) as Arc<dyn IStandardExpression>);
    }
    if let Some((question, colon)) = find_default_operator(input) {
        let queried = parse_range(&input[..question])?;
        let default = parse_range(&input[colon + 1..])?;
        return DefaultExpression::new(Some(queried), Some(default))
            .ok()
            .map(|value| Arc::new(value) as Arc<dyn IStandardExpression>);
    }

    macro_rules! binary_group {
        ($operators:expr) => {
            if let Some((position, operator)) = find_binary_operator(input, $operators) {
                let left = parse_range(&input[..position])?;
                let right = parse_range(&input[position + operator.len()..])?;
                return build_binary(operator, left, right);
            }
        };
    }

    binary_group!(&[OP_OR, OP_DOUBLE_PIPE]);
    binary_group!(&[OP_AND, OP_DOUBLE_AMPERSAND]);
    binary_group!(&[OP_NEQ, OP_NE, OP_NOT_EQUALS, OP_EQ, OP_EQUALS]);
    binary_group!(&[
        OP_GTE,
        OP_GE,
        OP_GREATER_EQUAL,
        OP_GT,
        OP_GREATER,
        OP_LTE,
        OP_LE,
        OP_LESS_EQUAL,
        OP_LT,
        OP_LESS,
    ]);
    binary_group!(&[OP_PLUS, OP_MINUS]);
    binary_group!(&[OP_MULTIPLY, OP_DIV, OP_DIVIDE, OP_MOD, OP_REMAINDER]);

    if input[0] == b'-' as u16 {
        let operand = parse_range(&input[1..])?;
        return MinusExpression::new(Some(operand))
            .ok()
            .map(|value| Arc::new(value) as Arc<dyn IStandardExpression>);
    }
    if input[0] == b'+' as u16 {
        return parse_range(&input[1..]);
    }
    if input[0] == b'!' as u16 {
        let operand = parse_range(&input[1..])?;
        return NegationExpression::new(Some(operand))
            .ok()
            .map(|value| Arc::new(value) as Arc<dyn IStandardExpression>);
    }
    if starts_with_word(input, "not") {
        let operand = parse_range(&input[3..])?;
        return NegationExpression::new(Some(operand))
            .ok()
            .map(|value| Arc::new(value) as Arc<dyn IStandardExpression>);
    }

    parse_simple(input)
}

fn parse_simple(input: &[u16]) -> Option<Arc<dyn IStandardExpression>> {
    let value = Utf16String::from_utf16(input.to_vec());
    if input[0] == b'\'' as u16 && input[input.len() - 1] == b'\'' as u16 {
        return Some(Arc::new(
            TextLiteralExpression::parse_text_literal_expression(&value),
        ));
    }
    if is_complete_selector(input, b'$' as u16) {
        return VariableExpression::parse_variable_expression(&value)
            .map(|expression| Arc::new(expression) as Arc<dyn IStandardExpression>);
    }
    if is_complete_selector(input, b'*' as u16) {
        return SelectionVariableExpression::parse_selection_variable_expression(&value)
            .map(|expression| Arc::new(expression) as Arc<dyn IStandardExpression>);
    }
    if is_complete_selector(input, b'#' as u16) {
        return parse_message(input);
    }
    if is_complete_selector(input, b'@' as u16) {
        return parse_link(input);
    }
    if is_complete_selector(input, b'~' as u16) {
        return FragmentExpression::parse_fragment_expression(Some(&value))
            .map(|expression| Arc::new(expression) as Arc<dyn IStandardExpression>);
    }
    if let Some(expression) = NumberTokenExpression::parse_number_token_expression(Some(&value)) {
        return Some(Arc::new(expression));
    }
    if let Some(expression) = BooleanTokenExpression::parse_boolean_token_expression(Some(&value)) {
        return Some(Arc::new(expression));
    }
    if let Some(expression) = NullTokenExpression::parse_null_token_expression(Some(&value)) {
        return Some(expression);
    }
    if let Some(expression) = NoOpTokenExpression::parse_no_op_token_expression(Some(&value)) {
        return Some(expression);
    }
    GenericTokenExpression::parse_generic_token_expression(Some(&value))
        .map(|expression| Arc::new(expression) as Arc<dyn IStandardExpression>)
}

fn parse_message(input: &[u16]) -> Option<Arc<dyn IStandardExpression>> {
    let content = java_trim(&input[2..input.len() - 1]);
    let (base_input, parameters_input) = split_trailing_parameters(content);
    let base = parse_link_base_default_as_literal(base_input)?;
    let parameters = match parameters_input {
        None => None,
        Some(parameters) => Some(Arc::new(parse_expression_sequence(parameters)?)),
    };
    MessageExpression::new(Some(base), parameters)
        .ok()
        .map(|value| Arc::new(value) as Arc<dyn IStandardExpression>)
}

fn parse_link(input: &[u16]) -> Option<Arc<dyn IStandardExpression>> {
    let content = java_trim(&input[2..input.len() - 1]);
    let (base_input, parameters_input) = split_trailing_parameters(content);
    let base = parse_link_base_default_as_literal(base_input)?;
    let parameters = match parameters_input {
        None => None,
        Some(parameters) => Some(Arc::new(parse_assignation_sequence(parameters, true)?)),
    };
    LinkExpression::new(Some(base), parameters)
        .ok()
        .map(|value| Arc::new(value) as Arc<dyn IStandardExpression>)
}

fn parse_link_base_default_as_literal(input: &[u16]) -> Option<Arc<dyn IStandardExpression>> {
    let input = java_trim(input);
    if input.len() >= 2 && input[0] == b'\'' as u16 && input[input.len() - 1] == b'\'' as u16 {
        // 显式文本字面量必须先按 TextLiteralExpression 解引号；路径兜底仅服务
        // `@{/path}` 这种无引号 base，不能把 `@{'/path'}` 的引号并入 URL。
        return parse_default_as_literal(input);
    }
    let contains_standard_expression = input.windows(2).any(|window| {
        matches!(
            window,
            [selector, open]
                if *open == b'{' as u16
                    && matches!(*selector, value if value == b'$' as u16
                        || value == b'*' as u16
                        || value == b'#' as u16
                        || value == b'@' as u16
                        || value == b'~' as u16)
        )
    });
    if input.first() != Some(&(b'(' as u16))
        && input.contains(&(b'/' as u16))
        && !contains_standard_expression
    {
        let literal = TextLiteralExpression::wrap_string_into_literal(Some(
            &Utf16String::from_utf16(input.to_vec()),
        ))?;
        return Some(Arc::new(
            TextLiteralExpression::parse_text_literal_expression(&literal),
        ));
    }
    parse_default_as_literal(input)
}

fn parse_default_as_literal(input: &[u16]) -> Option<Arc<dyn IStandardExpression>> {
    let input = java_trim(input);
    parse_range(input).or_else(|| {
        let literal = TextLiteralExpression::wrap_string_into_literal(Some(
            &Utf16String::from_utf16(input.to_vec()),
        ))?;
        Some(Arc::new(
            TextLiteralExpression::parse_text_literal_expression(&literal),
        ))
    })
}

fn parse_expression_sequence(input: &[u16]) -> Option<ExpressionSequence> {
    let ranges = split_top_level(input, b',' as u16)?;
    let expressions = ranges
        .into_iter()
        .map(|range| parse_range(range).map(Some))
        .collect::<Option<Vec<_>>>()?;
    ExpressionSequence::new(Some(Arc::new(RwLock::new(expressions)))).ok()
}

fn parse_assignation_sequence(
    input: &[u16],
    allow_parameters_without_value: bool,
) -> Option<AssignationSequence> {
    let ranges = split_top_level(input, b',' as u16)?;
    let mut assignations = Vec::with_capacity(ranges.len());
    for range in ranges {
        let equals = find_top_level_character(range, b'=' as u16);
        let (left, right) = match equals {
            Some(position) => {
                let left = parse_default_as_literal(&range[..position])?;
                let right_input = java_trim(&range[position + 1..]);
                let right = if right_input.is_empty() {
                    if !allow_parameters_without_value {
                        return None;
                    }
                    None
                } else {
                    Some(parse_range(right_input)?)
                };
                (left, right)
            }
            None if allow_parameters_without_value => (parse_default_as_literal(range)?, None),
            None => return None,
        };
        assignations.push(Some(Arc::new(Assignation::new(Some(left), right).ok()?)));
    }
    AssignationSequence::new(Some(Arc::new(RwLock::new(assignations)))).ok()
}

fn build_binary(
    operator: &[u16],
    left: Arc<dyn IStandardExpression>,
    right: Arc<dyn IStandardExpression>,
) -> Option<Arc<dyn IStandardExpression>> {
    let operator = String::from_utf16_lossy(operator).to_ascii_lowercase();
    macro_rules! create {
        ($kind:ty) => {
            <$kind>::new(Some(left), Some(right))
                .ok()
                .map(|value| Arc::new(value) as Arc<dyn IStandardExpression>)
        };
    }
    match operator.as_str() {
        "or" | "||" => create!(OrExpression),
        "and" | "&&" => create!(AndExpression),
        "eq" | "==" => create!(EqualsExpression),
        "neq" | "ne" | "!=" => create!(NotEqualsExpression),
        "gt" | ">" => create!(GreaterThanExpression),
        "gte" | "ge" | ">=" => create!(GreaterOrEqualToExpression),
        "lt" | "<" => create!(LessThanExpression),
        "lte" | "le" | "<=" => create!(LessOrEqualToExpression),
        "+" => create!(AdditionExpression),
        "-" => create!(SubtractionExpression),
        "*" => create!(MultiplicationExpression),
        "div" | "/" => create!(DivisionExpression),
        "mod" | "%" => create!(RemainderExpression),
        _ => None,
    }
}

fn find_conditional(input: &[u16]) -> Option<(usize, Option<usize>)> {
    let mut question = None;
    let mut colon = None;
    scan_top_level(input, |position, unit| {
        if unit == b'?' as u16
            && next_non_whitespace(input, position + 1)
                .is_none_or(|next| input[next] != b':' as u16)
            && question.is_none()
        {
            question = Some(position);
        } else if unit == b':' as u16 && question.is_some() && colon.is_none() {
            colon = Some(position);
        }
    });
    question.map(|position| (position, colon))
}

fn find_default_operator(input: &[u16]) -> Option<(usize, usize)> {
    let mut found = None;
    scan_top_level(input, |position, unit| {
        if found.is_none()
            && unit == b'?' as u16
            && let Some(colon) = next_non_whitespace(input, position + 1)
            && input[colon] == b':' as u16
        {
            found = Some((position, colon));
        }
    });
    found
}

fn next_non_whitespace(input: &[u16], mut position: usize) -> Option<usize> {
    while input.get(position).is_some_and(|unit| *unit <= 0x20) {
        position += 1;
    }
    (position < input.len()).then_some(position)
}

fn find_binary_operator<'a>(
    input: &[u16],
    operators: &'a [&'a [u16]],
) -> Option<(usize, &'a [u16])> {
    let mut found = None;
    scan_top_level(input, |position, _| {
        for operator in operators {
            if position + operator.len() <= input.len()
                && eq_ignore_ascii_case(&input[position..position + operator.len()], operator)
                && operator_boundary(input, position, operator)
                && !((operator == &OP_MINUS || operator == &OP_PLUS)
                    && is_unary_sign_position(input, position))
            {
                let replace =
                    found
                        .as_ref()
                        .is_none_or(|(old_position, old_operator): &(usize, &[u16])| {
                            position > *old_position
                                || (position == *old_position
                                    && operator.len() > old_operator.len())
                        });
                if replace {
                    found = Some((position, *operator));
                }
            }
        }
    });
    found.filter(|(position, operator)| {
        !java_trim(&input[..*position]).is_empty()
            && !java_trim(&input[*position + operator.len()..]).is_empty()
    })
}

fn is_unary_sign_position(input: &[u16], position: usize) -> bool {
    let prefix = java_trim(&input[..position]);
    let Some(last) = prefix.last().copied() else {
        return true;
    };
    if [
        b'+' as u16,
        b'-' as u16,
        b'*' as u16,
        b'/' as u16,
        b'%' as u16,
        b'<' as u16,
        b'>' as u16,
        b'=' as u16,
        b'!' as u16,
        b'&' as u16,
        b'|' as u16,
        b'?' as u16,
        b':' as u16,
        b',' as u16,
        b'(' as u16,
        b'[' as u16,
        b'{' as u16,
    ]
    .contains(&last)
    {
        return true;
    }
    [OP_DIV, OP_MOD, OP_AND, OP_OR].iter().any(|operator| {
        prefix.len() >= operator.len()
            && eq_ignore_ascii_case(&prefix[prefix.len() - operator.len()..], operator)
            && prefix
                .get(prefix.len().saturating_sub(operator.len() + 1))
                .is_none_or(|unit| !is_word_unit(*unit))
    })
}

fn operator_boundary(input: &[u16], position: usize, operator: &[u16]) -> bool {
    if operator.iter().all(|unit| is_ascii_alphabetic(*unit)) {
        let before = position.checked_sub(1).and_then(|index| input.get(index));
        let after = input.get(position + operator.len());
        return before.is_none_or(|unit| !is_word_unit(*unit))
            && after.is_none_or(|unit| !is_word_unit(*unit));
    }
    if operator == OP_MINUS {
        // Java 的 TokenParsingTracer 会先判断连字符是否属于 token；例如
        // `data-id` 是一个 GenericToken，而 `10-2` 才是减法表达式。
        let context = Utf16String::from_utf16(input.to_vec());
        return !Token::<Utf16String>::is_token_char(
            Some(&context),
            i32::try_from(position).unwrap_or(i32::MAX),
        )
        .unwrap_or(false);
    }
    true
}

fn split_trailing_parameters(input: &[u16]) -> (&[u16], Option<&[u16]>) {
    if input.last() != Some(&(b')' as u16)) {
        return (input, None);
    }
    let mut level = 0_i32;
    let mut in_literal = false;
    for position in (0..input.len()).rev() {
        let unit = input[position];
        if unit == b'\'' as u16 && !is_escaped(input, position) {
            in_literal = !in_literal;
        } else if !in_literal && unit == b')' as u16 {
            level += 1;
        } else if !in_literal && unit == b'(' as u16 {
            level -= 1;
            if level == 0 && position > 0 {
                return (
                    java_trim(&input[..position]),
                    Some(java_trim(&input[position + 1..input.len() - 1])),
                );
            }
        }
    }
    (input, None)
}

fn split_top_level(input: &[u16], separator: u16) -> Option<Vec<&[u16]>> {
    let mut result = Vec::new();
    let mut start = 0;
    scan_top_level(input, |position, unit| {
        if unit == separator {
            result.push(java_trim(&input[start..position]));
            start = position + 1;
        }
    });
    result.push(java_trim(&input[start..]));
    (!result.iter().any(|part| part.is_empty())).then_some(result)
}

fn find_top_level_character(input: &[u16], character: u16) -> Option<usize> {
    let mut found = None;
    scan_top_level(input, |position, unit| {
        if unit == character && found.is_none() {
            found = Some(position);
        }
    });
    found
}

fn scan_top_level(input: &[u16], mut visitor: impl FnMut(usize, u16)) {
    let mut parenthesis = 0_i32;
    let mut braces = 0_i32;
    let mut in_literal = false;
    for (position, unit) in input.iter().copied().enumerate() {
        if unit == b'\'' as u16 && !is_escaped(input, position) {
            in_literal = !in_literal;
            continue;
        }
        if in_literal {
            continue;
        }
        match unit {
            value if value == b'{' as u16 => braces += 1,
            value if value == b'}' as u16 => braces -= 1,
            value if value == b'(' as u16 && braces == 0 => parenthesis += 1,
            value if value == b')' as u16 && braces == 0 => parenthesis -= 1,
            _ if braces == 0 && parenthesis == 0 => visitor(position, unit),
            _ => {}
        }
    }
}

fn is_outer_parenthesized(input: &[u16]) -> bool {
    if input.first() != Some(&(b'(' as u16)) || input.last() != Some(&(b')' as u16)) {
        return false;
    }
    let mut level = 0_i32;
    let mut in_literal = false;
    for (position, unit) in input.iter().copied().enumerate() {
        if unit == b'\'' as u16 && !is_escaped(input, position) {
            in_literal = !in_literal;
        } else if !in_literal && unit == b'(' as u16 {
            level += 1;
        } else if !in_literal && unit == b')' as u16 {
            level -= 1;
            if level == 0 && position + 1 != input.len() {
                return false;
            }
        }
    }
    level == 0 && !in_literal
}

fn is_complete_selector(input: &[u16], selector: u16) -> bool {
    input.len() >= 3
        && input[0] == selector
        && input[1] == b'{' as u16
        && input[input.len() - 1] == b'}' as u16
}

fn starts_with_word(input: &[u16], word: &str) -> bool {
    let word = word.as_bytes();
    input.len() > word.len()
        && input[..word.len()]
            .iter()
            .zip(word)
            .all(|(left, right)| ascii_lower(*left) == u16::from(right.to_ascii_lowercase()))
        && !is_word_unit(input[word.len()])
}

fn eq_ignore_ascii_case(left: &[u16], right: &[u16]) -> bool {
    left.iter()
        .zip(right)
        .all(|(left, right)| ascii_lower(*left) == ascii_lower(*right))
}

fn is_word_unit(unit: u16) -> bool {
    is_ascii_alphabetic(unit) || (b'0' as u16..=b'9' as u16).contains(&unit) || unit == b'_' as u16
}

fn is_ascii_alphabetic(unit: u16) -> bool {
    (b'a' as u16..=b'z' as u16).contains(&unit) || (b'A' as u16..=b'Z' as u16).contains(&unit)
}

fn ascii_lower(unit: u16) -> u16 {
    if (b'A' as u16..=b'Z' as u16).contains(&unit) {
        unit + u16::from(b'a' - b'A')
    } else {
        unit
    }
}

fn is_escaped(input: &[u16], position: usize) -> bool {
    let mut slash_count = 0;
    let mut current = position;
    while current > 0 && input[current - 1] == b'\\' as u16 {
        slash_count += 1;
        current -= 1;
    }
    slash_count % 2 == 1
}

fn java_trim(input: &[u16]) -> &[u16] {
    let mut start = 0;
    while start < input.len() && input[start] <= 0x20 {
        start += 1;
    }
    let mut end = input.len();
    while end > start && input[end - 1] <= 0x20 {
        end -= 1;
    }
    &input[start..end]
}

const OP_OR: &[u16] = &[b'o' as u16, b'r' as u16];
const OP_DOUBLE_PIPE: &[u16] = &[b'|' as u16, b'|' as u16];
const OP_AND: &[u16] = &[b'a' as u16, b'n' as u16, b'd' as u16];
const OP_DOUBLE_AMPERSAND: &[u16] = &[b'&' as u16, b'&' as u16];
const OP_NEQ: &[u16] = &[b'n' as u16, b'e' as u16, b'q' as u16];
const OP_NE: &[u16] = &[b'n' as u16, b'e' as u16];
const OP_NOT_EQUALS: &[u16] = &[b'!' as u16, b'=' as u16];
const OP_EQ: &[u16] = &[b'e' as u16, b'q' as u16];
const OP_EQUALS: &[u16] = &[b'=' as u16, b'=' as u16];
const OP_GTE: &[u16] = &[b'g' as u16, b't' as u16, b'e' as u16];
const OP_GE: &[u16] = &[b'g' as u16, b'e' as u16];
const OP_GREATER_EQUAL: &[u16] = &[b'>' as u16, b'=' as u16];
const OP_GT: &[u16] = &[b'g' as u16, b't' as u16];
const OP_GREATER: &[u16] = &[b'>' as u16];
const OP_LTE: &[u16] = &[b'l' as u16, b't' as u16, b'e' as u16];
const OP_LE: &[u16] = &[b'l' as u16, b'e' as u16];
const OP_LESS_EQUAL: &[u16] = &[b'<' as u16, b'=' as u16];
const OP_LT: &[u16] = &[b'l' as u16, b't' as u16];
const OP_LESS: &[u16] = &[b'<' as u16];
const OP_PLUS: &[u16] = &[b'+' as u16];
const OP_MINUS: &[u16] = &[b'-' as u16];
const OP_MULTIPLY: &[u16] = &[b'*' as u16];
const OP_DIV: &[u16] = &[b'd' as u16, b'i' as u16, b'v' as u16];
const OP_DIVIDE: &[u16] = &[b'/' as u16];
const OP_MOD: &[u16] = &[b'm' as u16, b'o' as u16, b'd' as u16];
const OP_REMAINDER: &[u16] = &[b'%' as u16];
