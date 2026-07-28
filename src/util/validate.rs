use std::error::Error;
use std::fmt::{Display, Formatter};

/// Thymeleaf 参数校验失败。
///
/// 对应 Java:
/// `org.thymeleaf.util.Validate` 抛出的 `IllegalArgumentException`，以及遍历 null
/// `Iterable`/数组时由 JVM 抛出的 `NullPointerException`。
///
/// 显式校验保留调用方提供的可空消息；隐式 null 遍历错误不伪造依赖 JDK/Javac
/// 版本的 helpful-NPE 文本，只保留稳定异常类别。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidateError {
    /// 对应 Java `IllegalArgumentException`。
    IllegalArgument {
        /// Java 异常的可空 detail message。
        message: Option<String>,
    },
    /// 对应遍历 null `Iterable` 或数组产生的 Java `NullPointerException`。
    NullPointer,
}

impl ValidateError {
    fn illegal_argument(message: Option<&str>) -> Self {
        Self::IllegalArgument {
            message: message.map(str::to_owned),
        }
    }

    /// 返回对应的 Java 异常类名。
    ///
    /// # 返回
    /// `java.lang.IllegalArgumentException` 或 `java.lang.NullPointerException`。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::IllegalArgument { .. } => "java.lang.IllegalArgumentException",
            Self::NullPointer => "java.lang.NullPointerException",
        }
    }

    /// 返回显式 Java detail message。
    ///
    /// # 返回
    /// `IllegalArgumentException` 的调用方消息；消息为 Java null 或属于隐式 JVM
    /// `NullPointerException` 时返回 `None`。
    #[must_use]
    pub fn get_message(&self) -> Option<&str> {
        match self {
            Self::IllegalArgument { message } => message.as_deref(),
            Self::NullPointer => None,
        }
    }
}

impl Display for ValidateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(message) = self.get_message() {
            formatter.write_str(message)?;
        }
        Ok(())
    }
}

impl Error for ValidateError {}

/// Thymeleaf 内部的参数与集合前置条件校验工具。
///
/// 对应 Java: `org.thymeleaf.util.Validate`。
///
/// 本对象不包含状态。Rust 用静态关联函数保留 Java 的校验顺序、空值规则、字符串
/// whitespace 定义、调用方消息以及显式/隐式异常类别。成功时返回 `Ok(())`，失败时
/// 返回 [`ValidateError`]，避免把可恢复的 Java 参数错误转换成 panic。
pub struct Validate;

impl Validate {
    /// 断言对象非 null。
    ///
    /// 对应 Java: `Validate#notNull(Object, String)`。
    ///
    /// # 参数
    /// - `object`：Java 参数 `object`，`None` 对应 null；
    /// - `message`：Java 参数 `message`，允许为 null。
    pub fn not_null<T: ?Sized>(
        object: Option<&T>,
        message: Option<&str>,
    ) -> Result<(), ValidateError> {
        if object.is_none() {
            return Err(ValidateError::illegal_argument(message));
        }
        Ok(())
    }

    /// 断言字符串非 null、非空且不全为 Java whitespace。
    ///
    /// 对应 Java: `Validate#notEmpty(String, String)`。
    ///
    /// # 参数
    /// - `object`：待校验字符串；
    /// - `message`：失败消息，允许为 null。
    pub fn not_empty_str(object: Option<&str>, message: Option<&str>) -> Result<(), ValidateError> {
        if object.is_none_or(is_java_empty_or_whitespace) {
            return Err(ValidateError::illegal_argument(message));
        }
        Ok(())
    }

    /// 断言 Java Collection 对应的 Rust 集合非 null 且至少有一个元素。
    ///
    /// 对应 Java: `Validate#notEmpty(Collection, String)`。
    ///
    /// # 参数
    /// - `object`：可借用迭代的集合；`None` 对应 Java null；
    /// - `message`：失败消息，允许为 null。
    pub fn not_empty_collection<'a, C: ?Sized>(
        object: Option<&'a C>,
        message: Option<&str>,
    ) -> Result<(), ValidateError>
    where
        &'a C: IntoIterator,
    {
        if object.is_none_or(|collection| collection.into_iter().next().is_none()) {
            return Err(ValidateError::illegal_argument(message));
        }
        Ok(())
    }

    /// 断言 Java Object[] 对应的 Rust slice 非 null 且非空。
    ///
    /// 对应 Java: `Validate#notEmpty(Object[], String)`。
    pub fn not_empty_array<T>(
        object: Option<&[T]>,
        message: Option<&str>,
    ) -> Result<(), ValidateError> {
        if object.is_none_or(<[T]>::is_empty) {
            return Err(ValidateError::illegal_argument(message));
        }
        Ok(())
    }

    /// 断言 Iterable 的全部元素非 null。
    ///
    /// 对应 Java: `Validate#containsNoNulls(Iterable, String)`。
    ///
    /// Java 方法没有预先校验 Iterable 自身；传入 null 时由增强 for 循环产生
    /// `NullPointerException`。Rust 保留这一错误类别。遍历遇到首个 `None` 时立即
    /// 返回调用方消息对应的 `IllegalArgumentException`。
    pub fn contains_no_nulls_iterable<'a, C: ?Sized, T: 'a>(
        collection: Option<&'a C>,
        message: Option<&str>,
    ) -> Result<(), ValidateError>
    where
        &'a C: IntoIterator<Item = &'a Option<T>>,
    {
        let collection = collection.ok_or(ValidateError::NullPointer)?;
        for object in collection {
            Self::not_null(object.as_ref(), message)?;
        }
        Ok(())
    }

    /// 断言字符串 Iterable 的全部元素非 null、非空且不全为 Java whitespace。
    ///
    /// 对应 Java: `Validate#containsNoEmpties(Iterable<String>, String)`。
    ///
    /// 集合自身为 null 时返回 `NullPointer`；元素不满足 `notEmpty` 时返回调用方
    /// 消息对应的 `IllegalArgument`。
    pub fn contains_no_empties<'a, C: ?Sized, S: AsRef<str> + 'a>(
        collection: Option<&'a C>,
        message: Option<&str>,
    ) -> Result<(), ValidateError>
    where
        &'a C: IntoIterator<Item = &'a Option<S>>,
    {
        let collection = collection.ok_or(ValidateError::NullPointer)?;
        for object in collection {
            Self::not_empty_str(object.as_ref().map(AsRef::as_ref), message)?;
        }
        Ok(())
    }

    /// 断言 Object[] 的全部元素非 null。
    ///
    /// 对应 Java: `Validate#containsNoNulls(Object[], String)`。
    ///
    /// 数组自身为 null 时保留 JVM 隐式 `NullPointerException` 类别。
    pub fn contains_no_nulls_array<T>(
        array: Option<&[Option<T>]>,
        message: Option<&str>,
    ) -> Result<(), ValidateError> {
        let array = array.ok_or(ValidateError::NullPointer)?;
        for object in array {
            Self::not_null(object.as_ref(), message)?;
        }
        Ok(())
    }

    /// 断言条件为 true。
    ///
    /// 对应 Java: `Validate#isTrue(boolean, String)`。
    ///
    /// # 参数
    /// - `condition`：待断言条件；
    /// - `message`：失败消息，允许为 null。
    pub fn is_true(condition: bool, message: Option<&str>) -> Result<(), ValidateError> {
        if !condition {
            return Err(ValidateError::illegal_argument(message));
        }
        Ok(())
    }
}

fn is_java_empty_or_whitespace(value: &str) -> bool {
    value.is_empty()
        || value.chars().all(|character| {
            matches!(
                character,
                '\u{0009}'..='\u{000D}'
                    | '\u{001C}'..='\u{0020}'
                    | '\u{1680}'
                    | '\u{2000}'..='\u{2006}'
                    | '\u{2008}'..='\u{200A}'
                    | '\u{2028}'
                    | '\u{2029}'
                    | '\u{205F}'
                    | '\u{3000}'
            )
        })
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fmt::Write;

    use super::{Validate, ValidateError};

    struct FailingWriter {
        remaining_writes: usize,
    }

    impl Write for FailingWriter {
        fn write_str(&mut self, _value: &str) -> std::fmt::Result {
            if self.remaining_writes == 0 {
                return Err(std::fmt::Error);
            }
            self.remaining_writes -= 1;
            Ok(())
        }
    }

    #[test]
    fn preserves_not_null_and_nullable_message() {
        assert_eq!(Validate::not_null(Some(&7), Some("failure")), Ok(()));
        assert_eq!(
            Validate::not_null::<i32>(None, Some("failure")),
            Err(ValidateError::IllegalArgument {
                message: Some("failure".to_owned())
            })
        );
        let error_with_message =
            Validate::not_null::<i32>(None, Some("failure")).expect_err("error");
        assert_eq!(error_with_message.to_string(), "failure");
        assert!(
            write!(
                &mut FailingWriter {
                    remaining_writes: 0
                },
                "{error_with_message}"
            )
            .is_err()
        );
        assert!(
            write!(
                &mut FailingWriter {
                    remaining_writes: 1
                },
                "{error_with_message}"
            )
            .is_ok()
        );
        let error = Validate::not_null::<i32>(None, None).expect_err("error");
        assert_eq!(error.get_message(), None);
        assert_eq!(error.to_string(), "");
        assert_eq!(
            error.java_class_name(),
            "java.lang.IllegalArgumentException"
        );
    }

    #[test]
    fn applies_exact_java_whitespace_rules() {
        for invalid in [
            None,
            Some(""),
            Some(" "),
            Some("\t\n"),
            Some("\u{001C}"),
            Some("\u{1680}"),
            Some("\u{2008}"),
            Some("\u{2028}"),
            Some("\u{205F}"),
            Some("\u{3000}"),
        ] {
            assert!(Validate::not_empty_str(invalid, Some("empty")).is_err());
        }
        for valid in [Some("a"), Some("  a  "), Some("\u{00A0}")] {
            assert_eq!(Validate::not_empty_str(valid, Some("empty")), Ok(()));
        }
    }

    #[test]
    fn distinguishes_collection_and_array_overloads() {
        let empty_vec: Vec<i32> = Vec::new();
        let values = vec![1, 2];
        let set = HashSet::from([1]);
        assert!(Validate::not_empty_collection(None::<&Vec<i32>>, Some("empty")).is_err());
        assert!(Validate::not_empty_collection(Some(&empty_vec), Some("empty")).is_err());
        assert_eq!(
            Validate::not_empty_collection(Some(&values), Some("empty")),
            Ok(())
        );
        assert_eq!(
            Validate::not_empty_collection(Some(&set), Some("empty")),
            Ok(())
        );
        assert!(Validate::not_empty_array::<i32>(None, Some("empty")).is_err());
        assert!(Validate::not_empty_array::<i32>(Some(&[]), Some("empty")).is_err());
        assert_eq!(
            Validate::not_empty_array(Some(&[1, 2]), Some("empty")),
            Ok(())
        );
    }

    #[test]
    fn validates_iterable_elements_and_short_circuits_on_first_failure() {
        let valid = vec![Some(1), Some(2)];
        let invalid = vec![Some(1), None, Some(3)];
        assert_eq!(
            Validate::contains_no_nulls_iterable(Some(&valid), Some("null")),
            Ok(())
        );
        assert_eq!(
            Validate::contains_no_nulls_iterable(Some(&invalid), Some("null")),
            Err(ValidateError::IllegalArgument {
                message: Some("null".to_owned())
            })
        );
        assert_eq!(
            Validate::contains_no_nulls_iterable::<Vec<Option<i32>>, i32>(None, Some("ignored")),
            Err(ValidateError::NullPointer)
        );
    }

    #[test]
    fn validates_string_iterables_with_not_empty_semantics() {
        let valid = vec![Some("value".to_owned()), Some("\u{00A0}".to_owned())];
        let empty = vec![Some("value".to_owned()), Some(" ".to_owned())];
        let null = vec![Some("value".to_owned()), None];
        assert_eq!(
            Validate::contains_no_empties(Some(&valid), Some("empty")),
            Ok(())
        );
        assert!(Validate::contains_no_empties(Some(&empty), Some("empty")).is_err());
        assert!(Validate::contains_no_empties(Some(&null), Some("empty")).is_err());
        assert_eq!(
            Validate::contains_no_empties::<Vec<Option<String>>, String>(None, Some("ignored")),
            Err(ValidateError::NullPointer)
        );
    }

    #[test]
    fn validates_array_elements_and_null_array_category() {
        assert_eq!(
            Validate::contains_no_nulls_array(Some(&[Some(1), Some(2)]), Some("null")),
            Ok(())
        );
        assert_eq!(
            Validate::contains_no_nulls_array(Some(&[Some(1), None]), Some("null")),
            Err(ValidateError::IllegalArgument {
                message: Some("null".to_owned())
            })
        );
        let error =
            Validate::contains_no_nulls_array::<i32>(None, Some("ignored")).expect_err("error");
        assert_eq!(error, ValidateError::NullPointer);
        assert_eq!(error.java_class_name(), "java.lang.NullPointerException");
        assert_eq!(error.get_message(), None);
    }

    #[test]
    fn preserves_is_true_success_and_failure() {
        assert_eq!(Validate::is_true(true, Some("failure")), Ok(()));
        assert_eq!(
            Validate::is_true(false, Some("failure")),
            Err(ValidateError::IllegalArgument {
                message: Some("failure".to_owned())
            })
        );
    }
}
