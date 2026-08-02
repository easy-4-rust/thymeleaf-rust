use std::sync::Arc;

use indexmap::IndexMap;
use thymeleaf::context::IExpressionContext;
use thymeleaf::exceptions::TemplateProcessingException;
use thymeleaf::expression::TemplateValue;
use thymeleaf::linkbuilder::{ILinkBuilder, StandardLinkBuilder};
use thymeleaf::util::JavaString;

/// 上游测试环境使用的链接构建器，保留标准构建器的链接安全限制。
///
/// 对应 Java:
/// `org.thymeleaf.testing.templateengine.context.web.TestLinkBuilder`。
pub struct TestLinkBuilder;

impl ILinkBuilder for TestLinkBuilder {
    fn get_name(&self) -> Option<&JavaString> {
        None
    }

    fn get_order(&self) -> Option<i32> {
        None
    }

    fn build_link(
        &self,
        context: &dyn IExpressionContext,
        base: Option<&JavaString>,
        parameters: Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
    ) -> Result<Option<JavaString>, TemplateProcessingException> {
        let Some(base) = base else {
            return Ok(None);
        };
        let text = base.to_string_lossy();
        let context_relative = text.starts_with('/') && !text.starts_with("//");
        // 测试 exchange 的 application path 固定为 /testing。去掉首斜杠后委托
        // StandardLinkBuilder 完成模板变量、查询参数、fragment 与 URI 转义。
        let delegated_base = if context_relative {
            JavaString::from_rust_str(&text[1..])
        } else {
            base.clone()
        };
        let built = StandardLinkBuilder::new()
            .build_link(context, Some(&delegated_base), parameters)?
            .unwrap_or(delegated_base);
        Ok(Some(if context_relative {
            JavaString::from_rust_str(&format!("/testing/{}", built.to_string_lossy()))
        } else {
            built
        }))
    }
}
