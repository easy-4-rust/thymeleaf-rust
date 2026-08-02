use std::sync::Arc;

use super::IProcessor;

/// Java `Set<IProcessor>` 的 Rust 等价集合。
///
/// 这是 `org.thymeleaf.dialect.IProcessorDialect#getProcessors(String)` 返回边界所需的
/// Rust 扩展类型，不计入 Java 对象迁移分子。集合保留以下 Java 可观察合同：
///
/// - 每个逻辑元素最多出现一次；
/// - `None` 对应集合中的 Java `null`，同样最多出现一次；
/// - 迭代顺序保留具体 Java `Set` 的实际迭代顺序；
/// - 非空元素使用 [`Arc`] 保留同一 Processor 对象身份。
///
/// 固定上游 Processor 均继承 `Object` 的身份相等语义，所以 [`Self::insert`] 使用
/// [`Arc::ptr_eq`] 去重。第三方 Processor 若在 Java 中覆盖 `equals`，可使用
/// [`Self::insert_with`] 显式提供同一等价关系。
#[derive(Default)]
/// 对应 Java 语义：Rust 侧内部类型（Java 无直接对应对象）。
pub struct ProcessorSet {
    entries: Vec<Option<Arc<dyn IProcessor>>>,
}

impl ProcessorSet {
    /// 创建空 Processor 集合。
    ///
    /// # 返回
    ///
    /// 不包含任何元素的集合。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 按 Java `Object` 默认身份相等语义插入 Processor。
    ///
    /// # 参数
    ///
    /// - `processor`：待插入的共享 Processor；`None` 对应 Java `null`。
    ///
    /// # 返回
    ///
    /// 元素此前不存在并完成插入时返回 `true`；相同 `Arc` 身份或第二个 `None`
    /// 已存在时返回 `false`。
    pub fn insert(&mut self, processor: Option<Arc<dyn IProcessor>>) -> bool {
        self.insert_with(processor, |candidate, existing| {
            std::ptr::eq(candidate, existing)
        })
    }

    /// 按显式 Java `equals` 等价关系插入 Processor。
    ///
    /// 该入口用于第三方 Processor 在 Java 中覆盖 `Object#equals` 的场景。谓词参数
    /// 顺序与 `HashSet#add` 的关键比较方向一致：新候选在前、已存在元素在后。
    /// `None` 唯一性由集合自身处理，不会调用谓词。
    ///
    /// # 参数
    ///
    /// - `processor`：待插入的共享 Processor；`None` 对应 Java `null`；
    /// - `equivalent`：判断新候选与已有非空元素是否逻辑相等的谓词。
    ///
    /// # 返回
    ///
    /// 没有等价元素并完成插入时返回 `true`，否则返回 `false`。
    pub fn insert_with<F>(
        &mut self,
        processor: Option<Arc<dyn IProcessor>>,
        mut equivalent: F,
    ) -> bool
    where
        F: FnMut(&dyn IProcessor, &dyn IProcessor) -> bool,
    {
        let duplicate = self
            .entries
            .iter()
            .any(|existing| match (&processor, existing) {
                (None, None) => true,
                (Some(candidate), Some(existing)) => {
                    equivalent(candidate.as_ref(), existing.as_ref())
                }
                _ => false,
            });

        if duplicate {
            return false;
        }

        self.entries.push(processor);
        true
    }

    /// 返回集合中的唯一元素数。
    ///
    /// # 返回
    ///
    /// 包含可选 `null` 元素在内的元素数量。
    #[must_use]
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 判断集合是否为空。
    ///
    /// # 返回
    ///
    /// 没有任何元素时返回 `true`。
    #[must_use]
    /// 对应 Java 语义：Java 接口/超类方法 `isEmpty()` 的 Rust 移植（`None` 继承路径）。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 按底层集合的实际迭代顺序访问 Processor。
    ///
    /// # 返回
    ///
    /// 对集合中共享 Processor 身份的只读迭代器；`None` 对应 Java `null` 元素。
    pub fn iter(&self) -> impl ExactSizeIterator<Item = Option<&Arc<dyn IProcessor>>> {
        self.entries.iter().map(Option::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::ProcessorSet;
    use crate::processor::IProcessor;
    use crate::templatemode::TemplateMode;

    struct ProbeProcessor {
        precedence: i32,
    }

    impl IProcessor for ProbeProcessor {
        fn get_template_mode(&self) -> Option<TemplateMode> {
            Some(TemplateMode::HTML)
        }

        fn get_precedence(&self) -> i32 {
            self.precedence
        }
    }

    #[test]
    fn preserves_null_identity_uniqueness_iteration_order_and_custom_equality() {
        let mut processors = ProcessorSet::new();
        assert!(processors.is_empty());
        assert_eq!(processors.len(), 0);

        let first: Arc<dyn IProcessor> = Arc::new(ProbeProcessor { precedence: 10 });
        let same_value: Arc<dyn IProcessor> = Arc::new(ProbeProcessor { precedence: 10 });
        let distinct: Arc<dyn IProcessor> = Arc::new(ProbeProcessor { precedence: 20 });

        assert!(processors.insert(None));
        assert!(!processors.insert(None));
        assert!(processors.insert(Some(Arc::clone(&first))));
        assert!(!processors.insert(Some(Arc::clone(&first))));
        assert!(processors.insert(Some(Arc::clone(&same_value))));
        assert!(processors.insert(Some(Arc::clone(&distinct))));

        assert!(!processors.insert_with(
            Some(Arc::new(ProbeProcessor { precedence: 10 })),
            |candidate, existing| candidate.get_precedence() == existing.get_precedence(),
        ));
        assert!(processors.insert_with(
            Some(Arc::new(ProbeProcessor { precedence: 30 })),
            |candidate, existing| candidate.get_precedence() == existing.get_precedence(),
        ));

        let observed = processors
            .iter()
            .map(|processor| {
                processor.map(|value| (value.get_template_mode(), value.get_precedence()))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            observed,
            [
                None,
                Some((Some(TemplateMode::HTML), 10)),
                Some((Some(TemplateMode::HTML), 10)),
                Some((Some(TemplateMode::HTML), 20)),
                Some((Some(TemplateMode::HTML), 30)),
            ]
        );
        assert_eq!(processors.iter().len(), 5);
        assert!(!processors.is_empty());
        assert_eq!(processors.len(), 5);
    }
}
