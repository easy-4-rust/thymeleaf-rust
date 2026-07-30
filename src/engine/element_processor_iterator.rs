#![expect(
    dead_code,
    reason = "直接消费者 ProcessorTemplateHandler 将在同一 Engine 批次后续接入"
)]

use std::cmp::Ordering;
use std::sync::Arc;

use crate::element::IElementProcessor;
use crate::exceptions::TemplateProcessingException;

use super::AbstractProcessableElementTag;

/// 在标签属性动态变化时保持已访问状态的元素 Processor 迭代器。
///
/// 新出现的 Processor 即使优先级高于最后执行项也会被执行；仍存在的 Processor
/// 保留 visited 状态；被删除项从快照消失。还支持处理器要求立即重复自身的流程。
///
/// 对应 Java: `org.thymeleaf.engine.ElementProcessorIterator`。
pub(crate) struct ElementProcessorIterator {
    last: isize,
    processors: Vec<Arc<dyn IElementProcessor>>,
    visited: Vec<bool>,
    current_tag: Option<*const AbstractProcessableElementTag>,
    last_to_be_repeated: bool,
    last_was_repeated: bool,
}

impl ElementProcessorIterator {
    /// 创建尚未绑定标签的迭代器。
    pub(crate) const fn new() -> Self {
        Self {
            last: -1,
            processors: Vec::new(),
            visited: Vec::new(),
            current_tag: None,
            last_to_be_repeated: false,
            last_was_repeated: false,
        }
    }

    /// 清除当前迭代状态并复用已分配空间。
    pub(crate) fn reset(&mut self) {
        self.processors.clear();
        self.visited.clear();
        self.last = -1;
        self.current_tag = None;
        self.last_to_be_repeated = false;
        self.last_was_repeated = false;
    }

    /// 返回下一未访问 Processor，必要时按新标签快照重算。
    pub(crate) fn next(
        &mut self,
        tag: &AbstractProcessableElementTag,
    ) -> Result<Option<Arc<dyn IElementProcessor>>, TemplateProcessingException> {
        let tag_identity = tag as *const AbstractProcessableElementTag;
        if self.last_to_be_repeated {
            if self.current_tag != Some(tag_identity) {
                return Err(TemplateProcessingException::new(Some(
                    "Cannot return last processor to be repeated: changes were made and processor recompute is needed!"
                        .to_owned(),
                )));
            }
            let processor = self
                .processors
                .get(self.last as usize)
                .cloned()
                .ok_or_else(|| {
                    TemplateProcessingException::new(Some(
                        "Cannot return last processor to be repeated: no processors in tag!"
                            .to_owned(),
                    ))
                })?;
            self.last_to_be_repeated = false;
            self.last_was_repeated = true;
            return Ok(Some(processor));
        }
        self.last_was_repeated = false;
        if self.current_tag != Some(tag_identity) {
            self.recompute(tag)?;
            self.current_tag = Some(tag_identity);
            self.last = -1;
        }
        let start = usize::try_from(self.last + 1).unwrap_or(0);
        if let Some(index) = (start..self.processors.len()).find(|index| !self.visited[*index]) {
            self.visited[index] = true;
            self.last = index as isize;
            return Ok(Some(Arc::clone(&self.processors[index])));
        }
        self.last = self.processors.len() as isize;
        Ok(None)
    }

    /// 返回上次结果是否来自显式重复请求。
    pub(crate) const fn last_was_repeated(&self) -> bool {
        self.last_was_repeated
    }

    /// 要求下一次返回当前标签最后一个 Processor。
    pub(crate) fn set_last_to_be_repeated(
        &mut self,
        tag: &AbstractProcessableElementTag,
    ) -> Result<(), TemplateProcessingException> {
        if self.current_tag != Some(tag as *const AbstractProcessableElementTag) {
            return Err(TemplateProcessingException::new(Some(
                "Cannot set last processor to be repeated: processor recompute is needed!"
                    .to_owned(),
            )));
        }
        if self.processors.is_empty() || self.last < 0 {
            return Err(TemplateProcessingException::new(Some(
                "Cannot set last processor to be repeated: no processors in tag!".to_owned(),
            )));
        }
        self.last_to_be_repeated = true;
        Ok(())
    }

    /// 复制原迭代器的处理快照和访问进度。
    pub(crate) fn reset_as_clone_of(&mut self, original: &Self) {
        self.last = original.last;
        self.processors.clone_from(&original.processors);
        self.visited.clone_from(&original.visited);
        self.current_tag = original.current_tag;
        self.last_to_be_repeated = original.last_to_be_repeated;
        self.last_was_repeated = original.last_was_repeated;
    }

    fn recompute(
        &mut self,
        tag: &AbstractProcessableElementTag,
    ) -> Result<(), TemplateProcessingException> {
        let associated_processors = tag.get_associated_processors().map_err(|error| {
            TemplateProcessingException::with_cause(
                Some("Could not recompute associated element processors".to_owned()),
                error,
            )
        })?;
        let old_processors = std::mem::take(&mut self.processors);
        let old_visited = std::mem::take(&mut self.visited);
        self.visited = vec![false; associated_processors.len()];
        for (new_index, new_processor) in associated_processors.iter().enumerate() {
            if let Some(old_index) = old_processors
                .iter()
                .position(|old_processor| Arc::ptr_eq(new_processor, old_processor))
            {
                self.visited[new_index] = old_visited[old_index];
            } else if old_processors.iter().any(|old_processor| {
                compare_processors(new_processor, old_processor) == Ordering::Equal
            }) {
                return Err(TemplateProcessingException::new(Some(format!(
                    "Two different registered processors have returned zero as a result of their comparison, which is forbidden. Offending processors are {}",
                    new_processor.java_class_name()
                ))));
            }
        }
        self.processors = associated_processors;
        Ok(())
    }
}

fn compare_processors(
    left: &Arc<dyn IElementProcessor>,
    right: &Arc<dyn IElementProcessor>,
) -> Ordering {
    if Arc::ptr_eq(left, right) {
        return Ordering::Equal;
    }
    left.get_precedence()
        .cmp(&right.get_precedence())
        .then_with(|| left.java_class_name().cmp(right.java_class_name()))
}
