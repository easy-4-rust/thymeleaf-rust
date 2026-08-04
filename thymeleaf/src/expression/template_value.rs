use std::any::Any;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use crate::util::{EvaluationValue, NumberValue, Utf16String, double_string};

use super::LiteralValue;

/// Rust 用户对象接入 Thymeleaf 动态值模型的合同。
///
/// Java `Object` 同时具有运行时类、引用身份和 `toString()`。实现此 trait 可在不
/// 依赖 JVM 反射的情况下向表达式求值、序列化器和属性访问器提供这些信息。
/// 对应 Java 语义：Rust 侧内部类型（Java 无直接对应对象）。
pub trait TemplateObject: Any + Send + Sync {
    /// 返回用于诊断和类型分派的 Java 风格运行时类名。
    fn class_name(&self) -> &str;

    /// 返回 Java `Object#toString()` 对应 UTF-16 文本。
    fn to_utf16_string(&self) -> Utf16String;

    /// 返回 `Any` 视图，供已注册的属性和方法访问器安全向下转型。
    fn as_any(&self) -> &dyn Any;

    /// 执行 Java `Object#equals` 等价比较。
    fn template_equals(&self, other: &dyn TemplateObject) -> bool {
        std::ptr::eq(self.as_any(), other.as_any())
    }

    /// 若对象实现 Java Comparable，则执行同类对象比较。
    fn template_compare_to(
        &self,
        _other: &dyn TemplateObject,
    ) -> Option<Result<Ordering, TemplateObjectComparisonError>> {
        None
    }

    /// 若对象实现 Java `Iterable`，返回当前迭代顺序的值快照。
    ///
    /// 默认对象不可迭代；Link 参数归一化等 Java 反射路径通过此 capability 保留
    /// 动态对象行为。
    fn iterable_values(&self) -> Option<Vec<Arc<TemplateValue>>> {
        None
    }

    /// 返回 JavaBean/record 序列化可见属性的稳定顺序快照。
    ///
    /// Standard JavaScript Serializer 对 JavaBean 使用 Introspector、对 record 使用
    /// record component 顺序。Rust 宿主对象通过该 capability 暴露等价属性；返回
    /// `None` 表示对象没有可枚举属性。
    fn serializable_properties(&self) -> Option<Vec<(Utf16String, Option<Arc<TemplateValue>>)>> {
        None
    }

    /// 返回对象在 Jackson 模块中的代理序列化值。
    ///
    /// 外层 `None` 表示没有代理；`Some(None)` 表示序列化为 JSON null。Optional 等
    /// 包装类型可借此保留 Java 模块的解包语义。
    fn serializable_value(&self) -> Option<Option<Arc<TemplateValue>>> {
        None
    }

    /// 按 JavaBean/OGNL 属性名读取动态属性。
    ///
    /// 外层 `None` 表示对象不提供该属性访问器；`Some(Ok(None))` 表示属性存在但值为
    /// Java null；错误保留宿主 getter 抛出的运行时异常。
    fn get_property(
        &self,
        _property_name: &Utf16String,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        None
    }

    /// 按 OGNL/Java 方法名调用宿主对象方法。
    ///
    /// 外层 `None` 表示对象不提供该方法；`Some(Ok(None))` 表示方法返回 Java
    /// null；错误保留宿主方法抛出的运行时异常。
    fn invoke_method(
        &self,
        _method_name: &Utf16String,
        _arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectMethodError>> {
        None
    }

    /// 若对象实现 `ILazyContextVariable`，解析并返回其缓存值。
    ///
    /// 外层 `None` 表示普通对象；`Some(None)` 表示惰性变量解析为 Java null。
    /// 对应 Java: `EngineContext#resolveLazy(Object)`。
    fn resolve_lazy_context_variable(&self) -> Option<Option<Arc<TemplateValue>>> {
        None
    }
}

/// 宿主对象 Comparable 调用的动态错误。
pub type TemplateObjectComparisonError = Box<dyn Error + Send + Sync>;

/// 宿主对象属性 getter 的动态错误。
pub type TemplateObjectPropertyError = Box<dyn Error + Send + Sync>;

/// 宿主对象方法调用的动态错误。
pub type TemplateObjectMethodError = Box<dyn Error + Send + Sync>;

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
    Number(NumberValue),
    /// Java `Character` 的单个 UTF-16 代码单元。
    Character(u16),
    /// Java `String`。
    String(Arc<Utf16String>),
    /// Java `byte[]`。
    Bytes(Arc<Vec<i8>>),
    /// Java 数组或 List 的有序元素；null 使用 `TemplateValue::Null`。
    List(Arc<Vec<Arc<TemplateValue>>>),
    /// Java Map 的有序 key/value 条目；null 使用 `TemplateValue::Null`。
    Map(Arc<Vec<(Arc<TemplateValue>, Arc<TemplateValue>)>>),
    /// 文本字面量包装，阻止后续算术阶段重新解释其内容。
    Literal(Arc<LiteralValue>),
    /// `NoOpToken.VALUE` 的 Rust 等价单例值。
    NoOp,
    /// 宿主注册的任意 Java 对象等价物。
    Object(Arc<dyn TemplateObject>),
    /// 已由应用确认无需 HTML 转义的文本。
    SafeHtml(Arc<Utf16String>),
}

impl TemplateValue {
    /// 创建 Java `String` 模板值。
    #[must_use]
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn string(value: Utf16String) -> Self {
        Self::String(Arc::new(value))
    }

    /// 创建受信任的免 HTML 转义文本。
    #[must_use]
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn safe_html(value: Utf16String) -> Self {
        Self::SafeHtml(Arc::new(value))
    }

    /// 转换为 `EvaluationUtils` 使用的 Java 标量运行时分类。
    ///
    /// 集合、数组、宿主对象和 NoOp 保留为其他对象；SafeHtml 在 Java 中仍是 String。
    #[must_use]
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn to_evaluation_value(&self) -> EvaluationValue {
        match self {
            Self::Null => EvaluationValue::Null,
            Self::Boolean(value) => EvaluationValue::Boolean(*value),
            Self::Number(value) => EvaluationValue::Number(value.clone()),
            Self::Character(value) => EvaluationValue::Character(*value),
            Self::String(value) | Self::SafeHtml(value) => {
                EvaluationValue::String(value.as_ref().clone())
            }
            Self::Literal(value) => EvaluationValue::LiteralValue(Arc::clone(value)),
            Self::Bytes(_) => EvaluationValue::Other("[B".to_owned()),
            Self::List(_) => EvaluationValue::Other("java.util.List".to_owned()),
            Self::Map(_) => EvaluationValue::Other("java.util.Map".to_owned()),
            Self::Object(value) => EvaluationValue::Other(value.class_name().to_owned()),
            Self::NoOp => {
                EvaluationValue::Other("org.thymeleaf.standard.expression.NoOpToken".to_owned())
            }
        }
    }

    /// 执行 Java `Object#toString()` 等价转换。
    ///
    /// `None` 仅表示 `LiteralValue` 内部为 Java null；普通 `TemplateValue::Null`
    /// 仍按 `String.valueOf`/拼接语义返回文本 `null`。
    #[must_use]
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn to_utf16_string(&self) -> Option<Utf16String> {
        let text = match self {
            Self::Null => return Some(Utf16String::from_rust_str("null")),
            Self::Boolean(value) => value.to_string(),
            Self::Number(NumberValue::BigDecimal(value)) => value.to_string(),
            Self::Number(NumberValue::BigInteger(value)) => value.to_string(),
            Self::Number(NumberValue::Byte(value)) => value.to_string(),
            Self::Number(NumberValue::Short(value)) => value.to_string(),
            Self::Number(NumberValue::Integer(value)) => value.to_string(),
            Self::Number(NumberValue::Long(value)) => value.to_string(),
            Self::Number(NumberValue::Float(value)) => value.to_string(),
            Self::Number(NumberValue::Double(value)) => double_string(*value),
            Self::Number(NumberValue::Other { double_value, .. }) => double_value.to_string(),
            Self::Character(value) => {
                return Some(Utf16String::from_utf16(vec![*value]));
            }
            Self::String(value) | Self::SafeHtml(value) => return Some(value.as_ref().clone()),
            Self::Bytes(value) => format!("[B@{:x}", Arc::as_ptr(value) as usize),
            Self::List(values) => {
                let mut units = vec![b'[' as u16];
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        units.extend_from_slice(&[b',' as u16, b' ' as u16]);
                    }
                    units.extend_from_slice(
                        value
                            .to_utf16_string()
                            .unwrap_or_else(|| Utf16String::from_rust_str("null"))
                            .as_utf16(),
                    );
                }
                units.push(b']' as u16);
                return Some(Utf16String::from_utf16(units));
            }
            Self::Map(entries) => {
                let mut units = vec![b'{' as u16];
                for (index, (key, value)) in entries.iter().enumerate() {
                    if index != 0 {
                        units.extend_from_slice(&[b',' as u16, b' ' as u16]);
                    }
                    units.extend_from_slice(
                        key.to_utf16_string()
                            .unwrap_or_else(|| Utf16String::from_rust_str("null"))
                            .as_utf16(),
                    );
                    units.push(b'=' as u16);
                    units.extend_from_slice(
                        value
                            .to_utf16_string()
                            .unwrap_or_else(|| Utf16String::from_rust_str("null"))
                            .as_utf16(),
                    );
                }
                units.push(b'}' as u16);
                return Some(Utf16String::from_utf16(units));
            }
            Self::Literal(value) => return value.get_value().cloned(),
            Self::NoOp => return Some(Utf16String::from_rust_str("_")),
            Self::Object(value) => return Some(value.to_utf16_string()),
        };
        Some(Utf16String::from_rust_str(&text))
    }

    /// 执行 Java 对象的 `equals` 等价比较。
    #[must_use]
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn template_equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::Number(left), Self::Number(right)) => java_number_equals(left, right),
            (Self::Character(left), Self::Character(right)) => left == right,
            (Self::String(left), Self::String(right))
            | (Self::SafeHtml(left), Self::SafeHtml(right))
            | (Self::String(left), Self::SafeHtml(right))
            | (Self::SafeHtml(left), Self::String(right)) => left == right,
            (Self::Bytes(left), Self::Bytes(right)) => Arc::ptr_eq(left, right),
            (Self::List(left), Self::List(right)) => {
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right.iter())
                        .all(|(left, right)| left.template_equals(right))
            }
            (Self::Map(left), Self::Map(right)) => java_map_equals(left, right),
            (Self::Object(left), Self::Object(right)) => {
                left.class_name() == right.class_name() && left.template_equals(right.as_ref())
            }
            (Self::Literal(left), Self::Literal(right)) => Arc::ptr_eq(left, right),
            (Self::NoOp, Self::NoOp) => true,
            _ => false,
        }
    }

    /// 若两个对象具有相同 Java 运行时类且实现 Comparable，则返回比较结果。
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn template_compare_to(
        &self,
        other: &Self,
    ) -> Option<Result<Ordering, TemplateObjectComparisonError>> {
        match (self, other) {
            (Self::Boolean(left), Self::Boolean(right)) => Some(Ok(left.cmp(right))),
            (Self::Character(left), Self::Character(right)) => Some(Ok(left.cmp(right))),
            (Self::String(left), Self::String(right))
            | (Self::SafeHtml(left), Self::SafeHtml(right))
            | (Self::String(left), Self::SafeHtml(right))
            | (Self::SafeHtml(left), Self::String(right)) => {
                Some(Ok(left.as_utf16().cmp(right.as_utf16())))
            }
            (Self::Number(left), Self::Number(right)) => java_number_compare(left, right).map(Ok),
            (Self::Object(left), Self::Object(right))
                if left.class_name() == right.class_name() =>
            {
                left.template_compare_to(right.as_ref())
            }
            _ => None,
        }
    }

    /// 返回 Java 风格运行时类名。
    #[must_use]
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    pub fn class_name(&self) -> &str {
        match self {
            Self::Null => "null",
            Self::Boolean(_) => "java.lang.Boolean",
            Self::Number(number) => java_number_class_name(number),
            Self::Character(_) => "java.lang.Character",
            Self::String(_) | Self::SafeHtml(_) => "java.lang.String",
            Self::Bytes(_) => "[B",
            Self::List(_) => "java.util.List",
            Self::Map(_) => "java.util.Map",
            Self::Literal(_) => "org.thymeleaf.standard.expression.LiteralValue",
            Self::NoOp => "org.thymeleaf.standard.expression.NoOpToken",
            Self::Object(object) => object.class_name(),
        }
    }
}

impl super::ConversionObject for TemplateValue {
    fn java_to_string(
        &self,
    ) -> Result<super::Utf16StringConversionResult<'_>, super::StandardConversionError> {
        Ok(match self {
            Self::String(value) | Self::SafeHtml(value) => {
                super::Utf16StringConversionResult::Borrowed(value)
            }
            _ => match self.to_utf16_string() {
                Some(value) => super::Utf16StringConversionResult::Owned(value),
                None => super::Utf16StringConversionResult::Null,
            },
        })
    }
}

fn java_number_equals(left: &NumberValue, right: &NumberValue) -> bool {
    match (left, right) {
        (NumberValue::BigDecimal(left), NumberValue::BigDecimal(right)) => left == right,
        (NumberValue::BigInteger(left), NumberValue::BigInteger(right)) => left == right,
        (NumberValue::Byte(left), NumberValue::Byte(right)) => left == right,
        (NumberValue::Short(left), NumberValue::Short(right)) => left == right,
        (NumberValue::Integer(left), NumberValue::Integer(right)) => left == right,
        (NumberValue::Long(left), NumberValue::Long(right)) => left == right,
        (NumberValue::Float(left), NumberValue::Float(right)) => {
            normalized_f32_bits(*left) == normalized_f32_bits(*right)
        }
        (NumberValue::Double(left), NumberValue::Double(right)) => {
            normalized_f64_bits(*left) == normalized_f64_bits(*right)
        }
        (
            NumberValue::Other {
                class_name: left_class,
                double_value: left,
            },
            NumberValue::Other {
                class_name: right_class,
                double_value: right,
            },
        ) => left_class == right_class && normalized_f64_bits(*left) == normalized_f64_bits(*right),
        _ => false,
    }
}

fn java_number_compare(left: &NumberValue, right: &NumberValue) -> Option<Ordering> {
    match (left, right) {
        (NumberValue::BigDecimal(left), NumberValue::BigDecimal(right)) => {
            Some(left.compare_java(right))
        }
        (NumberValue::BigInteger(left), NumberValue::BigInteger(right)) => Some(left.cmp(right)),
        (NumberValue::Byte(left), NumberValue::Byte(right)) => Some(left.cmp(right)),
        (NumberValue::Short(left), NumberValue::Short(right)) => Some(left.cmp(right)),
        (NumberValue::Integer(left), NumberValue::Integer(right)) => Some(left.cmp(right)),
        (NumberValue::Long(left), NumberValue::Long(right)) => Some(left.cmp(right)),
        (NumberValue::Float(left), NumberValue::Float(right)) => Some(java_f32_cmp(*left, *right)),
        (NumberValue::Double(left), NumberValue::Double(right)) => {
            Some(java_f64_cmp(*left, *right))
        }
        _ => None,
    }
}

fn normalized_f32_bits(value: f32) -> u32 {
    if value.is_nan() {
        f32::NAN.to_bits()
    } else {
        value.to_bits()
    }
}

fn normalized_f64_bits(value: f64) -> u64 {
    if value.is_nan() {
        f64::NAN.to_bits()
    } else {
        value.to_bits()
    }
}

fn java_f32_cmp(left: f32, right: f32) -> Ordering {
    if left < right {
        return Ordering::Less;
    }
    if left > right {
        return Ordering::Greater;
    }
    i32::from_ne_bytes(normalized_f32_bits(left).to_ne_bytes()).cmp(&i32::from_ne_bytes(
        normalized_f32_bits(right).to_ne_bytes(),
    ))
}

fn java_f64_cmp(left: f64, right: f64) -> Ordering {
    if left < right {
        return Ordering::Less;
    }
    if left > right {
        return Ordering::Greater;
    }
    i64::from_ne_bytes(normalized_f64_bits(left).to_ne_bytes()).cmp(&i64::from_ne_bytes(
        normalized_f64_bits(right).to_ne_bytes(),
    ))
}

fn java_map_equals(
    left: &[(Arc<TemplateValue>, Arc<TemplateValue>)],
    right: &[(Arc<TemplateValue>, Arc<TemplateValue>)],
) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut matched = vec![false; right.len()];
    left.iter().all(|(left_key, left_value)| {
        right
            .iter()
            .enumerate()
            .find(|(index, (right_key, right_value))| {
                !matched[*index]
                    && left_key.template_equals(right_key)
                    && left_value.template_equals(right_value)
            })
            .is_some_and(|(index, _)| {
                matched[index] = true;
                true
            })
    })
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
            Self::Literal(value) => formatter.debug_tuple("Literal").field(value).finish(),
            Self::NoOp => formatter.write_str("NoOp"),
            Self::Object(value) => formatter
                .debug_struct("Object")
                .field("class_name", &value.class_name())
                .finish_non_exhaustive(),
            Self::SafeHtml(value) => formatter.debug_tuple("SafeHtml").field(value).finish(),
        }
    }
}

fn java_number_class_name(number: &NumberValue) -> &str {
    match number {
        NumberValue::BigDecimal(_) => "java.math.BigDecimal",
        NumberValue::BigInteger(_) => "java.math.BigInteger",
        NumberValue::Byte(_) => "java.lang.Byte",
        NumberValue::Short(_) => "java.lang.Short",
        NumberValue::Integer(_) => "java.lang.Integer",
        NumberValue::Long(_) => "java.lang.Long",
        NumberValue::Float(_) => "java.lang.Float",
        NumberValue::Double(_) => "java.lang.Double",
        NumberValue::Other { class_name, .. } => class_name,
    }
}
