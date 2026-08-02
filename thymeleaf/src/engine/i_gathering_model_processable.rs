use std::sync::Arc;

use crate::exceptions::TemplateEngineException;
use crate::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IOpenElementTag, IProcessingInstruction,
    IStandaloneElementTag, IText, IXMLDeclaration,
};

use super::IEngineProcessable;
use super::model::Model;
use super::processor_execution_vars::ProcessorExecutionVars;

/// 收集一段模板事件并在之后重放的 Engine processable 合同。
///
/// 对应 Java: `org.thymeleaf.engine.IGatheringModelProcessable`。
#[expect(
    dead_code,
    reason = "完整保留 Java IGatheringModelProcessable 内部 SPI"
)]
/// 对应 Java: `IGatheringModelProcessable`。
pub(crate) trait IGatheringModelProcessable: IEngineProcessable {
    /// 判断对应元素的事件边界是否已经完整收集。
    fn is_gathering_finished(&self) -> bool;
    /// 返回内部合成 Model。
    fn get_inner_model(&self) -> &Model;
    /// 恢复收集前的 body/close skip 标志。
    fn reset_gathered_skip_flags(&self);
    /// 返回本次或本次迭代使用的 Processor 状态。
    fn initialize_processor_execution_vars(&self) -> ProcessorExecutionVars;
    /// 收集 Text。
    fn gather_text(&mut self, text: Arc<dyn IText>)
    -> Result<(), Box<dyn TemplateEngineException>>;
    /// 收集 Comment。
    fn gather_comment(
        &mut self,
        comment: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 收集 CDATA。
    fn gather_cdata_section(
        &mut self,
        cdata_section: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 收集独立元素。
    fn gather_standalone_element(
        &mut self,
        tag: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 收集开放元素。
    fn gather_open_element(
        &mut self,
        tag: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 收集匹配关闭元素。
    fn gather_close_element(
        &mut self,
        tag: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 收集不匹配关闭元素。
    fn gather_unmatched_close_element(
        &mut self,
        tag: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 收集 DOCTYPE。
    fn gather_doc_type(
        &mut self,
        doc_type: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 收集 XML declaration。
    fn gather_xml_declaration(
        &mut self,
        declaration: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
    /// 收集 processing instruction。
    fn gather_processing_instruction(
        &mut self,
        instruction: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
}
