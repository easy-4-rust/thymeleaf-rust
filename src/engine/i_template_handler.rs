use std::rc::Rc;
use std::sync::Arc;

use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
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
    fn handle_template_start(
        &mut self,
        template_start: Arc<dyn ITemplateStart>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 处理模板结束。
    fn handle_template_end(
        &mut self,
        template_end: Arc<dyn ITemplateEnd>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 处理 XML declaration。
    fn handle_xml_declaration(
        &mut self,
        xml_declaration: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 处理 DOCTYPE。
    fn handle_doc_type(
        &mut self,
        doc_type: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 处理 CDATA。
    fn handle_cdata_section(
        &mut self,
        cdata_section: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 处理注释。
    fn handle_comment(
        &mut self,
        comment: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 处理文本。
    fn handle_text(&mut self, text: Arc<dyn IText>)
    -> Result<(), Box<dyn TemplateEngineException>>;
    /// 处理独立标签。
    fn handle_standalone_element(
        &mut self,
        tag: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 处理开始标签。
    fn handle_open_element(
        &mut self,
        tag: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 处理结束标签。
    fn handle_close_element(
        &mut self,
        tag: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 处理 processing instruction。
    fn handle_processing_instruction(
        &mut self,
        instruction: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
}
