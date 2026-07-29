use super::IElementTag;

/// 关闭元素标签合同。
///
/// 对应 Java: `org.thymeleaf.model.ICloseElementTag`。
pub trait ICloseElementTag: IElementTag {
    /// 判断该关闭标签此前是否不存在对应打开标签。
    fn is_unmatched(&self) -> bool;
}
