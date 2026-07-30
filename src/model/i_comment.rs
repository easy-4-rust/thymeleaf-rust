use crate::util::{JavaCharSequence, JavaString, TextUtilsError};

use super::ITemplateEvent;

/// 包含 `<!--` 与 `-->` 边界的不可变注释事件。
///
/// 对应 Java: `org.thymeleaf.model.IComment`。
pub trait IComment: ITemplateEvent + JavaCharSequence {
    /// 返回引擎内建 Comment，供 Processor 保留 parser 原始前后缀。
    fn as_engine_comment(&self) -> Option<&crate::engine::Comment> {
        None
    }

    /// 返回包含前后缀的完整注释。
    fn get_comment(&self) -> Result<Option<JavaString>, TextUtilsError>;

    /// 返回不含前后缀的注释内容。
    fn get_content(&self) -> Result<Option<JavaString>, TextUtilsError>;
}
