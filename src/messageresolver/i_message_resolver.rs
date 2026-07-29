use std::any::TypeId;
use std::sync::Arc;

use crate::context::ITemplateContext;
use crate::expression::TemplateValue;
use crate::util::JavaString;

/// 外部化消息解析器合同。
///
/// 对应 Java: `org.thymeleaf.messageresolver.IMessageResolver`。
pub trait IMessageResolver: Send + Sync {
    /// 返回可空解析器名称。
    fn get_name(&self) -> Option<&JavaString>;
    /// 返回可空链式顺序。
    fn get_order(&self) -> Option<i32>;
    /// 解析消息；未命中时返回 `None`。
    fn resolve_message(
        &self,
        context: &dyn ITemplateContext,
        origin: TypeId,
        key: &JavaString,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> Option<JavaString>;
    /// 创建未命中消息的表示；不提供表示时返回 `None`。
    fn create_absent_message_representation(
        &self,
        context: &dyn ITemplateContext,
        origin: TypeId,
        key: &JavaString,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> Option<JavaString>;
}
