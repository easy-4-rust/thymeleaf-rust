use crate::util::{CharSequenceValue, TextUtilsError, Utf16String};

use super::ITemplateEvent;

/// 模板中不具有结构含义的不可变文本事件。
///
/// 对应 Java: `org.thymeleaf.model.IText`。
pub trait IText: ITemplateEvent + CharSequenceValue {
    /// 返回完整文本；`None` 保留自定义实现返回 null 的接口边界。
    fn get_text(&self) -> Result<Option<Utf16String>, TextUtilsError>;
}
