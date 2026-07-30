use crate::TemplateMode;
use crate::engine::ITemplateHandler;

/// 创建一个全新 TemplateHandler 实例的工厂函数。
pub type PreProcessorHandlerFactory = fn() -> Box<dyn ITemplateHandler>;

/// 模板事件进入 Processor 链之前执行的预处理器配置合同。
///
/// 对应 Java: `org.thymeleaf.preprocessor.IPreProcessor`。
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
    /// 注入全局属性定义仓库；aware 实现需要覆盖。
    fn set_attribute_definitions(
        &self,
        _attribute_definitions: std::sync::Arc<crate::engine::AttributeDefinitions>,
    ) {
    }
    /// 判断该预处理器是否需要元素定义仓库。
    fn is_element_definitions_aware(&self) -> bool {
        false
    }
    /// 注入全局元素定义仓库；aware 实现需要覆盖。
    fn set_element_definitions(
        &self,
        _element_definitions: std::sync::Arc<crate::engine::ElementDefinitions>,
    ) {
    }

    /// 返回唯一适用模板模式。
    fn get_template_mode(&self) -> TemplateMode;
    /// 返回方言优先级之后应用的处理器优先级。
    fn get_precedence(&self) -> i32;
    /// 返回 Java handler class 对应的构造工厂。
    fn get_handler_factory(&self) -> PreProcessorHandlerFactory;
    /// 返回 handler 的稳定类名，保留 Java `Class#getName()` 可观察信息。
    fn get_handler_class_name(&self) -> &'static str;
}
