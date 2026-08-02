use crate::util::{JavaCharSequence, JavaString, TextUtilsError};

use super::ITemplateEvent;

/// 包含 `<![CDATA[` 与 `]]>` 边界的不可变 CDATA 事件。
///
/// 对应 Java: `org.thymeleaf.model.ICDATASection`。
pub trait ICDATASection: ITemplateEvent + JavaCharSequence {
    /// 返回引擎内建 CDATA，供 Processor 保留 parser 原始前后缀。
    fn as_engine_cdata_section(&self) -> Option<&crate::engine::CDATASection> {
        None
    }

    /// 返回包含前后缀的完整 CDATA section。
    fn get_cdata_section(&self) -> Result<Option<JavaString>, TextUtilsError>;

    /// 返回不含前后缀的 CDATA 内容。
    fn get_content(&self) -> Result<Option<JavaString>, TextUtilsError>;
}
