use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::sync::Arc;

use indexmap::IndexSet;
use thiserror::Error;

use crate::util::{ListView, ObjectUtils, SetView, Validate, ValidateError};

type ComponentPredicate<T> = dyn Fn(&T) -> bool + Send + Sync;

/// `Objects` 数组操作使用的 Java 引用数组适配值。
///
/// 对应 Java: `T[]`，由 `org.thymeleaf.expression.Objects#arrayNullSafe` 接收和返回。
///
/// Java 数组在运行时保存组件类，并在每次写入时执行赋值兼容性检查。Rust 泛型本身
/// 无法表达 `Number[]` 接受 `Integer`、而 `String[]` 拒绝 `Integer` 的 JVM
/// 规则，因此本适配值同时保存组件类名和调用方提供的运行时兼容性谓词。
#[derive(Clone)]
pub struct JavaObjectArray<T> {
    elements: Vec<Option<T>>,
    component_class_name: String,
    component_predicate: Arc<ComponentPredicate<T>>,
}

impl<T> JavaObjectArray<T> {
    /// 对应 Java 语义：`Objects` 的 `from_parts` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn from_parts(
        component_class_name: String,
        elements: Vec<Option<T>>,
        component_predicate: Arc<ComponentPredicate<T>>,
    ) -> Self {
        Self {
            elements,
            component_class_name,
            component_predicate,
        }
    }

    /// 对应 Java 语义：`Objects` 的 `component_predicate` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn component_predicate(&self) -> Arc<ComponentPredicate<T>> {
        Arc::clone(&self.component_predicate)
    }

    /// 创建具有 JVM 运行时组件类型约束的引用数组。
    ///
    /// # 参数
    /// - `component_class_name`：Java `Class#getName()` 形式的组件类名；
    /// - `elements`：数组当前元素，`None` 对应 Java null；
    /// - `component_predicate`：判断非 null 元素能否写入该组件类型。
    ///
    /// # 返回
    /// 保存运行时数组类型和元素的新数组。
    ///
    /// # 错误
    /// 任一已有元素不兼容时返回 Java `ArrayStoreException` 等价错误。
    pub fn typed(
        component_class_name: impl Into<String>,
        elements: Vec<Option<T>>,
        component_predicate: impl Fn(&T) -> bool + Send + Sync + 'static,
    ) -> Result<Self, ObjectsError> {
        let component_class_name = component_class_name.into();
        let component_predicate: Arc<ComponentPredicate<T>> = Arc::new(component_predicate);
        if elements
            .iter()
            .flatten()
            .any(|element| !component_predicate(element))
        {
            return Err(ObjectsError::ArrayStore {
                component_class_name,
            });
        }
        Ok(Self {
            elements,
            component_class_name,
            component_predicate,
        })
    }

    /// 创建组件类型为 `java.lang.Object` 的引用数组。
    ///
    /// # 参数
    /// - `elements`：数组元素，`None` 对应 Java null。
    ///
    /// # 返回
    /// 接受任意 `T` 值的对象数组。
    #[must_use]
    pub fn object(elements: Vec<Option<T>>) -> Self {
        Self {
            elements,
            component_class_name: "java.lang.Object".to_owned(),
            component_predicate: Arc::new(|_| true),
        }
    }

    /// 返回 Java 运行时组件类名。
    ///
    /// # 返回
    /// 创建数组时保存的 `Class#getName()` 等价值。
    #[must_use]
    /// 对应 Java 语义：`Objects` 的 `component_class_name` 行为（Rust 侧辅助/私有路径）。
    pub fn component_class_name(&self) -> &str {
        &self.component_class_name
    }

    /// 返回数组长度。
    ///
    /// # 返回
    /// 当前元素槽位数。
    #[must_use]
    /// 对应 Java 语义：`Objects` 的 `len` 行为（Rust 侧辅助/私有路径）。
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// 判断数组是否为空。
    ///
    /// # 返回
    /// 数组长度为零时返回 `true`。
    #[must_use]
    /// 对应 Java 语义：Java 接口/超类方法 `isEmpty()` 的 Rust 移植（`Objects` 继承路径）。
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// 返回数组元素切片。
    ///
    /// # 返回
    /// 按索引排列的只读元素。
    #[must_use]
    /// 对应 Java 语义：`Objects` 的 `as_slice` 行为（Rust 侧辅助/私有路径）。
    pub fn as_slice(&self) -> &[Option<T>] {
        &self.elements
    }

    /// 按 Java 数组写入规则修改指定槽位。
    ///
    /// # 参数
    /// - `index`：从零开始的数组索引；
    /// - `value`：新元素；`None` 对应 Java null。
    ///
    /// # 错误
    /// 索引越界时返回 `ArrayIndexOutOfBoundsException` 等价错误；非 null 值与运行时
    /// 组件类型不兼容时返回 `ArrayStoreException` 等价错误。
    pub fn set(&mut self, index: usize, value: Option<T>) -> Result<(), ObjectsError> {
        if index >= self.elements.len() {
            return Err(ObjectsError::ArrayIndexOutOfBounds {
                index,
                length: self.elements.len(),
            });
        }
        if value
            .as_ref()
            .is_some_and(|value| !(self.component_predicate)(value))
        {
            return Err(ObjectsError::ArrayStore {
                component_class_name: self.component_class_name.clone(),
            });
        }
        self.elements[index] = value;
        Ok(())
    }
}

impl<T: Debug> Debug for JavaObjectArray<T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("JavaObjectArray")
            .field("elements", &self.elements)
            .field("component_class_name", &self.component_class_name)
            .finish_non_exhaustive()
    }
}

/// `Objects` 表达式对象的类型化错误。
///
/// 对应 Java: `org.thymeleaf.expression.Objects` 可抛出的参数错误以及 JVM
/// 引用数组写入错误。
#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum ObjectsError {
    /// Java `Validate.notNull` 对应的参数错误。
    #[error(transparent)]
    Validation(#[from] ValidateError),

    /// 非兼容值写入具有具体运行时组件类的 Java 引用数组。
    #[error("value cannot be stored in array with component class \"{component_class_name}\"")]
    ArrayStore {
        /// Java `Class#getName()` 形式的数组组件类名。
        component_class_name: String,
    },

    /// Java 数组索引越界。
    #[error("index {index} out of bounds for array length {length}")]
    ArrayIndexOutOfBounds {
        /// 请求索引。
        index: usize,
        /// 数组长度。
        length: usize,
    },
}

impl ObjectsError {
    /// 返回对应的 Java 异常类名。
    ///
    /// # 返回
    /// `IllegalArgumentException`、`ArrayStoreException` 或
    /// `ArrayIndexOutOfBoundsException`。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::Validation(error) => error.java_class_name(),
            Self::ArrayStore { .. } => "java.lang.ArrayStoreException",
            Self::ArrayIndexOutOfBounds { .. } => "java.lang.ArrayIndexOutOfBoundsException",
        }
    }
}

/// Thymeleaf 标准表达式中的 null 安全对象。
///
/// 对应 Java: `org.thymeleaf.expression.Objects`。
///
/// 该无状态对象通常以 `#objects` 暴露。标量操作委托 [`ObjectUtils`]；数组、
/// 列表和集合操作创建独立可变结果，并保留 Java 数组组件检查、列表顺序以及
/// `LinkedHashSet` 首次插入顺序和去重语义。
#[derive(Debug, Default, Clone, Copy)]
pub struct Objects;

impl Objects {
    /// 创建无状态对象表达式实例。
    ///
    /// 对应 Java: `Objects#Objects()`。
    ///
    /// # 返回
    /// 新的 `#objects` 表达式对象。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 目标非 null 时返回目标，否则返回默认值。
    ///
    /// 对应 Java: `Objects#nullSafe(Object,Object)`。
    ///
    /// # 参数
    /// - `target`：首选对象；
    /// - `default_value`：target 为 null 时的默认对象。
    ///
    /// # 返回
    /// 被选中的同一对象，不执行克隆。
    #[must_use]
    pub fn null_safe<T>(&self, target: Option<T>, default_value: Option<T>) -> Option<T> {
        ObjectUtils::null_safe(target, default_value)
    }

    /// 将数组中的每个 null 替换为默认值并返回同运行时类型的独立数组。
    ///
    /// 对应 Java: `Objects#arrayNullSafe(Object[],Object)`。
    ///
    /// # 参数
    /// - `target`：目标 Java 引用数组；
    /// - `default_value`：用于替换 null 的可空默认值。
    ///
    /// # 返回
    /// 保留组件类和长度的数组克隆，原数组不变。
    ///
    /// # 错误
    /// target 为 null 时返回消息为 `Target cannot be null` 的参数错误；仅当数组
    /// 实际含 null 且非 null 默认值与组件类不兼容时返回数组存储错误。
    pub fn array_null_safe<T>(
        &self,
        target: Option<&JavaObjectArray<T>>,
        default_value: Option<&T>,
    ) -> Result<JavaObjectArray<T>, ObjectsError>
    where
        T: Clone,
    {
        Validate::not_null(target, Some("Target cannot be null"))?;
        let target = target.expect("validated target");
        let mut result = target.clone();
        for index in 0..result.elements.len() {
            if result.elements[index].is_none() {
                result.set(index, default_value.cloned())?;
            }
        }
        Ok(result)
    }

    /// 将列表中的每个 null 替换为默认值并返回新的可变列表。
    ///
    /// 对应 Java: `Objects#listNullSafe(List,Object)`。
    ///
    /// # 参数
    /// - `target`：目标列表；
    /// - `default_value`：用于替换 null 的可空默认值。
    ///
    /// # 返回
    /// Java `ArrayList` 等价的独立 [`Vec`]，保留顺序、重复项和长度。
    ///
    /// # 错误
    /// target 为 null 时返回消息为 `Target cannot be null` 的参数错误。
    pub fn list_null_safe<T>(
        &self,
        target: Option<&dyn ListView<Option<T>>>,
        default_value: Option<&T>,
    ) -> Result<Vec<Option<T>>, ObjectsError>
    where
        T: Clone,
    {
        Validate::not_null(target, Some("Target cannot be null"))?;
        let target = target.expect("validated target");
        Ok(target
            .iter()
            .map(|element| ObjectUtils::null_safe(element.clone(), default_value.cloned()))
            .collect())
    }

    /// 将集合中的每个 null 替换为默认值并返回新的可变有序集合。
    ///
    /// 对应 Java: `Objects#setNullSafe(Set,Object)`。
    ///
    /// # 参数
    /// - `target`：目标集合，按其迭代顺序读取；
    /// - `default_value`：用于替换 null 的可空默认值。
    ///
    /// # 返回
    /// Java `LinkedHashSet` 等价的独立 [`IndexSet`]；替换结果相等时按首次插入
    /// 位置去重。
    ///
    /// # 错误
    /// target 为 null 时返回消息为 `Target cannot be null` 的参数错误。
    pub fn set_null_safe<T>(
        &self,
        target: Option<&dyn SetView<Option<T>>>,
        default_value: Option<&T>,
    ) -> Result<IndexSet<Option<T>>, ObjectsError>
    where
        T: Clone + Eq + Hash,
    {
        Validate::not_null(target, Some("Target cannot be null"))?;
        let target = target.expect("validated target");
        Ok(target
            .iter()
            .map(|element| ObjectUtils::null_safe(element.clone(), default_value.cloned()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use indexmap::IndexSet;

    use super::{JavaObjectArray, Objects, ObjectsError};
    use crate::util::{ListView, SetView, ValidateError};

    #[test]
    fn scalar_selection_preserves_selected_identity() {
        let objects = Objects::new();
        let target = Rc::new("target".to_owned());
        let default_value = Rc::new("default".to_owned());
        let selected = objects
            .null_safe(Some(Rc::clone(&target)), Some(Rc::clone(&default_value)))
            .expect("selected");
        assert!(Rc::ptr_eq(&selected, &target));
    }

    #[test]
    fn array_result_is_independent_and_preserves_runtime_component_type() {
        let objects = Objects;
        let source = JavaObjectArray::typed(
            "java.lang.Number",
            vec![Some(1_i32), None, Some(3_i32)],
            |_| true,
        )
        .expect("source");
        assert_eq!(
            format!("{source:?}"),
            "JavaObjectArray { elements: [Some(1), None, Some(3)], component_class_name: \
             \"java.lang.Number\", .. }"
        );
        let mut result = objects
            .array_null_safe(Some(&source), Some(&2))
            .expect("result");

        assert_eq!(result.component_class_name(), "java.lang.Number");
        assert_eq!(result.as_slice(), &[Some(1), Some(2), Some(3)]);
        assert_eq!(source.as_slice(), &[Some(1), None, Some(3)]);
        result.set(0, Some(9)).expect("mutable result");
        assert_eq!(source.as_slice()[0], Some(1));
    }

    #[test]
    fn array_store_check_runs_only_when_a_null_slot_is_replaced() {
        #[derive(Clone, Debug, Eq, PartialEq)]
        enum Value {
            Text(&'static str),
            Number(i32),
        }

        let accepts_text = |value: &Value| matches!(value, Value::Text(_));
        let with_null = JavaObjectArray::typed(
            "java.lang.String",
            vec![Some(Value::Text("one")), None],
            accepts_text,
        )
        .expect("source");
        let error = Objects
            .array_null_safe(Some(&with_null), Some(&Value::Number(2)))
            .expect_err("array store");
        assert_eq!(
            error,
            ObjectsError::ArrayStore {
                component_class_name: "java.lang.String".to_owned()
            }
        );
        assert_eq!(
            error.to_string(),
            "value cannot be stored in array with component class \"java.lang.String\""
        );
        assert_eq!(error.java_class_name(), "java.lang.ArrayStoreException");

        let without_null = JavaObjectArray::typed(
            "java.lang.String",
            vec![Some(Value::Text("one"))],
            accepts_text,
        )
        .expect("source");
        assert!(
            Objects
                .array_null_safe(Some(&without_null), Some(&Value::Number(2)))
                .is_ok()
        );
    }

    #[test]
    fn array_adapter_enforces_store_and_index_contracts() {
        let invalid =
            JavaObjectArray::typed("positive.Integer", vec![Some(-1)], |value| *value > 0)
                .expect_err("invalid source");
        assert_eq!(invalid.java_class_name(), "java.lang.ArrayStoreException");

        let mut target = JavaObjectArray::object(vec![Some("one")]);
        let error = target.set(1, Some("two")).expect_err("bounds");
        assert_eq!(
            error,
            ObjectsError::ArrayIndexOutOfBounds {
                index: 1,
                length: 1
            }
        );
        assert_eq!(
            error.to_string(),
            "index 1 out of bounds for array length 1"
        );
        assert_eq!(
            error.java_class_name(),
            "java.lang.ArrayIndexOutOfBoundsException"
        );
        assert_eq!(target.len(), 1);
        assert!(!target.is_empty());
    }

    #[test]
    fn list_result_is_mutable_ordered_and_independent() {
        let source = vec![Some("one".to_owned()), None, Some("one".to_owned())];
        let view: &dyn ListView<Option<String>> = &source;
        let mut result = Objects
            .list_null_safe(Some(view), Some(&"default".to_owned()))
            .expect("result");
        result.push(Some("tail".to_owned()));

        assert_eq!(
            result,
            vec![
                Some("one".to_owned()),
                Some("default".to_owned()),
                Some("one".to_owned()),
                Some("tail".to_owned())
            ]
        );
        assert_eq!(source[1], None);
    }

    #[test]
    fn set_result_preserves_first_order_and_deduplicates_replacement() {
        let source = IndexSet::from([Some("default".to_owned()), None, Some("other".to_owned())]);
        let view: &dyn SetView<Option<String>> = &source;
        let mut result = Objects
            .set_null_safe(Some(view), Some(&"default".to_owned()))
            .expect("result");
        result.insert(Some("tail".to_owned()));

        assert_eq!(
            result.into_iter().collect::<Vec<_>>(),
            vec![
                Some("default".to_owned()),
                Some("other".to_owned()),
                Some("tail".to_owned())
            ]
        );
        assert!(source.contains(&None));
    }

    #[test]
    fn collection_and_array_null_targets_use_exact_validation_message() {
        let array_error = Objects
            .array_null_safe::<String>(None, None)
            .expect_err("array null");
        assert_eq!(
            array_error,
            ObjectsError::Validation(ValidateError::IllegalArgument {
                message: Some("Target cannot be null".to_owned())
            })
        );
        assert_eq!(
            array_error.java_class_name(),
            "java.lang.IllegalArgumentException"
        );
        assert_eq!(array_error.to_string(), "Target cannot be null");
        assert!(Objects.list_null_safe::<String>(None, None).is_err());
        assert!(Objects.set_null_safe::<String>(None, None).is_err());
    }
}
