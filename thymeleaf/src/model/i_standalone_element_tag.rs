use super::IProcessableElementTag;
use std::sync::Arc;

/// 独立元素标签合同。
///
/// 对应 Java: `org.thymeleaf.model.IStandaloneElementTag`。
pub trait IStandaloneElementTag: IProcessableElementTag {
    /// 若当前对象已经是引擎内建 standalone 标签，则消费共享引用并保持身份。
    fn into_engine_standalone_element_tag(
        self: Arc<Self>,
    ) -> Option<Arc<crate::engine::StandaloneElementTag>> {
        None
    }

    /// 返回引擎内建 standalone 标签，供内部创建 open/close 等价模型。
    ///
    /// 第三方实现返回 `None`，引擎会按公开标签合同重建等价事件。
    fn as_engine_standalone_element_tag(&self) -> Option<&crate::engine::StandaloneElementTag> {
        None
    }

    /// 判断输出时是否使用最小化标签形式。
    fn is_minimized(&self) -> bool;
}
