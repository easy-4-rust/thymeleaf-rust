use std::sync::Arc;

use indexmap::IndexMap;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::expression::TemplateValue;
use crate::util::JavaString;

/// 模板 URL 构建器合同。
///
/// 对应 Java: `org.thymeleaf.linkbuilder.ILinkBuilder`。
pub trait ILinkBuilder: Send + Sync {
    /// 返回日志和配置使用的可空名称。
    fn get_name(&self) -> Option<&JavaString>;
    /// 返回链式执行顺序；`None` 的构建器最后执行。
    fn get_order(&self) -> Option<i32>;
    /// 尝试构建链接；不能处理时返回 `None` 交给下一个构建器。
    fn build_link(
        &self,
        context: &dyn IExpressionContext,
        base: Option<&JavaString>,
        parameters: Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
    ) -> Result<Option<JavaString>, TemplateProcessingException>;
}
