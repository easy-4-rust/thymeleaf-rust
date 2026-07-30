use std::sync::Arc;

use indexmap::IndexMap;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::expression::TemplateValue;
use crate::util::JavaString;

use super::ILinkBuilder;

/// 保存名称和执行顺序，并由闭包实现具体链接构建逻辑的抽象 LinkBuilder。
///
/// 对应 Java: `org.thymeleaf.linkbuilder.AbstractLinkBuilder`。
pub struct AbstractLinkBuilder<F> {
    name: Option<JavaString>,
    order: Option<i32>,
    build_link: F,
}

impl<F> AbstractLinkBuilder<F> {
    /// 创建默认名称为具体 Java 类名、顺序为 null 的构建器。
    pub fn new(java_class_name: &'static str, build_link: F) -> Self {
        Self {
            name: Some(JavaString::from_rust_str(java_class_name)),
            order: None,
            build_link,
        }
    }

    /// 设置可空构建器名称。
    pub fn set_name(&mut self, name: Option<JavaString>) {
        self.name = name;
    }

    /// 设置可空链式顺序。
    pub fn set_order(&mut self, order: Option<i32>) {
        self.order = order;
    }
}

impl<F> ILinkBuilder for AbstractLinkBuilder<F>
where
    F: Fn(
            &dyn IExpressionContext,
            Option<&JavaString>,
            Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
        ) -> Result<Option<JavaString>, TemplateProcessingException>
        + Send
        + Sync,
{
    fn get_name(&self) -> Option<&JavaString> {
        self.name.as_ref()
    }

    fn get_order(&self) -> Option<i32> {
        self.order
    }

    fn build_link(
        &self,
        context: &dyn IExpressionContext,
        base: Option<&JavaString>,
        parameters: Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
    ) -> Result<Option<JavaString>, TemplateProcessingException> {
        (self.build_link)(context, base, parameters)
    }
}
