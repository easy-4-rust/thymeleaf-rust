use crate::util::ExpressionUtils;

/// Standard Expression 的快速扫描与受限模式安全检查。
///
/// 对应 Java: `org.thymeleaf.standard.util.StandardExpressionUtils`。
pub struct StandardExpressionUtils;

impl StandardExpressionUtils {
    /// 判断表达式是否可能引用 `#expressionObject`。
    /// 对应 Java: `StandardExpressionUtils#mightNeedExpressionObjects()`。
    #[must_use]
    pub fn might_need_expression_objects(expression: &str) -> bool {
        expression.contains('#')
    }

    /// 检测 OGNL `new`、`param` 或 `@Type@` 外部访问语法。
    /// 对应 Java: `StandardExpressionUtils#containsExternalAccess()`。
    #[must_use]
    pub fn contains_external_access(expression: &str) -> bool {
        let expression =
            ExpressionUtils::normalize(Some(expression), true).expect("non-null expression");
        let chars = expression.chars().collect::<Vec<_>>();
        for index in 0..chars.len() {
            if matches_keyword(&chars, index, "new", true)
                || matches_keyword(&chars, index, "param", false)
            {
                return true;
            }
        }
        let mut opening = None;
        for (index, character) in chars.iter().enumerate() {
            if *character == '@' {
                if opening.take().is_some() {
                    return true;
                }
                opening = Some(index);
            } else if let Some(start) = opening
                && index > start
                && !(character.is_alphanumeric()
                    || character.is_whitespace()
                    || matches!(character, '_' | '$' | '.'))
            {
                opening = None;
            }
        }
        false
    }
}

fn matches_keyword(
    chars: &[char],
    index: usize,
    keyword: &str,
    requires_space_after: bool,
) -> bool {
    let keyword = keyword.chars().collect::<Vec<_>>();
    if chars.get(index..index + keyword.len()) != Some(keyword.as_slice()) {
        return false;
    }
    let before_safe = index > 0 && is_safe_identifier_char(chars[index - 1]);
    let after = chars.get(index + keyword.len()).copied();
    let after_valid = if requires_space_after {
        after.is_some_and(char::is_whitespace)
    } else {
        after.is_none_or(|character| !is_safe_identifier_char(character))
    };
    !before_safe && after_valid
}

fn is_safe_identifier_char(character: char) -> bool {
    character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
}
