use crate::util::JavaString;

/// 将 `|...|` 字面量替换语法转换为标准连接表达式。
///
/// 对应 Java: `org.thymeleaf.standard.expression.LiteralSubstitutionUtil`。
pub(crate) struct LiteralSubstitutionUtil;

impl LiteralSubstitutionUtil {
    /// 执行字面量替换；不包含替换定界符时保留输入值。
    pub(crate) fn perform_literal_substitution(input: Option<&JavaString>) -> Option<JavaString> {
        let input = input?;
        let units = input.as_utf16();
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

fn is_escaped(input: &[u16], position: usize) -> bool {
    let mut slashes = 0;
    let mut index = position;
    while index > 0 && input[index - 1] == b'\\' as u16 {
        slashes += 1;
        index -= 1;
    }
    slashes % 2 == 1
}
