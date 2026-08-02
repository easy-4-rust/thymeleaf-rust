use std::sync::Arc;
use thiserror::Error;

use crate::util::JavaString;

/// Token 值执行 Java `toString()` 时的可空、借用或新建结果。
///
/// 该适配保留 `String#toString()` 返回原实例，以及自定义对象返回共享字符串的
/// 引用身份；Java 允许覆写 `toString()` 后返回 null。
pub enum JavaTokenStringResult<'a> {
    /// Java null。
    Null,
    /// 借用的既有 Java 字符串。
    Borrowed(&'a JavaString),
    /// 新建 Java 字符串。
    Owned(JavaString),
}

/// 可被 `Token` 保存并按 Java 规则转换为字符串的值。
pub trait JavaTokenValue {
    /// 执行 Java `Object#toString()` 等价操作。
    ///
    /// # 返回
    /// 可空、借用或新建的 UTF-16 字符串。
    ///
    /// # 错误
    /// 自定义 `toString()` 抛出的运行时异常必须保留类别和消息。
    fn java_token_to_string(&self) -> Result<JavaTokenStringResult<'_>, TokenError>;
}

impl JavaTokenValue for JavaString {
    fn java_token_to_string(&self) -> Result<JavaTokenStringResult<'_>, TokenError> {
        Ok(JavaTokenStringResult::Borrowed(self))
    }
}

impl<T: JavaTokenValue> JavaTokenValue for Arc<T> {
    fn java_token_to_string(&self) -> Result<JavaTokenStringResult<'_>, TokenError> {
        self.as_ref().java_token_to_string()
    }
}

/// `Token` 操作中可观察的 Java 异常。
#[derive(Debug, Error, Eq, PartialEq)]
pub enum TokenError {
    /// 对 null String 或 null token 值调用实例方法。
    #[error("")]
    NullPointer,
    /// `String#charAt(int)` 的索引越界。
    #[error("String index out of range: {position}")]
    StringIndexOutOfBounds {
        /// Java 参数 `pos`。
        position: i32,
    },
    /// Token 值的自定义 `toString()` 抛出运行时异常。
    #[error("{message}")]
    Runtime {
        /// 原 Java 异常类名。
        exception_class_name: String,
        /// 原 Java detail message；null 用空字符串表达。
        message: String,
    },
}

impl TokenError {
    /// 返回对应 Java 异常类名。
    ///
    /// # 返回
    /// NullPointer、StringIndexOutOfBounds 或保存的运行时异常类名。
    #[must_use]
    pub fn java_class_name(&self) -> &str {
        match self {
            Self::NullPointer => "java.lang.NullPointerException",
            Self::StringIndexOutOfBounds { .. } => "java.lang.StringIndexOutOfBoundsException",
            Self::Runtime {
                exception_class_name,
                ..
            } => exception_class_name,
        }
    }

    /// 创建自定义 token 值 `toString()` 抛出的运行时错误。
    ///
    /// # 参数
    /// - `exception_class_name`：Java 异常类名；
    /// - `message`：可观察 detail message。
    ///
    /// # 返回
    /// 保留类别和消息的错误。
    #[must_use]
    pub fn runtime(exception_class_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Runtime {
            exception_class_name: exception_class_name.into(),
            message: message.into(),
        }
    }
}

/// Thymeleaf 标准表达式 token 的公共基对象。
///
/// Rust 以泛型组合代替 Java `Token extends SimpleExpression` 的字段继承；值仍然
/// 只保存一次，并向 Boolean/Number/Generic/Null/NoOp token 子对象提供共同实现。
/// 公开构造器对应 Java protected 构造能力，供外部自定义 token 组合使用。
///
/// 对应 Java: `org.thymeleaf.standard.expression.Token`。
pub struct Token<T: JavaTokenValue> {
    value: Option<T>,
}

impl<T: JavaTokenValue> Token<T> {
    /// 创建保存指定可空对象的 token。对应 Java: `Token#Token(Object)`。
    ///
    /// # 参数
    /// - `value`：token 值；`None` 对应 Java null。
    ///
    /// # 返回
    /// 保存该值的新 token 基对象。
    #[must_use]
    pub const fn new(value: Option<T>) -> Self {
        Self { value }
    }

    /// 返回 token 的原始值。对应 Java: `Token#getValue()`。
    ///
    /// # 返回
    /// 原值借用；构造时为 Java null 则返回 `None`。
    #[must_use]
    pub const fn get_value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// 返回 token 的字符串表示。
    ///
    /// 对应 Java: `Token#getStringRepresentation()`，同时也是
    /// `Token#toString()` 的完整委托目标。
    ///
    /// # 返回
    /// 值的 `toString()` 所产生的可空、借用或新建字符串。
    ///
    /// # 错误
    /// token 值为 null 时返回 NullPointer 类别；自定义值异常原样传播。
    pub fn get_string_representation(&self) -> Result<JavaTokenStringResult<'_>, TokenError> {
        self.value
            .as_ref()
            .ok_or(TokenError::NullPointer)?
            .java_token_to_string()
    }

    /// 返回 Java `toString()` 的结果。
    ///
    /// 对应 Java: `Token#toString()`；上游实现完整委托
    /// `getStringRepresentation()`，因此这里保留相同的可空结果、引用身份和错误。
    ///
    /// # 返回
    /// 值的 `toString()` 所产生的可空、借用或新建字符串。
    ///
    /// # 错误
    /// token 值为 null 时返回 NullPointer 类别；自定义值异常原样传播。
    pub fn to_string(&self) -> Result<JavaTokenStringResult<'_>, TokenError> {
        self.get_string_representation()
    }

    /// 判断指定 UTF-16 位置是否属于标准表达式 token。
    ///
    /// 对应 Java: `Token#isTokenChar(String,int)`。
    ///
    /// # 参数
    /// - `context`：完整 Java 字符串上下文，`None` 对应 null；
    /// - `pos`：按 UTF-16 code unit 计数的 Java int 索引。
    ///
    /// # 返回
    /// ASCII、规定的国际字符区间及符合上下文条件的连字符返回 `true`。
    ///
    /// # 错误
    /// context 为 null 或 pos 越界时，保留 Java 异常类别和校验顺序。
    pub fn is_token_char(context: Option<&JavaString>, pos: i32) -> Result<bool, TokenError> {
        let context = context.ok_or(TokenError::NullPointer)?;
        let position = position_in(context, pos)?;
        Ok(is_token_char_at(context.as_utf16(), position))
    }
}

/// 在表达式解析诊断中把所有 token 字符替换成 `#` 的追踪器。
///
/// 这是 `Token` 的紧耦合 Java 静态内部类，按对象组织规则与主对象同文件。
///
/// 对应 Java:
/// `org.thymeleaf.standard.expression.Token.TokenParsingTracer`。
pub struct TokenParsingTracer {
    _private: (),
}

impl TokenParsingTracer {
    /// token 字符的追踪替代码元 `#`。
    pub const TOKEN_SUBSTITUTE: u16 = 0x0023;

    /// 生成与输入等长的 UTF-16 token 追踪文本。
    ///
    /// 对应 Java: `Token.TokenParsingTracer#trace(String)`。
    ///
    /// # 参数
    /// - `input`：待追踪 Java 字符串；`None` 对应 null。
    ///
    /// # 返回
    /// 每个 token code unit 替换为 `#`，其他 code unit 原样保留。
    ///
    /// # 错误
    /// 输入为 null 时返回 Java NullPointer 类别。
    pub fn trace(input: Option<&JavaString>) -> Result<JavaString, TokenError> {
        let input = input.ok_or(TokenError::NullPointer)?;
        let input_units = input.as_utf16();
        let mut traced = Vec::with_capacity(input_units.len().saturating_add(1));
        for position in 0..input_units.len() {
            if is_token_char_at(input_units, position) {
                traced.push(Self::TOKEN_SUBSTITUTE);
            } else {
                traced.push(input_units[position]);
            }
        }
        Ok(JavaString::from_utf16(traced))
    }
}

fn position_in(context: &JavaString, position: i32) -> Result<usize, TokenError> {
    let Ok(position_usize) = usize::try_from(position) else {
        return Err(TokenError::StringIndexOutOfBounds { position });
    };
    if position_usize >= context.len() {
        return Err(TokenError::StringIndexOutOfBounds { position });
    }
    Ok(position_usize)
}

fn is_token_char_at(context: &[u16], position: usize) -> bool {
    let current = context[position];

    if is_ascii_lower(current) || is_ascii_upper(current) || is_ascii_digit(current) {
        return true;
    }
    if matches!(
        current,
        0x0020
            | 0x000A
            | 0x0028
            | 0x0029
            | 0x0027
            | 0x0022
            | 0x003C
            | 0x003E
            | 0x007B
            | 0x007D
            | 0x003D
            | 0x002C
            | 0x003B
            | 0x003A
            | 0x002B
            | 0x002A
            | 0x0024
            | 0x0025
            | 0x0026
            | 0x0023
    ) {
        return false;
    }
    if matches!(current, 0x005B | 0x005D | 0x002E | 0x005F) {
        return true;
    }
    if current == u16::from(b'-') {
        // 向后扫描连续 token；发现非数字/点 code unit 时，连字符属于标识符。
        for index in (0..position).rev() {
            if !is_token_char_at(context, index) {
                break;
            }
            let candidate = context[index];
            if !is_ascii_digit(candidate) && candidate != u16::from(b'.') {
                return true;
            }
        }

        // 向前扫描时先识别另一个连字符，避免两个递归调用形成循环。
        for index in position.saturating_add(1)..context.len() {
            let candidate = context[index];
            if candidate == u16::from(b'-') {
                return true;
            }
            if !is_token_char_at(context, index) {
                break;
            }
            if !is_ascii_digit(candidate) && candidate != u16::from(b'.') {
                return true;
            }
        }
        return false;
    }

    current == 0x00B7
        || (0x00C0..=0x00D6).contains(&current)
        || (0x00D8..=0x00F6).contains(&current)
        || (0x00F8..=0x02FF).contains(&current)
        || (0x0300..=0x036F).contains(&current)
        || (0x0370..=0x037D).contains(&current)
        || (0x037F..=0x1FFF).contains(&current)
        || (0x200C..=0x200D).contains(&current)
        || (0x203F..=0x2040).contains(&current)
        || (0x2070..=0x218F).contains(&current)
        || (0x2C00..=0x2FEF).contains(&current)
        || (0x3001..=0xD7FF).contains(&current)
        || (0xF900..=0xFDCF).contains(&current)
        || (0xFDF0..=0xFFFD).contains(&current)
}

const fn is_ascii_lower(value: u16) -> bool {
    value >= 0x0061 && value <= 0x007A
}

const fn is_ascii_upper(value: u16) -> bool {
    value >= 0x0041 && value <= 0x005A
}

const fn is_ascii_digit(value: u16) -> bool {
    value >= 0x0030 && value <= 0x0039
}

#[cfg(test)]
mod tests {
    use super::{JavaTokenStringResult, JavaTokenValue, Token, TokenError, TokenParsingTracer};
    use crate::util::JavaString;

    struct Probe {
        result: ProbeResult,
    }

    enum ProbeResult {
        Null,
        Value(JavaString),
        Error,
    }

    impl JavaTokenValue for Probe {
        fn java_token_to_string(&self) -> Result<JavaTokenStringResult<'_>, TokenError> {
            match &self.result {
                ProbeResult::Null => Ok(JavaTokenStringResult::Null),
                ProbeResult::Value(value) => Ok(JavaTokenStringResult::Borrowed(value)),
                ProbeResult::Error => Err(TokenError::runtime(
                    "java.lang.IllegalStateException",
                    "boom",
                )),
            }
        }
    }

    #[test]
    fn preserves_value_identity_nullable_string_and_runtime_errors() {
        let string = JavaString::from_rust_str("token");
        let token = Token::new(Some(string.clone()));
        assert_eq!(token.get_value(), Some(&string));
        assert!(matches!(
            token.get_string_representation(),
            Ok(JavaTokenStringResult::Borrowed(value)) if value == &string
        ));

        let null_token = Token::<JavaString>::new(None);
        assert_eq!(
            null_token.get_string_representation().err(),
            Some(TokenError::NullPointer)
        );

        let null_result = Token::new(Some(Probe {
            result: ProbeResult::Null,
        }));
        assert_eq!(
            std::mem::discriminant(&null_result.get_string_representation().unwrap()),
            std::mem::discriminant(&JavaTokenStringResult::Null)
        );
        let owned_result = Token::new(Some(Probe {
            result: ProbeResult::Value(JavaString::from_rust_str("owned")),
        }));
        let borrowed_result = owned_result.get_string_representation().unwrap();
        let expected_borrowed_value = JavaString::from_rust_str("expected");
        assert_eq!(
            std::mem::discriminant(&borrowed_result),
            std::mem::discriminant(&JavaTokenStringResult::Borrowed(&expected_borrowed_value))
        );
        let error_token = Token::new(Some(Probe {
            result: ProbeResult::Error,
        }));
        let error = error_token
            .get_string_representation()
            .err()
            .expect("runtime error");
        assert_eq!(error.java_class_name(), "java.lang.IllegalStateException");
        assert_eq!(error.to_string(), "boom");
    }

    #[test]
    fn preserves_null_index_and_trace_boundaries() {
        assert_eq!(
            Token::<JavaString>::is_token_char(None, 0).err(),
            Some(TokenError::NullPointer)
        );
        let empty = JavaString::from_rust_str("");
        for position in [-1, 0, i32::MAX] {
            let error = Token::<JavaString>::is_token_char(Some(&empty), position)
                .expect_err("index failure");
            assert_eq!(
                error.java_class_name(),
                "java.lang.StringIndexOutOfBoundsException"
            );
        }
        assert_eq!(
            TokenParsingTracer::trace(None).err(),
            Some(TokenError::NullPointer)
        );
        assert_eq!(
            TokenParsingTracer::trace(Some(&empty))
                .expect("empty trace")
                .as_utf16(),
            &[] as &[u16]
        );
    }
}
