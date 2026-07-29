use crate::TemplateMode;
use crate::engine::ITemplateHandler;

/// 创建一个全新 TemplateHandler 实例的工厂函数。
pub type PreProcessorHandlerFactory = fn() -> Box<dyn ITemplateHandler>;

/// 模板事件进入 Processor 链之前执行的预处理器配置合同。
///
/// 对应 Java: `org.thymeleaf.preprocessor.IPreProcessor`。
pub trait IPreProcessor {
    /// 返回唯一适用模板模式。
    fn get_template_mode(&self) -> TemplateMode;
    /// 返回方言优先级之后应用的处理器优先级。
    fn get_precedence(&self) -> i32;
    /// 返回 Java handler class 对应的构造工厂。
    fn get_handler_factory(&self) -> PreProcessorHandlerFactory;
    /// 返回 handler 的稳定类名，保留 Java `Class#getName()` 可观察信息。
    fn get_handler_class_name(&self) -> &'static str;
}
