use crate::model::ITemplateEvent;

/// 引擎内部可直接分派给 TemplateHandler 的事件合同。
///
/// 对应 Java: `org.thymeleaf.engine.IEngineTemplateEvent`。
pub trait IEngineTemplateEvent: ITemplateEvent {}
