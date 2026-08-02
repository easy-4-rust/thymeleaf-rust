use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IOpenElementTag, IProcessingInstruction,
    IStandaloneElementTag, ITemplateEnd, ITemplateStart, IText, IXMLDeclaration,
};

use super::gathering_model_execution_state::GatheringModelExecutionState;

/// 模板事件处理流水线合同。
///
/// 对应 Java: `org.thymeleaf.engine.ITemplateHandler`。
pub trait ITemplateHandler {
    /// 为允许 Java handler 重入的内部处理链创建轻量代理。
    ///
    /// 普通第三方 Handler 不需要实现；`ProcessorTemplateHandler` 使用该入口避免在
    /// gathering/iteration 重放期间跨事件持有 `RefCell` 可变借用。
    fn create_reentrant_handler(&self) -> Option<Box<dyn ITemplateHandler>> {
        None
    }

    /// 设置链中的下一处理器。
    fn set_next(&mut self, next: Option<TemplateHandlerHandle>);
    /// 设置本次模板执行上下文。
    fn set_context(&mut self, context: Arc<dyn ITemplateContext>);

    /// 设置下一次 gathering Model 首事件消费的执行快照。
    ///
    /// 对应 Java 引擎内部语义：gathering 状态仅 `ProcessorTemplateHandler` 消费，
    /// 其余 Handler 保持默认忽略（no-op）。
    fn set_current_gathering_model(&mut self, _state: Option<GatheringModelExecutionState>) {}
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
/// Java Handler 引用的共享可变身份。
pub type TemplateHandlerHandle = Rc<RefCell<Box<dyn ITemplateHandler>>>;
