use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use thiserror::Error;

/// 表达式缓存使用的不可变复合键。
///
/// 对应 Java: `org.thymeleaf.cache.ExpressionCacheKey`。
///
/// 键由表达式类型、第一表达式和可选第二表达式构成。构造时预计算 Java
/// `String.hashCode()` 组合值，后续相等和哈希操作可先快速比较该值，再比较全部字段。
/// 本对象应只由模板引擎内部创建。
#[derive(Clone)]
pub struct ExpressionCacheKey {
    expression_type: String,
    expression0: String,
    expression1: Option<String>,
    hash_code: i32,
}

impl ExpressionCacheKey {
    /// 使用类型和第一表达式创建缓存键。
    ///
    /// 对应 Java: `ExpressionCacheKey#ExpressionCacheKey(String, String)`。
    ///
    /// # 参数
    /// - `expression_type`：Java 参数 `type`；`None` 对应 Java `null`。
    /// - `expression0`：Java 参数 `expression0`；`None` 对应 Java `null`。
    ///
    /// # 错误
    /// 类型或第一表达式为 `None` 时返回对应的类型化校验错误。
    pub fn new(
        expression_type: Option<&str>,
        expression0: Option<&str>,
    ) -> Result<Self, ExpressionCacheKeyError> {
        Self::with_expression1(expression_type, expression0, None)
    }

    /// 使用类型、第一表达式和可选第二表达式创建缓存键。
    ///
    /// 对应 Java: `ExpressionCacheKey#ExpressionCacheKey(String, String, String)`。
    ///
    /// # 参数
    /// - `expression_type`：Java 参数 `type`；不能为 `None`，空字符串允许。
    /// - `expression0`：Java 参数 `expression0`；不能为 `None`，空字符串允许。
    /// - `expression1`：Java 参数 `expression1`；可以为 `None` 或空字符串。
    ///
    /// # 错误
    /// 类型或第一表达式为 `None` 时返回对应的类型化校验错误。
    pub fn with_expression1(
        expression_type: Option<&str>,
        expression0: Option<&str>,
        expression1: Option<&str>,
    ) -> Result<Self, ExpressionCacheKeyError> {
        let expression_type = expression_type.ok_or(ExpressionCacheKeyError::TypeCannotBeNull)?;
        let expression0 = expression0.ok_or(ExpressionCacheKeyError::ExpressionCannotBeNull)?;
        let hash_code = compute_hash_code(expression_type, expression0, expression1);
        Ok(Self {
            expression_type: expression_type.to_owned(),
            expression0: expression0.to_owned(),
            expression1: expression1.map(str::to_owned),
            hash_code,
        })
    }

    /// 返回表达式缓存的类型区分符。
    ///
    /// 对应 Java: `ExpressionCacheKey#getType()`。
    ///
    /// # 返回
    /// 构造时传入的非空类型字符串。
    #[must_use]
    pub fn get_type(&self) -> &str {
        &self.expression_type
    }

    /// 返回第一表达式。
    ///
    /// 对应 Java: `ExpressionCacheKey#getExpression0()`。
    ///
    /// # 返回
    /// 构造时传入的非空第一表达式。
    #[must_use]
    pub fn get_expression0(&self) -> &str {
        &self.expression0
    }

    /// 返回可选第二表达式。
    ///
    /// 对应 Java: `ExpressionCacheKey#getExpression1()`。
    ///
    /// # 返回
    /// 第二表达式；`None` 对应 Java `null`。
    #[must_use]
    pub fn get_expression1(&self) -> Option<&str> {
        self.expression1.as_deref()
    }

    /// 返回与 Java `hashCode()` 完全相同的预计算值。
    ///
    /// 对应 Java: `ExpressionCacheKey#hashCode()`。
    ///
    /// # 返回
    /// 基于 UTF-16 代码单元和 Java `int` 环绕乘加得到的哈希值。
    #[must_use]
    pub const fn hash_code(&self) -> i32 {
        self.hash_code
    }
}

impl PartialEq for ExpressionCacheKey {
    fn eq(&self, other: &Self) -> bool {
        if std::ptr::eq(self, other) {
            return true;
        }
        if self.hash_code != other.hash_code {
            return false;
        }
        if self.expression_type != other.expression_type {
            return false;
        }
        if self.expression0 != other.expression0 {
            return false;
        }
        self.expression1 == other.expression1
    }
}

impl Eq for ExpressionCacheKey {}

impl Hash for ExpressionCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.hash_code);
    }
}

impl Display for ExpressionCacheKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}|{}", self.expression_type, self.expression0)?;
        if let Some(expression1) = &self.expression1 {
            write!(formatter, "|{expression1}")?;
        }
        Ok(())
    }
}

/// 创建 `ExpressionCacheKey` 时的参数校验错误。
///
/// 对应 Java: `org.thymeleaf.cache.ExpressionCacheKey` 构造器抛出的
/// `IllegalArgumentException`。该类型是 Rust 类型化错误扩展，不计入 Java
/// 对象迁移分子。
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ExpressionCacheKeyError {
    /// 表达式类型对应 Java `null`。
    #[error("Type cannot be null")]
    TypeCannotBeNull,
    /// 第一表达式对应 Java `null`。
    #[error("Expression cannot be null")]
    ExpressionCannotBeNull,
}

fn compute_hash_code(expression_type: &str, expression0: &str, expression1: Option<&str>) -> i32 {
    let mut result = utf16_string_hash_code(expression_type);
    result = result
        .wrapping_mul(31)
        .wrapping_add(utf16_string_hash_code(expression0));
    result = result
        .wrapping_mul(31)
        .wrapping_add(expression1.map(utf16_string_hash_code).unwrap_or(0));
    result
}

fn utf16_string_hash_code(value: &str) -> i32 {
    value.encode_utf16().fold(0_i32, |hash, code_unit| {
        hash.wrapping_mul(31).wrapping_add(i32::from(code_unit))
    })
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::fmt::Write;
    use std::hash::{Hash, Hasher};

    use super::{ExpressionCacheKey, ExpressionCacheKeyError, utf16_string_hash_code};

    fn rust_hash(value: &ExpressionCacheKey) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    struct FailingWriter {
        remaining_writes: usize,
    }

    impl Write for FailingWriter {
        fn write_str(&mut self, value: &str) -> std::fmt::Result {
            if self.remaining_writes == 0 {
                return Err(std::fmt::Error);
            }
            self.remaining_writes -= 1;
            let _ = value;
            Ok(())
        }
    }

    #[test]
    fn validates_required_fields_and_allows_empty_strings() {
        assert_eq!(
            ExpressionCacheKey::new(None, Some("x")).err(),
            Some(ExpressionCacheKeyError::TypeCannotBeNull)
        );
        assert_eq!(
            ExpressionCacheKey::new(Some("TYPE"), None).err(),
            Some(ExpressionCacheKeyError::ExpressionCannotBeNull)
        );
        assert_eq!(
            ExpressionCacheKeyError::TypeCannotBeNull.to_string(),
            "Type cannot be null"
        );
        assert_eq!(
            ExpressionCacheKeyError::ExpressionCannotBeNull.to_string(),
            "Expression cannot be null"
        );

        let empty = ExpressionCacheKey::with_expression1(Some(""), Some(""), Some(""))
            .expect("empty strings are legal");
        assert_eq!(empty.get_type(), "");
        assert_eq!(empty.get_expression0(), "");
        assert_eq!(empty.get_expression1(), Some(""));
        assert_eq!(empty.to_string(), "||");
    }

    #[test]
    fn exposes_fields_display_and_exact_java_utf16_hash() {
        let key =
            ExpressionCacheKey::new(Some("EXPRESSION"), Some("😀")).expect("valid expression key");

        assert_eq!(key.get_type(), "EXPRESSION");
        assert_eq!(key.get_expression0(), "😀");
        assert_eq!(key.get_expression1(), None);
        assert_eq!(key.to_string(), "EXPRESSION|😀");
        assert_eq!(utf16_string_hash_code("😀"), 1_772_899);
        assert_eq!(key.hash_code(), -775_497_835);
    }

    #[test]
    fn equality_checks_identity_cached_hash_and_every_field() {
        let left = ExpressionCacheKey::with_expression1(Some("T"), Some("A"), Some("B"))
            .expect("valid key");
        assert!(left == left);

        let different_hash = ExpressionCacheKey::new(Some("T"), Some("A")).expect("valid key");
        assert!(left != different_hash);

        let mut different_type =
            ExpressionCacheKey::with_expression1(Some("X"), Some("A"), Some("B"))
                .expect("valid key");
        different_type.hash_code = left.hash_code;
        assert!(left != different_type);

        let mut different_expression0 =
            ExpressionCacheKey::with_expression1(Some("T"), Some("X"), Some("B"))
                .expect("valid key");
        different_expression0.hash_code = left.hash_code;
        assert!(left != different_expression0);

        let mut different_expression1 =
            ExpressionCacheKey::with_expression1(Some("T"), Some("A"), Some("X"))
                .expect("valid key");
        different_expression1.hash_code = left.hash_code;
        assert!(left != different_expression1);

        let equal = ExpressionCacheKey::with_expression1(Some("T"), Some("A"), Some("B"))
            .expect("valid key");
        assert!(left == equal);
        assert_eq!(rust_hash(&left), rust_hash(&equal));
    }

    #[test]
    fn display_propagates_formatter_failures_from_each_segment() {
        let key = ExpressionCacheKey::with_expression1(Some("TYPE"), Some("A"), Some("B"))
            .expect("valid key");

        for remaining_writes in 0..8 {
            let mut writer = FailingWriter { remaining_writes };
            let _ = write!(&mut writer, "{key}");
        }
    }
}
