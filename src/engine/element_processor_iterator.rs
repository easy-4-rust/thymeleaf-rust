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
    current_tag_identity: Option<u64>,
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
            current_tag_identity: None,
            last_to_be_repeated: false,
            last_was_repeated: false,
        }
    }

    /// 清除当前迭代状态并复用已分配空间。
    pub(crate) fn reset(&mut self) {
        self.processors.clear();
        self.visited.clear();
        self.last = -1;
        self.current_tag_identity = None;
        self.last_to_be_repeated = false;
        self.last_was_repeated = false;
    }

    /// 返回下一未访问 Processor，必要时按新标签快照重算。
    pub(crate) fn next(
        &mut self,
        tag: &AbstractProcessableElementTag,
    ) -> Result<Option<Arc<dyn IElementProcessor>>, TemplateProcessingException> {
        let tag_identity = tag.identity();
        if self.last_to_be_repeated {
            if self.current_tag_identity != Some(tag_identity) {
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
        if self.current_tag_identity != Some(tag_identity) {
            self.recompute(tag)?;
            self.current_tag_identity = Some(tag_identity);
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
        if self.current_tag_identity != Some(tag.identity()) {
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
        self.current_tag_identity = original.current_tag_identity;
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
                    "Two different registered processors have returned zero as a result of their comparison, which is forbidden. Offending processors are {} and {}",
                    new_processor.java_class_name(),
                    old_processors
                        .iter()
                        .find(|old_processor| {
                            compare_processors(new_processor, old_processor) == Ordering::Equal
                        })
                        .expect("the preceding equality check must identify an offending processor")
                        .java_class_name()
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
    crate::util::ProcessorComparators::compare_processors(left.as_ref(), right.as_ref())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use super::ElementProcessorIterator;
    use crate::element::{IElementProcessor, MatchingAttributeName};
    use crate::engine::{
        Attribute, AttributeDefinitionValue, AttributeDefinitions, ElementDefinitionValue,
        ElementDefinitions, ElementProcessorsByTemplateMode, OpenElementTag,
    };
    use crate::model::{AttributeValueQuotes, IProcessableElementTag};
    use crate::processor::IProcessor;
    use crate::templatemode::TemplateMode;
    use crate::util::JavaString;

    /// 对应 Java: `ProcessorAggregationTestDialect.TestElementProcessor`。
    ///
    /// 该最小测试方言 Processor 只描述匹配规则与 precedence，实际关联、重建和
    /// 迭代均由生产 `AttributeDefinitions`、`ElementDefinitions` 与标签对象完成。
    struct TestElementProcessor {
        name: &'static str,
        precedence: i32,
        matching_attribute_name: MatchingAttributeName,
    }

    impl IProcessor for TestElementProcessor {
        fn as_element_processor(&self) -> Option<&dyn IElementProcessor> {
            Some(self)
        }

        fn java_class_name(&self) -> &'static str {
            self.name
        }

        fn get_template_mode(&self) -> Option<TemplateMode> {
            Some(TemplateMode::HTML)
        }

        fn get_precedence(&self) -> i32 {
            self.precedence
        }
    }

    impl IElementProcessor for TestElementProcessor {
        fn get_matching_element_name(&self) -> Option<&crate::element::MatchingElementName> {
            None
        }

        fn get_matching_attribute_name(&self) -> Option<&MatchingAttributeName> {
            Some(&self.matching_attribute_name)
        }
    }

    fn java(value: &str) -> JavaString {
        JavaString::from_rust_str(value)
    }

    fn processor(
        name: &'static str,
        attribute: &str,
        precedence: i32,
    ) -> Arc<dyn IElementProcessor> {
        let matching_name = crate::engine::AttributeNames::for_html_name(Some(&java(attribute)))
            .expect("test attribute name");
        Arc::new(TestElementProcessor {
            name,
            precedence,
            matching_attribute_name: MatchingAttributeName::for_attribute_name(
                Some(TemplateMode::HTML),
                Some(crate::engine::AttributeNameValue::Html(matching_name)),
            )
            .expect("test matching attribute name"),
        })
    }

    fn definitions(
        processors: Vec<Arc<dyn IElementProcessor>>,
    ) -> (AttributeDefinitions, ElementDefinitions) {
        let processor_map: ElementProcessorsByTemplateMode =
            HashMap::from([(TemplateMode::HTML, processors)]);
        let attribute_definitions =
            AttributeDefinitions::new(processor_map.clone()).expect("test attribute definitions");
        let element_definitions =
            ElementDefinitions::new(processor_map).expect("test element definitions");
        (attribute_definitions, element_definitions)
    }

    fn tag_with_src(
        attribute_definitions: &AttributeDefinitions,
        element_definitions: &ElementDefinitions,
    ) -> Arc<OpenElementTag> {
        let element_definition = ElementDefinitionValue::Html(
            element_definitions
                .for_html_name(Some(&java("element")))
                .expect("element definition"),
        );
        let src_definition = AttributeDefinitionValue::Html(
            attribute_definitions
                .for_html_name(Some(&java("data-th-src")))
                .expect("src definition"),
        );
        let attributes = crate::engine::Attributes::new(
            Some(vec![Arc::new(Attribute::new(
                src_definition,
                java("data-th-src"),
                None,
                Some(java("src")),
                Some(AttributeValueQuotes::DOUBLE),
                None,
                -1,
                -1,
            ))]),
            Some(vec![java(" ")]),
        );
        Arc::new(OpenElementTag::new(
            TemplateMode::HTML,
            element_definition,
            java("element"),
            Some(attributes),
            false,
        ))
    }

    fn add_one(
        tag: &Arc<OpenElementTag>,
        definitions: &AttributeDefinitions,
    ) -> Arc<OpenElementTag> {
        let definition = AttributeDefinitionValue::Html(
            definitions
                .for_html_name(Some(&java("data-th-one")))
                .expect("one definition"),
        );
        tag.set_attribute(
            definitions,
            Some(&definition),
            java("data-th-one"),
            Some(java("one")),
            Some(AttributeValueQuotes::DOUBLE),
        )
        .expect("add one attribute")
    }

    fn next_precedence(iterator: &mut ElementProcessorIterator, tag: &OpenElementTag) -> String {
        iterator
            .next(
                tag.as_engine_processable_element_tag()
                    .expect("engine processable tag"),
            )
            .expect("iterator next")
            .map_or_else(
                || "null".to_owned(),
                |processor| processor.get_precedence().to_string(),
            )
    }

    fn assert_golden(case: &str, output: String) {
        let java_trace = include_str!("../../tests/fixtures/element_processor_iterator_golden.txt")
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{case}=")))
            .expect("Java Golden case");
        let mut expected = java_trace
            .split("N-ELEMENT-")
            .skip(1)
            .map(|entry| {
                entry
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>()
            })
            .collect::<Vec<_>>();
        if java_trace.ends_with("null") {
            expected.push("null".to_owned());
        }
        let expected = expected.join(",");
        assert_eq!(output, expected, "Java Golden {case}");
    }

    #[test]
    fn preserves_java_dynamic_attribute_processor_iteration_cases() {
        for (case, one_precedence, operation) in [
            ("case01", 0, "none"),
            ("case02", 15, "add_after_first"),
            ("case03", 7, "add_after_first"),
            ("case04", 2, "add_after_first"),
            ("case06", 0, "remove_before_first"),
            ("case07", 2, "remove_then_add"),
            ("case08", 2, "add_then_remove_after_first"),
            ("case09", 2, "add_after_first_then_remove"),
        ] {
            let mut processors = vec![
                processor("N-ELEMENT-5", "data-th-src", 5),
                processor("N-ELEMENT-10", "data-th-src", 10),
            ];
            if operation != "none" {
                processors.push(processor("N-ELEMENT-ONE", "data-th-one", one_precedence));
            }
            let (attribute_definitions, element_definitions) = definitions(processors);
            let mut tag = tag_with_src(&attribute_definitions, &element_definitions);
            let mut iterator = ElementProcessorIterator::new();
            let mut actual = Vec::new();

            match operation {
                "remove_before_first" => {
                    tag = tag
                        .remove_attribute(&java("data-th-src"))
                        .expect("remove src");
                }
                "remove_then_add" => {
                    tag = tag
                        .remove_attribute(&java("data-th-src"))
                        .expect("remove src");
                    tag = add_one(&tag, &attribute_definitions);
                }
                "add_after_first" => {
                    actual.push(next_precedence(&mut iterator, &tag));
                    tag = add_one(&tag, &attribute_definitions);
                }
                "add_then_remove_after_first" => {
                    actual.push(next_precedence(&mut iterator, &tag));
                    tag = add_one(&tag, &attribute_definitions);
                    tag = tag
                        .remove_attribute(&java("data-th-src"))
                        .expect("remove src");
                }
                "add_after_first_then_remove" => {
                    actual.push(next_precedence(&mut iterator, &tag));
                    tag = add_one(&tag, &attribute_definitions);
                    actual.push(next_precedence(&mut iterator, &tag));
                    tag = tag
                        .remove_attribute(&java("data-th-src"))
                        .expect("remove src");
                }
                "none" => {}
                _ => unreachable!("fixed test operation"),
            }
            while actual.last().is_none_or(|value| value != "null") {
                actual.push(next_precedence(&mut iterator, &tag));
            }
            assert_golden(case, actual.join(","));
        }
    }
}
