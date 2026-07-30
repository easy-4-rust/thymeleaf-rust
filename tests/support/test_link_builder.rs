use std::sync::Arc;

use indexmap::IndexMap;
use thymeleaf::context::IExpressionContext;
use thymeleaf::exceptions::TemplateProcessingException;
use thymeleaf::expression::TemplateValue;
use thymeleaf::linkbuilder::ILinkBuilder;
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
        _context: &dyn IExpressionContext,
        base: Option<&JavaString>,
        _parameters: Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
    ) -> Result<Option<JavaString>, TemplateProcessingException> {
        let Some(base) = base else {
            return Ok(None);
        };
        let text = base.to_string_lossy();
        if text
            .get(..11)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("javascript:"))
        {
            return Err(TemplateProcessingException::new(Some(
                "'javascript:' is forbidden in this context. Link expressions cannot contain \
                 inlined JavaScript code."
                    .to_owned(),
            )));
        }
        Ok(Some(if text.starts_with('/') {
            JavaString::from_rust_str(&format!("/testing{text}"))
        } else {
            base.clone()
        }))
    }
}
