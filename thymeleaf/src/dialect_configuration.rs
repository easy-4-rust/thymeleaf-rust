use std::sync::Arc;

use thiserror::Error;

use crate::IDialect;

/// 单个方言及其处理器前缀的模板引擎配置。
///
/// 对应 Java: `org.thymeleaf.DialectConfiguration`。
///
/// “没有指定前缀”和“显式指定空前缀”是两个不同状态：前者使用方言自身的默认
/// 前缀（如果该方言提供 Processor），后者要求 Processor 匹配无前缀的元素或
/// 属性。显式的非空前缀会覆盖方言默认前缀。
///
/// Java 1.0 起已有同名类，但当前语义对应 Thymeleaf 3.0 的完全重实现。
pub struct DialectConfiguration {
    prefix_specified: bool,
    prefix: Option<String>,
    dialect: Arc<dyn IDialect>,
}

impl DialectConfiguration {
    /// 使用方言自身的默认前缀创建配置。
    ///
    /// 对应 Java: `DialectConfiguration#DialectConfiguration(IDialect)`。
    ///
    /// # 参数
    /// - `dialect`：Java 参数 `dialect`；`None` 对应 Java `null`。
    ///
    /// # 返回
    /// `prefix_specified` 为 `false`、前缀为 `None` 的方言配置。
    ///
    /// # 错误
    /// `dialect` 为 `None` 时返回
    /// `DialectConfigurationError::DialectCannotBeNull`。
    pub fn new(dialect: Option<Arc<dyn IDialect>>) -> Result<Self, DialectConfigurationError> {
        Self::build(false, None, dialect)
    }

    /// 使用显式前缀创建配置。
    ///
    /// 对应 Java: `DialectConfiguration#DialectConfiguration(String, IDialect)`。
    ///
    /// # 参数
    /// - `prefix`：Java 参数 `prefix`；`None` 表示显式匹配无前缀元素或属性，
    ///   空字符串仍按原值保存。
    /// - `dialect`：Java 参数 `dialect`；`None` 对应 Java `null`。
    ///
    /// # 返回
    /// `prefix_specified` 为 `true` 的方言配置。
    ///
    /// # 错误
    /// `dialect` 为 `None` 时返回
    /// `DialectConfigurationError::DialectCannotBeNull`。
    pub fn with_prefix(
        prefix: Option<&str>,
        dialect: Option<Arc<dyn IDialect>>,
    ) -> Result<Self, DialectConfigurationError> {
        Self::build(true, prefix, dialect)
    }

    fn build(
        prefix_specified: bool,
        prefix: Option<&str>,
        dialect: Option<Arc<dyn IDialect>>,
    ) -> Result<Self, DialectConfigurationError> {
        let dialect = dialect.ok_or(DialectConfigurationError::DialectCannotBeNull)?;
        Ok(Self {
            prefix_specified,
            prefix: prefix.map(str::to_owned),
            dialect,
        })
    }

    /// 返回配置持有的同一方言实例。
    ///
    /// 对应 Java: `DialectConfiguration#getDialect()`。
    ///
    /// # 返回
    /// 构造时传入方言的共享引用，不创建替代方言。
    #[must_use]
    pub fn get_dialect(&self) -> &dyn IDialect {
        self.dialect.as_ref()
    }

    /// 返回构造时传入的共享方言实例。
    ///
    /// 这是 Rust 所有权适配入口，保留 Java `getDialect()` 的同一对象身份。
    /// 对应 Java 语义：`DialectConfiguration` 的 `get_dialect_arc` 行为（Rust 侧辅助/私有路径）。
    pub fn get_dialect_arc(&self) -> Arc<dyn IDialect> {
        Arc::clone(&self.dialect)
    }

    /// 返回配置的显式前缀。
    ///
    /// 对应 Java: `DialectConfiguration#getPrefix()`。
    ///
    /// # 返回
    /// 显式字符串，或 Java `null` 对应的 `None`。必须结合
    /// `is_prefix_specified` 区分“未指定”和“显式无前缀”。
    #[must_use]
    pub fn get_prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    /// 判断调用方是否显式指定了前缀参数。
    ///
    /// 对应 Java: `DialectConfiguration#isPrefixSpecified()`。
    ///
    /// # 返回
    /// 使用二参数构造器时始终为 `true`，即使 `prefix` 为 `None`。
    #[must_use]
    pub const fn is_prefix_specified(&self) -> bool {
        self.prefix_specified
    }
}

/// 创建 `DialectConfiguration` 时可能发生的校验错误。
///
/// 对应 Java: `org.thymeleaf.DialectConfiguration` 构造器抛出的
/// `IllegalArgumentException`。该类型是 Rust 的类型化错误扩展，不计入 Java
/// 对象迁移分子。
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DialectConfigurationError {
    /// 方言实例对应 Java `null`。
    #[error("Dialect cannot be null")]
    DialectCannotBeNull,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{DialectConfiguration, DialectConfigurationError};
    use crate::{AbstractDialect, IDialect};

    fn dialect() -> Arc<dyn IDialect> {
        Arc::new(AbstractDialect::new(Some("Test")).expect("valid dialect"))
    }

    #[test]
    fn constructor_without_prefix_preserves_the_unspecified_state() {
        let configuration =
            DialectConfiguration::new(Some(dialect())).expect("valid configuration");

        assert!(!configuration.is_prefix_specified());
        assert_eq!(configuration.get_prefix(), None);
        assert_eq!(configuration.get_dialect().get_name(), Some("Test"));
    }

    #[test]
    fn explicit_null_empty_and_non_empty_prefixes_remain_distinct() {
        let null_prefix =
            DialectConfiguration::with_prefix(None, Some(dialect())).expect("null prefix is legal");
        let empty_prefix = DialectConfiguration::with_prefix(Some(""), Some(dialect()))
            .expect("empty prefix is legal");
        let named_prefix = DialectConfiguration::with_prefix(Some("th"), Some(dialect()))
            .expect("named prefix is legal");

        assert!(null_prefix.is_prefix_specified());
        assert_eq!(null_prefix.get_prefix(), None);
        assert!(empty_prefix.is_prefix_specified());
        assert_eq!(empty_prefix.get_prefix(), Some(""));
        assert!(named_prefix.is_prefix_specified());
        assert_eq!(named_prefix.get_prefix(), Some("th"));
    }

    #[test]
    fn both_java_constructors_reject_a_null_dialect() {
        let without_prefix = DialectConfiguration::new(None).err();
        let with_prefix = DialectConfiguration::with_prefix(Some("th"), None).err();

        assert_eq!(
            without_prefix,
            Some(DialectConfigurationError::DialectCannotBeNull)
        );
        assert_eq!(with_prefix, without_prefix);
        assert_eq!(
            without_prefix.expect("null dialect must fail").to_string(),
            "Dialect cannot be null"
        );
    }
}
