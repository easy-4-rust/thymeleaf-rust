use std::cmp::Ordering;
use std::collections::{LinkedList, VecDeque};
use std::ptr;

use thiserror::Error;

use super::{Validate, ValidateError};

/// Java 列表的运行时实现类别。
///
/// `ListUtils#sort` 会反射调用输入列表类型的公开无参构造器；构造失败时才回退
/// `ArrayList`。Rust 使用该值显式保存这项 JVM 运行时信息。
///
/// 对应 Java: `org.thymeleaf.util.ListUtils#fillNewList` 中的 `Class<? extends List>`。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ListTypeValue {
    /// `java.util.ArrayList`。
    ArrayList,
    /// `java.util.LinkedList`。
    LinkedList,
    /// 其他 Java `List` 实现。
    Custom {
        /// Java `Class#getName()` 形式的类型名。
        class_name: String,
        /// 是否能通过公开无参构造器创建新实例。
        public_no_arg_constructor: bool,
    },
}

impl ListTypeValue {
    /// 创建自定义 Java 列表类型描述。
    ///
    /// # 参数
    /// - `class_name`：Java 运行时类名；
    /// - `public_no_arg_constructor`：公开无参构造器是否可用。
    ///
    /// # 返回
    /// 保存反射构造信息的列表类型。
    /// 对应 Java 语义：`ListUtils` 的 `custom` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn custom(class_name: impl Into<String>, public_no_arg_constructor: bool) -> Self {
        Self::Custom {
            class_name: class_name.into(),
            public_no_arg_constructor,
        }
    }

    /// 返回 Java 运行时类名。
    ///
    /// # 返回
    /// 与 `Class#getName()` 对应的类名。
    /// 对应 Java 语义：`ListUtils` 的 `class_name` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn class_name(&self) -> &str {
        match self {
            Self::ArrayList => "java.util.ArrayList",
            Self::LinkedList => "java.util.LinkedList",
            Self::Custom { class_name, .. } => class_name,
        }
    }

    fn sorted_result_type(&self) -> Self {
        match self {
            Self::ArrayList => Self::ArrayList,
            Self::LinkedList => Self::LinkedList,
            Self::Custom {
                class_name,
                public_no_arg_constructor: true,
            } => Self::custom(class_name, true),
            Self::Custom {
                public_no_arg_constructor: false,
                ..
            } => Self::ArrayList,
        }
    }
}

/// `ListUtils` 的类型化错误。
///
/// 对应 Java: `org.thymeleaf.util.ListUtils` 抛出的参数、转换、排序和列表运行时异常。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ListUtilsError {
    /// Java `Validate.notNull` 对应的参数错误。
    #[error(transparent)]
    Validation(#[from] ValidateError),

    /// 输入不能转换为 Java `List`。
    #[error("Cannot convert object of class \"{class_name}\" to a list")]
    CannotConvert {
        /// Java `Class#getName()` 形式的类型名。
        class_name: String,
    },

    /// primitive array 无法强制转换成 Java `Object[]`。
    #[error("class \"{class_name}\" cannot be cast to class \"[Ljava.lang.Object;\"")]
    ClassCast {
        /// Java primitive array 的运行时类名，例如 `[I`。
        class_name: String,
    },

    /// 自然排序遇到 Java null 元素。
    #[error("natural ordering cannot compare null")]
    NaturalOrderingNull,

    /// 自然排序遇到不可相互比较的运行时类型。
    #[error("class \"{left_class}\" cannot be compared to class \"{right_class}\"")]
    NaturalOrderingClassCast {
        /// 左操作数 Java 类名。
        left_class: String,
        /// 右操作数 Java 类名。
        right_class: String,
    },

    /// Comparator 或列表实现抛出的 Java 运行时异常。
    #[error("{class_name}:{message}")]
    Runtime {
        /// Java 异常类名。
        class_name: String,
        /// Java 异常消息。
        message: String,
    },
}

impl ListUtilsError {
    /// 创建 Comparator/列表实现对应的运行时异常。
    ///
    /// # 参数
    /// - `class_name`：Java 异常类名；
    /// - `message`：Java 异常消息。
    ///
    /// # 返回
    /// 可在稳定排序过程中传播的类型化错误。
    /// 对应 Java 语义：`ListUtils` 的 `runtime` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn runtime(class_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Runtime {
            class_name: class_name.into(),
            message: message.into(),
        }
    }
}

/// Java `Comparable` 的 Rust 等价契约。
///
/// `String` 实现按 UTF-16 code unit 比较，而不是 Rust Unicode scalar 顺序；
/// `Option<T>` 显式保留 Java null 在自然排序中抛 `NullPointerException` 的边界。
///
/// 对应 Java: `java.lang.Comparable`，由 `org.thymeleaf.util.ListUtils#sort(List)` 使用。
pub trait ComparableValue {
    /// 执行 Java `Comparable#compareTo`。
    ///
    /// # 参数
    /// - `other`：右操作数。
    ///
    /// # 返回
    /// Java 负数/零/正数结果归一化后的 [`Ordering`]。
    ///
    /// # 错误
    /// null、运行时类型不兼容或用户比较逻辑失败时返回类型化错误。
    fn template_compare_to(&self, other: &Self) -> Result<Ordering, ListUtilsError>;
}

macro_rules! impl_java_comparable_ord {
    ($($type:ty),+ $(,)?) => {
        $(
            impl ComparableValue for $type {
                fn template_compare_to(&self, other: &Self) -> Result<Ordering, ListUtilsError> {
                    Ok(self.cmp(other))
                }
            }
        )+
    };
}

impl_java_comparable_ord!(bool, i8, i16, i32, i64, u16);

impl ComparableValue for String {
    fn template_compare_to(&self, other: &Self) -> Result<Ordering, ListUtilsError> {
        Ok(self.encode_utf16().cmp(other.encode_utf16()))
    }
}

impl ComparableValue for f32 {
    fn template_compare_to(&self, other: &Self) -> Result<Ordering, ListUtilsError> {
        Ok(f32_compare(*self, *other))
    }
}

impl ComparableValue for f64 {
    fn template_compare_to(&self, other: &Self) -> Result<Ordering, ListUtilsError> {
        Ok(f64_compare(*self, *other))
    }
}

impl<T> ComparableValue for Option<T>
where
    T: ComparableValue,
{
    fn template_compare_to(&self, other: &Self) -> Result<Ordering, ListUtilsError> {
        match (self, other) {
            (Some(left), Some(right)) => left.template_compare_to(right),
            _ => Err(ListUtilsError::NaturalOrderingNull),
        }
    }
}

/// Java `Comparator<? super T>` 的 Rust 等价契约。
///
/// 对应 Java: `java.util.Comparator`，由
/// `org.thymeleaf.util.ListUtils#sort(List, Comparator)` 使用。
pub trait ComparatorValue<T> {
    /// 比较两个列表元素。
    ///
    /// # 参数
    /// - `left`：左元素；
    /// - `right`：右元素。
    ///
    /// # 返回
    /// Comparator 结果归一化后的 [`Ordering`]。
    ///
    /// # 错误
    /// Comparator 抛出运行时异常时返回 [`ListUtilsError`]。
    fn compare(&mut self, left: &T, right: &T) -> Result<Ordering, ListUtilsError>;
}

impl<T, F> ComparatorValue<T> for F
where
    F: FnMut(&T, &T) -> Result<Ordering, ListUtilsError>,
{
    fn compare(&mut self, left: &T, right: &T) -> Result<Ordering, ListUtilsError> {
        self(left, right)
    }
}

/// Java `List` 的只读 Rust 视图。
///
/// 视图保留元素顺序、重复项、null 等价值和运行时列表类型，并允许自定义实现覆写
/// `snapshot` 来复现 `List#toArray()` 的运行时失败。
///
/// 对应 Java: `java.util.List`，由 `org.thymeleaf.util.ListUtils` 接收。
pub trait ListView<T> {
    /// 返回列表元素数。
    ///
    /// # 返回
    /// Java `List#size()` 对应长度。
    fn len(&self) -> usize;

    /// 判断列表是否为空。
    ///
    /// # 返回
    /// 长度为零时返回 `true`。
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 返回指定位置元素。
    ///
    /// # 参数
    /// - `index`：从零开始的位置。
    ///
    /// # 返回
    /// 位置存在时返回借用元素。
    fn get(&self, index: usize) -> Option<&T>;

    /// 按列表顺序访问元素。
    ///
    /// # 返回
    /// 借用元素的只读迭代器。
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_>;

    /// 返回 Java 列表运行时类型。
    ///
    /// # 返回
    /// 用于排序结果反射构造的类型描述。
    fn list_type(&self) -> ListTypeValue;

    /// 复制 `List#toArray()` 的元素快照。
    ///
    /// # 返回
    /// 与原列表顺序一致的独立元素向量。
    ///
    /// # 错误
    /// 自定义列表的 `toArray()` 失败时可返回对应运行时错误。
    fn snapshot(&self) -> Result<Vec<T>, ListUtilsError>
    where
        T: Clone,
    {
        Ok(self.iter().cloned().collect())
    }

    /// 使用当前列表运行时类型承接已排序元素。
    ///
    /// 对应 Java: `ListUtils#fillNewList(Object[], Class<? extends List>)`。默认实现
    /// 模拟公开无参构造器；构造失败回退 `ArrayList`。自定义列表可覆写此方法，
    /// 以保留构造成功后 `List#add` 抛出的异常不会被回退逻辑吞掉这一语义。
    ///
    /// # 参数
    /// - `elements`：已经按 Java 排序规则排列的独立元素。
    ///
    /// # 返回
    /// 原类型新列表，或仅在反射构造失败时返回 ArrayList 等价值。
    ///
    /// # 错误
    /// 新列表的 `add` 操作失败时原样返回运行时错误。
    fn fill_sorted(&self, elements: Vec<T>) -> Result<ListValue<'static, T>, ListUtilsError>
    where
        T: 'static,
    {
        Ok(fill_new_list(elements, &self.list_type()))
    }
}

impl<T> ListView<T> for Vec<T> {
    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.as_slice().get(index)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.as_slice().iter())
    }

    fn list_type(&self) -> ListTypeValue {
        ListTypeValue::ArrayList
    }
}

impl<T> ListView<T> for LinkedList<T> {
    fn len(&self) -> usize {
        LinkedList::len(self)
    }

    fn get(&self, index: usize) -> Option<&T> {
        LinkedList::iter(self).nth(index)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(LinkedList::iter(self))
    }

    fn list_type(&self) -> ListTypeValue {
        ListTypeValue::LinkedList
    }
}

/// `ListUtils#toList(Object)` 的动态输入分类。
///
/// 对应 Java: `org.thymeleaf.util.ListUtils#toList(Object)` 的运行时类型分派。
pub enum ListTarget<'a, T> {
    /// 输入已经是列表；结果借用同一实例。
    List(&'a dyn ListView<T>),
    /// 输入是 Java 对象数组；Rust 通过切片表达。
    Array(&'a [T]),
    /// 输入是普通 Iterable。
    Iterable(Box<dyn Iterator<Item = T> + 'a>),
    /// 输入是 primitive array，Java `Object[]` 强转失败。
    PrimitiveArray(&'a str),
    /// 输入不属于 List、对象数组或 Iterable。
    Unsupported(&'a str),
}

/// Java 列表结果的借用身份或独立存储。
///
/// 对应 Java: `org.thymeleaf.util.ListUtils#toList` 与 `#sort` 返回的 `List<?>`。
enum ListStorage<'a, T> {
    Borrowed(&'a dyn ListView<T>),
    Owned {
        elements: Vec<T>,
        list_type: ListTypeValue,
    },
}

/// `ListUtils` 返回的列表值。
///
/// 对应 Java `List<?>`。借用分支保留已有列表身份，拥有分支保留顺序、重复项和
/// 运行时列表类型。Rust 不依赖 JVM 反射，但通过 [`ListTypeValue`] 保留排序结果
/// “同类型构造或回退 ArrayList”的可观察合同。
///
/// 对应 Java: `java.util.List` 返回值，来源为 `org.thymeleaf.util.ListUtils`。
pub struct ListValue<'a, T> {
    storage: ListStorage<'a, T>,
}

impl<'a, T> ListValue<'a, T> {
    fn borrowed(target: &'a dyn ListView<T>) -> Self {
        Self {
            storage: ListStorage::Borrowed(target),
        }
    }

    fn owned(elements: Vec<T>, list_type: ListTypeValue) -> Self {
        Self {
            storage: ListStorage::Owned {
                elements,
                list_type,
            },
        }
    }

    /// 返回列表元素数。
    ///
    /// # 返回
    /// 当前列表长度。
    /// 对应 Java 语义：`ListUtils` 的 `len` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn len(&self) -> usize {
        match &self.storage {
            ListStorage::Borrowed(target) => target.len(),
            ListStorage::Owned { elements, .. } => elements.len(),
        }
    }

    /// 判断列表是否为空。
    ///
    /// # 返回
    /// 长度为零时返回 `true`。
    /// 对应 Java: `ListUtils#isEmpty()`。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 返回指定位置元素。
    ///
    /// # 参数
    /// - `index`：从零开始的位置。
    ///
    /// # 返回
    /// 位置存在时返回借用元素。
    /// 对应 Java 语义：Java 接口/超类方法 `get()` 的 Rust 移植（`ListUtils` 继承路径）。
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        match &self.storage {
            ListStorage::Borrowed(target) => target.get(index),
            ListStorage::Owned { elements, .. } => elements.get(index),
        }
    }

    /// 按列表顺序访问元素。
    ///
    /// # 返回
    /// 借用元素的只读迭代器。
    /// 对应 Java 语义：`ListUtils` 的 `iter` 行为（Rust 侧辅助/私有路径）。
    pub fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match &self.storage {
            ListStorage::Borrowed(target) => target.iter(),
            ListStorage::Owned { elements, .. } => Box::new(elements.iter()),
        }
    }

    /// 判断列表是否包含指定元素。
    ///
    /// # 参数
    /// - `element`：待查找元素。
    ///
    /// # 返回
    /// 任一元素按 Java `equals` 对应值相等时返回 `true`。
    /// 对应 Java: `ListUtils#contains()`。
    #[must_use]
    pub fn contains(&self, element: &T) -> bool
    where
        T: PartialEq,
    {
        self.iter().any(|candidate| candidate == element)
    }

    /// 返回列表运行时类型。
    ///
    /// # 返回
    /// Java 类型描述。
    /// 对应 Java 语义：`ListUtils` 的 `list_type` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn list_type(&self) -> ListTypeValue {
        match &self.storage {
            ListStorage::Borrowed(target) => target.list_type(),
            ListStorage::Owned { list_type, .. } => list_type.clone(),
        }
    }

    /// 判断结果是否借用指定列表实例。
    ///
    /// # 参数
    /// - `target`：用于身份比较的原列表。
    ///
    /// # 返回
    /// `toList` 直接返回该列表时返回 `true`。
    /// 对应 Java 语义：`ListUtils` 的 `is_borrowed_from` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn is_borrowed_from(&self, target: &dyn ListView<T>) -> bool {
        match self.storage {
            ListStorage::Borrowed(source) => ptr::eq(source, target),
            ListStorage::Owned { .. } => false,
        }
    }
}

impl<T> ListView<T> for ListValue<'_, T> {
    fn len(&self) -> usize {
        ListValue::len(self)
    }

    fn get(&self, index: usize) -> Option<&T> {
        ListValue::get(self, index)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        ListValue::iter(self)
    }

    fn list_type(&self) -> ListTypeValue {
        ListValue::list_type(self)
    }
}

/// Thymeleaf 的列表工具。
///
/// 对应 Java: `org.thymeleaf.util.ListUtils`。
///
/// 保留动态转换、列表身份、顺序与重复项、两个 `containsAll` 重载、稳定排序、
/// 原列表不变、nullable Comparator 和反射构造回退语义。
pub struct ListUtils;

impl ListUtils {
    /// 将动态目标转换为列表。
    ///
    /// 对应 Java: `ListUtils#toList(Object)`。
    ///
    /// # 参数
    /// - `target`：Java 参数 `target`；`None` 对应 Java null。
    ///
    /// # 返回
    /// 已有列表的借用视图，或新建的 ArrayList 等价值。
    ///
    /// # 错误
    /// null、primitive array 或不支持类型返回对应 Java 异常类别。
    pub fn to_list<'a, T>(
        target: Option<ListTarget<'a, T>>,
    ) -> Result<ListValue<'a, T>, ListUtilsError>
    where
        T: Clone,
    {
        Validate::not_null(target.as_ref(), Some("Cannot convert null to list"))?;
        match target.expect("validated target") {
            ListTarget::List(target) => Ok(ListValue::borrowed(target)),
            ListTarget::Array(target) => {
                Ok(ListValue::owned(target.to_vec(), ListTypeValue::ArrayList))
            }
            ListTarget::Iterable(target) => {
                Ok(ListValue::owned(target.collect(), ListTypeValue::ArrayList))
            }
            ListTarget::PrimitiveArray(class_name) => Err(ListUtilsError::ClassCast {
                class_name: class_name.to_owned(),
            }),
            ListTarget::Unsupported(class_name) => Err(ListUtilsError::CannotConvert {
                class_name: class_name.to_owned(),
            }),
        }
    }

    /// 返回列表大小。
    ///
    /// 对应 Java: `ListUtils#size(List)`。
    ///
    /// # 参数
    /// - `target`：目标列表；`None` 对应 Java null。
    ///
    /// # 返回
    /// Java `List#size()` 对应大小。
    ///
    /// # 错误
    /// target 为 null 时返回精确参数错误。
    pub fn size<T>(target: Option<&dyn ListView<T>>) -> Result<i32, ValidateError> {
        Validate::not_null(target, Some("Cannot get list size of null"))?;
        Ok(list_size(target.expect("validated target").len()))
    }

    /// 判断列表是否为 null 或为空。
    ///
    /// 对应 Java: `ListUtils#isEmpty(List)`。
    ///
    /// # 参数
    /// - `target`：待判断列表。
    ///
    /// # 返回
    /// target 为 null 或空列表时返回 `true`。
    #[must_use]
    pub fn is_empty<T>(target: Option<&dyn ListView<T>>) -> bool {
        target.is_none_or(ListView::is_empty)
    }

    /// 判断列表是否包含指定元素。
    ///
    /// 对应 Java: `ListUtils#contains(List,Object)`。
    ///
    /// # 参数
    /// - `target`：目标列表；
    /// - `element`：待查找元素。
    ///
    /// # 返回
    /// 元素存在时返回 `true`。
    ///
    /// # 错误
    /// target 为 null 时返回精确参数错误。
    pub fn contains<T>(target: Option<&dyn ListView<T>>, element: &T) -> Result<bool, ValidateError>
    where
        T: PartialEq,
    {
        Validate::not_null(target, Some("Cannot execute list contains: target is null"))?;
        Ok(target
            .expect("validated target")
            .iter()
            .any(|candidate| candidate == element))
    }

    /// 判断列表是否包含数组中的全部元素。
    ///
    /// 对应 Java: `ListUtils#containsAll(List,Object[])`。
    ///
    /// # 参数
    /// - `target`：目标列表；
    /// - `elements`：请求元素数组。
    ///
    /// # 返回
    /// 每个请求元素至少出现一次时返回 `true`；重复请求不要求重复条目。
    ///
    /// # 错误
    /// 按 Java 顺序先校验 target，再校验 elements。
    pub fn contains_all_array<T>(
        target: Option<&dyn ListView<T>>,
        elements: Option<&[T]>,
    ) -> Result<bool, ValidateError>
    where
        T: PartialEq,
    {
        Validate::not_null(
            target,
            Some("Cannot execute list containsAll: target is null"),
        )?;
        Validate::not_null(
            elements,
            Some("Cannot execute list containsAll: elements is null"),
        )?;
        let target = target.expect("validated target");
        Ok(elements
            .expect("validated elements")
            .iter()
            .all(|element| target.iter().any(|candidate| candidate == element)))
    }

    /// 判断列表是否包含 Collection 中的全部元素。
    ///
    /// 对应 Java: `ListUtils#containsAll(List,Collection)`。
    ///
    /// # 参数
    /// - `target`：目标列表；
    /// - `elements`：请求 Collection 迭代器。
    ///
    /// # 返回
    /// 每个请求元素至少出现一次时返回 `true`。
    ///
    /// # 错误
    /// 保留该重载与数组重载不同的 target 错误文本。
    pub fn contains_all_collection<'a, T, I>(
        target: Option<&dyn ListView<T>>,
        elements: Option<I>,
    ) -> Result<bool, ValidateError>
    where
        T: PartialEq + 'a,
        I: IntoIterator<Item = &'a T>,
    {
        Validate::not_null(target, Some("Cannot execute list contains: target is null"))?;
        Validate::not_null(
            elements.as_ref(),
            Some("Cannot execute list containsAll: elements is null"),
        )?;
        let target = target.expect("validated target");
        Ok(elements
            .expect("validated elements")
            .into_iter()
            .all(|element| target.iter().any(|candidate| candidate == element)))
    }

    /// 按 Java 自然顺序稳定排序列表副本。
    ///
    /// 对应 Java: `ListUtils#sort(List<T>)`。
    ///
    /// # 参数
    /// - `list`：待排序列表；原列表不会被修改。
    ///
    /// # 返回
    /// 使用原列表可公开构造类型或 ArrayList 回退类型的新列表。
    ///
    /// # 错误
    /// null 列表、`toArray()`、null 元素或自然比较失败时返回对应错误。
    pub fn sort<T>(list: Option<&dyn ListView<T>>) -> Result<ListValue<'static, T>, ListUtilsError>
    where
        T: Clone + ComparableValue + 'static,
    {
        Self::sort_with_comparator(list, None)
    }

    /// 使用 nullable Comparator 稳定排序列表副本。
    ///
    /// 对应 Java: `ListUtils#sort(List<T>,Comparator<? super T>)`。
    ///
    /// # 参数
    /// - `list`：待排序列表；
    /// - `comparator`：Java Comparator；`None` 对应 null 并回退自然顺序。
    ///
    /// # 返回
    /// 同类型构造或 ArrayList 回退的新列表；原列表保持不变。
    ///
    /// # 错误
    /// 传播列表快照、自然比较或 Comparator 错误。
    pub fn sort_with_comparator<T>(
        list: Option<&dyn ListView<T>>,
        comparator: Option<&mut dyn ComparatorValue<T>>,
    ) -> Result<ListValue<'static, T>, ListUtilsError>
    where
        T: Clone + ComparableValue + 'static,
    {
        Validate::not_null(list, Some("Cannot execute list sort: list is null"))?;
        let list = list.expect("validated list");
        let elements = list.snapshot()?;
        let sorted = match comparator {
            Some(comparator) => stable_sort(elements, comparator),
            None => {
                let mut natural = |left: &T, right: &T| left.template_compare_to(right);
                stable_sort(elements, &mut natural)
            }
        };
        sorted.and_then(|elements| list.fill_sorted(elements))
    }

    /// 使用非 null Comparator 排序不实现 [`ComparableValue`] 的元素。
    ///
    /// 这是 Java `<T>` Comparator 重载在 Rust 静态类型系统中的完整入口。
    ///
    /// # 参数
    /// - `list`：待排序列表；
    /// - `comparator`：非 null Comparator。
    ///
    /// # 返回
    /// 同类型构造或 ArrayList 回退的新列表。
    ///
    /// # 错误
    /// null 列表、列表快照或 Comparator 失败时返回对应错误。
    /// 对应 Java 语义：`ListUtils` 的 `sort_with_required_comparator` 行为（Rust 侧辅助/私有路径）。
    pub fn sort_with_required_comparator<T>(
        list: Option<&dyn ListView<T>>,
        comparator: &mut dyn ComparatorValue<T>,
    ) -> Result<ListValue<'static, T>, ListUtilsError>
    where
        T: Clone + 'static,
    {
        Validate::not_null(list, Some("Cannot execute list sort: list is null"))?;
        let list = list.expect("validated list");
        let elements = list.snapshot()?;
        stable_sort(elements, comparator).and_then(|sorted| list.fill_sorted(sorted))
    }
}

fn fill_new_list<T>(elements: Vec<T>, list_type: &ListTypeValue) -> ListValue<'static, T> {
    ListValue::owned(elements, list_type.sorted_result_type())
}

fn stable_sort<T>(
    mut elements: Vec<T>,
    comparator: &mut dyn ComparatorValue<T>,
) -> Result<Vec<T>, ListUtilsError> {
    if elements.len() < 2 {
        return Ok(elements);
    }
    let right = elements.split_off(elements.len() / 2);
    let left = stable_sort(elements, comparator)?;
    let right = stable_sort(right, comparator)?;
    merge(left, right, comparator)
}

fn merge<T>(
    left: Vec<T>,
    right: Vec<T>,
    comparator: &mut dyn ComparatorValue<T>,
) -> Result<Vec<T>, ListUtilsError> {
    let capacity = left.len().saturating_add(right.len());
    let mut left = VecDeque::from(left);
    let mut right = VecDeque::from(right);
    let mut result = Vec::with_capacity(capacity);

    while let (Some(left_value), Some(right_value)) = (left.front(), right.front()) {
        if comparator.compare(left_value, right_value)? == Ordering::Greater {
            result.push(right.pop_front().expect("right front exists"));
        } else {
            // 相等时先取左半边，保留 Arrays.sort(Object[]) 的稳定性。
            result.push(left.pop_front().expect("left front exists"));
        }
    }
    result.extend(left);
    result.extend(right);
    Ok(result)
}

fn list_size(size: usize) -> i32 {
    i32::try_from(size).unwrap_or(i32::MAX)
}

fn f32_compare(left: f32, right: f32) -> Ordering {
    if left < right {
        Ordering::Less
    } else if left > right {
        Ordering::Greater
    } else {
        let left_bits = f32_bits(left);
        let right_bits = f32_bits(right);
        left_bits.cmp(&right_bits)
    }
}

fn f32_bits(value: f32) -> i32 {
    if value.is_nan() {
        0x7fc0_0000_u32 as i32
    } else {
        value.to_bits() as i32
    }
}

fn f64_compare(left: f64, right: f64) -> Ordering {
    if left < right {
        Ordering::Less
    } else if left > right {
        Ordering::Greater
    } else {
        let left_bits = f64_bits(left);
        let right_bits = f64_bits(right);
        left_bits.cmp(&right_bits)
    }
}

fn f64_bits(value: f64) -> i64 {
    if value.is_nan() {
        0x7ff8_0000_0000_0000_u64 as i64
    } else {
        value.to_bits() as i64
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::collections::LinkedList;

    use super::{
        ComparableValue, ListTarget, ListTypeValue, ListUtils, ListUtilsError, ListValue, ListView,
    };
    use crate::util::ValidateError;

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Item {
        key: i32,
        id: &'static str,
    }

    struct CustomList<T> {
        values: Vec<T>,
        list_type: ListTypeValue,
        snapshot_error: Option<(&'static str, &'static str)>,
    }

    struct AddFailingList<T> {
        values: Vec<T>,
    }

    impl<T> ListView<T> for CustomList<T> {
        fn len(&self) -> usize {
            self.values.len()
        }

        fn get(&self, index: usize) -> Option<&T> {
            self.values.get(index)
        }

        fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
            Box::new(self.values.iter())
        }

        fn list_type(&self) -> ListTypeValue {
            self.list_type.clone()
        }

        fn snapshot(&self) -> Result<Vec<T>, ListUtilsError>
        where
            T: Clone,
        {
            match self.snapshot_error {
                Some((class_name, message)) => Err(ListUtilsError::runtime(class_name, message)),
                None => Ok(self.values.clone()),
            }
        }
    }

    impl<T> ListView<T> for AddFailingList<T> {
        fn len(&self) -> usize {
            self.values.len()
        }

        fn get(&self, index: usize) -> Option<&T> {
            self.values.get(index)
        }

        fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
            Box::new(self.values.iter())
        }

        fn list_type(&self) -> ListTypeValue {
            ListTypeValue::custom("example.AddFailingList", true)
        }

        fn fill_sorted(&self, _elements: Vec<T>) -> Result<ListValue<'static, T>, ListUtilsError>
        where
            T: 'static,
        {
            Err(ListUtilsError::runtime(
                "java.lang.UnsupportedOperationException",
                "add failed",
            ))
        }
    }

    #[test]
    fn converts_list_array_and_iterable_with_identity_and_order() {
        let source = vec![Some("one".to_owned()), None, Some("one".to_owned())];
        let view: &dyn ListView<Option<String>> = &source;
        let borrowed = ListUtils::to_list(Some(ListTarget::List(view))).unwrap();
        assert!(borrowed.is_borrowed_from(view));
        assert_eq!(borrowed.len(), 3);
        assert_eq!(borrowed.list_type(), ListTypeValue::ArrayList);
        assert_eq!(borrowed.get(1), Some(&None));
        assert_eq!(borrowed.get(10), None);

        let array = [Some("two".to_owned()), None, Some("two".to_owned())];
        let converted = ListUtils::to_list(Some(ListTarget::Array(&array))).unwrap();
        assert_eq!(
            converted.iter().cloned().collect::<Vec<_>>(),
            array.to_vec()
        );
        assert!(!converted.is_borrowed_from(view));

        let iterable = vec![Some("b".to_owned()), Some("a".to_owned())];
        let converted = ListUtils::to_list(Some(ListTarget::Iterable(Box::new(
            iterable.clone().into_iter(),
        ))))
        .unwrap();
        assert_eq!(converted.iter().cloned().collect::<Vec<_>>(), iterable);
    }

    #[test]
    fn preserves_conversion_errors_and_runtime_error_factory() {
        assert_eq!(
            ListUtils::to_list(None::<ListTarget<'_, Option<String>>>)
                .err()
                .expect("null"),
            ListUtilsError::Validation(ValidateError::IllegalArgument {
                message: Some("Cannot convert null to list".to_owned())
            })
        );
        assert_eq!(
            ListUtils::to_list(Some(ListTarget::<Option<String>>::PrimitiveArray("[I")))
                .err()
                .expect("primitive"),
            ListUtilsError::ClassCast {
                class_name: "[I".to_owned()
            }
        );
        assert_eq!(
            ListUtils::to_list(Some(ListTarget::<Option<String>>::Unsupported(
                "java.lang.Integer",
            )))
            .err()
            .expect("unsupported"),
            ListUtilsError::CannotConvert {
                class_name: "java.lang.Integer".to_owned()
            }
        );
        assert_eq!(
            ListUtilsError::runtime("java.lang.IllegalStateException", "boom"),
            ListUtilsError::Runtime {
                class_name: "java.lang.IllegalStateException".to_owned(),
                message: "boom".to_owned()
            }
        );
        assert_eq!(
            ListUtilsError::NaturalOrderingClassCast {
                left_class: "java.lang.String".to_owned(),
                right_class: "java.lang.Integer".to_owned()
            }
            .to_string(),
            "class \"java.lang.String\" cannot be compared to class \"java.lang.Integer\""
        );
    }

    #[test]
    fn checks_size_empty_contains_and_both_contains_all_overloads() {
        let source = vec![Some("one".to_owned()), None, Some("one".to_owned())];
        let view: &dyn ListView<Option<String>> = &source;
        let empty = Vec::<Option<String>>::new();
        let empty_view: &dyn ListView<Option<String>> = &empty;

        assert_eq!(ListUtils::size(Some(view)), Ok(3));
        assert_eq!(
            ListUtils::size(None::<&dyn ListView<Option<String>>>),
            Err(ValidateError::IllegalArgument {
                message: Some("Cannot get list size of null".to_owned())
            })
        );
        assert!(!ListUtils::is_empty(Some(view)));
        assert!(ListUtils::is_empty(Some(empty_view)));
        assert!(ListUtils::is_empty(None::<&dyn ListView<Option<String>>>));
        assert_eq!(super::list_size(usize::MAX), i32::MAX);

        assert_eq!(ListUtils::contains(Some(view), &None), Ok(true));
        assert_eq!(
            ListUtils::contains(Some(view), &Some("missing".to_owned())),
            Ok(false)
        );
        assert!(ListUtils::contains(None::<&dyn ListView<Option<String>>>, &None).is_err());

        let present = [Some("one".to_owned()), None, Some("one".to_owned())];
        let missing = [Some("one".to_owned()), Some("missing".to_owned())];
        assert_eq!(
            ListUtils::contains_all_array(Some(view), Some(&present)),
            Ok(true)
        );
        assert_eq!(
            ListUtils::contains_all_array(Some(view), Some(&missing)),
            Ok(false)
        );
        assert_eq!(
            ListUtils::contains_all_collection(Some(view), Some(present.iter())),
            Ok(true)
        );
        assert_eq!(
            ListUtils::contains_all_collection(Some(view), Some(missing.iter())),
            Ok(false)
        );
        let target_error = ListUtils::contains_all_array(
            None::<&dyn ListView<Option<String>>>,
            None::<&[Option<String>]>,
        )
        .unwrap_err();
        assert_eq!(
            target_error.get_message(),
            Some("Cannot execute list containsAll: target is null")
        );
        let collection_target_error = ListUtils::contains_all_collection(
            None::<&dyn ListView<Option<String>>>,
            None::<std::slice::Iter<'_, Option<String>>>,
        )
        .unwrap_err();
        assert_eq!(
            collection_target_error.get_message(),
            Some("Cannot execute list contains: target is null")
        );
        assert!(ListUtils::contains_all_array(Some(view), None::<&[Option<String>]>).is_err());
        assert!(
            ListUtils::contains_all_collection(
                Some(view),
                None::<std::slice::Iter<'_, Option<String>>>
            )
            .is_err()
        );
    }

    #[test]
    fn stable_sort_preserves_source_and_runtime_list_type_or_fallback() {
        let source = LinkedList::from(["c".to_owned(), "a".to_owned(), "b".to_owned()]);
        let source_view: &dyn ListView<String> = &source;
        assert_eq!(source_view.get(1), Some(&"a".to_owned()));
        let sorted = ListUtils::sort(Some(source_view)).unwrap();
        assert_eq!(
            sorted.iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
        assert_eq!(
            source.iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["c", "a", "b"]
        );
        assert_eq!(sorted.list_type(), ListTypeValue::LinkedList);

        let constructible = CustomList {
            values: vec!["b".to_owned(), "a".to_owned()],
            list_type: ListTypeValue::custom("example.PublicList", true),
            snapshot_error: None,
        };
        assert_eq!(constructible.len(), 2);
        assert_eq!(constructible.get(0), Some(&"b".to_owned()));
        assert_eq!(constructible.iter().count(), 2);
        let sorted = ListUtils::sort(Some(&constructible as &dyn ListView<String>)).unwrap();
        assert_eq!(
            sorted.list_type(),
            ListTypeValue::custom("example.PublicList", true)
        );

        let fallback = CustomList {
            values: vec!["b".to_owned(), "a".to_owned()],
            list_type: ListTypeValue::custom("example.PrivateList", false),
            snapshot_error: None,
        };
        let sorted = ListUtils::sort(Some(&fallback as &dyn ListView<String>)).unwrap();
        assert_eq!(sorted.list_type(), ListTypeValue::ArrayList);
        assert_eq!(fallback.list_type.class_name(), "example.PrivateList");

        let add_failing = AddFailingList {
            values: vec!["b".to_owned(), "a".to_owned()],
        };
        assert_eq!(add_failing.len(), 2);
        assert_eq!(add_failing.get(1), Some(&"a".to_owned()));
        assert_eq!(
            add_failing.list_type(),
            ListTypeValue::custom("example.AddFailingList", true)
        );
        assert_eq!(
            ListUtils::sort(Some(&add_failing as &dyn ListView<String>))
                .err()
                .expect("add failure"),
            ListUtilsError::runtime("java.lang.UnsupportedOperationException", "add failed")
        );
    }

    #[test]
    fn comparator_sort_is_stable_nullable_and_supports_non_comparable_types() {
        let source = vec!["c".to_owned(), "a".to_owned(), "b".to_owned()];
        let view: &dyn ListView<String> = &source;
        let mut descending = |left: &String, right: &String| right.template_compare_to(left);
        let sorted = ListUtils::sort_with_comparator(Some(view), Some(&mut descending)).unwrap();
        assert_eq!(
            sorted.iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["c", "b", "a"]
        );
        let natural = ListUtils::sort_with_comparator(Some(view), None).unwrap();
        assert_eq!(
            natural.iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );

        let items = CustomList {
            values: vec![
                Item {
                    key: 2,
                    id: "first",
                },
                Item {
                    key: 1,
                    id: "middle",
                },
                Item {
                    key: 2,
                    id: "second",
                },
            ],
            list_type: ListTypeValue::ArrayList,
            snapshot_error: None,
        };
        assert_eq!(items.len(), 3);
        assert_eq!(items.get(1).map(|item| item.id), Some("middle"));
        assert_eq!(items.iter().count(), 3);
        assert_eq!(items.list_type(), ListTypeValue::ArrayList);
        let item_view: &dyn ListView<Item> = &items;
        let mut by_key = |left: &Item, right: &Item| Ok(left.key.cmp(&right.key));
        let sorted =
            ListUtils::sort_with_required_comparator(Some(item_view), &mut by_key).unwrap();
        assert_eq!(
            sorted.iter().map(|item| item.id).collect::<Vec<_>>(),
            vec!["middle", "first", "second"]
        );

        assert!(
            ListUtils::sort_with_required_comparator(None::<&dyn ListView<Item>>, &mut by_key,)
                .is_err()
        );
        let snapshot_failing = CustomList {
            values: Vec::<Item>::new(),
            list_type: ListTypeValue::ArrayList,
            snapshot_error: Some(("java.lang.IllegalStateException", "toArray failed")),
        };
        assert!(
            ListUtils::sort_with_required_comparator(Some(&snapshot_failing), &mut by_key).is_err()
        );
        let four_items = vec![
            Item { key: 1, id: "1" },
            Item { key: 2, id: "2" },
            Item { key: 3, id: "3" },
            Item { key: 4, id: "4" },
        ];
        let mut failing = |_left: &Item, _right: &Item| {
            Err(ListUtilsError::runtime(
                "java.lang.IllegalStateException",
                "compare failed",
            ))
        };
        assert!(ListUtils::sort_with_required_comparator(Some(&four_items), &mut failing).is_err());

        let mut comparisons = 0_u8;
        let mut fail_in_right_half = |left: &Item, right: &Item| {
            comparisons = comparisons.saturating_add(1);
            if comparisons == 2 {
                Err(ListUtilsError::runtime(
                    "java.lang.IllegalStateException",
                    "right comparison failed",
                ))
            } else {
                Ok(left.key.cmp(&right.key))
            }
        };
        assert!(
            ListUtils::sort_with_required_comparator(Some(&four_items), &mut fail_in_right_half,)
                .is_err()
        );

        let mut comparisons = 0_u8;
        let mut fail_in_outer_merge = |left: &Item, right: &Item| {
            comparisons = comparisons.saturating_add(1);
            if comparisons == 3 {
                Err(ListUtilsError::runtime(
                    "java.lang.IllegalStateException",
                    "outer comparison failed",
                ))
            } else {
                Ok(left.key.cmp(&right.key))
            }
        };
        assert!(
            ListUtils::sort_with_required_comparator(Some(&four_items), &mut fail_in_outer_merge,)
                .is_err()
        );
    }

    #[test]
    fn sorting_propagates_null_snapshot_comparator_and_empty_boundaries() {
        assert_eq!(
            ListUtils::sort(None::<&dyn ListView<String>>)
                .err()
                .expect("null sort"),
            ListUtilsError::Validation(ValidateError::IllegalArgument {
                message: Some("Cannot execute list sort: list is null".to_owned())
            })
        );

        let nullable = vec![Some("a".to_owned()), None];
        assert_eq!(
            ListUtils::sort(Some(&nullable as &dyn ListView<Option<String>>))
                .err()
                .expect("null element"),
            ListUtilsError::NaturalOrderingNull
        );
        let null_in_left_half = vec![
            Some("a".to_owned()),
            None,
            Some("b".to_owned()),
            Some("c".to_owned()),
        ];
        let null_in_right_half = vec![
            Some("a".to_owned()),
            Some("b".to_owned()),
            Some("c".to_owned()),
            None,
        ];
        assert!(ListUtils::sort(Some(&null_in_left_half)).is_err());
        assert!(ListUtils::sort(Some(&null_in_right_half)).is_err());

        let failing = CustomList {
            values: vec!["b".to_owned(), "a".to_owned()],
            list_type: ListTypeValue::custom("example.FailingList", true),
            snapshot_error: Some(("java.lang.IllegalStateException", "toArray failed")),
        };
        assert_eq!(
            ListUtils::sort(Some(&failing as &dyn ListView<String>))
                .err()
                .expect("snapshot"),
            ListUtilsError::runtime("java.lang.IllegalStateException", "toArray failed")
        );

        let source = vec!["b".to_owned(), "a".to_owned()];
        let mut comparator = |_left: &String, _right: &String| {
            Err(ListUtilsError::runtime(
                "java.lang.IllegalStateException",
                "compare failed",
            ))
        };
        assert_eq!(
            ListUtils::sort_with_comparator(
                Some(&source as &dyn ListView<String>),
                Some(&mut comparator),
            )
            .err()
            .expect("comparator"),
            ListUtilsError::runtime("java.lang.IllegalStateException", "compare failed")
        );

        let four_strings = vec![
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        ];
        let mut fail_in_left_half = |_left: &String, _right: &String| {
            Err(ListUtilsError::runtime(
                "java.lang.IllegalStateException",
                "left comparison failed",
            ))
        };
        assert!(
            ListUtils::sort_with_comparator(Some(&four_strings), Some(&mut fail_in_left_half),)
                .is_err()
        );
        let mut comparisons = 0_u8;
        let mut fail_in_right_half = |left: &String, right: &String| {
            comparisons = comparisons.saturating_add(1);
            if comparisons == 2 {
                Err(ListUtilsError::runtime(
                    "java.lang.IllegalStateException",
                    "right comparison failed",
                ))
            } else {
                left.template_compare_to(right)
            }
        };
        assert!(
            ListUtils::sort_with_comparator(Some(&four_strings), Some(&mut fail_in_right_half),)
                .is_err()
        );

        let empty = Vec::<String>::new();
        let singleton = vec!["one".to_owned()];
        assert!(
            ListUtils::sort(Some(&empty as &dyn ListView<String>))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            ListUtils::sort(Some(&singleton as &dyn ListView<String>))
                .unwrap()
                .get(0),
            Some(&"one".to_owned())
        );
    }

    #[test]
    fn comparable_matches_utf16_and_float_wrapper_ordering() {
        assert_eq!(
            "\u{1f600}"
                .to_owned()
                .template_compare_to(&"\u{e000}".to_owned()),
            Ok(Ordering::Less)
        );
        assert_eq!((-0.0_f64).template_compare_to(&0.0_f64), Ok(Ordering::Less));
        assert_eq!(1.0_f64.template_compare_to(&2.0_f64), Ok(Ordering::Less));
        assert_eq!(2.0_f64.template_compare_to(&1.0_f64), Ok(Ordering::Greater));
        assert_eq!(
            f64::NAN.template_compare_to(&f64::INFINITY),
            Ok(Ordering::Greater)
        );
        assert_eq!((-0.0_f32).template_compare_to(&0.0_f32), Ok(Ordering::Less));
        assert_eq!(1.0_f32.template_compare_to(&2.0_f32), Ok(Ordering::Less));
        assert_eq!(2.0_f32.template_compare_to(&1.0_f32), Ok(Ordering::Greater));
        assert_eq!(
            f32::NAN.template_compare_to(&f32::INFINITY),
            Ok(Ordering::Greater)
        );
        assert_eq!(
            Some("a".to_owned()).template_compare_to(&Some("b".to_owned())),
            Ok(Ordering::Less)
        );
        assert_eq!(false.template_compare_to(&true), Ok(Ordering::Less));
        assert_eq!(1_i8.template_compare_to(&2), Ok(Ordering::Less));
        assert_eq!(1_i16.template_compare_to(&2), Ok(Ordering::Less));
        assert_eq!(1_i32.template_compare_to(&2), Ok(Ordering::Less));
        assert_eq!(1_i64.template_compare_to(&2), Ok(Ordering::Less));
        assert_eq!(1_u16.template_compare_to(&2), Ok(Ordering::Less));
    }

    #[test]
    fn list_delegates_list_view_operations() {
        let list = ListValue::owned(vec![Some("one".to_owned()), None], ListTypeValue::ArrayList);
        let view: &dyn ListView<Option<String>> = &list;
        assert_eq!(view.len(), 2);
        assert!(!view.is_empty());
        assert_eq!(view.get(1), Some(&None));
        assert_eq!(view.iter().count(), 2);
        assert_eq!(view.list_type(), ListTypeValue::ArrayList);
        assert!(list.contains(&None));
    }
}
