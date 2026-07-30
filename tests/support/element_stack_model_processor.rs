use thymeleaf::TemplateMode;
use thymeleaf::context::ITemplateContext;
use thymeleaf::element::{
    AbstractAttributeModelProcessor, IElementModelProcessor, IElementModelStructureHandler,
    IElementProcessor, MatchingAttributeName, MatchingElementName,
};
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::model::IModel;
use thymeleaf::processor::IProcessor;
use thymeleaf::util::JavaString;

type ProcessResult = Result<(), Box<dyn TemplateEngineException>>;
type ProcessCallback = fn(
    &dyn ITemplateContext,
    &mut dyn IModel,
    &thymeleaf::engine::AttributeName,
    Option<JavaString>,
    &mut dyn IElementModelStructureHandler,
) -> ProcessResult;

/// 触发模型收集以验证元素栈语义的测试 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.features.elementstack.ElementStackModelProcessor`。
pub struct ElementStackModelProcessor {
    processor: AbstractAttributeModelProcessor<ProcessCallback>,
}

impl ElementStackModelProcessor {
    /// 创建匹配 `stack:model` 并在执行后删除该属性的模型 Processor。
    pub fn new(dialect_prefix: Option<&str>) -> Self {
        Self {
            processor: AbstractAttributeModelProcessor::new(
                Some(TemplateMode::HTML),
                dialect_prefix.map(JavaString::from_rust_str),
                None,
                false,
                Some(JavaString::from_rust_str("model")),
                true,
                100,
                true,
                "org.thymeleaf.templateengine.features.elementstack.ElementStackModelProcessor",
                process_model as ProcessCallback,
            )
            .expect("the fixed test processor configuration is valid"),
        }
    }
}

impl IProcessor for ElementStackModelProcessor {
    fn as_element_processor(&self) -> Option<&dyn IElementProcessor> {
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

impl IElementProcessor for ElementStackModelProcessor {
    fn as_element_model_processor(&self) -> Option<&dyn IElementModelProcessor> {
        Some(self)
    }

    fn get_matching_element_name(&self) -> Option<&MatchingElementName> {
        self.processor.get_matching_element_name()
    }

    fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
        self.processor.get_matching_attribute_name()
    }
}

impl IElementModelProcessor for ElementStackModelProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    ) -> ProcessResult {
        self.processor.process(context, model, structure_handler)
    }
}

fn process_model(
    _context: &dyn ITemplateContext,
    _model: &mut dyn IModel,
    _attribute_name: &thymeleaf::engine::AttributeName,
    _attribute_value: Option<JavaString>,
    _structure_handler: &mut dyn IElementModelStructureHandler,
) -> ProcessResult {
    // Java Processor 有意不修改模型；进入模型收集路径本身就是被验证的行为。
    Ok(())
}
