use std::sync::Arc;

use thymeleaf::TemplateMode;
use thymeleaf::dialect::{AbstractDialect, IDialect, IPostProcessorDialect, IPreProcessorDialect};
use thymeleaf::engine::ITemplateHandler;
use thymeleaf::postprocessor::{IPostProcessor, PostProcessor};
use thymeleaf::preprocessor::{IPreProcessor, PreProcessor};

use super::{Dialect01PostProcessor, Dialect01PreProcessor};

const TEMPLATE_MODES: [TemplateMode; 5] = [
    TemplateMode::HTML,
    TemplateMode::XML,
    TemplateMode::TEXT,
    TemplateMode::JAVASCRIPT,
    TemplateMode::CSS,
];

/// 为五种可解析模式注册同一对 pre/post template handler 的测试方言。
///
/// 对应 Java: `org.thymeleaf.templateengine.prepostprocessors.dialect.Dialect01`。
pub struct PrePostProcessorsDialect01 {
    dialect: AbstractDialect,
}

impl PrePostProcessorsDialect01 {
    /// 创建方言级 precedence 为 100 的 `Dialect01`。
    pub fn new() -> Self {
        Self {
            dialect: AbstractDialect::new(Some("Dialect01"))
                .expect("the fixed pre/post processor dialect name is valid"),
        }
    }
}

impl Default for PrePostProcessorsDialect01 {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for PrePostProcessorsDialect01 {
    fn as_pre_processor_dialect(&self) -> Option<&dyn IPreProcessorDialect> {
        Some(self)
    }

    fn as_post_processor_dialect(&self) -> Option<&dyn IPostProcessorDialect> {
        Some(self)
    }

    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IPreProcessorDialect for PrePostProcessorsDialect01 {
    fn get_dialect_pre_processor_precedence(&self) -> i32 {
        100
    }

    fn get_pre_processors(&self) -> Vec<Arc<dyn IPreProcessor>> {
        TEMPLATE_MODES
            .into_iter()
            .map(|template_mode| {
                Arc::new(PreProcessor::new(
                    template_mode,
                    new_pre_processor,
                    "org.thymeleaf.templateengine.prepostprocessors.dialect.Dialect01PreProcessor",
                    1000,
                )) as Arc<dyn IPreProcessor>
            })
            .collect()
    }
}

impl IPostProcessorDialect for PrePostProcessorsDialect01 {
    fn get_dialect_post_processor_precedence(&self) -> i32 {
        100
    }

    fn get_post_processors(&self) -> Vec<Arc<dyn IPostProcessor>> {
        TEMPLATE_MODES
            .into_iter()
            .map(|template_mode| {
                Arc::new(PostProcessor::new(
                    template_mode,
                    new_post_processor,
                    "org.thymeleaf.templateengine.prepostprocessors.dialect.Dialect01PostProcessor",
                    1000,
                )) as Arc<dyn IPostProcessor>
            })
            .collect()
    }
}

fn new_pre_processor() -> Box<dyn ITemplateHandler> {
    Box::new(Dialect01PreProcessor::new())
}

fn new_post_processor() -> Box<dyn ITemplateHandler> {
    Box::new(Dialect01PostProcessor::new())
}
