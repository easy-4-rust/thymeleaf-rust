use std::sync::Arc;

use indexmap::IndexMap;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::expression::TemplateValue;
use crate::util::JavaString;

use super::ILinkBuilder;

/// 保存链接构建器名称和执行顺序的抽象基类等价对象。
///
/// 对应 Java: `org.thymeleaf.linkbuilder.AbstractLinkBuilder`。
///
/// Java 类型只持有公共名称和顺序，具体子类负责 `buildLink`。Rust 没有类继承，
/// 因此本对象额外接收一个闭包承接子类逻辑，同时完整保留基类状态合同。
///
/// 自 Thymeleaf 3.0.0 起提供。
pub struct AbstractLinkBuilder<F> {
    name: Option<JavaString>,
    order: Option<i32>,
    build_link: F,
}

impl<F> AbstractLinkBuilder<F> {
    /// 创建默认名称为具体 Java 类名、顺序为 null 的构建器。
    ///
    /// 对应 Java: `AbstractLinkBuilder#AbstractLinkBuilder()`。
    ///
    /// # 参数
    ///
    /// - `java_class_name`：具体 Java 子类的全限定名。
    /// - `build_link`：承接具体子类链接构建逻辑的线程安全闭包。
    ///
    /// # 返回值
    ///
    /// 名称已初始化、顺序为空的抽象构建器等价对象。
    pub fn new(java_class_name: &'static str, build_link: F) -> Self {
        Self {
            name: Some(JavaString::from_rust_str(java_class_name)),
            order: None,
            build_link,
        }
    }

    /// 返回可空构建器名称。
    ///
    /// 对应 Java: `AbstractLinkBuilder#getName()`。
    ///
    /// # 返回值
    ///
    /// 当前名称；`None` 对应 Java `null`。
    #[must_use]
    pub const fn get_name(&self) -> Option<&JavaString> {
        self.name.as_ref()
    }

    /// 设置可空构建器名称。
    ///
    /// 对应 Java: `AbstractLinkBuilder#setName(String)`。
    ///
    /// # 参数
    ///
    /// - `name`：新的可空名称。
    pub fn set_name(&mut self, name: Option<JavaString>) {
        self.name = name;
    }

    /// 返回可空链式执行顺序。
    ///
    /// 对应 Java: `AbstractLinkBuilder#getOrder()`。
    ///
    /// # 返回值
    ///
    /// 当前顺序；`None` 对应 Java `null`。
    #[must_use]
    pub const fn get_order(&self) -> Option<i32> {
        self.order
    }

    /// 设置可空链式顺序。
    ///
    /// 对应 Java: `AbstractLinkBuilder#setOrder(Integer)`。
    ///
    /// # 参数
    ///
    /// - `order`：新的可空顺序。
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
        self.get_name()
    }

    fn get_order(&self) -> Option<i32> {
        self.get_order()
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
