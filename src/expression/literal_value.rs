use std::any::Any;
use std::ptr;

use crate::util::JavaString;

/// 表达式解析期间使用的文本字面量包装器。
///
/// 本对象避免 `'4'`、`'2.'` 等文本字面量在算术运算中被再次解释为数字。
/// 构造器保留 Java 允许内部字符串为 null 的行为，且未覆写值相等语义。
///
/// 对应 Java: `org.thymeleaf.standard.expression.LiteralValue`。
#[derive(Debug)]
pub struct LiteralValue {
    value: Option<JavaString>,
}

impl PartialEq for LiteralValue {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Eq for LiteralValue {}

impl LiteralValue {
    /// 创建文本字面量包装器。对应 Java: `LiteralValue#LiteralValue(String)`。
    ///
    /// # 参数
    /// - `value`：字面量文本；`None` 对应 Java null。
    ///
    /// # 返回
    /// 保存原始可空文本的新包装器。
    #[must_use]
    pub const fn new(value: Option<JavaString>) -> Self {
        Self { value }
    }

    /// 返回包装的原始文本。对应 Java: `LiteralValue#getValue()`。
    ///
    /// # 返回
    /// 字面量文本的借用；内部 Java null 返回 `None`。
    #[must_use]
    pub const fn get_value(&self) -> Option<&JavaString> {
        self.value.as_ref()
    }

    /// 解包任意对象中的 `LiteralValue`。对应 Java: `LiteralValue#unwrap(Object)`。
    ///
    /// # 参数
    /// - `object`：可空的动态对象引用。
    ///
    /// # 返回
    /// null 原样返回 `None`；`LiteralValue` 返回其内部字符串（内部 null 仍为
    /// `None`）；其他对象返回完全相同的引用。
    #[must_use]
    pub fn unwrap(object: Option<&dyn Any>) -> Option<&dyn Any> {
        let object = object?;
        if let Some(literal_value) = object.downcast_ref::<Self>() {
            literal_value.get_value().map(|value| value as &dyn Any)
        } else {
            Some(object)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::ptr;

    use super::LiteralValue;
    use crate::util::JavaString;

    #[test]
    fn preserves_nullable_value_and_unwrap_identity() {
        let value = JavaString::from_rust_str("4");
        let literal = LiteralValue::new(Some(value.clone()));
        assert_eq!(literal.get_value(), Some(&value));

        let unwrapped =
            LiteralValue::unwrap(Some(&literal as &dyn Any)).expect("non-null literal must unwrap");
        assert_eq!(unwrapped.downcast_ref::<JavaString>(), Some(&value));
        assert!(LiteralValue::unwrap(Some(&LiteralValue::new(None) as &dyn Any)).is_none());
        assert!(LiteralValue::unwrap(None).is_none());

        let other = String::from("other");
        let other_object = &other as &dyn Any;
        let same = LiteralValue::unwrap(Some(other_object)).expect("other object");
        assert!(ptr::eq(same, other_object));
    }
}
