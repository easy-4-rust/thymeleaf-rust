use crate::context::ITemplateContext;
use crate::model::IProcessableElementTag;

use super::{IElementProcessor, IElementTagStructureHandler};

/// 以 open/standalone 标签为处理单位的 Processor 合同。
///
/// 对应 Java: `org.thymeleaf.processor.element.IElementTagProcessor`。
pub trait IElementTagProcessor: IElementProcessor {
    /// 处理元素标签。
    fn process(
        &self,
        context: &dyn ITemplateContext,
        tag: &dyn IProcessableElementTag,
        structure_handler: &mut dyn IElementTagStructureHandler,
    );
}
