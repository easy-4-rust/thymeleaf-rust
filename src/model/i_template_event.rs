use std::io;

use crate::util::{JavaString, JavaWriter};

use super::IModelVisitor;

/// parser 产生并由模板 handler 处理的不可变模板事件合同。
///
/// 对应 Java: `org.thymeleaf.model.ITemplateEvent`。
pub trait ITemplateEvent {
    /// 判断事件是否携带原模板的位置。
    fn has_location(&self) -> bool;

    /// 返回可空原模板名称。
    fn get_template_name(&self) -> Option<&JavaString>;

    /// 返回事件在模板中的一基行号；无位置事件保留实现的占位整数。
    fn get_line(&self) -> i32;

    /// 返回事件在模板中的一基列号；无位置事件保留实现的占位整数。
    fn get_col(&self) -> i32;

    /// 按 Visitor 模式把事件分派给对应重载。
    fn accept(&self, visitor: &mut dyn IModelVisitor);

    /// 将事件完整写入输出。
    ///
    /// # 错误
    ///
    /// 底层输出失败时返回 Java `IOException` 对应错误。
    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()>;
}
