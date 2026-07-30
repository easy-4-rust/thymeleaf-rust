use std::sync::Arc;

use crate::model::IModel;
use crate::util::JavaString;

/// XMLDeclaration Processor 的结构变更合同。
///
/// 对应 Java:
/// `org.thymeleaf.processor.xmldeclaration.IXMLDeclarationStructureHandler`。
pub trait IXMLDeclarationStructureHandler {
    /// 清除已指定动作。
    fn reset(&mut self);
    /// 设置 declaration 的全部属性。
    fn set_xml_declaration(
        &mut self,
        keyword: JavaString,
        version: Option<JavaString>,
        encoding: Option<JavaString>,
        standalone: Option<JavaString>,
    );
    /// 使用模型替换当前事件。
    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 删除当前 declaration。
    fn remove_xml_declaration(&mut self);
}
