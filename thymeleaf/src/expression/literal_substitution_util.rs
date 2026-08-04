use crate::util::Utf16String;

/// 将 `|...|` 字面量替换语法转换为标准连接表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.LiteralSubstitutionUtil`。
pub(crate) struct LiteralSubstitutionUtil;

impl LiteralSubstitutionUtil {
    /// 执行字面量替换；不包含替换定界符时保留输入值。
    /// 对应 Java: `LiteralSubstitutionUtil#performLiteralSubstitution()`。
    pub(crate) fn perform_literal_substitution(input: Option<&Utf16String>) -> Option<Utf16String> {
        Self::perform_literal_substitution_inner(input, 0)
    }

    /// 嵌套 selector 递归处理的最大深度。
    ///
    /// Java 上游是零递归的单遍状态机；本递归是为 `@{|/orders/${id}|}` 这类
    /// "字面量替换嵌在另一个 simple expression 内"的场景增加的 Rust 侧辅助
    /// （等价于 Java 对 link 内容二次调用 substitution）。深度上限保证病态
    /// 嵌套输入绝对终止，不改变合法输入的处理结果。
    const MAX_SUBSTITUTION_DEPTH: usize = 16;

    fn perform_literal_substitution_inner(
        input: Option<&Utf16String>,
        depth: usize,
    ) -> Option<Utf16String> {
        let input = input?;
        if depth >= Self::MAX_SUBSTITUTION_DEPTH {
            return Some(input.clone());
        }
        let units = input.as_utf16();

        // Java 允许字面量替换出现在另一个 simple expression 内，例如
        // `@{|/orders/${id}|}`。先递归处理最外层 selector 的内容，否则状态机会把
        // 整个 `@{...}` 当成一个不可进入的插值区间而漏掉内层 `|...|`。
        if is_complete_outer_selector(units) && units[2..units.len() - 1].contains(&(b'|' as u16)) {
            let content = Utf16String::from_utf16(units[2..units.len() - 1].to_vec());
            let substituted = Self::perform_literal_substitution_inner(Some(&content), depth + 1)
                .expect("non-null selector content remains non-null");
            if substituted != content {
                let mut nested = Vec::with_capacity(substituted.len() + 3);
                nested.extend_from_slice(&units[..2]);
                nested.extend_from_slice(substituted.as_utf16());
                nested.push(b'}' as u16);
                return Some(Utf16String::from_utf16(nested));
            }
        }
        if let Some((start, end)) = find_nested_selector_with_substitution(units) {
            let selector = Utf16String::from_utf16(units[start..=end].to_vec());
            let substituted = Self::perform_literal_substitution_inner(Some(&selector), depth + 1)
                .expect("non-null nested selector remains non-null");
            if substituted != selector {
                let mut nested = Vec::with_capacity(units.len() + substituted.len());
                nested.extend_from_slice(&units[..start]);
                nested.extend_from_slice(substituted.as_utf16());
                nested.extend_from_slice(&units[end + 1..]);
                return Self::perform_literal_substitution_inner(
                    Some(&Utf16String::from_utf16(nested)),
                    depth + 1,
                );
            }
            // substituted == selector：递归不会产生任何变化（例如 `${${||}}` 的
            // 空字面量 `||` 按 Java 语义原样保留），此时以相同输入递归会无限
            // 循环，落到主状态机直接处理（与 Java 单遍状态机行为一致）。
        }

        let mut output: Option<Vec<u16>> = None;
        let mut in_substitution = false;
        let mut in_insertion = false;
        let mut substitution_index = usize::MAX;
        let mut expression_level = 0_i32;
        let mut in_literal = false;
        let mut in_nothing = true;
        let mut index = 0;

        while index < units.len() {
            let current = units[index];
            if current == b'|' as u16 && !in_substitution && in_nothing {
                output.get_or_insert_with(|| units[..index].to_vec());
                in_substitution = true;
                substitution_index = index;
            } else if current == b'|' as u16 && in_substitution && in_nothing {
                let target = output.as_mut().expect("substitution initializes output");
                if index - substitution_index == 1 {
                    target.extend_from_slice(&[b'|' as u16, b'|' as u16]);
                } else if in_insertion {
                    target.push(b'\'' as u16);
                    in_insertion = false;
                }
                in_substitution = false;
                substitution_index = usize::MAX;
            } else if in_nothing
                && matches!(current, value if value == b'$' as u16 || value == b'*' as u16 || value == b'#' as u16 || value == b'@' as u16)
                && units.get(index + 1) == Some(&(b'{' as u16))
            {
                if in_substitution && in_insertion {
                    output
                        .as_mut()
                        .expect("substitution initializes output")
                        .extend(" ' + ".trim_start().encode_utf16());
                    in_insertion = false;
                } else if in_substitution && index > 0 && units[index - 1] == b'}' as u16 {
                    output
                        .as_mut()
                        .expect("substitution initializes output")
                        .extend(" + '' + ".encode_utf16());
                }
                if let Some(target) = output.as_mut() {
                    target.extend_from_slice(&[current, b'{' as u16]);
                }
                expression_level = 1;
                index += 1;
                in_nothing = false;
            } else if expression_level == 1 && current == b'}' as u16 {
                if let Some(target) = output.as_mut() {
                    target.push(current);
                }
                expression_level = 0;
                in_nothing = true;
            } else if expression_level > 0 && current == b'{' as u16 {
                if let Some(target) = output.as_mut() {
                    target.push(current);
                }
                expression_level += 1;
            } else if expression_level > 1 && current == b'}' as u16 {
                if let Some(target) = output.as_mut() {
                    target.push(current);
                }
                expression_level -= 1;
            } else if expression_level > 0 {
                if let Some(target) = output.as_mut() {
                    target.push(current);
                }
            } else if in_nothing
                && !in_substitution
                && current == b'\'' as u16
                && !is_escaped(units, index)
            {
                in_nothing = false;
                in_literal = true;
                if let Some(target) = output.as_mut() {
                    target.push(current);
                }
            } else if in_literal
                && !in_substitution
                && current == b'\'' as u16
                && !is_escaped(units, index)
            {
                in_literal = false;
                in_nothing = true;
                if let Some(target) = output.as_mut() {
                    target.push(current);
                }
            } else if in_substitution && in_nothing {
                let target = output.as_mut().expect("substitution initializes output");
                if !in_insertion {
                    if units[index - 1] != b'|' as u16 {
                        target.extend(" + ".encode_utf16());
                    }
                    target.push(b'\'' as u16);
                    in_insertion = true;
                }
                if matches!(current, value if value == b'\'' as u16 || value == b'\\' as u16) {
                    target.push(b'\\' as u16);
                }
                target.push(current);
            } else if let Some(target) = output.as_mut() {
                target.push(current);
            }
            index += 1;
        }

        Some(output.map_or_else(|| input.clone(), Utf16String::from_utf16))
    }
}

fn is_complete_outer_selector(input: &[u16]) -> bool {
    input.len() >= 3
        && matches!(
            input[0],
            value if value == b'$' as u16
                || value == b'*' as u16
                || value == b'#' as u16
                || value == b'@' as u16
                || value == b'~' as u16
        )
        && input[1] == b'{' as u16
        && input[input.len() - 1] == b'}' as u16
}

fn find_nested_selector_with_substitution(input: &[u16]) -> Option<(usize, usize)> {
    let mut start = 1_usize;
    while start + 2 < input.len() {
        if matches!(
            input[start],
            value if value == b'$' as u16
                || value == b'*' as u16
                || value == b'#' as u16
                || value == b'@' as u16
                || value == b'~' as u16
        ) && input[start + 1] == b'{' as u16
        {
            let mut level = 1_i32;
            let mut position = start + 2;
            while position < input.len() {
                match input[position] {
                    value if value == b'{' as u16 => level += 1,
                    value if value == b'}' as u16 => {
                        level -= 1;
                        if level == 0 {
                            if input[start + 2..position].contains(&(b'|' as u16)) {
                                return Some((start, position));
                            }
                            break;
                        }
                    }
                    _ => {}
                }
                position += 1;
            }
        }
        start += 1;
    }
    None
}

fn is_escaped(input: &[u16], position: usize) -> bool {
    let mut slashes = 0;
    let mut index = position;
    while index > 0 && input[index - 1] == b'\\' as u16 {
        slashes += 1;
        index -= 1;
    }
    slashes % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::{LiteralSubstitutionUtil, Utf16String};

    #[test]
    fn empty_literal_substitution_inside_nested_selector_terminates() {
        // 回归：`${${||}}` 曾因 substituted == selector 以完全相同输入无限递归
        // （render smoke fuzz 超时根因）。Java 状态机对空字面量 `||` 原样保留，
        // 结果必须与输入一致且立即返回。
        let input = Utf16String::from_rust_str("${${||}}");
        let result =
            LiteralSubstitutionUtil::perform_literal_substitution(Some(&input)).expect("non-null");
        assert_eq!(result, input);
    }

    #[test]
    fn link_expression_literal_substitution_still_processed() {
        // 守卫不得破坏 `@{|...|}` 的正常处理（等价 Java 对 link 内容二次
        // substitution 的收敛结果）。
        let input = Utf16String::from_rust_str("@{|/orders/${id}|}");
        let result =
            LiteralSubstitutionUtil::perform_literal_substitution(Some(&input)).expect("non-null");
        assert_eq!(result.to_string_lossy(), "@{'/orders/' + ${id}}");
    }

    #[test]
    fn basic_literal_substitution_matches_java_doc_example() {
        // Java 文档示例：|${onevar} ${twovar}| --> ${onevar} + ' ' + ${twovar}
        let input = Utf16String::from_rust_str("|${onevar} ${twovar}|");
        let result =
            LiteralSubstitutionUtil::perform_literal_substitution(Some(&input)).expect("non-null");
        assert_eq!(result.to_string_lossy(), "${onevar} + ' ' + ${twovar}");
    }

    #[test]
    fn pathological_nesting_beyond_depth_cap_terminates() {
        // 深度上限：20 层嵌套超过 MAX_SUBSTITUTION_DEPTH=16，必须原样返回。
        let nested = format!("{}||{}", "${".repeat(20), "}".repeat(20));
        let input = Utf16String::from_rust_str(&nested);
        let result =
            LiteralSubstitutionUtil::perform_literal_substitution(Some(&input)).expect("non-null");
        assert_eq!(result, input);
    }
}
