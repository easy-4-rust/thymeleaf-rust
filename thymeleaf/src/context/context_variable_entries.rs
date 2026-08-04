use std::sync::Arc;

use crate::expression::TemplateValue;
use crate::util::Utf16String;

/// Java `Map<String,Object>` 构造参数的有序、可空 Rust 表示。
///
/// 这是上下文对象共享的签名别名，不对应独立 Java 对象。
pub type ContextVariableEntry = (Option<Utf16String>, Option<Arc<TemplateValue>>);

/// 可空上下文变量切片；`None` 对应 Java `null` Map。
pub type ContextVariableEntries<'a> = Option<&'a [ContextVariableEntry]>;
