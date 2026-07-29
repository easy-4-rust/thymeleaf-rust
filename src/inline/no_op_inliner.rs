use std::sync::OnceLock;

use crate::context::ITemplateContext;
use crate::model::{ICDATASection, IComment, IText};
use crate::util::{JavaString, TextUtilsError};

use super::IInliner;

/// 不执行任何内联处理的共享单例。
///
/// 三种 inline 重载始终返回 Java null；正常引擎路径不会主动调用这些方法。
///
/// 对应 Java: `org.thymeleaf.inline.NoOpInliner`。
pub struct NoOpInliner;

impl NoOpInliner {
    /// Java `NoOpInliner.INSTANCE` 对应的零大小共享值。
    pub const INSTANCE: Self = Self;
}

impl IInliner for NoOpInliner {
    fn get_name(&self) -> &JavaString {
        static NAME: OnceLock<JavaString> = OnceLock::new();
        NAME.get_or_init(|| JavaString::from_rust_str("NOOP"))
    }

    fn inline_text(
        &self,
        _context: &dyn ITemplateContext,
        _text: &dyn IText,
    ) -> Result<Option<JavaString>, TextUtilsError> {
        Ok(None)
    }

    fn inline_cdata_section(
        &self,
        _context: &dyn ITemplateContext,
        _cdata_section: &dyn ICDATASection,
    ) -> Result<Option<JavaString>, TextUtilsError> {
        Ok(None)
    }

    fn inline_comment(
        &self,
        _context: &dyn ITemplateContext,
        _comment: &dyn IComment,
    ) -> Result<Option<JavaString>, TextUtilsError> {
        Ok(None)
    }
}
