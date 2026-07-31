use std::sync::Arc;

use crate::util::JavaString;

use super::{ExpressionObjectNames, StandardExpressionResult, TemplateValue};

/// 一次模板执行期间全部表达式工具对象的容器合同。
///
/// 容器让大多数工具对象在整次模板执行中保持同一实例，同时保留创建它们所使用的
/// [`crate::context::IExpressionContext`]，使上下文相关对象随时读取当前状态。
///
/// 对应 Java: `org.thymeleaf.expression.IExpressionObjects`。
pub trait IExpressionObjects {
    /// 返回工厂声明的表达式对象名称数量。
    ///
    /// # 返回值
    ///
    /// 返回名称集合的 Java `int` 大小。
    fn size(&self) -> i32;

    /// 判断指定名称是否由容器声明。
    ///
    /// # 参数
    ///
    /// - `name`：待查找的可空对象名称。
    ///
    /// # 返回值
    ///
    /// 名称存在于工厂完整名称集合时返回 `true`。
    fn contains_object(&self, name: Option<&JavaString>) -> bool;

    /// 返回对象名称集合。
    ///
    /// # 返回值
    ///
    /// 返回构造容器时从工厂取得的同一共享只读集合。
    fn get_object_names(&self) -> ExpressionObjectNames;

    /// 按名称取得并在策略允许时缓存表达式对象。
    ///
    /// # 参数
    ///
    /// - `name`：待取得对象的可空名称。
    ///
    /// # 返回值
    ///
    /// 未声明名称或工厂构建 Java `null` 时返回 `None`；可缓存的 `None` 也会被记录，
    /// 后续读取不会再次调用工厂。
    ///
    /// # 错误
    ///
    /// 原样返回表达式对象工厂的构建错误。
    fn get_object(
        &self,
        name: Option<&JavaString>,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>>;
}
