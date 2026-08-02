use std::sync::{Arc, RwLock, RwLockReadGuard};

use super::IElementProcessor;

/// Java `Set<IElementProcessor>` 的有序身份集合适配。
///
/// 该支撑类型保留可空元素、迭代顺序及同一 Processor 引用只出现一次的语义。
#[derive(Default)]
/// 对应 Java 语义：Rust 侧内部类型（Java 无直接对应对象）。
pub struct ElementProcessorSet {
    entries: Vec<Option<Arc<dyn IElementProcessor>>>,
}

impl ElementProcessorSet {
    /// 创建空集合。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 按 Java 默认引用身份插入可空 Processor。
    ///
    /// # 返回
    ///
    /// 元素此前不存在时返回 `true`。
    /// 对应 Java 语义：Java 接口/超类方法 `insert()` 的 Rust 移植（`None` 继承路径）。
    pub fn insert(&mut self, processor: Option<Arc<dyn IElementProcessor>>) -> bool {
        let duplicate = self
            .entries
            .iter()
            .any(|existing| match (&processor, existing) {
                (None, None) => true,
                (Some(left), Some(right)) => Arc::ptr_eq(left, right),
                _ => false,
            });
        if duplicate {
            return false;
        }
        self.entries.push(processor);
        true
    }

    /// 返回包含可空元素在内的集合大小。
    #[must_use]
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 判断集合是否为空。
    #[must_use]
    /// 对应 Java 语义：Java 接口/超类方法 `isEmpty()` 的 Rust 移植（`None` 继承路径）。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 按集合实际迭代顺序访问元素。
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn iter(&self) -> impl ExactSizeIterator<Item = Option<&Arc<dyn IElementProcessor>>> {
        self.entries.iter().map(Option::as_ref)
    }
}

/// `Collections.unmodifiableSet` 对原 Processor Set 的实时只读视图。
/// 对应 Java 语义：Rust 侧内部类型（Java 无直接对应对象）。
pub struct UnmodifiableElementProcessorSet {
    source: Arc<RwLock<ElementProcessorSet>>,
}

impl UnmodifiableElementProcessorSet {
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub(crate) fn new(source: Arc<RwLock<ElementProcessorSet>>) -> Self {
        Self { source }
    }

    /// 返回当前原集合大小。
    #[must_use]
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn len(&self) -> usize {
        read_recovering_poison(&self.source).len()
    }

    /// 判断当前原集合是否为空。
    #[must_use]
    /// 对应 Java 语义：Java 接口/超类方法 `isEmpty()` 的 Rust 移植（`None` 继承路径）。
    pub fn is_empty(&self) -> bool {
        read_recovering_poison(&self.source).is_empty()
    }

    /// 返回当前原集合迭代顺序的独立身份快照。
    #[must_use]
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn snapshot(&self) -> Vec<Option<Arc<dyn IElementProcessor>>> {
        read_recovering_poison(&self.source)
            .iter()
            .map(|value| value.cloned())
            .collect()
    }
}
/// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。

pub(crate) fn read_set(
    source: &RwLock<ElementProcessorSet>,
) -> RwLockReadGuard<'_, ElementProcessorSet> {
    read_recovering_poison(source)
}

fn read_recovering_poison<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
