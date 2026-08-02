use crate::TemplateMode;
use crate::engine::TemplateHandlerClass;

/// Processor 链完成后、实际输出前执行的后处理器配置合同。
///
/// PostProcessor 是完整 [`crate::engine::ITemplateHandler`]，在模板事件经过所有适用
/// Processor 之后、真实输出之前接收事件，因此可以在输出前重塑处理结果。通常应使用
/// [`super::PostProcessor`] 注册配置。
///
/// 对应 Java: `org.thymeleaf.postprocessor.IPostProcessor`。
///
/// # 起始版本
///
/// 上游自 Thymeleaf 3.0.0 提供该接口。
pub trait IPostProcessor: Send + Sync {
    /// 返回包装器附加的方言级优先级；普通实现返回 `None`。
    fn get_dialect_precedence(&self) -> Option<i32> {
        None
    }
    /// 返回包装前的 PostProcessor；普通实现返回 `None`。
    fn get_wrapped_post_processor(&self) -> Option<&dyn IPostProcessor> {
        None
    }
    /// 判断该后处理器是否需要属性定义仓库。
    fn is_attribute_definitions_aware(&self) -> bool {
        false
    }
    /// 注入全局属性定义仓库。
    ///
    /// 对应 Java: `IAttributeDefinitionsAware#setAttributeDefinitions()`。Java 侧 awareness
    /// 是可选标记接口：未实现它的 Processor/PreProcessor/PostProcessor 不需要仓库，
    /// 此处空默认即等价于未实现该标记接口（no-op）。
    fn set_attribute_definitions(
        &self,
        _attribute_definitions: std::sync::Arc<crate::engine::AttributeDefinitions>,
    ) {
    }
    /// 判断该后处理器是否需要元素定义仓库。
    fn is_element_definitions_aware(&self) -> bool {
        false
    }
    /// 注入全局元素定义仓库。
    ///
    /// 对应 Java: `IElementDefinitionsAware#setElementDefinitions()`；未实现该可选标记
    /// 接口的对象保持 no-op 默认，与 `IAttributeDefinitionsAware` 同机制。
    fn set_element_definitions(
        &self,
        _element_definitions: std::sync::Arc<crate::engine::ElementDefinitions>,
    ) {
    }

    /// 返回该 PostProcessor 唯一适用的模板模式。
    ///
    /// # 返回值
    ///
    /// 返回一个具体模板模式；`None` 对应非法第三方 Java 实现返回 `null`，由
    /// `DialectSetConfiguration` 在聚合阶段拒绝。
    fn get_template_mode(&self) -> Option<TemplateMode>;

    /// 返回方言级优先级之后应用的 PostProcessor 优先级。
    ///
    /// # 返回值
    ///
    /// 返回用于确定同一方言级别内执行顺序的完整有符号 32 位优先级。
    fn get_precedence(&self) -> i32;

    /// 返回实现实际后处理逻辑的 Handler 类型令牌。
    ///
    /// Handler 必须正确实现完整模板事件合同。组合
    /// [`crate::engine::AbstractTemplateHandler`] 可以复用默认转发行为。
    ///
    /// # 返回值
    ///
    /// 返回可重复创建全新 Handler 实例的稳定类型令牌；`None` 对应非法第三方
    /// Java 实现返回 `null`，由配置聚合阶段拒绝。
    fn get_handler_class(&self) -> Option<&TemplateHandlerClass>;

    /// 返回 PostProcessor 配置对象本身的稳定类名。
    ///
    /// 该名称用于复现 Java 比较器在优先级相同时按 PostProcessor 实现类排序的规则，
    /// 不能替换成 Handler 类名。
    ///
    /// # 返回值
    ///
    /// 返回当前具体实现的稳定完整类名。
    fn java_class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
