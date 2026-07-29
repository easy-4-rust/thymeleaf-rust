use super::IProcessableElementTag;

/// 独立元素标签合同。
///
/// 对应 Java: `org.thymeleaf.model.IStandaloneElementTag`。
pub trait IStandaloneElementTag: IProcessableElementTag {
    /// 判断输出时是否使用最小化标签形式。
    fn is_minimized(&self) -> bool;
}
