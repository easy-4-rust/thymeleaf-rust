use std::rc::Rc;
use std::sync::Arc;

use crate::IEngineConfiguration;
use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IModelError, IOpenElementTag,
    IProcessingInstruction, IStandaloneElementTag, ITemplateEnd, ITemplateEvent, ITemplateStart,
    IText, IXMLDeclaration,
};

use super::{
    AbstractTemplateHandler, ITemplateHandler, TemplateData, TemplateEnd, TemplateModel,
    TemplateStart,
};

/// 将解析事件收集为不可变 `TemplateModel`，同时保持处理器链透明。
///
/// 收集时保存同一不可变事件引用，转发时仍传递原引用；模板开始和结束统一规范化为
/// 引擎单例。对应 Java: `org.thymeleaf.engine.ModelBuilderTemplateHandler`。
pub struct ModelBuilderTemplateHandler {
    base: AbstractTemplateHandler,
    events: Vec<Arc<dyn ITemplateEvent>>,
    configuration: Arc<dyn IEngineConfiguration>,
    template_data: Arc<TemplateData>,
}

impl ModelBuilderTemplateHandler {
    /// 创建容量初值为 100 的模型构建处理器。
    pub fn new(
        configuration: Arc<dyn IEngineConfiguration>,
        template_data: Arc<TemplateData>,
    ) -> Self {
        Self {
            base: AbstractTemplateHandler::new(),
            events: Vec::with_capacity(100),
            configuration,
            template_data,
        }
    }

    /// 返回当前已收集事件的不可变完整模板模型。
    pub fn get_model(&self) -> Result<TemplateModel, IModelError> {
        TemplateModel::new(
            Arc::clone(&self.configuration),
            Arc::clone(&self.template_data),
            self.events.clone(),
        )
    }
}

impl ITemplateHandler for ModelBuilderTemplateHandler {
    fn set_next(&mut self, next: Option<Box<dyn ITemplateHandler>>) {
        self.base.set_next(next);
    }

    fn set_context(&mut self, context: Rc<dyn ITemplateContext>) {
        // Java 未覆盖此方法；继承的基础实现保存上下文但构建过程不读取它。
        self.base.set_context(context);
    }

    fn handle_template_start(
        &mut self,
        template_start: Arc<dyn ITemplateStart>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.events.push(TemplateStart::instance());
        self.base.handle_template_start(template_start)
    }

    fn handle_template_end(
        &mut self,
        template_end: Arc<dyn ITemplateEnd>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.events.push(TemplateEnd::instance());
        self.base.handle_template_end(template_end)
    }

    fn handle_xml_declaration(
        &mut self,
        xml_declaration: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.events.push(xml_declaration.clone());
        self.base.handle_xml_declaration(xml_declaration)
    }

    fn handle_doc_type(
        &mut self,
        doc_type: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.events.push(doc_type.clone());
        self.base.handle_doc_type(doc_type)
    }

    fn handle_cdata_section(
        &mut self,
        cdata_section: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.events.push(cdata_section.clone());
        self.base.handle_cdata_section(cdata_section)
    }

    fn handle_comment(
        &mut self,
        comment: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.events.push(comment.clone());
        self.base.handle_comment(comment)
    }

    fn handle_text(
        &mut self,
        text: Arc<dyn IText>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.events.push(text.clone());
        self.base.handle_text(text)
    }

    fn handle_standalone_element(
        &mut self,
        tag: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.events.push(tag.clone());
        self.base.handle_standalone_element(tag)
    }

    fn handle_open_element(
        &mut self,
        tag: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.events.push(tag.clone());
        self.base.handle_open_element(tag)
    }

    fn handle_close_element(
        &mut self,
        tag: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.events.push(tag.clone());
        self.base.handle_close_element(tag)
    }

    fn handle_processing_instruction(
        &mut self,
        instruction: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.events.push(instruction.clone());
        self.base.handle_processing_instruction(instruction)
    }
}
