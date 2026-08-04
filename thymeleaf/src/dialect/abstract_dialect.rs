use thiserror::Error;

use super::IDialect;

/// Thymeleaf 方言的基础实现。
///
/// 对应 Java: `org.thymeleaf.dialect.AbstractDialect`。
///
/// Java 抽象基类只保存一个构造时校验为非空、之后不可变的名称。Rust 不模拟类
/// 继承；具体方言可以组合本对象，并把 `IDialect` 调用委托给它。
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AbstractDialect {
    name: String,
}

impl AbstractDialect {
    /// 创建具有指定名称的基础方言。
    ///
    /// 对应 Java: `AbstractDialect#AbstractDialect(String)`。
    ///
    /// # 参数
    /// - `name`：Java 参数 `name`；允许空字符串，但不能为 `None`。
    ///
    /// # 返回
    /// 名称已经防御性持有的基础方言。
    ///
    /// # 错误
    /// `name` 为 `None` 时返回 `AbstractDialectError::DialectNameCannotBeNull`，
    /// 对应 Java `IllegalArgumentException("Dialect name cannot be null")`。
    pub fn new(name: Option<&str>) -> Result<Self, AbstractDialectError> {
        let name = name.ok_or(AbstractDialectError::DialectNameCannotBeNull)?;
        Ok(Self {
            name: name.to_owned(),
        })
    }

    /// 返回构造时指定的非空方言名称。
    ///
    /// 对应 Java: `AbstractDialect#getName()`。
    ///
    /// # 返回
    /// 方言名称；包括 Java 允许的空字符串。
    #[must_use]
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl IDialect for AbstractDialect {
    fn class_name(&self) -> &'static str {
        "org.thymeleaf.dialect.AbstractDialect"
    }

    fn get_name(&self) -> Option<&str> {
        Some(self.get_name())
    }
}

/// 创建 `AbstractDialect` 时可能发生的校验错误。
///
/// 对应 Java: `org.thymeleaf.dialect.AbstractDialect` 构造器抛出的
/// `IllegalArgumentException`。该类型是 Rust 的类型化错误扩展，不计入 Java
/// 对象迁移分子。
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum AbstractDialectError {
    /// 方言名称对应 Java `null`。
    #[error("Dialect name cannot be null")]
    DialectNameCannotBeNull,
}

#[cfg(test)]
mod tests {
    use super::{AbstractDialect, AbstractDialectError};
    use crate::IDialect;

    #[test]
    fn accepts_empty_and_unicode_names_and_exposes_them_immutably() {
        let empty = AbstractDialect::new(Some("")).expect("empty name is legal");
        let unicode = AbstractDialect::new(Some("标准方言")).expect("unicode name is legal");

        assert_eq!(empty.get_name(), "");
        assert_eq!(unicode.get_name(), "标准方言");
        assert_eq!(IDialect::get_name(&unicode), Some("标准方言"));
    }

    #[test]
    fn rejects_a_java_null_name_with_the_exact_message() {
        let error = AbstractDialect::new(None).expect_err("null name must fail");

        assert_eq!(error, AbstractDialectError::DialectNameCannotBeNull);
        assert_eq!(error.to_string(), "Dialect name cannot be null");
    }
}
