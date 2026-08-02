use std::sync::OnceLock;

use crate::context::ITemplateContext;
use crate::exceptions::TemplateProcessingException;
use crate::expression::StandardExpressionResult;
use crate::model::{ICDATASection, IComment, IText};
use crate::serializer::StandardSerializers;
use crate::util::{JavaCharSequence, JavaString};
use crate::{IEngineConfiguration, TemplateMode};

use super::{AbstractStandardInliner, IInliner, StandardInlinerEscaping};

/// JAVASCRIPT 模式 Standard 内联器。
///
/// 对应 Java: `org.thymeleaf.standard.inline.StandardJavaScriptInliner`。
pub struct StandardJavaScriptInliner(AbstractStandardInliner);

impl StandardJavaScriptInliner {
    /// 创建并绑定配置中 JavaScript Serializer 的内联器。
    pub fn new(
        configuration: &dyn IEngineConfiguration,
    ) -> Result<Self, TemplateProcessingException> {
        Ok(Self(AbstractStandardInliner::new(
            configuration,
            TemplateMode::JAVASCRIPT,
            StandardInlinerEscaping::JavaScript(StandardSerializers::get_java_script_serializer(
                configuration,
            )?),
        )))
    }
}

impl IInliner for StandardJavaScriptInliner {
    fn get_name(&self) -> &JavaString {
        static NAME: OnceLock<JavaString> = OnceLock::new();
        NAME.get_or_init(|| JavaString::from_rust_str("StandardJavaScriptInliner"))
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
