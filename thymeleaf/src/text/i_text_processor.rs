use crate::context::ITemplateContext;
use crate::exceptions::TemplateEngineException;
use crate::model::IText;
use crate::processor::IProcessor;

use super::ITextStructureHandler;

/// Text 事件 Processor 合同。
///
/// 对应 Java: `org.thymeleaf.processor.text.ITextProcessor`。
pub trait ITextProcessor: IProcessor {
    /// 处理文本并通过结构处理器声明变更。
    ///
    /// 对应 Java: `ITextProcessor#process(ITemplateContext, IText,
    /// ITextStructureHandler)`。事件不可变，所有输出变更必须写入
    /// `structure_handler`；失败返回 Java 模板引擎异常的 Rust 对应值。
    fn process(
        &self,
        context: &dyn ITemplateContext,
        text: &dyn IText,
        structure_handler: &mut dyn ITextStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>;
}
