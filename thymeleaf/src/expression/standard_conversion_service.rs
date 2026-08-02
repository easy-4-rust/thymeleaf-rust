use super::AbstractStandardConversionService;

/// Thymeleaf 标准方言注册的默认表达式转换服务。
///
/// 本对象继承抽象转换服务的全部行为：支持 null/String 身份快路径及任意对象的
/// `toString()`，其他目标类型返回不可用转换错误。对象无可变状态，可安全共享。
///
/// 对应 Java:
/// `org.thymeleaf.standard.expression.StandardConversionService`。
#[derive(Debug, Default)]
pub struct StandardConversionService {
    _private: (),
}

impl StandardConversionService {
    /// 创建默认转换服务。对应 Java: `StandardConversionService#StandardConversionService()`。
    ///
    /// 上游注释要求通常仅由 `StandardDialect` 实例化，但构造器本身保持公开。
    ///
    /// # 返回
    /// 无状态、线程安全的默认转换服务。
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

impl AbstractStandardConversionService for StandardConversionService {}
