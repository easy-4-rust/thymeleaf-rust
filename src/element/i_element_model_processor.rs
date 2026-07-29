use crate::context::ITemplateContext;
use crate::model::IModel;

use super::{IElementModelStructureHandler, IElementProcessor};

/// 以完整元素模型为处理单位的 Processor 合同。
///
/// 对应 Java: `org.thymeleaf.processor.element.IElementModelProcessor`。
pub trait IElementModelProcessor: IElementProcessor {
    /// 处理元素模型。
    fn process(
        &self,
        context: &dyn ITemplateContext,
        model: &mut dyn IModel,
        structure_handler: &mut dyn IElementModelStructureHandler,
    );
}
