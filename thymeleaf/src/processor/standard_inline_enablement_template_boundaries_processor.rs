use std::sync::Arc;

use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::inline::{
    IInliner, StandardCSSInliner, StandardHTMLInliner, StandardJavaScriptInliner,
    StandardTextInliner, StandardXMLInliner,
};
use crate::model::{ITemplateEnd, ITemplateStart};
use crate::processor::IProcessor;
use crate::templateboundaries::{
    AbstractTemplateBoundariesProcessor, ITemplateBoundariesProcessor,
    ITemplateBoundariesStructureHandler,
};

type StartCallback = Box<
    dyn Fn(
            &dyn ITemplateContext,
            &dyn ITemplateStart,
            &mut dyn ITemplateBoundariesStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
>;
type EndCallback = Box<
    dyn Fn(
            &dyn ITemplateContext,
            &dyn ITemplateEnd,
            &mut dyn ITemplateBoundariesStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
>;

/// 在模板开始时按模板模式安装 Standard Inliner 的边界 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardInlineEnablementTemplateBoundariesProcessor`。
pub struct StandardInlineEnablementTemplateBoundariesProcessor {
    processor: AbstractTemplateBoundariesProcessor<StartCallback, EndCallback>,
}

impl StandardInlineEnablementTemplateBoundariesProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 10;

    /// 创建指定模板模式的边界 Processor。
    pub fn new(template_mode: TemplateMode) -> Result<Self, TemplateProcessingException> {
        let start: StartCallback = Box::new(move |context, _start, structure_handler| {
            let inliner: Option<Arc<dyn IInliner>> = match template_mode {
                TemplateMode::HTML => Some(Arc::new(StandardHTMLInliner::new(
                    context.get_configuration(),
                ))),
                TemplateMode::XML => Some(Arc::new(StandardXMLInliner::new(
                    context.get_configuration(),
                ))),
                TemplateMode::TEXT => Some(Arc::new(StandardTextInliner::new(
                    context.get_configuration(),
                ))),
                TemplateMode::JAVASCRIPT => Some(Arc::new(
                    StandardJavaScriptInliner::new(context.get_configuration())
                        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?,
                )),
                TemplateMode::CSS => Some(Arc::new(
                    StandardCSSInliner::new(context.get_configuration())
                        .map_err(|error| Box::new(error) as Box<dyn TemplateEngineException>)?,
                )),
                TemplateMode::RAW => None,
            };
            structure_handler.set_inliner(inliner);
            Ok(())
        });
        let end: EndCallback = Box::new(|_context, _end, _structure_handler| Ok(()));
        Ok(Self {
            processor: AbstractTemplateBoundariesProcessor::new(
                Some(template_mode),
                Self::PRECEDENCE,
                "org.thymeleaf.standard.processor.StandardInlineEnablementTemplateBoundariesProcessor",
                start,
                end,
            )
            .map_err(|error| {
                TemplateProcessingException::with_cause(
                    Some("Could not create inline enablement processor".to_owned()),
                    error,
                )
            })?,
        })
    }
}

impl IProcessor for StandardInlineEnablementTemplateBoundariesProcessor {
    fn as_template_boundaries_processor(&self) -> Option<&dyn ITemplateBoundariesProcessor> {
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

impl ITemplateBoundariesProcessor for StandardInlineEnablementTemplateBoundariesProcessor {
    fn process_template_start(
        &self,
        context: &dyn ITemplateContext,
        template_start: &dyn ITemplateStart,
        structure_handler: &mut dyn ITemplateBoundariesStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor
            .process_template_start(context, template_start, structure_handler)
    }

    fn process_template_end(
        &self,
        context: &dyn ITemplateContext,
        template_end: &dyn ITemplateEnd,
        structure_handler: &mut dyn ITemplateBoundariesStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor
            .process_template_end(context, template_end, structure_handler)
    }
}
