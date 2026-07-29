use super::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IOpenElementTag, IProcessingInstruction,
    IStandaloneElementTag, ITemplateEnd, ITemplateStart, IText, IXMLDeclaration,
};

/// 按具体事件类型访问模型的 Visitor 合同。
///
/// 对应 Java: `org.thymeleaf.model.IModelVisitor`。
pub trait IModelVisitor {
    /// 访问模板开始事件。
    fn visit_template_start(&mut self, template_start: &dyn ITemplateStart);
    /// 访问模板结束事件。
    fn visit_template_end(&mut self, template_end: &dyn ITemplateEnd);
    /// 访问 XML declaration。
    fn visit_xml_declaration(&mut self, xml_declaration: &dyn IXMLDeclaration);
    /// 访问 DOCTYPE。
    fn visit_doc_type(&mut self, doc_type: &dyn IDocType);
    /// 访问 CDATA section。
    fn visit_cdata_section(&mut self, cdata_section: &dyn ICDATASection);
    /// 访问注释。
    fn visit_comment(&mut self, comment: &dyn IComment);
    /// 访问文本。
    fn visit_text(&mut self, text: &dyn IText);
    /// 访问独立元素标签。
    fn visit_standalone_element_tag(&mut self, standalone_element_tag: &dyn IStandaloneElementTag);
    /// 访问打开元素标签。
    fn visit_open_element_tag(&mut self, open_element_tag: &dyn IOpenElementTag);
    /// 访问关闭元素标签。
    fn visit_close_element_tag(&mut self, close_element_tag: &dyn ICloseElementTag);
    /// 访问 processing instruction。
    fn visit_processing_instruction(&mut self, processing_instruction: &dyn IProcessingInstruction);
}
