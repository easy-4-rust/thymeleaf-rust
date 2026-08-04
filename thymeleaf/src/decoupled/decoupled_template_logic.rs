use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::util::Utf16String;

use super::DecoupledInjectedAttribute;

/// 保存解耦模板逻辑中按 selector 分组的待注入属性。
///
/// 对象在构建阶段可变且不保证线程安全；完成构建后由一次模板解析独占读取。selector
/// 的键集合保持 HashMap 语义，调试输出则按 Java 版排序，保证稳定可诊断。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.markup.decoupled.DecoupledTemplateLogic`。
pub struct DecoupledTemplateLogic {
    injected_attributes: HashMap<Utf16String, Vec<Arc<DecoupledInjectedAttribute>>>,
}

impl DecoupledTemplateLogic {
    /// 创建空的解耦逻辑容器。
    ///
    /// 对应 Java: `DecoupledTemplateLogic#DecoupledTemplateLogic()`。
    #[must_use]
    pub fn new() -> Self {
        Self {
            injected_attributes: HashMap::with_capacity(20),
        }
    }

    /// 判断是否存在至少一组注入属性。
    ///
    /// 对应 Java: `DecoupledTemplateLogic#hasInjectedAttributes()`。
    #[must_use]
    pub fn has_injected_attributes(&self) -> bool {
        !self.injected_attributes.is_empty()
    }

    /// 返回全部 selector 的只读快照。
    ///
    /// 对应 Java: `DecoupledTemplateLogic#getAllInjectedAttributeSelectors()`。
    #[must_use]
    pub fn get_all_injected_attribute_selectors(&self) -> Vec<&Utf16String> {
        self.injected_attributes.keys().collect()
    }

    /// 返回指定 selector 的注入属性；不存在时返回 `None`。
    ///
    /// 对应 Java:
    /// `DecoupledTemplateLogic#getInjectedAttributesForSelector(String)`。
    #[must_use]
    pub fn get_injected_attributes_for_selector(
        &self,
        selector: &Utf16String,
    ) -> Option<&[Arc<DecoupledInjectedAttribute>]> {
        self.injected_attributes.get(selector).map(Vec::as_slice)
    }

    /// 在指定 selector 尾部追加一个注入属性。
    ///
    /// 参数均为非空 Rust 引用，等价于 Java 通过 `Validate.notNull` 后执行；同一
    /// selector 的属性严格保持加入顺序。
    ///
    /// 对应 Java: `DecoupledTemplateLogic#addInjectedAttribute`。
    pub fn add_injected_attribute(
        &mut self,
        selector: Utf16String,
        injected_attribute: Arc<DecoupledInjectedAttribute>,
    ) {
        self.injected_attributes
            .entry(selector)
            .or_insert_with(|| Vec::with_capacity(2))
            .push(injected_attribute);
    }
}

impl Default for DecoupledTemplateLogic {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for DecoupledTemplateLogic {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let mut keys: Vec<&Utf16String> = self.injected_attributes.keys().collect();
        keys.sort_by(|left, right| left.as_utf16().cmp(right.as_utf16()));
        formatter.write_str("{")?;
        for (index, key) in keys.into_iter().enumerate() {
            if index > 0 {
                formatter.write_str(", ")?;
            }
            write!(formatter, "{}=[", key.to_string_lossy())?;
            if let Some(attributes) = self.injected_attributes.get(key) {
                for (attribute_index, attribute) in attributes.iter().enumerate() {
                    if attribute_index > 0 {
                        formatter.write_str(", ")?;
                    }
                    formatter.write_str(&attribute.to_utf16_string().to_string_lossy())?;
                }
            }
            formatter.write_str("]")?;
        }
        formatter.write_str("}")
    }
}
