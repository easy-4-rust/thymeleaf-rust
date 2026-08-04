use std::sync::Arc;

use crate::util::Utf16String;

use super::{DecoupledInjectedAttribute, DecoupledTemplateLogic};

/// 根据 node selector 的当前选择结果提供解耦注入属性。
///
/// Java 对象位于 markup handler 链中并在 injection level 0 消费 `ParseSelection`；
/// Rust parser 将相同 level 的 selector 列表直接传入本对象，再由 Engine adapter
/// 依次发送空白与属性事件。selector 顺序和每组属性顺序均保持不变。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.markup.decoupled.DecoupledTemplateLogicMarkupHandler`。
pub struct DecoupledTemplateLogicMarkupHandler {
    decoupled_template_logic: Arc<DecoupledTemplateLogic>,
    inject_attributes: bool,
}

impl DecoupledTemplateLogicMarkupHandler {
    /// 创建解耦逻辑注入 handler。
    ///
    /// 对应 Java:
    /// `DecoupledTemplateLogicMarkupHandler#DecoupledTemplateLogicMarkupHandler`。
    #[must_use]
    pub fn new(decoupled_template_logic: Arc<DecoupledTemplateLogic>) -> Self {
        let inject_attributes = decoupled_template_logic.has_injected_attributes();
        Self {
            decoupled_template_logic,
            inject_attributes,
        }
    }

    /// 按当前 node selection 顺序收集需要注入的全部属性。
    ///
    /// `None`/空选择等价于 Java `ParseSelection` 在 injection level 0 未匹配；
    /// 同一个节点同时匹配多个 selector 时，各组属性按 selector 顺序连续注入。
    ///
    /// 对应 Java:
    /// `DecoupledTemplateLogicMarkupHandler#processInjectedAttributes`。
    #[must_use]
    pub fn process_injected_attributes(
        &self,
        selectors: Option<&[Utf16String]>,
    ) -> Vec<Arc<DecoupledInjectedAttribute>> {
        if !self.inject_attributes {
            return Vec::new();
        }
        let Some(selectors) = selectors.filter(|values| !values.is_empty()) else {
            return Vec::new();
        };
        let mut result = Vec::new();
        for selector in selectors {
            if let Some(attributes) = self
                .decoupled_template_logic
                .get_injected_attributes_for_selector(selector)
            {
                result.extend(attributes.iter().cloned());
            }
        }
        result
    }
}
