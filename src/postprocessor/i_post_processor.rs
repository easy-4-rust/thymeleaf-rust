use crate::TemplateMode;
use crate::engine::ITemplateHandler;

/// 创建一个全新后处理 TemplateHandler 实例的工厂函数。
pub type PostProcessorHandlerFactory = fn() -> Box<dyn ITemplateHandler>;

/// Processor 链完成后、实际输出前执行的后处理器配置合同。
///
/// 对应 Java: `org.thymeleaf.postprocessor.IPostProcessor`。
pub trait IPostProcessor {
    /// 返回唯一适用模板模式。
    fn get_template_mode(&self) -> TemplateMode;
    /// 返回方言优先级之后应用的处理器优先级。
    fn get_precedence(&self) -> i32;
    /// 返回 Java handler class 对应的构造工厂。
    fn get_handler_factory(&self) -> PostProcessorHandlerFactory;
    /// 返回 handler 的稳定类名。
    fn get_handler_class_name(&self) -> &'static str;
}
