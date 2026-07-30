use std::sync::Arc;

use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::engine::EngineEventUtils;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::IText;
use crate::processor::IProcessor;
use crate::text::{AbstractTextProcessor, ITextProcessor, ITextStructureHandler};

use super::expression_processing_error;

type TextCallback = Box<
    dyn Fn(
            &dyn ITemplateContext,
            &dyn IText,
            &mut dyn ITextStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
>;

/// 对非空白 Text 事件应用当前上下文 Inliner 的 Processor。
///
/// 内联结果以 CharSequence 直接交给引擎，保留延迟 Writer 输出。对应 Java:
/// `org.thymeleaf.standard.processor.StandardInliningTextProcessor`。
pub struct StandardInliningTextProcessor {
    processor: AbstractTextProcessor<TextCallback>,
}

impl StandardInliningTextProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;

    /// 创建指定模板模式的内联 Processor。
    pub fn new(template_mode: TemplateMode) -> Result<Self, TemplateProcessingException> {
        let callback: TextCallback = Box::new(|context, text, structure_handler| {
            if EngineEventUtils::is_whitespace_text(Some(text)).map_err(|error| {
                Box::new(TemplateProcessingException::with_cause(
                    Some("Could not inspect text whitespace".to_owned()),
                    error,
                )) as Box<dyn TemplateEngineException>
            })? {
                return Ok(());
            }
            let Some(inliner) = context.get_inliner() else {
                return Ok(());
            };
            if let Some(inlined) = inliner
                .inline_text(context, text)
                .map_err(|error| expression_processing_error("Could not inline text", error))?
            {
                structure_handler.set_text_sequence(Arc::from(inlined));
            }
            Ok(())
        });
        Ok(Self {
            processor: AbstractTextProcessor::new(
                Some(template_mode),
                Self::PRECEDENCE,
                "org.thymeleaf.standard.processor.StandardInliningTextProcessor",
                callback,
            )
            .map_err(|error| {
                TemplateProcessingException::with_cause(
                    Some("Could not create inlining text processor".to_owned()),
                    error,
                )
            })?,
        })
    }
}

impl IProcessor for StandardInliningTextProcessor {
    fn as_text_processor(&self) -> Option<&dyn ITextProcessor> {
        Some(self)
    }

    fn java_class_name(&self) -> &'static str {
        self.processor.java_class_name()
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.processor.get_template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.processor.get_precedence()
    }
}

impl ITextProcessor for StandardInliningTextProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        text: &dyn IText,
        structure_handler: &mut dyn ITextStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, text, structure_handler)
    }
}
