use std::rc::Rc;

use crate::context::ITemplateContext;
use crate::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IOpenElementTag, IProcessingInstruction,
    IStandaloneElementTag, ITemplateEnd, ITemplateStart, IText, IXMLDeclaration,
};

/// 模板事件处理流水线合同。
///
/// 对应 Java: `org.thymeleaf.engine.ITemplateHandler`。
pub trait ITemplateHandler {
    /// 设置链中的下一处理器。
    fn set_next(&mut self, next: Option<Box<dyn ITemplateHandler>>);
    /// 设置本次模板执行上下文。
    fn set_context(&mut self, context: Rc<dyn ITemplateContext>);
    /// 处理模板开始。
    fn handle_template_start(&mut self, template_start: &dyn ITemplateStart);
    /// 处理模板结束。
    fn handle_template_end(&mut self, template_end: &dyn ITemplateEnd);
    /// 处理 XML declaration。
    fn handle_xml_declaration(&mut self, xml_declaration: &dyn IXMLDeclaration);
    /// 处理 DOCTYPE。
    fn handle_doc_type(&mut self, doc_type: &dyn IDocType);
    /// 处理 CDATA。
    fn handle_cdata_section(&mut self, cdata_section: &dyn ICDATASection);
    /// 处理注释。
    fn handle_comment(&mut self, comment: &dyn IComment);
    /// 处理文本。
    fn handle_text(&mut self, text: &dyn IText);
    /// 处理独立标签。
    fn handle_standalone_element(&mut self, tag: &dyn IStandaloneElementTag);
    /// 处理开始标签。
    fn handle_open_element(&mut self, tag: &dyn IOpenElementTag);
    /// 处理结束标签。
    fn handle_close_element(&mut self, tag: &dyn ICloseElementTag);
    /// 处理 processing instruction。
    fn handle_processing_instruction(&mut self, instruction: &dyn IProcessingInstruction);
}
