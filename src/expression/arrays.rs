use std::hash::Hash;

use crate::util::{ArrayTarget, ArrayUtils, ArrayUtilsError, JavaArray, JavaArrayElement};

/// Thymeleaf 标准表达式中的数组工具对象。
///
/// 对应 Java: `org.thymeleaf.expression.Arrays`。
///
/// 该无状态对象通常以 `#arrays` 暴露，所有行为均委托 [`ArrayUtils`]，包括
/// 引用数组身份、Iterable 组件类推断、primitive array 异常和两个
/// `containsAll` 重载的不同校验文本。
#[derive(Debug, Default, Clone, Copy)]
pub struct Arrays;

impl Arrays {
    /// 创建无状态数组表达式对象。对应 Java: `Arrays#Arrays()`。
    ///
    /// # 返回
    /// 新的 `#arrays` 表达式对象。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 转换为运行时推断组件类的数组。对应 Java: `Arrays#toArray(Object)`。
    ///
    /// # 参数
    /// - `target`：Java 引用数组、primitive 数组、Iterable 或其他对象。
    ///
    /// # 返回
    /// 原引用数组或按 Iterable 新建的运行时类型数组。
    pub fn to_array<'a, T>(
        &self,
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        ArrayUtils::to_array(target)
    }

    /// 转换为 `String[]`。对应 Java: `Arrays#toStringArray(Object)`。
    ///
    /// # 参数
    /// - `target`：待转换的 Java 运行时目标。
    ///
    /// # 返回
    /// 组件类为 `java.lang.String` 的数组。
    pub fn to_string_array<'a, T>(
        &self,
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        ArrayUtils::to_string_array(target)
    }

    /// 转换为 `Integer[]`。对应 Java: `Arrays#toIntegerArray(Object)`。
    ///
    /// # 参数
    /// - `target`：待转换的 Java 运行时目标。
    ///
    /// # 返回
    /// 组件类为 `java.lang.Integer` 的数组。
    pub fn to_integer_array<'a, T>(
        &self,
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        ArrayUtils::to_integer_array(target)
    }

    /// 转换为 `Long[]`。对应 Java: `Arrays#toLongArray(Object)`。
    ///
    /// # 参数
    /// - `target`：待转换的 Java 运行时目标。
    ///
    /// # 返回
    /// 组件类为 `java.lang.Long` 的数组。
    pub fn to_long_array<'a, T>(
        &self,
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        ArrayUtils::to_long_array(target)
    }

    /// 转换为 `Double[]`。对应 Java: `Arrays#toDoubleArray(Object)`。
    ///
    /// # 参数
    /// - `target`：待转换的 Java 运行时目标。
    ///
    /// # 返回
    /// 组件类为 `java.lang.Double` 的数组。
    pub fn to_double_array<'a, T>(
        &self,
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        ArrayUtils::to_double_array(target)
    }

    /// 转换为 `Float[]`。对应 Java: `Arrays#toFloatArray(Object)`。
    ///
    /// # 参数
    /// - `target`：待转换的 Java 运行时目标。
    ///
    /// # 返回
    /// 组件类为 `java.lang.Float` 的数组。
    pub fn to_float_array<'a, T>(
        &self,
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        ArrayUtils::to_float_array(target)
    }

    /// 转换为 `Boolean[]`。对应 Java: `Arrays#toBooleanArray(Object)`。
    ///
    /// # 参数
    /// - `target`：待转换的 Java 运行时目标。
    ///
    /// # 返回
    /// 组件类为 `java.lang.Boolean` 的数组。
    pub fn to_boolean_array<'a, T>(
        &self,
        target: Option<ArrayTarget<'a, T>>,
    ) -> Result<JavaArray<'a, T>, ArrayUtilsError>
    where
        T: Clone + JavaArrayElement + 'static,
    {
        ArrayUtils::to_boolean_array(target)
    }

    /// 返回数组长度。对应 Java: `Arrays#length(Object[])`。
    ///
    /// # 参数
    /// - `target`：可空引用数组。
    ///
    /// # 返回
    /// Java `int` 范围的数组长度。
    pub fn length<T>(&self, target: Option<&[Option<T>]>) -> Result<i32, ArrayUtilsError> {
        ArrayUtils::length(target)
    }

    /// 判断数组为 null 或空。对应 Java: `Arrays#isEmpty(Object[])`。
    ///
    /// # 参数
    /// - `target`：可空引用数组。
    ///
    /// # 返回
    /// target 为 null 或零长度时返回 `true`。
    #[must_use]
    pub fn is_empty<T>(&self, target: Option<&[Option<T>]>) -> bool {
        ArrayUtils::is_empty(target)
    }

    /// 判断数组是否包含元素。对应 Java: `Arrays#contains(Object[],Object)`。
    ///
    /// # 参数
    /// - `target`：可空目标数组；
    /// - `element`：待查询的可空元素。
    ///
    /// # 返回
    /// 存在相等元素时返回 `true`。
    pub fn contains<T>(
        &self,
        target: Option<&[Option<T>]>,
        element: &Option<T>,
    ) -> Result<bool, ArrayUtilsError>
    where
        T: PartialEq,
    {
        ArrayUtils::contains(target, element)
    }

    /// 判断数组是否包含另一个数组的全部不同元素。
    ///
    /// 对应 Java: `Arrays#containsAll(Object[],Object[])`。
    ///
    /// # 参数
    /// - `target`：可空目标数组；
    /// - `elements`：可空请求数组。
    ///
    /// # 返回
    /// 全部不同请求元素都存在时返回 `true`。
    pub fn contains_all_array<T>(
        &self,
        target: Option<&[Option<T>]>,
        elements: Option<&[Option<T>]>,
    ) -> Result<bool, ArrayUtilsError>
    where
        T: Clone + Eq + Hash,
    {
        ArrayUtils::contains_all_array(target, elements)
    }

    /// 判断数组是否包含 Collection 的全部不同元素。
    ///
    /// 对应 Java: `Arrays#containsAll(Object[],Collection)`。
    ///
    /// # 参数
    /// - `target`：可空目标数组；
    /// - `elements`：按 Java Collection 语义提供的可空元素序列。
    ///
    /// # 返回
    /// 全部不同请求元素都存在时返回 `true`。
    pub fn contains_all_collection<T>(
        &self,
        target: Option<&[Option<T>]>,
        elements: Option<&[Option<T>]>,
    ) -> Result<bool, ArrayUtilsError>
    where
        T: Clone + Eq + Hash,
    {
        ArrayUtils::contains_all_collection(target, elements)
    }
}

#[cfg(test)]
mod tests {
    use super::Arrays;
    use crate::util::{ArrayTarget, JavaArrayElement};

    #[derive(Clone)]
    struct Text(String);

    impl JavaArrayElement for Text {
        fn java_class_name(&self) -> &str {
            "java.lang.String"
        }
    }

    #[test]
    fn facade_delegates_conversion_and_queries() {
        let values = [Some(Text("one".to_owned())), None];
        let result = Arrays::new()
            .to_string_array(Some(ArrayTarget::Iterable(&values)))
            .expect("array");
        assert_eq!(result.as_array().component_class_name(), "java.lang.String");
        assert_eq!(
            result.as_array().as_slice()[0]
                .as_ref()
                .map(|v| v.0.as_str()),
            Some("one")
        );

        let strings = [Some("one".to_owned()), None];
        assert_eq!(Arrays.length(Some(&strings)).expect("length"), 2);
        assert!(Arrays.contains(Some(&strings), &None).expect("contains"));
        assert!(
            Arrays
                .contains_all_collection(Some(&strings), Some(&[None]))
                .expect("all")
        );
    }
}
