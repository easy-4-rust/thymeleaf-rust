use std::sync::Arc;

use crate::IEngineConfiguration;
use crate::expression::IExpressionObjects;

use super::IContext;

/// 表达式求值所需的上下文合同。
///
/// 对应 Java: `org.thymeleaf.context.IExpressionContext`。
///
/// 本接口扩展 [`IContext`]，增加表达式执行所需的引擎配置和惰性表达式对象容器。
/// 与 Java Javadoc 一致，实现面向一次模板执行，不应跨不同执行共享业务状态。
pub trait IExpressionContext: IContext {
    /// 返回当前模板引擎配置。
    ///
    /// # 返回值
    ///
    /// 返回创建当前表达式 Context 时传入的同一配置对象。
    fn get_configuration(&self) -> &dyn IEngineConfiguration;
    /// 返回当前模板引擎配置的共享身份。
    ///
    /// Rust Handler/Model 需要把配置安全保存到延迟执行对象中；该共享引用与 Java
    /// 上下文持有的同一配置对象身份等价。
    ///
    /// # 返回值
    ///
    /// 返回与 [`Self::get_configuration`] 指向同一逻辑对象的共享引用。
    fn get_configuration_arc(&self) -> Arc<dyn IEngineConfiguration>;
    /// 返回表达式工具对象容器。
    ///
    /// 第一次调用时可以惰性创建，后续调用必须返回同一容器身份。
    ///
    /// # 返回值
    ///
    /// 返回本次表达式执行使用的对象容器。
    fn get_expression_objects(&self) -> &dyn IExpressionObjects;
}
