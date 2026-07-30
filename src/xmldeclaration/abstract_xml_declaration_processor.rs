use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::IXMLDeclaration;
use crate::processor::{AbstractProcessorAdapter, IProcessor};
use crate::util::ValidateError;

use super::{IXMLDeclarationProcessor, IXMLDeclarationStructureHandler};

/// 捕获 `doProcess` 异常并补充 XML declaration 位置的抽象 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.processor.xmldeclaration.AbstractXMLDeclarationProcessor`。
pub struct AbstractXMLDeclarationProcessor<F> {
    adapter: AbstractProcessorAdapter<F>,
}

impl<F> AbstractXMLDeclarationProcessor<F> {
    /// 创建以闭包表达 Java 抽象 `doProcess` 方法的 Processor。
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

impl<F> IProcessor for AbstractXMLDeclarationProcessor<F>
where
    F: Send + Sync,
{
    fn java_class_name(&self) -> &'static str {
        self.adapter.processor_class_name()
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.adapter.template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.adapter.precedence()
    }
}

impl<F> IXMLDeclarationProcessor for AbstractXMLDeclarationProcessor<F>
where
    F: Fn(
            &dyn ITemplateContext,
            &dyn IXMLDeclaration,
            &mut dyn IXMLDeclarationStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
{
    fn process(
        &self,
        context: &dyn ITemplateContext,
        xml_declaration: &dyn IXMLDeclaration,
        structure_handler: &mut dyn IXMLDeclarationStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.adapter.execute(xml_declaration, |callback| {
            callback(context, xml_declaration, structure_handler)
        })
    }
}
