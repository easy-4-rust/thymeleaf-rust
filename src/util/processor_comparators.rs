use std::cmp::Ordering;

use crate::postprocessor::IPostProcessor;
use crate::preprocessor::IPreProcessor;
use crate::processor::IProcessor;

/// 提供 Processor、PreProcessor 和 PostProcessor 的 Java 排序规则。
///
/// 对应 Java: `org.thymeleaf.util.ProcessorComparators`。
pub struct ProcessorComparators;

impl ProcessorComparators {
    /// 比较两个 Processor。
    ///
    /// 配置包装器先比较方言 precedence，再比较 Processor precedence、Java 类名和
    /// 对象身份；同一对象是唯一返回 `Equal` 的场景。
    pub fn compare_processors<T>(left: &T, right: &T) -> Ordering
    where
        T: IProcessor + ?Sized,
    {
        ProcessorPrecedenceComparator::compare(left, right)
    }

    /// 比较 PreProcessor。对应 Java:
    /// `ProcessorComparators.PRE_PROCESSOR_COMPARATOR`。
    pub fn compare_pre_processors(left: &dyn IPreProcessor, right: &dyn IPreProcessor) -> Ordering {
        PreProcessorPrecedenceComparator::compare(left, right)
    }

    /// 比较 PostProcessor。对应 Java:
    /// `ProcessorComparators.POST_PROCESSOR_COMPARATOR`。
    pub fn compare_post_processors(
        left: &dyn IPostProcessor,
        right: &dyn IPostProcessor,
    ) -> Ordering {
        PostProcessorPrecedenceComparator::compare(left, right)
    }
}

/// Processor precedence、类名与身份的全序比较器。
///
/// 对应 Java: `ProcessorComparators.ProcessorPrecedenceComparator`。
struct ProcessorPrecedenceComparator;

impl ProcessorPrecedenceComparator {
    fn compare<T>(left: &T, right: &T) -> Ordering
    where
        T: IProcessor + ?Sized,
    {
        if std::ptr::eq(left, right) {
            return Ordering::Equal;
        }
        if let (Some(left_dialect), Some(right_dialect)) = (
            left.get_dialect_precedence(),
            right.get_dialect_precedence(),
        ) {
            let dialect = left_dialect.cmp(&right_dialect);
            if dialect != Ordering::Equal {
                return dialect;
            }
        }
        let left_wrapped = left.get_wrapped_processor();
        let right_wrapped = right.get_wrapped_processor();
        let left_precedence =
            left_wrapped.map_or_else(|| left.get_precedence(), IProcessor::get_precedence);
        let right_precedence =
            right_wrapped.map_or_else(|| right.get_precedence(), IProcessor::get_precedence);
        let left_class =
            left_wrapped.map_or_else(|| left.java_class_name(), IProcessor::java_class_name);
        let right_class =
            right_wrapped.map_or_else(|| right.java_class_name(), IProcessor::java_class_name);
        left_precedence
            .cmp(&right_precedence)
            .then_with(|| left_class.cmp(right_class))
            .then_with(|| {
                processor_identity_value(left, left_wrapped)
                    .cmp(&processor_identity_value(right, right_wrapped))
            })
    }
}

/// PreProcessor precedence、handler 类名与身份的全序比较器。
///
/// 对应 Java: `ProcessorComparators.PreProcessorPrecedenceComparator`。
struct PreProcessorPrecedenceComparator;

impl PreProcessorPrecedenceComparator {
    fn compare(left: &dyn IPreProcessor, right: &dyn IPreProcessor) -> Ordering {
        if std::ptr::eq(left, right) {
            return Ordering::Equal;
        }
        let left_wrapped = left.get_wrapped_pre_processor();
        let right_wrapped = right.get_wrapped_pre_processor();
        let dialect_order = match (
            left.get_dialect_precedence(),
            right.get_dialect_precedence(),
        ) {
            (Some(left), Some(right)) => left.cmp(&right),
            _ => Ordering::Equal,
        };
        let left_processor = left_wrapped.unwrap_or(left);
        let right_processor = right_wrapped.unwrap_or(right);
        dialect_order
            .then_with(|| {
                left_processor
                    .get_precedence()
                    .cmp(&right_processor.get_precedence())
            })
            .then_with(|| {
                left_processor
                    .java_class_name()
                    .cmp(right_processor.java_class_name())
            })
            .then_with(|| {
                pre_processor_identity(left_processor).cmp(&pre_processor_identity(right_processor))
            })
    }
}

/// PostProcessor precedence、handler 类名与身份的全序比较器。
///
/// 对应 Java: `ProcessorComparators.PostProcessorPrecedenceComparator`。
struct PostProcessorPrecedenceComparator;

impl PostProcessorPrecedenceComparator {
    fn compare(left: &dyn IPostProcessor, right: &dyn IPostProcessor) -> Ordering {
        if std::ptr::eq(left, right) {
            return Ordering::Equal;
        }
        let left_wrapped = left.get_wrapped_post_processor();
        let right_wrapped = right.get_wrapped_post_processor();
        let dialect_order = match (
            left.get_dialect_precedence(),
            right.get_dialect_precedence(),
        ) {
            (Some(left), Some(right)) => left.cmp(&right),
            _ => Ordering::Equal,
        };
        let left_processor = left_wrapped.unwrap_or(left);
        let right_processor = right_wrapped.unwrap_or(right);
        dialect_order
            .then_with(|| {
                left_processor
                    .get_precedence()
                    .cmp(&right_processor.get_precedence())
            })
            .then_with(|| {
                left_processor
                    .java_class_name()
                    .cmp(right_processor.java_class_name())
            })
            .then_with(|| {
                post_processor_identity(left_processor)
                    .cmp(&post_processor_identity(right_processor))
            })
    }
}

fn processor_identity(processor: &dyn IProcessor) -> usize {
    std::ptr::from_ref(processor).cast::<()>() as usize
}

fn processor_identity_value<T>(processor: &T, wrapped: Option<&dyn IProcessor>) -> usize
where
    T: IProcessor + ?Sized,
{
    wrapped.map_or_else(
        || std::ptr::from_ref(processor).cast::<()>() as usize,
        processor_identity,
    )
}

fn pre_processor_identity(processor: &dyn IPreProcessor) -> usize {
    std::ptr::from_ref(processor).cast::<()>() as usize
}

fn post_processor_identity(processor: &dyn IPostProcessor) -> usize {
    std::ptr::from_ref(processor).cast::<()>() as usize
}
