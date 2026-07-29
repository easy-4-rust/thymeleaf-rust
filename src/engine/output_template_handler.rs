use std::rc::Rc;
use std::sync::Arc;

use crate::context::ITemplateContext;
use crate::exceptions::{TemplateEngineException, TemplateOutputException};
use crate::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IOpenElementTag, IProcessingInstruction,
    IStandaloneElementTag, ITemplateEnd, ITemplateEvent, ITemplateStart, IText, IXMLDeclaration,
};
use crate::util::JavaWriter;

use super::{AbstractTemplateHandler, ITemplateHandler};

const OUTPUT_ERROR_MESSAGE: &str = "An error happened during template rendering";

/// 将处理后的模板事件写入目标 Writer 的终端处理器。
///
/// 任意事件写出失败都会携带该事件的模板名、行、列包装为
/// `TemplateOutputException`，成功写出后仍会转发给可能存在的下一处理器。
/// 对应 Java: `org.thymeleaf.engine.OutputTemplateHandler`。
pub struct OutputTemplateHandler {
    base: AbstractTemplateHandler,
    writer: Box<dyn JavaWriter>,
}

impl OutputTemplateHandler {
    /// 创建拥有指定输出 Writer 的处理器。
    pub fn new(writer: Box<dyn JavaWriter>) -> Self {
        Self {
            base: AbstractTemplateHandler::new(),
            writer,
        }
    }

    fn write_event(
        &mut self,
        event: &dyn ITemplateEvent,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        event.write(self.writer.as_mut()).map_err(|cause| {
            let template_name = event.get_template_name().map(|name| name.to_string_lossy());
            Box::new(TemplateOutputException::new(
                Some(OUTPUT_ERROR_MESSAGE.to_owned()),
                template_name,
                event.get_line(),
                event.get_col(),
                cause,
            )) as Box<dyn TemplateEngineException>
        })
    }
}

impl ITemplateHandler for OutputTemplateHandler {
    fn set_next(&mut self, next: Option<Box<dyn ITemplateHandler>>) {
        self.base.set_next(next);
    }

    fn set_context(&mut self, context: Rc<dyn ITemplateContext>) {
        self.base.set_context(context);
    }

    fn handle_template_start(
        &mut self,
        template_start: Arc<dyn ITemplateStart>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base.handle_template_start(template_start)
    }

    fn handle_template_end(
        &mut self,
        template_end: Arc<dyn ITemplateEnd>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.base.handle_template_end(template_end)
    }

    fn handle_xml_declaration(
        &mut self,
        xml_declaration: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.write_event(xml_declaration.as_ref())?;
        self.base.handle_xml_declaration(xml_declaration)
    }

    fn handle_doc_type(
        &mut self,
        doc_type: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.write_event(doc_type.as_ref())?;
        self.base.handle_doc_type(doc_type)
    }

    fn handle_cdata_section(
        &mut self,
        cdata_section: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.write_event(cdata_section.as_ref())?;
        self.base.handle_cdata_section(cdata_section)
    }

    fn handle_comment(
        &mut self,
        comment: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.write_event(comment.as_ref())?;
        self.base.handle_comment(comment)
    }

    fn handle_text(
        &mut self,
        text: Arc<dyn IText>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.write_event(text.as_ref())?;
        self.base.handle_text(text)
    }

    fn handle_standalone_element(
        &mut self,
        tag: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.write_event(tag.as_ref())?;
        self.base.handle_standalone_element(tag)
    }

    fn handle_open_element(
        &mut self,
        tag: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.write_event(tag.as_ref())?;
        self.base.handle_open_element(tag)
    }

    fn handle_close_element(
        &mut self,
        tag: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.write_event(tag.as_ref())?;
        self.base.handle_close_element(tag)
    }

    fn handle_processing_instruction(
        &mut self,
        instruction: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.write_event(instruction.as_ref())?;
        self.base.handle_processing_instruction(instruction)
    }
}
