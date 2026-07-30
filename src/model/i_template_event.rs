use std::io;
use std::sync::Arc;

use crate::engine::ITemplateHandler;
use crate::exceptions::TemplateEngineException;
use crate::util::{JavaString, JavaWriter};

use super::{ICloseElementTag, IModelVisitor, IOpenElementTag, IProcessableElementTag, IText};

/// parser 产生并由模板 handler 处理的不可变模板事件合同。
///
/// 对应 Java: `org.thymeleaf.model.ITemplateEvent`。
pub trait ITemplateEvent: Send + Sync {
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

    /// 将事件分派给模板处理器的对应重载。
    ///
    /// Java 将该内部能力放在 `IEngineTemplateEvent`；Rust 为了让自定义公共事件在
    /// 进入 `Model` 后仍可无损分派，将同一动态分派能力显式放在基础事件合同中。
    fn be_handled(
        self: Arc<Self>,
        handler: &mut dyn ITemplateHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>>;

    /// 判断事件是否为只允许解析器写入的模板开始单例。
    fn is_template_start(&self) -> bool {
        false
    }

    /// 判断事件是否为只允许解析器写入的模板结束单例。
    fn is_template_end(&self) -> bool {
        false
    }

    /// 若当前事件为可处理元素标签，则返回保持同一对象身份的标签引用。
    ///
    /// 对应 Java 的 `event instanceof IProcessableElementTag` 与强制类型转换。
    fn into_processable_element_tag(self: Arc<Self>) -> Option<Arc<dyn IProcessableElementTag>> {
        None
    }

    /// 若事件是 Text，则返回同一对象的只读 Text 视图。
    ///
    /// 对应 Java 内部对 `event instanceof IText` 的动态类型检查。
    fn as_text(&self) -> Option<&dyn IText> {
        None
    }

    /// 若事件是 Text，则消费共享引用并返回保持同一身份的 Text trait object。
    fn into_text(self: Arc<Self>) -> Option<Arc<dyn IText>> {
        None
    }

    /// 若事件是开放元素，则返回同一对象的只读标签视图。
    ///
    /// 对应 Java 内部对 `event instanceof IOpenElementTag` 的动态类型检查。
    fn as_open_element_tag(&self) -> Option<&dyn IOpenElementTag> {
        None
    }

    /// 若事件是关闭元素，则返回同一对象的只读标签视图。
    ///
    /// 对应 Java 内部对 `event instanceof ICloseElementTag` 的动态类型检查。
    fn as_close_element_tag(&self) -> Option<&dyn ICloseElementTag> {
        None
    }

    /// 将事件完整写入输出。
    ///
    /// # 错误
    ///
    /// 底层输出失败时返回 Java `IOException` 对应错误。
    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()>;
}
