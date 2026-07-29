use super::ITemplateHandler;
use crate::model::ITemplateEvent;

/// 引擎内部可直接分派给 TemplateHandler 的事件合同。
///
/// 对应 Java: `org.thymeleaf.engine.IEngineTemplateEvent`。
pub trait IEngineTemplateEvent: ITemplateEvent {
    /// 把当前事件交给其对应的 handler 重载。
    fn be_handled(&self, handler: &mut dyn ITemplateHandler);
}
