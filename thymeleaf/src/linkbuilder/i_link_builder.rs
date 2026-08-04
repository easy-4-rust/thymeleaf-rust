use std::error::Error;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::expression::TemplateValue;
use crate::util::Utf16String;

/// 链接构建器动态边界的错误结果。
///
/// 同时保留 Java 参数校验异常和模板处理异常的具体错误类型。
pub type LinkBuilderResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// 模板 URL 构建器合同，是不同 Web 执行环境接入 Thymeleaf 的扩展点。
///
/// 对应 Java: `org.thymeleaf.linkbuilder.ILinkBuilder`。
///
/// 同一模板引擎可以配置多个构建器。引擎按 [`ILinkBuilder::get_order`] 排序并依次
/// 调用，首个返回非空链接的构建器胜出；`None` 表示当前构建器不负责该链接，整条链
/// 都返回 `None` 时由引擎抛出处理异常。实现必须能够被多个渲染线程安全共享。
///
/// 自 Thymeleaf 3.0.0 起提供。
pub trait ILinkBuilder: Send + Sync {
    /// 返回日志和配置使用的可空名称。
    ///
    /// 对应 Java: `ILinkBuilder#getName()`。
    ///
    /// # 返回值
    ///
    /// 当前构建器名称；`None` 对应 Java `null`。
    fn get_name(&self) -> Option<&Utf16String>;

    /// 返回构建器在链中的可空执行顺序。
    ///
    /// 对应 Java: `ILinkBuilder#getOrder()`。
    ///
    /// # 返回值
    ///
    /// 数值越小越先执行；`None` 的构建器排在显式顺序之后。
    fn get_order(&self) -> Option<i32>;

    /// 尝试构建链接；不能处理时返回 `None` 交给下一个构建器。
    ///
    /// 对应 Java: `ILinkBuilder#buildLink(IExpressionContext,String,Map)`。
    ///
    /// # 参数
    ///
    /// - `context`：当前表达式上下文，不能为空。
    /// - `base`：链接基础路径，允许为空。
    /// - `parameters`：可选 URL 参数，键和值均保留 Java 可空边界。
    ///
    /// # 返回值
    ///
    /// 构建完成的链接；无法处理或 `base` 为空时返回 `None`。
    ///
    /// # 错误
    ///
    /// URL 不允许、上下文能力不足或宿主扩展处理失败时返回模板处理异常。
    fn build_link(
        &self,
        context: &dyn IExpressionContext,
        base: Option<&Utf16String>,
        parameters: Option<&IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>>,
    ) -> Result<Option<Utf16String>, TemplateProcessingException>;

    /// 保留 Java 公共入口的可空上下文校验边界。
    ///
    /// # 参数
    ///
    /// 参数语义与 [`ILinkBuilder::build_link`] 相同，`None` 上下文对应 Java `null`。
    ///
    /// # 返回值
    ///
    /// 构建结果或未处理的 `None`。
    ///
    /// # 错误
    ///
    /// 上下文为空时返回 Java `IllegalArgumentException` 等价的
    /// [`crate::util::ValidateError`]；其余错误保留原具体类型。
    fn build_link_nullable(
        &self,
        context: Option<&dyn IExpressionContext>,
        base: Option<&Utf16String>,
        parameters: Option<&IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>>,
    ) -> LinkBuilderResult<Option<Utf16String>> {
        crate::util::Validate::not_null(context, Some("Expression context cannot be null"))
            .map_err(|error| Box::new(error) as Box<dyn Error + Send + Sync>)?;
        self.build_link(
            context.expect("context was validated as non-null"),
            base,
            parameters,
        )
        .map_err(|error| Box::new(error) as Box<dyn Error + Send + Sync>)
    }
}
