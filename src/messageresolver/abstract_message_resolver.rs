use std::any::TypeId;
use std::sync::Arc;

use crate::context::ITemplateContext;
use crate::expression::TemplateValue;
use crate::util::JavaString;

use super::{IMessageResolver, MessageResolutionResult};

/// 保存名称和顺序，并由闭包实现消息解析行为的抽象 MessageResolver。
///
/// 对应 Java: `org.thymeleaf.messageresolver.AbstractMessageResolver`。
pub struct AbstractMessageResolver<FResolve, FAbsent> {
    name: Option<JavaString>,
    order: Option<i32>,
    resolve_message: FResolve,
    absent_message: FAbsent,
}

impl<FResolve, FAbsent> AbstractMessageResolver<FResolve, FAbsent> {
    /// 创建默认名称为具体 Java 类名、顺序为 null 的解析器。
    pub fn new(
        java_class_name: &'static str,
        resolve_message: FResolve,
        absent_message: FAbsent,
    ) -> Self {
        Self {
            name: Some(JavaString::from_rust_str(java_class_name)),
            order: None,
            resolve_message,
            absent_message,
        }
    }

    /// 设置可空解析器名称。
    pub fn set_name(&mut self, name: Option<JavaString>) {
        self.name = name;
    }

    /// 设置可空链式顺序。
    pub fn set_order(&mut self, order: Option<i32>) {
        self.order = order;
    }
}

impl<FResolve, FAbsent> IMessageResolver for AbstractMessageResolver<FResolve, FAbsent>
where
    FResolve: Fn(
            &dyn ITemplateContext,
            Option<TypeId>,
            &JavaString,
            Option<&[Option<Arc<TemplateValue>>]>,
        ) -> MessageResolutionResult<Option<JavaString>>
        + Send
        + Sync,
    FAbsent: Fn(
            &dyn ITemplateContext,
            Option<TypeId>,
            &JavaString,
            Option<&[Option<Arc<TemplateValue>>]>,
        ) -> MessageResolutionResult<Option<JavaString>>
        + Send
        + Sync,
{
    fn get_name(&self) -> Option<&JavaString> {
        self.name.as_ref()
    }

    fn get_order(&self) -> Option<i32> {
        self.order
    }

    fn resolve_message(
        &self,
        context: &dyn ITemplateContext,
        origin: Option<TypeId>,
        key: &JavaString,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>> {
        (self.resolve_message)(context, origin, key, message_parameters)
    }

    fn create_absent_message_representation(
        &self,
        context: &dyn ITemplateContext,
        origin: Option<TypeId>,
        key: &JavaString,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>> {
        (self.absent_message)(context, origin, key, message_parameters)
    }
}
