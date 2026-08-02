use crate::util::JavaString;

/// 将 `|...|` 字面量替换语法转换为标准连接表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.LiteralSubstitutionUtil`。
pub(crate) struct LiteralSubstitutionUtil;

impl LiteralSubstitutionUtil {
    /// 执行字面量替换；不包含替换定界符时保留输入值。
    /// 对应 Java: `LiteralSubstitutionUtil#performLiteralSubstitution()`。
    pub(crate) fn perform_literal_substitution(input: Option<&JavaString>) -> Option<JavaString> {
        let input = input?;
        let units = input.as_utf16();

        // Java 允许字面量替换出现在另一个 simple expression 内，例如
        // `@{|/orders/${id}|}`。先递归处理最外层 selector 的内容，否则状态机会把
        // 整个 `@{...}` 当成一个不可进入的插值区间而漏掉内层 `|...|`。
        if is_complete_outer_selector(units) && units[2..units.len() - 1].contains(&(b'|' as u16)) {
            let content = JavaString::from_utf16(units[2..units.len() - 1].to_vec());
            let substituted = Self::perform_literal_substitution(Some(&content))
                .expect("non-null selector content remains non-null");
            if substituted != content {
                let mut nested = Vec::with_capacity(substituted.len() + 3);
                nested.extend_from_slice(&units[..2]);
                nested.extend_from_slice(substituted.as_utf16());
                nested.push(b'}' as u16);
                return Some(JavaString::from_utf16(nested));
            }
        }
        if let Some((start, end)) = find_nested_selector_with_substitution(units) {
            let selector = JavaString::from_utf16(units[start..=end].to_vec());
            let substituted = Self::perform_literal_substitution(Some(&selector))
                .expect("non-null nested selector remains non-null");
            let mut nested = Vec::with_capacity(units.len() + substituted.len());
            nested.extend_from_slice(&units[..start]);
            nested.extend_from_slice(substituted.as_utf16());
            nested.extend_from_slice(&units[end + 1..]);
            return Self::perform_literal_substitution(Some(&JavaString::from_utf16(nested)));
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

        Some(output.map_or_else(|| input.clone(), JavaString::from_utf16))
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
