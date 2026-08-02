use std::sync::OnceLock;

use crate::context::ITemplateContext;
use crate::exceptions::TemplateProcessingException;
use crate::expression::StandardExpressionResult;
use crate::model::{ICDATASection, IComment, IText};
use crate::serializer::StandardSerializers;
use crate::util::{JavaCharSequence, JavaString};
use crate::{IEngineConfiguration, TemplateMode};

use super::{AbstractStandardInliner, IInliner, StandardInlinerEscaping};

/// CSS 模式 Standard 内联器。
///
/// 对应 Java: `org.thymeleaf.standard.inline.StandardCSSInliner`。
pub struct StandardCSSInliner(AbstractStandardInliner);

impl StandardCSSInliner {
    /// 创建并绑定配置中 CSS Serializer 的内联器。
    /// 对应 Java 语义：`StandardCSSInliner` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(
        configuration: &dyn IEngineConfiguration,
    ) -> Result<Self, TemplateProcessingException> {
        Ok(Self(AbstractStandardInliner::new(
            configuration,
            TemplateMode::CSS,
            StandardInlinerEscaping::Css(StandardSerializers::get_css_serializer(configuration)?),
        )))
    }
}

impl IInliner for StandardCSSInliner {
    fn get_name(&self) -> &JavaString {
        static NAME: OnceLock<JavaString> = OnceLock::new();
        NAME.get_or_init(|| JavaString::from_rust_str("StandardCSSInliner"))
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
