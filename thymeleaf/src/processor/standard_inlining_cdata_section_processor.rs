use std::sync::Arc;

use crate::TemplateMode;
use crate::cdatasection::{
    AbstractCDATASectionProcessor, ICDATASectionProcessor, ICDATASectionStructureHandler,
};
use crate::context::ITemplateContext;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::ICDATASection;
use crate::processor::IProcessor;

use super::expression_processing_error;

type CDATACallback = Box<
    dyn Fn(
            &dyn ITemplateContext,
            &dyn ICDATASection,
            &mut dyn ICDATASectionStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
>;

/// 对 CDATA 内容应用当前上下文 Inliner 的 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardInliningCDATASectionProcessor`。
pub struct StandardInliningCDATASectionProcessor {
    processor: AbstractCDATASectionProcessor<CDATACallback>,
}

impl StandardInliningCDATASectionProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;

    /// 创建指定模板模式的内联 Processor。
    pub fn new(template_mode: TemplateMode) -> Result<Self, TemplateProcessingException> {
        let callback: CDATACallback = Box::new(|context, cdata_section, structure_handler| {
            let Some(inliner) = context.get_inliner() else {
                return Ok(());
            };
            if let Some(inlined) = inliner
                .inline_cdata_section(context, cdata_section)
                .map_err(|error| {
                    expression_processing_error("Could not inline CDATA section", error)
                })?
            {
                structure_handler.set_content_sequence(Arc::from(inlined));
            }
            Ok(())
        });
        Ok(Self {
            processor: AbstractCDATASectionProcessor::new(
                Some(template_mode),
                Self::PRECEDENCE,
                "org.thymeleaf.standard.processor.StandardInliningCDATASectionProcessor",
                callback,
            )
            .map_err(|error| {
                TemplateProcessingException::with_cause(
                    Some("Could not create inlining CDATA processor".to_owned()),
                    error,
                )
            })?,
        })
    }
}

impl IProcessor for StandardInliningCDATASectionProcessor {
    fn as_cdata_section_processor(&self) -> Option<&dyn ICDATASectionProcessor> {
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

impl ICDATASectionProcessor for StandardInliningCDATASectionProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        cdata_section: &dyn ICDATASection,
        structure_handler: &mut dyn ICDATASectionStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor
            .process(context, cdata_section, structure_handler)
    }
}
