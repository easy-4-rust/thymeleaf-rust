use std::sync::OnceLock;

use crate::context::ITemplateContext;
use crate::expression::StandardExpressionResult;
use crate::model::{ICDATASection, IComment, IText};
use crate::util::{JavaCharSequence, JavaString};
use crate::{IEngineConfiguration, TemplateMode};

use super::{AbstractStandardInliner, IInliner, StandardInlinerEscaping};

/// HTML 模式 Standard 内联器。
///
/// 对应 Java: `org.thymeleaf.standard.inline.StandardHTMLInliner`。
pub struct StandardHTMLInliner(AbstractStandardInliner);

impl StandardHTMLInliner {
    /// 创建 HTML 模式内联器。
    #[must_use]
    pub fn new(configuration: &dyn IEngineConfiguration) -> Self {
        Self(AbstractStandardInliner::new(
            configuration,
            TemplateMode::HTML,
            StandardInlinerEscaping::Html,
        ))
    }
}

impl IInliner for StandardHTMLInliner {
    fn get_name(&self) -> &JavaString {
        static NAME: OnceLock<JavaString> = OnceLock::new();
        NAME.get_or_init(|| JavaString::from_rust_str("StandardHTMLInliner"))
    }

    fn inline_text(
        &self,
        context: &dyn ITemplateContext,
        text: &dyn IText,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        self.0.inline_text(context, text)
    }

    fn inline_cdata_section(
        &self,
        context: &dyn ITemplateContext,
        value: &dyn ICDATASection,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        self.0.inline_cdata_section(context, value)
    }

    fn inline_comment(
        &self,
        context: &dyn ITemplateContext,
        value: &dyn IComment,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        self.0.inline_comment(context, value)
    }
}
