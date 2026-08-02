use super::ITemplateEvent;

/// 标记一次模板处理结束的不可变事件。
///
/// 对应 Java: `org.thymeleaf.model.ITemplateEnd`。
pub trait ITemplateEnd: ITemplateEvent {}
