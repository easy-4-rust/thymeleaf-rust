use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use crate::util::{JavaNumber, JavaString};

/// Rust 用户对象接入 Thymeleaf 动态值模型的合同。
///
/// Java `Object` 同时具有运行时类、引用身份和 `toString()`。实现此 trait 可在不
/// 依赖 JVM 反射的情况下向表达式求值、序列化器和属性访问器提供这些信息。
pub trait TemplateObject: Any + Send + Sync {
    /// 返回用于诊断和类型分派的 Java 风格运行时类名。
    fn java_class_name(&self) -> &str;

    /// 返回 Java `Object#toString()` 对应 UTF-16 文本。
    fn to_java_string(&self) -> JavaString;

    /// 返回 `Any` 视图，供已注册的属性和方法访问器安全向下转型。
    fn as_any(&self) -> &dyn Any;
}

/// Thymeleaf 核心在上下文、表达式、序列化和 Processor 间传递的动态值。
///
/// 对应 Java: 模板 API 中反复出现的 `java.lang.Object`。
///
/// 集合以顺序条目表示，既保留 Java 集合迭代顺序，也允许 null、重复 Map key
/// 快照和不可 Hash 的任意模板值。用户对象以 `Arc` 保留跨上下文读取的引用身份。
#[derive(Clone)]
pub enum TemplateValue {
    /// Java null；与 Context 中“变量不存在”保持独立。
    Null,
    /// Java `Boolean`。
    Boolean(bool),
    /// 具有精确 Java 包装类语义的任意 `Number`。
    Number(JavaNumber),
    /// Java `Character` 的单个 UTF-16 代码单元。
    Character(u16),
    /// Java `String`。
    String(Arc<JavaString>),
    /// Java `byte[]`。
    Bytes(Arc<Vec<i8>>),
    /// Java 数组或 List 的有序元素；null 使用 `TemplateValue::Null`。
    List(Arc<Vec<Arc<TemplateValue>>>),
    /// Java Map 的有序 key/value 条目；null 使用 `TemplateValue::Null`。
    Map(Arc<Vec<(Arc<TemplateValue>, Arc<TemplateValue>)>>),
    /// 宿主注册的任意 Java 对象等价物。
    Object(Arc<dyn TemplateObject>),
    /// 已由应用确认无需 HTML 转义的文本。
    SafeHtml(Arc<JavaString>),
}

impl TemplateValue {
    /// 创建 Java `String` 模板值。
    #[must_use]
    pub fn string(value: JavaString) -> Self {
        Self::String(Arc::new(value))
    }

    /// 创建受信任的免 HTML 转义文本。
    #[must_use]
    pub fn safe_html(value: JavaString) -> Self {
        Self::SafeHtml(Arc::new(value))
    }

    /// 返回 Java 风格运行时类名。
    #[must_use]
    pub fn java_class_name(&self) -> &str {
        match self {
            Self::Null => "null",
            Self::Boolean(_) => "java.lang.Boolean",
            Self::Number(number) => java_number_class_name(number),
            Self::Character(_) => "java.lang.Character",
            Self::String(_) | Self::SafeHtml(_) => "java.lang.String",
            Self::Bytes(_) => "[B",
            Self::List(_) => "java.util.List",
            Self::Map(_) => "java.util.Map",
            Self::Object(object) => object.java_class_name(),
        }
    }
}

impl Debug for TemplateValue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => formatter.write_str("Null"),
            Self::Boolean(value) => formatter.debug_tuple("Boolean").field(value).finish(),
            Self::Number(value) => formatter.debug_tuple("Number").field(value).finish(),
            Self::Character(value) => formatter.debug_tuple("Character").field(value).finish(),
            Self::String(value) => formatter.debug_tuple("String").field(value).finish(),
            Self::Bytes(value) => formatter.debug_tuple("Bytes").field(value).finish(),
            Self::List(value) => formatter.debug_tuple("List").field(value).finish(),
            Self::Map(value) => formatter.debug_tuple("Map").field(value).finish(),
            Self::Object(value) => formatter
                .debug_struct("Object")
                .field("java_class_name", &value.java_class_name())
                .finish_non_exhaustive(),
            Self::SafeHtml(value) => formatter.debug_tuple("SafeHtml").field(value).finish(),
        }
    }
}

fn java_number_class_name(number: &JavaNumber) -> &str {
    match number {
        JavaNumber::BigDecimal(_) => "java.math.BigDecimal",
        JavaNumber::BigInteger(_) => "java.math.BigInteger",
        JavaNumber::Byte(_) => "java.lang.Byte",
        JavaNumber::Short(_) => "java.lang.Short",
        JavaNumber::Integer(_) => "java.lang.Integer",
        JavaNumber::Long(_) => "java.lang.Long",
        JavaNumber::Float(_) => "java.lang.Float",
        JavaNumber::Double(_) => "java.lang.Double",
        JavaNumber::Other { class_name, .. } => class_name,
    }
}
