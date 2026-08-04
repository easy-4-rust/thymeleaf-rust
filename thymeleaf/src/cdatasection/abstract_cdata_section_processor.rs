use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::ICDATASection;
use crate::processor::{AbstractProcessorAdapter, IProcessor};
use crate::util::ValidateError;

use super::{ICDATASectionProcessor, ICDATASectionStructureHandler};

/// 捕获 `doProcess` 异常并补充 CDATA 事件位置的抽象 Processor。
///
/// 对应 Java: `org.thymeleaf.processor.cdatasection.AbstractCDATASectionProcessor`。
pub struct AbstractCDATASectionProcessor<F> {
    adapter: AbstractProcessorAdapter<F>,
}

impl<F> AbstractCDATASectionProcessor<F> {
    /// 创建以闭包表达 Java 抽象 `doProcess` 方法的 Processor。
    ///
    /// 对应 Java:
    /// `AbstractCDATASectionProcessor#AbstractCDATASectionProcessor(TemplateMode, int)`。
    /// 构造时校验模板模式；回调异常按 Java `process` 规则保留或包装。
    pub fn new(
        template_mode: Option<TemplateMode>,
        precedence: i32,
        processor_class_name: &'static str,
        do_process: F,
    ) -> Result<Self, ValidateError> {
        Ok(Self {
            adapter: AbstractProcessorAdapter::new(
                template_mode,
                precedence,
                processor_class_name,
                do_process,
            )?,
        })
    }
}

impl<F> IProcessor for AbstractCDATASectionProcessor<F>
where
    F: Send + Sync,
{
    fn class_name(&self) -> &'static str {
        self.adapter.processor_class_name()
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.adapter.template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.adapter.precedence()
    }
}

impl<F> ICDATASectionProcessor for AbstractCDATASectionProcessor<F>
where
    F: Fn(
            &dyn ITemplateContext,
            &dyn ICDATASection,
            &mut dyn ICDATASectionStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
{
    fn process(
        &self,
        context: &dyn ITemplateContext,
        cdata_section: &dyn ICDATASection,
        structure_handler: &mut dyn ICDATASectionStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        // 对应 Java AbstractCDATASectionProcessor#process。
        self.adapter.execute(cdata_section, |callback| {
            callback(context, cdata_section, structure_handler)
        })
    }
}
