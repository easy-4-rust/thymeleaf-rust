use std::sync::Arc;

use super::ElementDefinitions;

/// 配置阶段需要注入全局元素定义管理器的内部标记合同。
///
/// 对应 Java: `org.thymeleaf.engine.IElementDefinitionsAware`。
///
/// Processor、PreProcessor 和 PostProcessor 均可实现该接口。
pub trait IElementDefinitionsAware {
    /// 注入引擎构建完成的元素定义管理器。
    fn set_element_definitions(&mut self, element_definitions: Arc<ElementDefinitions>);
}
