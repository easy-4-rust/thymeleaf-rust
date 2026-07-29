use crate::util::JavaString;

use super::ITemplateEvent;

/// 不可变 DOCTYPE 模板事件。
///
/// 对应 Java: `org.thymeleaf.model.IDocType`。
pub trait IDocType: ITemplateEvent {
    /// 返回保持原始大小写的 DOCTYPE 关键字。
    fn get_keyword(&self) -> Option<&JavaString>;
    /// 返回 DOCTYPE 根元素名。
    fn get_element_name(&self) -> Option<&JavaString>;
    /// 返回可空 DOCTYPE 类型，通常为 `PUBLIC` 或 `SYSTEM`。
    fn get_type(&self) -> Option<&JavaString>;
    /// 返回可空 PUBLIC ID。
    fn get_public_id(&self) -> Option<&JavaString>;
    /// 返回可空 SYSTEM ID。
    fn get_system_id(&self) -> Option<&JavaString>;
    /// 返回可空内部子集。
    fn get_internal_subset(&self) -> Option<&JavaString>;
    /// 返回完整 DOCTYPE 子句。
    fn get_doc_type(&self) -> Option<&JavaString>;
}
