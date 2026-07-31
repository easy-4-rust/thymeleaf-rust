use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::JavaString;

use super::{StandardExpressionResult, TemplateValue};

/// 表达式对象工厂返回的完整名称集合。
///
/// Java 使用 `Set<String>`。Rust 使用共享只读切片保留构造后不可变的集合身份，并由
/// 工厂负责保持名称唯一性和迭代顺序。
pub type ExpressionObjectNames = Arc<[Option<JavaString>]>;

/// 按需创建表达式工具对象的工厂合同。
///
/// 方言提供工厂而不是预先创建好的对象，使表达式工具只在模板表达式真正读取时才被
/// 实例化。工厂必须在名称集合中声明所有可能构建的对象。
///
/// 对应 Java: `org.thymeleaf.expression.IExpressionObjectFactory`。
pub trait IExpressionObjectFactory: Send + Sync {
    /// 返回工厂可能构建的全部对象名称。
    ///
    /// 该集合用于在构建前判断名称是否属于此工厂，因此必须完整。Java 集合和元素都
    /// 可以由第三方实现返回 `null`，外层及元素分别使用 `Option` 保留该边界。
    ///
    /// # 返回值
    ///
    /// 返回全部名称的共享只读集合；`None` 对应违反 SPI 约定的 Java `null`。
    fn get_all_expression_object_names(&self) -> Option<ExpressionObjectNames>;

    /// 构建指定名称的表达式对象。
    ///
    /// # 参数
    ///
    /// - `context`：当前模板执行使用的表达式上下文。
    /// - `expression_object_name`：待构建对象的可空名称。
    ///
    /// # 返回值
    ///
    /// 返回构建结果；内层 `None` 对应工厂无法构建时返回的 Java `null`。
    ///
    /// # 错误
    ///
    /// 返回对象构造或策略检查产生的原始动态错误。
    fn build_object(
        &self,
        context: Arc<dyn IExpressionContext>,
        expression_object_name: Option<&JavaString>,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>>;

    /// 判断指定对象能否在同一次模板执行的所有表达式之间缓存复用。
    ///
    /// 该标志只控制一次模板执行内部的对象复用，不表示跨模板或跨请求缓存。
    ///
    /// # 参数
    ///
    /// - `expression_object_name`：待检查对象的可空名称。
    ///
    /// # 返回值
    ///
    /// `true` 表示容器应缓存包括 Java `null` 在内的构建结果。
    fn is_cacheable(&self, expression_object_name: Option<&JavaString>) -> bool;
}
