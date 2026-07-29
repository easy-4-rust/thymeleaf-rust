use std::rc::Rc;
use std::sync::Arc;

use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IOpenElementTag, IProcessingInstruction,
    IStandaloneElementTag, ITemplateEnd, ITemplateStart, IText, IXMLDeclaration,
};

use super::ITemplateHandler;

/// 默认把全部模板事件转发给链中下一处理器的基础实现。
///
/// Rust 没有类继承，扩展处理器可以组合此对象，并将未消费的事件交给对应
/// `handle_*` 方法。对应 Java: `org.thymeleaf.engine.AbstractTemplateHandler`。
pub struct AbstractTemplateHandler {
    next: Option<Box<dyn ITemplateHandler>>,
    context: Option<Rc<dyn ITemplateContext>>,
}

impl AbstractTemplateHandler {
    /// 创建尚未连接下一处理器的基础处理器。
    pub fn new() -> Self {
        Self {
            next: None,
            context: None,
        }
    }

    /// 创建并连接下一处理器的基础处理器。
    ///
    /// 参数 `next` 对应 Java 构造方法中的下一链路节点。
    pub fn with_next(next: Box<dyn ITemplateHandler>) -> Self {
        Self {
            next: Some(next),
            context: None,
        }
    }

    /// 返回链中下一处理器，供组合型扩展处理器转发事件。
    pub fn get_next(&self) -> Option<&dyn ITemplateHandler> {
        self.next.as_deref()
    }

    /// 返回链中下一处理器的可变引用。
    pub fn get_next_mut(&mut self) -> Option<&mut (dyn ITemplateHandler + 'static)> {
        self.next.as_deref_mut()
    }

    /// 返回当前模板执行上下文；设置上下文前返回 `None`。
    pub fn get_context(&self) -> Option<&dyn ITemplateContext> {
        self.context.as_deref()
    }
}

impl Default for AbstractTemplateHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ITemplateHandler for AbstractTemplateHandler {
    fn set_next(&mut self, next: Option<Box<dyn ITemplateHandler>>) {
        self.next = next;
    }

    fn set_context(&mut self, context: Rc<dyn ITemplateContext>) {
        self.context = Some(context);
    }

    fn handle_template_start(
        &mut self,
        template_start: Arc<dyn ITemplateStart>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if let Some(next) = self.next.as_deref_mut() {
            return next.handle_template_start(template_start);
        }
        Ok(())
    }

    fn handle_template_end(
        &mut self,
        template_end: Arc<dyn ITemplateEnd>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if let Some(next) = self.next.as_deref_mut() {
            return next.handle_template_end(template_end);
        }
        Ok(())
    }

    fn handle_xml_declaration(
        &mut self,
        xml_declaration: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if let Some(next) = self.next.as_deref_mut() {
            return next.handle_xml_declaration(xml_declaration);
        }
        Ok(())
    }

    fn handle_doc_type(
        &mut self,
        doc_type: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if let Some(next) = self.next.as_deref_mut() {
            return next.handle_doc_type(doc_type);
        }
        Ok(())
    }

    fn handle_cdata_section(
        &mut self,
        cdata_section: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if let Some(next) = self.next.as_deref_mut() {
            return next.handle_cdata_section(cdata_section);
        }
        Ok(())
    }

    fn handle_comment(
        &mut self,
        comment: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if let Some(next) = self.next.as_deref_mut() {
            return next.handle_comment(comment);
        }
        Ok(())
    }

    fn handle_text(
        &mut self,
        text: Arc<dyn IText>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if let Some(next) = self.next.as_deref_mut() {
            return next.handle_text(text);
        }
        Ok(())
    }

    fn handle_standalone_element(
        &mut self,
        tag: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if let Some(next) = self.next.as_deref_mut() {
            return next.handle_standalone_element(tag);
        }
        Ok(())
    }

    fn handle_open_element(
        &mut self,
        tag: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if let Some(next) = self.next.as_deref_mut() {
            return next.handle_open_element(tag);
        }
        Ok(())
    }

    fn handle_close_element(
        &mut self,
        tag: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if let Some(next) = self.next.as_deref_mut() {
            return next.handle_close_element(tag);
        }
        Ok(())
    }

    fn handle_processing_instruction(
        &mut self,
        instruction: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        if let Some(next) = self.next.as_deref_mut() {
            return next.handle_processing_instruction(instruction);
        }
        Ok(())
    }
}
