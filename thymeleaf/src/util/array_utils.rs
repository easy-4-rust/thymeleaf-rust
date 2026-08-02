use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::sync::Arc;

use thiserror::Error;

use crate::expression::JavaObjectArray;

use super::{Validate, ValidateError};

type ArrayPredicate<T> = dyn Fn(&T) -> bool + Send + Sync;

/// Java 数组或 `Iterable` 中元素的运行时类信息。
///
/// 对应 Java: `java.lang.Object#getClass()` 以及
/// `org.thymeleaf.util.ArrayUtils#toArray(Class,Object)` 的组件类推断。
pub trait JavaArrayElement {
    /// 返回 Java `Class#getName()` 形式的精确运行时类名。
    ///
    /// # 返回
    /// 当前元素的精确 Java 运行时类名。
    fn java_class_name(&self) -> &str;

    /// 判断当前对象能否赋值给指定 Java 组件类。
    ///
    /// # 参数
    /// - `component_class_name`：目标数组的组件类名。
    ///
    /// # 返回
    /// Java `ArrayStoreException` 检查通过时返回 `true`。
    fn is_instance_of(&self, component_class_name: &str) -> bool {
        component_class_name == "java.lang.Object" || component_class_name == self.java_class_name()
    }
}

impl JavaArrayElement for String {
    fn java_class_name(&self) -> &str {
        "java.lang.String"
    }
}

impl JavaArrayElement for i32 {
    fn java_class_name(&self) -> &str {
        "java.lang.Integer"
    }
}

impl JavaArrayElement for i64 {
    fn java_class_name(&self) -> &str {
        "java.lang.Long"
    }
}

impl JavaArrayElement for f64 {
    fn java_class_name(&self) -> &str {
        "java.lang.Double"
    }
}

impl JavaArrayElement for f32 {
    fn java_class_name(&self) -> &str {
        "java.lang.Float"
    }
}

impl JavaArrayElement for bool {
    fn java_class_name(&self) -> &str {
        "java.lang.Boolean"
    }
}

/// Java `Class<? extends T[]>` 的 Rust 等价描述。
///
/// 对应 Java: `org.thymeleaf.util.ArrayUtils#copyOf(Object[],int,Class)` 的
/// `newType` 参数。
#[derive(Clone)]
pub struct JavaArrayType<T> {
    component_class_name: String,
    component_predicate: Arc<ArrayPredicate<T>>,
}

impl<T> JavaArrayType<T> {
    /// 创建具有运行时写入约束的引用数组类型。
    ///
    /// # 参数
    /// - `component_class_name`：Java 数组组件类名；
    /// - `component_predicate`：判断元素能否写入该类型。
    ///
    /// # 返回
    /// 可传给 [`ArrayUtils::copy_of_with_type`] 的类型描述。
    #[must_use]
    pub fn typed(
        component_class_name: impl Into<String>,
        component_predicate: impl Fn(&T) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            component_class_name: component_class_name.into(),
            component_predicate: Arc::new(component_predicate),
        }
    }

    /// 创建 Java `Object[]` 类型描述。
    ///
    /// # 返回
    /// 接受任意非 null 元素的数组类型。
    #[must_use]
    pub fn object() -> Self {
        Self::typed("java.lang.Object", |_| true)
    }

    /// 返回 Java 组件类名。
    ///
    /// # 返回
    /// `Class#getComponentType().getName()` 等价文本。
    #[must_use]
    pub fn component_class_name(&self) -> &str {
        &self.component_class_name
    }
}

impl<T> Debug for JavaArrayType<T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("JavaArrayType")
            .field("component_class_name", &self.component_class_name)
            .finish_non_exhaustive()
    }
}

/// `ArrayUtils#toArray` 可接受的 Java 运行时目标。
///
/// 对应 Java: `Object target`；枚举显式区分引用数组、primitive 数组、
/// `Iterable` 与其他对象，保留 JVM 分派及异常类别。
pub enum ArrayTarget<'a, T> {
    /// Java 引用数组。
    Reference(&'a JavaObjectArray<T>),
    /// Java primitive 数组，值为 `Class#getName()`，例如 `[I`。
    PrimitiveArray {
        /// primitive 数组运行时类名。
        class_name: &'a str,
        /// 诊断消息使用的组件类名，例如 `int`。
        component_class_name: &'a str,
    },
    /// Java `Iterable<?>`，按迭代顺序保存元素。
    Iterable(&'a [Option<T>]),
    /// 既不是数组也不是 `Iterable` 的对象。
    Other {
        /// Java `Class#getName()` 形式的运行时类名。
        class_name: &'a str,
    },
}

/// `ArrayUtils#toArray` 的借用或新建数组结果。
///
/// Java 在兼容引用数组输入时原样返回同一实例；`Iterable` 输入则反射创建新数组。
#[derive(Debug)]
pub enum JavaArray<'a, T> {
    /// 原数组的同一引用。
    Borrowed(&'a JavaObjectArray<T>),
    /// 从 `Iterable` 新建的数组。
    Owned(JavaObjectArray<T>),
}

impl<'a, T> JavaArray<'a, T> {
    /// 返回结果数组。
    ///
    /// # 返回
    /// 借用输入或持有新数组的统一只读引用。
    #[must_use]
    pub fn as_array(&self) -> &JavaObjectArray<T> {
        match self {
            Self::Borrowed(array) => array,
            Self::Owned(array) => array,
        }
    }

    /// 判断结果是否与指定输入数组为同一实例。
    ///
    /// # 参数
    /// - `target`：用于比较引用身份的输入数组。
    ///
    /// # 返回
    /// 结果借用该数组同一实例时返回 `true`。
    #[must_use]
    pub fn is_same_reference(&self, target: &JavaObjectArray<T>) -> bool {
        matches!(self, Self::Borrowed(array) if std::ptr::eq(*array, target))
    }

    /// 消费结果并返回 owned 数组；借用结果会按 Java 数组组件约束克隆。
    ///
    /// # 返回
    /// 与当前结果等值且独立的引用数组。
    #[must_use]
    pub fn into_owned(self) -> JavaObjectArray<T>
    where
        T: Clone,
    {
        match self {
            Self::Borrowed(array) => array.clone(),
            Self::Owned(array) => array,
        }
    }
}

/// `ArrayUtils` 的类型化异常。
///
/// 对应 Java: `org.thymeleaf.util.ArrayUtils` 显式参数异常及 JVM 数组运行时异常。
#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum ArrayUtilsError {
    /// `Validate.notNull` 的 `IllegalArgumentException`。
    #[error(transparent)]
    Validation(#[from] ValidateError),
    /// 目标无法转换为所需数组。
    #[error("{message}")]
    CannotConvert {
        /// 上游生成的精确 detail message。
        message: String,
    },
    /// primitive array 强制转换到 `Object[]` 失败。
    #[error("class {class_name} cannot be cast to class [Ljava.lang.Object;")]
    ClassCast {
        /// primitive 数组运行时类名。
        class_name: String,
    },
    /// 元素无法写入目标运行时组件类。
    #[error("element cannot be stored in array with component class \"{component_class_name}\"")]
    ArrayStore {
        /// Java 组件类名。
        component_class_name: String,
    },
    /// Java `NegativeArraySizeException`。
    #[error("{length}")]
    NegativeArraySize {
        /// 请求的负长度。
        length: i32,
    },
    /// Java 隐式 `NullPointerException`。
    #[error("")]
    NullPointer,
    /// Java `ArrayIndexOutOfBoundsException`。
    #[error("{message}")]
    ArrayIndexOutOfBounds {
        /// 与 JVM 边界原因等价的诊断消息。
        message: String,
    },
    /// `copyOfRange` 的反向范围参数异常。
    #[error("Cannot copy array range with indexes {from} and {to}")]
    InvalidRange {
        /// 起始索引。
        from: i32,
        /// 结束索引。
        to: i32,
    },
}

impl ArrayUtilsError {
    /// 返回对应 Java 异常类名。
    ///
    /// # 返回
    /// 当前错误对应的 JVM 异常全限定名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::Validation(error) => error.java_class_name(),
            Self::CannotConvert { .. } | Self::InvalidRange { .. } => {
                "java.lang.IllegalArgumentException"
            }
            Self::ClassCast { .. } => "java.lang.ClassCastException",
            Self::ArrayStore { .. } => "java.lang.ArrayStoreException",
            Self::NegativeArraySize { .. } => "java.lang.NegativeArraySizeException",
            Self::NullPointer => "java.lang.NullPointerException",
            Self::ArrayIndexOutOfBounds { .. } => "java.lang.ArrayIndexOutOfBoundsException",
        }
    }
}

/// Java 数组转换、查询和复制工具。
///
/// 对应 Java: `org.thymeleaf.util.ArrayUtils`。
pub struct ArrayUtils;

impl ArrayUtils {
    /// 将引用数组原样返回，或按元素运行时类从 Iterable 创建新数组。
    ///
    /// 对应 Java: `ArrayUtils#toArray(Object)`。
    ///
    /// # 参数
    /// - `target`：可空 Java 运行时目标。
    ///
    /// # 返回
    /// 原引用数组或新建数组。
    pub fn to_array<'a, T>(
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        Self::convert(None, target)
    }

    /// 转换为 `String[]`。对应 Java: `ArrayUtils#toStringArray(Object)`。
    ///
    /// # 参数
    /// - `target`：可空 Java 运行时目标。
    ///
    /// # 返回
    /// 组件类为 `java.lang.String` 的数组。
    pub fn to_string_array<'a, T>(
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        Self::convert(Some("java.lang.String"), target)
    }

    /// 转换为 `Integer[]`。对应 Java: `ArrayUtils#toIntegerArray(Object)`。
    ///
    /// # 参数
    /// - `target`：可空 Java 运行时目标。
    ///
    /// # 返回
    /// 组件类为 `java.lang.Integer` 的数组。
    pub fn to_integer_array<'a, T>(
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        Self::convert(Some("java.lang.Integer"), target)
    }

    /// 转换为 `Long[]`。对应 Java: `ArrayUtils#toLongArray(Object)`。
    ///
    /// # 参数
    /// - `target`：可空 Java 运行时目标。
    ///
    /// # 返回
    /// 组件类为 `java.lang.Long` 的数组。
    pub fn to_long_array<'a, T>(
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        Self::convert(Some("java.lang.Long"), target)
    }

    /// 转换为 `Double[]`。对应 Java: `ArrayUtils#toDoubleArray(Object)`。
    ///
    /// # 参数
    /// - `target`：可空 Java 运行时目标。
    ///
    /// # 返回
    /// 组件类为 `java.lang.Double` 的数组。
    pub fn to_double_array<'a, T>(
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        Self::convert(Some("java.lang.Double"), target)
    }

    /// 转换为 `Float[]`。对应 Java: `ArrayUtils#toFloatArray(Object)`。
    ///
    /// # 参数
    /// - `target`：可空 Java 运行时目标。
    ///
    /// # 返回
    /// 组件类为 `java.lang.Float` 的数组。
    pub fn to_float_array<'a, T>(
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        Self::convert(Some("java.lang.Float"), target)
    }

    /// 转换为 `Boolean[]`。对应 Java: `ArrayUtils#toBooleanArray(Object)`。
    ///
    /// # 参数
    /// - `target`：可空 Java 运行时目标。
    ///
    /// # 返回
    /// 组件类为 `java.lang.Boolean` 的数组。
    pub fn to_boolean_array<'a, T>(
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        Self::convert(Some("java.lang.Boolean"), target)
    }

    /// 返回数组长度。对应 Java: `ArrayUtils#length(Object[])`。
    ///
    /// # 参数
    /// - `target`：可空目标数组。
    ///
    /// # 返回
    /// Java `int` 范围的数组长度。
    pub fn length<T>(target: Option<&[Option<T>]>) -> Result<i32, ArrayUtilsError> {
        Validate::not_null(target, Some("Cannot get array length of null"))?;
        Ok(i32::try_from(target.expect("validated target").len()).unwrap_or(i32::MAX))
    }

    /// 判断数组为 null 或空。对应 Java: `ArrayUtils#isEmpty(Object[])`。
    ///
    /// # 参数
    /// - `target`：可空目标数组。
    ///
    /// # 返回
    /// null 或零长度时返回 `true`。
    #[must_use]
    pub fn is_empty<T>(target: Option<&[Option<T>]>) -> bool {
        target.is_none_or(<[Option<T>]>::is_empty)
    }

    /// 判断数组是否包含元素。对应 Java: `ArrayUtils#contains(Object[],Object)`。
    ///
    /// # 参数
    /// - `target`：可空目标数组；
    /// - `element`：可空请求元素。
    ///
    /// # 返回
    /// 找到相等元素时返回 `true`。
    pub fn contains<T>(
        target: Option<&[Option<T>]>,
        element: &Option<T>,
    ) -> Result<bool, ArrayUtilsError>
    where
        T: PartialEq,
    {
        Validate::not_null(
            target,
            Some("Cannot execute array contains: target is null"),
        )?;
        Ok(target
            .expect("validated target")
            .iter()
            .any(|target_element| element == target_element))
    }

    /// 判断数组是否包含另一个数组的全部不同元素。
    ///
    /// 对应 Java: `ArrayUtils#containsAll(Object[],Object[])`。
    ///
    /// # 参数
    /// - `target`：可空目标数组；
    /// - `elements`：可空请求数组。
    ///
    /// # 返回
    /// 全部不同请求元素都存在时返回 `true`。
    pub fn contains_all_array<T>(
        target: Option<&[Option<T>]>,
        elements: Option<&[Option<T>]>,
    ) -> Result<bool, ArrayUtilsError>
    where
        T: Clone + Eq + Hash,
    {
        Validate::not_null(
            target,
            Some("Cannot execute array containsAll: target is null"),
        )?;
        Validate::not_null(
            elements,
            Some("Cannot execute array containsAll: elements is null"),
        )?;
        Self::contains_all_collection(target, elements)
    }

    /// 判断数组是否包含 Collection 的全部不同元素。
    ///
    /// 对应 Java: `ArrayUtils#containsAll(Object[],Collection)`。
    ///
    /// # 参数
    /// - `target`：可空目标数组；
    /// - `elements`：可空 Collection 等价序列。
    ///
    /// # 返回
    /// 全部不同请求元素都存在时返回 `true`。
    pub fn contains_all_collection<T>(
        target: Option<&[Option<T>]>,
        elements: Option<&[Option<T>]>,
    ) -> Result<bool, ArrayUtilsError>
    where
        T: Clone + Eq + Hash,
    {
        Validate::not_null(
            target,
            Some("Cannot execute array contains: target is null"),
        )?;
        Validate::not_null(
            elements,
            Some("Cannot execute array containsAll: elements is null"),
        )?;
        let mut remaining: HashSet<Option<T>> = elements
            .expect("validated elements")
            .iter()
            .cloned()
            .collect();
        for target_element in target.expect("validated target") {
            remaining.remove(target_element);
        }
        Ok(remaining.is_empty())
    }

    /// 使用显式运行时数组类型复制引用数组。
    ///
    /// 对应 Java: `ArrayUtils#copyOf(T[],int,Class<? extends X[]>)`。
    ///
    /// # 参数
    /// - `original`：可空源引用数组；
    /// - `new_length`：请求的新长度；
    /// - `new_type`：可空目标运行时数组类型。
    ///
    /// # 返回
    /// 独立的目标类型数组。
    pub fn copy_of_with_type<T>(
        original: Option<&JavaObjectArray<T>>,
        new_length: i32,
        new_type: Option<&JavaArrayType<T>>,
    ) -> Result<JavaObjectArray<T>, ArrayUtilsError>
    where
        T: Clone,
    {
        let new_type = new_type.ok_or(ArrayUtilsError::NullPointer)?;
        if new_length < 0 {
            return Err(ArrayUtilsError::NegativeArraySize { length: new_length });
        }
        let mut elements = vec![None; new_length as usize];
        let original = original.ok_or(ArrayUtilsError::NullPointer)?;
        let copy_length = original.len().min(elements.len());
        for (index, element) in original.as_slice()[..copy_length].iter().enumerate() {
            if element
                .as_ref()
                .is_some_and(|value| !(new_type.component_predicate)(value))
            {
                return Err(ArrayUtilsError::ArrayStore {
                    component_class_name: new_type.component_class_name.clone(),
                });
            }
            elements[index] = element.clone();
        }
        Ok(JavaObjectArray::from_parts(
            new_type.component_class_name.clone(),
            elements,
            Arc::clone(&new_type.component_predicate),
        ))
    }

    /// 使用原数组运行时类型复制引用数组。
    ///
    /// 对应 Java: `ArrayUtils#copyOf(T[],int)`。
    ///
    /// # 参数
    /// - `original`：可空源引用数组；
    /// - `new_length`：请求的新长度。
    ///
    /// # 返回
    /// 独立且保留源运行时组件类的数组。
    pub fn copy_of<T>(
        original: Option<&JavaObjectArray<T>>,
        new_length: i32,
    ) -> Result<JavaObjectArray<T>, ArrayUtilsError>
    where
        T: Clone,
    {
        let original = original.ok_or(ArrayUtilsError::NullPointer)?;
        if new_length < 0 {
            return Err(ArrayUtilsError::NegativeArraySize { length: new_length });
        }
        let mut elements = vec![None; new_length as usize];
        let copy_length = original.len().min(elements.len());
        elements[..copy_length].clone_from_slice(&original.as_slice()[..copy_length]);
        Ok(JavaObjectArray::from_parts(
            original.component_class_name().to_owned(),
            elements,
            original.component_predicate(),
        ))
    }

    /// 复制 Java `char[]`，扩展位置以 `\0` 填充。
    ///
    /// 对应 Java: `ArrayUtils#copyOf(char[],int)`；Rust 用 `u16` 保留 Java
    /// UTF-16 code unit。
    ///
    /// # 参数
    /// - `original`：可空 Java char 序列；
    /// - `new_length`：请求的新长度。
    ///
    /// # 返回
    /// 截断或以零扩展的独立 UTF-16 code unit 向量。
    pub fn copy_of_chars(
        original: Option<&[u16]>,
        new_length: i32,
    ) -> Result<Vec<u16>, ArrayUtilsError> {
        if new_length < 0 {
            return Err(ArrayUtilsError::NegativeArraySize { length: new_length });
        }
        let mut copy = vec![0; new_length as usize];
        let original = original.ok_or(ArrayUtilsError::NullPointer)?;
        let copy_length = original.len().min(copy.len());
        copy[..copy_length].copy_from_slice(&original[..copy_length]);
        Ok(copy)
    }

    /// 复制 Java `char[]` 的半开区间，并在 `to` 超过源长度时以零扩展。
    ///
    /// 对应 Java: `ArrayUtils#copyOfRange(char[],int,int)`。
    ///
    /// # 参数
    /// - `original`：可空 Java char 序列；
    /// - `from`：包含的起始索引；
    /// - `to`：不包含的结束索引。
    ///
    /// # 返回
    /// 指定半开区间的独立 UTF-16 code unit 向量。
    pub fn copy_of_range(
        original: Option<&[u16]>,
        from: i32,
        to: i32,
    ) -> Result<Vec<u16>, ArrayUtilsError> {
        let new_length = to.wrapping_sub(from);
        if new_length < 0 {
            return Err(ArrayUtilsError::InvalidRange { from, to });
        }
        let mut copy = vec![0; new_length as usize];
        let original = original.ok_or(ArrayUtilsError::NullPointer)?;
        let available = i64::try_from(original.len()).unwrap_or(i64::MAX) - i64::from(from);
        let copy_length = available.min(i64::from(new_length));
        if from < 0 {
            return Err(ArrayUtilsError::ArrayIndexOutOfBounds {
                message: format!(
                    "arraycopy: source index {from} out of bounds for char[{}]",
                    original.len()
                ),
            });
        }
        if copy_length < 0 {
            return Err(ArrayUtilsError::ArrayIndexOutOfBounds {
                message: format!("arraycopy: length {copy_length} is negative"),
            });
        }
        let from = from as usize;
        let copy_length = copy_length as usize;
        if copy_length > 0 {
            copy[..copy_length].copy_from_slice(&original[from..from + copy_length]);
        }
        Ok(copy)
    }

    fn convert<'a, T>(
        component_class_name: Option<&'static str>,
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        let target = target.ok_or_else(|| ArrayUtilsError::CannotConvert {
            message: "Cannot convert null to array".to_owned(),
        })?;
        match target {
            ArrayTarget::Reference(array) => {
                if component_class_name.is_none()
                    || component_class_name == Some(array.component_class_name())
                {
                    return Ok(JavaArray::Borrowed(array));
                }
                Err(Self::incompatible_array(
                    array.component_class_name(),
                    component_class_name,
                ))
            }
            ArrayTarget::PrimitiveArray {
                class_name,
                component_class_name: primitive_component,
            } => {
                if component_class_name.is_none() {
                    Err(ArrayUtilsError::ClassCast {
                        class_name: class_name.to_owned(),
                    })
                } else {
                    Err(Self::incompatible_array(
                        primitive_component,
                        component_class_name,
                    ))
                }
            }
            ArrayTarget::Iterable(elements) => {
                let computed = component_class_name.map_or_else(
                    || {
                        let mut computed: Option<&str> = None;
                        for element in elements.iter().flatten() {
                            computed = match computed {
                                None => Some(element.java_class_name()),
                                Some("java.lang.Object") => Some("java.lang.Object"),
                                Some(current) if current == element.java_class_name() => {
                                    Some(current)
                                }
                                Some(_) => Some("java.lang.Object"),
                            };
                        }
                        computed.unwrap_or("java.lang.Object").to_owned()
                    },
                    str::to_owned,
                );
                let predicate_component = computed.clone();
                let predicate: Arc<ArrayPredicate<T>> =
                    Arc::new(move |value| value.is_instance_of(&predicate_component));
                if let Some(value) = elements.iter().flatten().find(|value| !predicate(value)) {
                    let _ = value;
                    return Err(ArrayUtilsError::ArrayStore {
                        component_class_name: computed,
                    });
                }
                Ok(JavaArray::Owned(JavaObjectArray::from_parts(
                    computed,
                    elements.to_vec(),
                    predicate,
                )))
            }
            ArrayTarget::Other { class_name } => Err(ArrayUtilsError::CannotConvert {
                message: format!(
                    "Cannot convert object of class \"{class_name}\" to an array{}",
                    component_class_name.map_or("", |_| " of Class")
                ),
            }),
        }
    }

    fn incompatible_array(
        component_class_name: &str,
        requested_component_class_name: Option<&str>,
    ) -> ArrayUtilsError {
        ArrayUtilsError::CannotConvert {
            message: format!(
                "Cannot convert object of class \"{component_class_name}[]\" to an array{}",
                requested_component_class_name.map_or("", |_| " of Class")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ArrayTarget, ArrayUtils, ArrayUtilsError, JavaArrayElement, JavaArrayType};
    use crate::expression::JavaObjectArray;

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    enum Value {
        Text(String),
        Number(i32),
    }

    impl JavaArrayElement for Value {
        fn java_class_name(&self) -> &str {
            match self {
                Self::Text(_) => "java.lang.String",
                Self::Number(_) => "java.lang.Integer",
            }
        }
    }

    #[test]
    fn conversion_preserves_identity_and_infers_exact_component_class() {
        let source = JavaObjectArray::typed(
            "java.lang.String",
            vec![Some(Value::Text("one".to_owned()))],
            |value| matches!(value, Value::Text(_)),
        )
        .expect("source");
        let result = ArrayUtils::to_array(Some(ArrayTarget::Reference(&source))).expect("array");
        assert!(result.is_same_reference(&source));

        let values = [
            Some(Value::Text("one".to_owned())),
            None,
            Some(Value::Text("two".to_owned())),
        ];
        let result = ArrayUtils::to_array(Some(ArrayTarget::Iterable(&values))).expect("iterable");
        assert_eq!(result.as_array().component_class_name(), "java.lang.String");
        assert_eq!(result.as_array().as_slice(), &values);
    }

    #[test]
    fn typed_conversion_and_primitive_errors_keep_java_categories() {
        let mixed = [
            Some(Value::Text("one".to_owned())),
            Some(Value::Number(2)),
            Some(Value::Text("two".to_owned())),
        ];
        let inferred =
            ArrayUtils::to_array(Some(ArrayTarget::Iterable(&mixed))).expect("object array");
        assert_eq!(
            inferred.as_array().component_class_name(),
            "java.lang.Object"
        );
        let error = ArrayUtils::to_string_array(Some(ArrayTarget::Iterable(&mixed)))
            .expect_err("array store");
        assert_eq!(error.java_class_name(), "java.lang.ArrayStoreException");

        let error = ArrayUtils::to_array::<Value>(Some(ArrayTarget::PrimitiveArray {
            class_name: "[I",
            component_class_name: "int",
        }))
        .expect_err("class cast");
        assert_eq!(error.java_class_name(), "java.lang.ClassCastException");
    }

    #[test]
    fn query_and_copy_contracts_are_preserved() {
        let values = [Some("one".to_owned()), None, Some("two".to_owned())];
        assert_eq!(ArrayUtils::length(Some(&values)).expect("length"), 3);
        assert!(ArrayUtils::contains(Some(&values), &None).expect("contains"));
        assert!(
            ArrayUtils::contains_all_array(Some(&values), Some(&[Some("one".to_owned()), None]))
                .expect("all")
        );

        let source =
            JavaObjectArray::typed("java.lang.String", values.to_vec(), |_| true).expect("source");
        let copied = ArrayUtils::copy_of(Some(&source), 5).expect("copy");
        assert_eq!(copied.len(), 5);
        assert_eq!(copied.component_class_name(), "java.lang.String");
        let integer_type = JavaArrayType::typed("java.lang.Integer", |_: &String| false);
        assert_eq!(
            ArrayUtils::copy_of_with_type(Some(&source), 1, Some(&integer_type))
                .expect_err("store")
                .java_class_name(),
            "java.lang.ArrayStoreException"
        );
    }

    #[test]
    fn char_copy_order_and_range_failures_match_java() {
        assert_eq!(
            ArrayUtils::copy_of_chars(Some(&[97, 0, 122]), 5).expect("copy"),
            vec![97, 0, 122, 0, 0]
        );
        assert_eq!(
            ArrayUtils::copy_of_range(Some(&[97, 98, 99, 100]), 2, 6).expect("range"),
            vec![99, 100, 0, 0]
        );
        assert_eq!(
            ArrayUtils::copy_of_range(Some(&[1]), 3, 1).expect_err("range"),
            ArrayUtilsError::InvalidRange { from: 3, to: 1 }
        );
        assert_eq!(
            ArrayUtils::copy_of_chars(None, -1).expect_err("negative"),
            ArrayUtilsError::NegativeArraySize { length: -1 }
        );
    }

    #[test]
    fn runtime_adapters_cover_builtin_classes_types_and_owned_results() {
        assert_eq!(String::new().java_class_name(), "java.lang.String");
        assert_eq!(0_i32.java_class_name(), "java.lang.Integer");
        assert_eq!(0_i64.java_class_name(), "java.lang.Long");
        assert_eq!(0_f64.java_class_name(), "java.lang.Double");
        assert_eq!(0_f32.java_class_name(), "java.lang.Float");
        assert_eq!(false.java_class_name(), "java.lang.Boolean");
        assert!("text".to_owned().is_instance_of("java.lang.Object"));
        assert!(!0_i32.is_instance_of("java.lang.String"));

        let array_type = JavaArrayType::<String>::object();
        assert_eq!(array_type.component_class_name(), "java.lang.Object");
        assert_eq!(
            format!("{array_type:?}"),
            "JavaArrayType { component_class_name: \"java.lang.Object\", .. }"
        );

        let source = JavaObjectArray::object(vec![Some("one".to_owned())]);
        let borrowed = ArrayUtils::to_array(Some(ArrayTarget::Reference(&source)))
            .expect("borrowed")
            .into_owned();
        assert_eq!(borrowed.as_slice(), source.as_slice());
        let values = [Some("one".to_owned())];
        let owned = ArrayUtils::to_array(Some(ArrayTarget::Iterable(&values)))
            .expect("owned")
            .into_owned();
        assert_eq!(owned.as_slice(), &values);

        assert_eq!(
            ArrayUtils::copy_of_with_type(Some(&source), -1, Some(&array_type))
                .expect_err("negative"),
            ArrayUtilsError::NegativeArraySize { length: -1 }
        );
        assert_eq!(
            ArrayUtils::copy_of_with_type(None, 1, Some(&array_type)).expect_err("null original"),
            ArrayUtilsError::NullPointer
        );
    }
}
