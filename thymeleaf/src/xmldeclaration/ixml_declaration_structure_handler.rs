use std::sync::Arc;

use crate::model::IModel;
use crate::util::Utf16String;

/// XMLDeclaration Processor 的结构变更合同。
///
/// 对应 Java:
/// `org.thymeleaf.processor.xmldeclaration.IXMLDeclarationStructureHandler`。
pub trait IXMLDeclarationStructureHandler {
    /// 清除已指定动作。对应 Java: `IXMLDeclarationStructureHandler#reset()`。
    fn reset(&mut self);
    /// 设置 declaration 的全部属性。
    ///
    /// 对应 Java: `IXMLDeclarationStructureHandler#setXMLDeclaration(String,
    /// String, String, String)`。keyword 非空，其他属性允许为空。
    fn set_xml_declaration(
        &mut self,
        keyword: Utf16String,
        version: Option<Utf16String>,
        encoding: Option<Utf16String>,
        standalone: Option<Utf16String>,
    );
    /// 使用模型替换当前事件。对应 Java:
    /// `IXMLDeclarationStructureHandler#replaceWith(IModel, boolean)`。
    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 删除当前 declaration。对应 Java:
    /// `IXMLDeclarationStructureHandler#removeXMLDeclaration()`。
    fn remove_xml_declaration(&mut self);
}
