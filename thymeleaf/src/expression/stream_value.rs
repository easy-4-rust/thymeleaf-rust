use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::util::{JavaNumber, Utf16String};

use super::{TemplateObject, TemplateObjectMethodError, TemplateValue};

/// OGNL 可见的只读 Java Stream 快照。
///
/// 对应 Java: `java.util.stream.Stream`。读取或 `count()` 后按 Java 单次消费语义关闭。
pub(crate) struct StreamValue {
    values: Arc<Vec<Arc<TemplateValue>>>,
    consumed: AtomicBool,
}

impl StreamValue {
    /// 从 List 当前顺序创建单次消费流。
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub(crate) fn new(values: Arc<Vec<Arc<TemplateValue>>>) -> Self {
        Self {
            values,
            consumed: AtomicBool::new(false),
        }
    }

    fn consume(&self) -> Option<Vec<Arc<TemplateValue>>> {
        (!self.consumed.swap(true, Ordering::AcqRel)).then(|| self.values.as_ref().clone())
    }
}

impl TemplateObject for StreamValue {
    fn java_class_name(&self) -> &str {
        "java.util.stream.Stream"
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str("java.util.stream.ReferencePipeline$Head")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_iterable_values(&self) -> Option<Vec<Arc<TemplateValue>>> {
        self.consume()
    }

    fn java_invoke_method(
        &self,
        method_name: &Utf16String,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        (method_name.to_string_lossy() == "count" && arguments.is_empty()).then(|| {
            self.consume().map_or_else(
                || {
                    Err("stream has already been operated upon or closed"
                        .to_owned()
                        .into())
                },
                |values| {
                    Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Long(
                        values.len() as i64,
                    )))))
                },
            )
        })
    }
}
