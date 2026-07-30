use super::IProcessableElementTag;
use std::sync::Arc;

/// 打开元素标签的标记合同。
///
/// 对应 Java: `org.thymeleaf.model.IOpenElementTag`。
pub trait IOpenElementTag: IProcessableElementTag {
    /// 若当前对象已经是引擎内建开放标签，则消费共享引用并保持身份。
    fn into_engine_open_element_tag(self: Arc<Self>) -> Option<Arc<crate::engine::OpenElementTag>> {
        None
    }
}
