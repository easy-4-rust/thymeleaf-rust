use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::IText;
use crate::processor::{AbstractProcessorAdapter, IProcessor};
use crate::util::ValidateError;

use super::{ITextProcessor, ITextStructureHandler};

/// 捕获 `doProcess` 异常并补充文本事件位置的抽象 Text Processor。
///
/// 对应 Java: `org.thymeleaf.processor.text.AbstractTextProcessor`。
pub struct AbstractTextProcessor<F> {
    adapter: AbstractProcessorAdapter<F>,
}

impl<F> AbstractTextProcessor<F> {
    /// 创建以闭包表达 Java 抽象 `doProcess` 方法的 Processor。
    ///
    /// 对应 Java: `AbstractTextProcessor#AbstractTextProcessor(TemplateMode, int)`。
    /// `template_mode` 为 `None` 时保留 `"Template mode cannot be null"` 校验；
    /// `processor_class_name` 用于异常包装，`do_process` 对应子类实现。
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

impl<F> IProcessor for AbstractTextProcessor<F>
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

impl<F> ITextProcessor for AbstractTextProcessor<F>
where
    F: Fn(
            &dyn ITemplateContext,
            &dyn IText,
            &mut dyn ITextStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
{
    fn process(
        &self,
        context: &dyn ITemplateContext,
        text: &dyn IText,
        structure_handler: &mut dyn ITextStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        // 对应 Java AbstractTextProcessor#process：委托 doProcess，并统一装饰异常位置。
        self.adapter
            .execute(text, |callback| callback(context, text, structure_handler))
    }
}
