use crate::TemplateMode;
use crate::engine::TemplateHandlerClass;

/// 模板事件进入 Processor 链之前执行的预处理器配置合同。
///
/// PreProcessor 是完整 [`crate::engine::ITemplateHandler`]，在模板解析或缓存读取之后、
/// 所有适用 Processor 之前接收模板模型事件，因此可以在正式处理前重塑模型。通常应
/// 使用 [`super::PreProcessor`] 注册配置。
///
/// 对应 Java: `org.thymeleaf.preprocessor.IPreProcessor`。
///
/// # 起始版本
///
/// 上游自 Thymeleaf 3.0.0 提供该接口。
pub trait IPreProcessor: Send + Sync {
    /// 返回包装器附加的方言级优先级；普通实现返回 `None`。
    fn get_dialect_precedence(&self) -> Option<i32> {
        None
    }
    /// 返回包装前的 PreProcessor；普通实现返回 `None`。
    fn get_wrapped_pre_processor(&self) -> Option<&dyn IPreProcessor> {
        None
    }
    /// 判断该预处理器是否需要属性定义仓库。
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
    /// 判断该预处理器是否需要元素定义仓库。
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

    /// 返回该 PreProcessor 唯一适用的模板模式。
    ///
    /// # 返回值
    ///
    /// 返回一个具体模板模式；`None` 对应非法第三方 Java 实现返回 `null`，由
    /// `DialectSetConfiguration` 在聚合阶段拒绝。
    fn get_template_mode(&self) -> Option<TemplateMode>;

    /// 返回方言级优先级之后应用的 PreProcessor 优先级。
    ///
    /// # 返回值
    ///
    /// 返回用于确定同一方言级别内执行顺序的完整有符号 32 位优先级。
    fn get_precedence(&self) -> i32;

    /// 返回实现实际预处理逻辑的 Handler 类型令牌。
    ///
    /// Handler 必须正确实现完整模板事件合同。组合
    /// [`crate::engine::AbstractTemplateHandler`] 可以复用默认转发行为。
    ///
    /// # 返回值
    ///
    /// 返回可重复创建全新 Handler 实例的稳定类型令牌；`None` 对应非法第三方
    /// Java 实现返回 `null`，由配置聚合阶段拒绝。
    fn get_handler_class(&self) -> Option<&TemplateHandlerClass>;

    /// 返回 PreProcessor 配置对象本身的稳定类名。
    ///
    /// 该名称用于复现 Java 比较器在优先级相同时按 PreProcessor 实现类排序的规则，
    /// 不能替换成 Handler 类名。
    ///
    /// # 返回值
    ///
    /// 返回当前具体实现的稳定完整类名。
    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
