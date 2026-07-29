use std::sync::Arc;

use super::AttributeDefinitions;

/// 配置阶段需要注入全局属性定义管理器的内部标记合同。
///
/// 对应 Java: `org.thymeleaf.engine.IAttributeDefinitionsAware`。
///
/// Processor、PreProcessor 和 PostProcessor 均可实现该接口。
pub trait IAttributeDefinitionsAware {
    /// 注入引擎构建完成的属性定义管理器。
    fn set_attribute_definitions(&mut self, attribute_definitions: Arc<AttributeDefinitions>);
}
