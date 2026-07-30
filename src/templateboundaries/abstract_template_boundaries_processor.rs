use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::{ITemplateEnd, ITemplateStart};
use crate::processor::{AbstractProcessorAdapter, IProcessor};
use crate::util::ValidateError;

use super::{ITemplateBoundariesProcessor, ITemplateBoundariesStructureHandler};

/// 分别执行模板开始与结束回调并补充事件位置的抽象边界 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.processor.templateboundaries.AbstractTemplateBoundariesProcessor`。
pub struct AbstractTemplateBoundariesProcessor<FStart, FEnd> {
    start_adapter: AbstractProcessorAdapter<FStart>,
    end_callback: FEnd,
}

impl<FStart, FEnd> AbstractTemplateBoundariesProcessor<FStart, FEnd> {
    /// 创建以两个闭包表达 Java 抽象边界处理方法的 Processor。
    pub fn new(
        template_mode: Option<TemplateMode>,
        precedence: i32,
        processor_class_name: &'static str,
        process_start: FStart,
        process_end: FEnd,
    ) -> Result<Self, ValidateError> {
        Ok(Self {
            start_adapter: AbstractProcessorAdapter::new(
                template_mode,
                precedence,
                processor_class_name,
                process_start,
            )?,
            end_callback: process_end,
        })
    }
}

impl<FStart, FEnd> IProcessor for AbstractTemplateBoundariesProcessor<FStart, FEnd>
where
    FStart: Send + Sync,
    FEnd: Send + Sync,
{
    fn java_class_name(&self) -> &'static str {
        self.start_adapter.processor_class_name()
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.start_adapter.template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.start_adapter.precedence()
    }
}

impl<FStart, FEnd> ITemplateBoundariesProcessor
    for AbstractTemplateBoundariesProcessor<FStart, FEnd>
where
    FStart: Fn(
            &dyn ITemplateContext,
            &dyn ITemplateStart,
            &mut dyn ITemplateBoundariesStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
    FEnd: Fn(
            &dyn ITemplateContext,
            &dyn ITemplateEnd,
            &mut dyn ITemplateBoundariesStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
{
    fn process_template_start(
        &self,
        context: &dyn ITemplateContext,
        template_start: &dyn ITemplateStart,
        structure_handler: &mut dyn ITemplateBoundariesStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.start_adapter.execute(template_start, |callback| {
            callback(context, template_start, structure_handler)
        })
    }

    fn process_template_end(
        &self,
        context: &dyn ITemplateContext,
        template_end: &dyn ITemplateEnd,
        structure_handler: &mut dyn ITemplateBoundariesStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.start_adapter.execute(template_end, |_| {
            (self.end_callback)(context, template_end, structure_handler)
        })
    }
}
