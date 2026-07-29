use crate::context::ITemplateContext;
use crate::model::{ICDATASection, IComment, IText};
use crate::util::JavaString;

/// 文本、CDATA 和注释节点的内联处理合同。
///
/// 对应 Java: `org.thymeleaf.inline.IInliner`。
pub trait IInliner {
    /// 返回可识别的内联器名称。
    fn get_name(&self) -> &JavaString;
    /// 处理 Text 节点并返回延迟或立即求值的字符序列。
    fn inline_text(
        &self,
        context: &dyn ITemplateContext,
        text: &dyn IText,
    ) -> Result<Option<JavaString>, crate::util::TextUtilsError>;
    /// 处理 CDATA 节点。
    fn inline_cdata_section(
        &self,
        context: &dyn ITemplateContext,
        cdata_section: &dyn ICDATASection,
    ) -> Result<Option<JavaString>, crate::util::TextUtilsError>;
    /// 处理 Comment 节点。
    fn inline_comment(
        &self,
        context: &dyn ITemplateContext,
        comment: &dyn IComment,
    ) -> Result<Option<JavaString>, crate::util::TextUtilsError>;
}
