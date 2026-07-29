use crate::util::JavaString;

use super::ITemplateEvent;

/// 不可变 XML declaration 事件。
///
/// 对应 Java: `org.thymeleaf.model.IXMLDeclaration`。
pub trait IXMLDeclaration: ITemplateEvent {
    /// 返回保持原始大小写的 XML 关键字。
    fn get_keyword(&self) -> Option<&JavaString>;
    /// 返回可空 XML version。
    fn get_version(&self) -> Option<&JavaString>;
    /// 返回可空 encoding。
    fn get_encoding(&self) -> Option<&JavaString>;
    /// 返回可空 standalone。
    fn get_standalone(&self) -> Option<&JavaString>;
    /// 返回完整 XML declaration。
    fn get_xml_declaration(&self) -> Option<&JavaString>;
}
