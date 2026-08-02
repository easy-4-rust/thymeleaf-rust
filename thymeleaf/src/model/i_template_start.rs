use super::ITemplateEvent;

/// 标记一次模板处理开始的不可变事件。
///
/// 对应 Java: `org.thymeleaf.model.ITemplateStart`。
pub trait ITemplateStart: ITemplateEvent {}
