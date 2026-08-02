use std::any::TypeId;
use std::sync::Arc;

use crate::context::ITemplateContext;
use crate::expression::TemplateValue;
use crate::util::JavaString;

use super::MessageResolutionResult;

/// 外部化、国际化消息解析器合同。
///
/// 对应 Java: `org.thymeleaf.messageresolver.IMessageResolver`。
///
/// 实现必须可被多个渲染线程安全共享。引擎按照 `get_order()` 排序解析器，并以
/// “首个非空结果胜出”的方式逐个调用；`None` 只表示当前解析器无法解析，并不是空消息。
/// 整条解析器链均未命中后，引擎再按相同顺序请求 absent representation；若仍全部返回
/// `None`，引擎最终使用空字符串。`origin` 表示触发消息解析的模板对象类型，参数数组
/// 本身以及数组中的元素都允许为空，与 Java 的 `Object[]` 边界一致。
pub trait IMessageResolver: Send + Sync {
    /// 返回可空解析器名称。
    ///
    /// # 返回值
    ///
    /// 配置的名称；`None` 对应 Java `null`。
    fn get_name(&self) -> Option<&JavaString>;

    /// 返回可空链式顺序。
    ///
    /// # 返回值
    ///
    /// 数值越小越先执行；`None` 表示未显式指定顺序。
    fn get_order(&self) -> Option<i32>;

    /// 根据上下文、可选 origin、key 与可选参数解析消息；未命中返回 `None`。
    ///
    /// 对应 Java: `IMessageResolver#resolveMessage(...)`。
    ///
    /// # 参数
    ///
    /// - `context`：当前模板上下文；实现可以像 Java 一样拒绝空值。
    /// - `origin`：消息来源对象类型；没有来源时为 `None`。
    /// - `key`：待解析的消息键；实现可以像 Java 一样拒绝空值。
    /// - `message_parameters`：可空参数数组；元素也可为空。
    ///
    /// # 返回值
    ///
    /// 命中的格式化消息、未命中的 `None`，或与 Java 运行时异常等价的错误。
    fn resolve_message_nullable(
        &self,
        context: Option<&dyn ITemplateContext>,
        origin: Option<TypeId>,
        key: Option<&JavaString>,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>>;

    /// 为 Rust 非空调用者提供的便利入口。
    ///
    /// # 参数
    ///
    /// 参数语义与 [`IMessageResolver::resolve_message_nullable`] 相同，但上下文和键非空。
    ///
    /// # 返回值
    ///
    /// 命中的格式化消息、未命中的 `None`，或解析错误。
    fn resolve_message(
        &self,
        context: &dyn ITemplateContext,
        origin: Option<TypeId>,
        key: &JavaString,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>> {
        self.resolve_message_nullable(Some(context), origin, Some(key), message_parameters)
    }
    /// 创建未命中消息的表示；无法创建时返回 `None`。
    ///
    /// 对应 Java: `IMessageResolver#createAbsentMessageRepresentation(...)`。
    ///
    /// # 参数
    ///
    /// 参数含义与 `resolveMessage` 相同。此方法只在所有解析器均未命中后调用。
    ///
    /// # 返回值
    ///
    /// 缺失消息的可显示文本、无法表示时的 `None`，或边界校验错误。
    fn create_absent_message_representation_nullable(
        &self,
        context: Option<&dyn ITemplateContext>,
        origin: Option<TypeId>,
        key: Option<&JavaString>,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>>;

    /// 为 Rust 非空调用者提供的 absent representation 便利入口。
    ///
    /// # 返回值
    ///
    /// 缺失消息表示、`None`，或解析错误。
    fn create_absent_message_representation(
        &self,
        context: &dyn ITemplateContext,
        origin: Option<TypeId>,
        key: &JavaString,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>> {
        self.create_absent_message_representation_nullable(
            Some(context),
            origin,
            Some(key),
            message_parameters,
        )
    }
}
