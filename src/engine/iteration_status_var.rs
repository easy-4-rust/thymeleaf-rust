use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::util::JavaString;

/// 迭代状态访问时产生的 Java 运行时错误。
///
/// 对应 Java: `org.thymeleaf.engine.IterationStatusVar` 中对可空 `Integer size`
/// 自动拆箱时可能抛出的异常。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IterationStatusVarError {
    /// `isLast()` 对 null `size` 执行自动拆箱。
    NullSize,
}

impl IterationStatusVarError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::NullSize => "java.lang.NullPointerException",
        }
    }
}

impl Display for IterationStatusVarError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(
            "Cannot invoke \"java.lang.Integer.intValue()\" because \"this.size\" is null",
        )
    }
}

impl Error for IterationStatusVarError {}

/// 模板循环中暴露给表达式的迭代状态。
///
/// 对应 Java: `org.thymeleaf.engine.IterationStatusVar`。
///
/// `size` 在迭代源无法预先确定长度时允许为 null；奇偶性按 CSS `:nth-child`
/// 习惯从 1 开始计数。内部字段只允许 engine 同包对象推进。
#[derive(Clone, Debug)]
pub struct IterationStatusVar<T> {
    pub(super) index: i32,
    pub(super) size: Option<i32>,
    pub(super) current: Option<T>,
}

impl<T> IterationStatusVar<T> {
    /// 返回从零开始的当前索引。
    #[must_use]
    pub const fn get_index(&self) -> i32 {
        self.index
    }

    /// 返回从一开始的当前计数。
    ///
    /// Java `int` 溢出按二进制补码回绕。
    #[must_use]
    pub const fn get_count(&self) -> i32 {
        self.index.wrapping_add(1)
    }

    /// 判断当前状态是否具有预先确定的总大小。
    #[must_use]
    pub const fn has_size(&self) -> bool {
        self.size.is_some()
    }

    /// 返回可空总大小。
    #[must_use]
    pub const fn get_size(&self) -> Option<i32> {
        self.size
    }

    /// 返回可空当前迭代对象。
    #[must_use]
    pub const fn get_current(&self) -> Option<&T> {
        self.current.as_ref()
    }

    /// 判断当前一基计数是否为偶数。
    #[must_use]
    pub const fn is_even(&self) -> bool {
        self.index.wrapping_add(1) % 2 == 0
    }

    /// 判断当前一基计数是否为奇数。
    #[must_use]
    pub const fn is_odd(&self) -> bool {
        !self.is_even()
    }

    /// 判断当前元素是否为首个元素。
    #[must_use]
    pub const fn is_first(&self) -> bool {
        self.index == 0
    }

    /// 判断当前元素是否为最后一个元素。
    ///
    /// # 错误
    ///
    /// `size` 未知时保留 Java 自动拆箱产生的 `NullPointerException`。
    pub const fn is_last(&self) -> Result<bool, IterationStatusVarError> {
        match self.size {
            Some(size) => Ok(self.index == size.wrapping_sub(1)),
            None => Err(IterationStatusVarError::NullSize),
        }
    }

    /// 按 Java `toString()` 布局生成状态文本。
    ///
    /// # 返回
    ///
    /// 包含 index、count、可空 size 和可空 current 的 UTF-16 字符串。
    #[must_use]
    pub fn to_java_string(&self) -> JavaString
    where
        T: Display,
    {
        let size = self
            .size
            .map_or_else(|| "null".to_owned(), |value| value.to_string());
        let current = self
            .current
            .as_ref()
            .map_or_else(|| "null".to_owned(), ToString::to_string);
        JavaString::from_rust_str(&format!(
            "{{index = {}, count = {}, size = {size}, current = {current}}}",
            self.index,
            self.index.wrapping_add(1)
        ))
    }
}
