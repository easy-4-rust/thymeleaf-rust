use super::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IModelVisitor, IOpenElementTag,
    IProcessingInstruction, IStandaloneElementTag, ITemplateEnd, ITemplateStart, IText,
    IXMLDeclaration,
};

/// 为全部模型事件提供空操作默认访问行为的 Visitor 基类。
///
/// 对应 Java: `org.thymeleaf.model.AbstractModelVisitor`。
///
/// Rust 用户可以组合该零状态对象，或直接参考这些精确空操作实现，仅覆盖感兴趣的
/// 事件类型。
#[derive(Clone, Copy, Debug, Default)]
pub struct AbstractModelVisitor;

impl AbstractModelVisitor {
    /// 创建无状态默认 Visitor。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl IModelVisitor for AbstractModelVisitor {
    /// 对应 Java: `AbstractModelVisitor#visitTemplateStart()` 的空默认实现（no-op）。
    fn visit_template_start(&mut self, _template_start: &dyn ITemplateStart) {}

    /// 对应 Java: `AbstractModelVisitor#visitTemplateEnd()` 的空默认实现（no-op）。
    fn visit_template_end(&mut self, _template_end: &dyn ITemplateEnd) {}

    /// 对应 Java: `AbstractModelVisitor#visitXMLDeclaration()` 的空默认实现（no-op）。
    fn visit_xml_declaration(&mut self, _xml_declaration: &dyn IXMLDeclaration) {}

    /// 对应 Java: `AbstractModelVisitor#visitDocType()` 的空默认实现（no-op）。
    fn visit_doc_type(&mut self, _doc_type: &dyn IDocType) {}

    /// 对应 Java: `AbstractModelVisitor#visitCDATASection()` 的空默认实现（no-op）。
    fn visit_cdata_section(&mut self, _cdata_section: &dyn ICDATASection) {}

    /// 对应 Java: `AbstractModelVisitor#visitComment()` 的空默认实现（no-op）。
    fn visit_comment(&mut self, _comment: &dyn IComment) {}

    /// 对应 Java: `AbstractModelVisitor#visitText()` 的空默认实现（no-op）。
    fn visit_text(&mut self, _text: &dyn IText) {}

    /// 对应 Java: `AbstractModelVisitor#visitStandaloneElementTag()` 的空默认实现（no-op）。
    fn visit_standalone_element_tag(
        &mut self,
        _standalone_element_tag: &dyn IStandaloneElementTag,
    ) {
    }

    /// 对应 Java: `AbstractModelVisitor#visitOpenElementTag()` 的空默认实现（no-op）。
    fn visit_open_element_tag(&mut self, _open_element_tag: &dyn IOpenElementTag) {}

    /// 对应 Java: `AbstractModelVisitor#visitCloseElementTag()` 的空默认实现（no-op）。
    fn visit_close_element_tag(&mut self, _close_element_tag: &dyn ICloseElementTag) {}

    /// 对应 Java: `AbstractModelVisitor#visitProcessingInstruction()` 的空默认实现（no-op）。
    fn visit_processing_instruction(
        &mut self,
        _processing_instruction: &dyn IProcessingInstruction,
    ) {
    }
}
