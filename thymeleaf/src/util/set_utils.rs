use std::collections::{BTreeSet, HashSet};
use std::hash::{BuildHasher, Hash};
use std::ptr;

use indexmap::IndexSet;
use thiserror::Error;

use super::{Validate, ValidateError};

/// Java `Set` 的只读 Rust 视图。
///
/// 这是 `org.thymeleaf.util.SetUtils` 所需的 Rust 等价适配：既允许借用
/// [`HashSet`]，也允许借用保持插入顺序的 [`IndexSet`]，从而使
/// [`SetUtils::to_set`] 在输入已经是集合时返回同一个集合视图，而不复制数据。
/// 对应 Java 语义：`SetUtils` 的 Rust 侧类型 `SetView`。
pub trait SetView<T> {
    /// 返回集合元素数。
    ///
    /// # 返回
    /// 当前集合包含的唯一元素数。
    fn len(&self) -> usize;

    /// 判断集合是否没有元素。
    ///
    /// # 返回
    /// 集合长度为零时返回 `true`。
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 判断集合是否包含指定元素。
    ///
    /// # 参数
    /// - `element`：待查找元素。
    ///
    /// # 返回
    /// 底层集合包含该元素时返回 `true`。
    fn contains(&self, element: &T) -> bool;

    /// 按底层集合自身的迭代顺序访问元素。
    ///
    /// # 返回
    /// 借用集合元素的只读迭代器。
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_>;
}

impl<T, S> SetView<T> for HashSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    fn len(&self) -> usize {
        HashSet::len(self)
    }

    fn contains(&self, element: &T) -> bool {
        HashSet::contains(self, element)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(HashSet::iter(self))
    }
}

impl<T, S> SetView<T> for IndexSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    fn len(&self) -> usize {
        IndexSet::len(self)
    }

    fn contains(&self, element: &T) -> bool {
        IndexSet::contains(self, element)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(IndexSet::iter(self))
    }
}

impl<T> SetView<T> for BTreeSet<T>
where
    T: Ord,
{
    fn len(&self) -> usize {
        BTreeSet::len(self)
    }

    fn contains(&self, element: &T) -> bool {
        BTreeSet::contains(self, element)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(BTreeSet::iter(self))
    }
}

/// `SetUtils#toSet(Object)` 的动态输入分类。
///
/// 对应 Java 运行时依次检查 `Set`、对象数组和 `Iterable` 的分支。
/// `Unsupported` 保存 Java `Class#getName()` 的结果，以生成相同错误消息。
pub enum SetTarget<'a, T> {
    /// 输入已经是集合；返回值必须继续借用同一实例。
    Set(&'a dyn SetView<T>),
    /// 输入是 Java 对象数组；Rust 通过切片表达。
    Array(&'a [T]),
    /// 输入是普通 `Iterable`；迭代产生的元素被加入新集合。
    Iterable(Box<dyn Iterator<Item = T> + 'a>),
    /// 输入是 Java primitive array；其 `Object[]` 强制转换必然失败。
    PrimitiveArray(&'a str),
    /// 输入既不是集合、对象数组，也不是可迭代对象。
    Unsupported(&'a str),
}

enum SetStorage<'a, T> {
    Borrowed(&'a dyn SetView<T>),
    Owned(IndexSet<T>),
}

/// `SetUtils` 返回的只读集合。
///
/// 对应 Java `Set<?>` 返回合同。借用分支保留原集合身份；拥有分支使用
/// [`IndexSet`] 精确保留 Java `LinkedHashSet` 的首次插入顺序与去重语义。
/// 本类型不暴露可变入口，因此也保留 `singletonSet` 的不可修改合同。
///
/// ```compile_fail
/// use thymeleaf::util::SetUtils;
///
/// let mut singleton = SetUtils::singleton_set("one");
/// singleton.insert("two");
/// ```
pub struct SetValue<'a, T> {
    storage: SetStorage<'a, T>,
}

impl<'a, T> SetValue<'a, T> {
    fn borrowed(target: &'a dyn SetView<T>) -> Self {
        Self {
            storage: SetStorage::Borrowed(target),
        }
    }

    fn owned(target: IndexSet<T>) -> Self {
        Self {
            storage: SetStorage::Owned(target),
        }
    }

    /// 返回集合元素数。
    ///
    /// # 返回
    /// 当前集合包含的唯一元素数。
    /// 对应 Java 语义：`SetUtils` 的 `len` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn len(&self) -> usize {
        match &self.storage {
            SetStorage::Borrowed(target) => target.len(),
            SetStorage::Owned(target) => target.len(),
        }
    }

    /// 判断集合是否没有元素。
    ///
    /// # 返回
    /// 集合长度为零时返回 `true`。
    /// 对应 Java: `SetUtils#isEmpty()`。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 判断集合是否包含指定元素。
    ///
    /// # 参数
    /// - `element`：待查找元素。
    ///
    /// # 返回
    /// 借用或拥有的底层集合包含该元素时返回 `true`。
    /// 对应 Java: `SetUtils#contains()`。
    #[must_use]
    pub fn contains(&self, element: &T) -> bool
    where
        T: Eq + Hash,
    {
        match &self.storage {
            SetStorage::Borrowed(target) => target.contains(element),
            SetStorage::Owned(target) => target.contains(element),
        }
    }

    /// 按底层集合顺序访问元素。
    ///
    /// # 返回
    /// 借用集合元素的只读迭代器。
    /// 对应 Java 语义：`SetUtils` 的 `iter` 行为（Rust 侧辅助/私有路径）。
    pub fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_>
    where
        T: Eq + Hash,
    {
        match &self.storage {
            SetStorage::Borrowed(target) => target.iter(),
            SetStorage::Owned(target) => Box::new(target.iter()),
        }
    }

    /// 判断返回值是否借用了指定集合实例。
    ///
    /// 该观察入口用于证明 Java `target instanceof Set` 分支直接返回
    /// `(Set<?>) target`，没有创建副本。
    ///
    /// # 参数
    /// - `target`：用于进行引用身份比较的原集合。
    ///
    /// # 返回
    /// 当前集合直接借用 `target` 时返回 `true`。
    /// 对应 Java 语义：`SetUtils` 的 `is_borrowed_from` 行为（Rust 侧辅助/私有路径）。
    #[must_use]
    pub fn is_borrowed_from(&self, target: &dyn SetView<T>) -> bool {
        match self.storage {
            SetStorage::Borrowed(source) => ptr::eq(source, target),
            SetStorage::Owned(_) => false,
        }
    }
}

impl<T> SetView<T> for SetValue<'_, T>
where
    T: Eq + Hash,
{
    fn len(&self) -> usize {
        SetValue::len(self)
    }

    fn contains(&self, element: &T) -> bool {
        SetValue::contains(self, element)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        SetValue::iter(self)
    }
}

/// `SetUtils#toSet(Object)` 的类型化错误。
/// 对应 Java 语义：`SetUtils` 的 Rust 侧类型 `SetUtilsError`。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SetUtilsError {
    /// Java `Validate.notNull` 对应的参数错误。
    #[error(transparent)]
    Validation(#[from] ValidateError),

    /// 输入不属于 Java `Set`、对象数组或 `Iterable`。
    #[error("Cannot convert object of class \"{class_name}\" to a set")]
    CannotConvert {
        /// Java `Class#getName()` 形式的类型名。
        class_name: String,
    },

    /// primitive array 无法强制转换成 Java `Object[]`。
    ///
    /// 上游在 `Arrays.asList((Object[]) target)` 处抛出
    /// `ClassCastException`。错误文本包含 JDK 模块/类加载器信息，Rust 只稳定保存
    /// 可移植的源数组类名和目标 `Object[]` 类别。
    #[error("class \"{class_name}\" cannot be cast to class \"[Ljava.lang.Object;\"")]
    ClassCast {
        /// Java primitive array 的运行时类名，例如 `[I`。
        class_name: String,
    },
}

/// Thymeleaf 的集合工具。
///
/// 对应 Java: `org.thymeleaf.util.SetUtils`。
///
/// Java 对象为无状态静态工具类。Rust 保留动态转换分支、已有集合身份、
/// `LinkedHashSet` 顺序、重复元素折叠、null 边界、校验顺序和只读单例集合。
pub struct SetUtils;

impl SetUtils {
    /// 将动态目标转换为只读集合。
    ///
    /// 对应 Java: `SetUtils#toSet(Object)`。
    ///
    /// # 参数
    /// - `target`：Java 参数 `target`；`None` 对应 Java null。
    ///
    /// # 返回
    /// 已有集合返回借用同一实例的视图；数组和 Iterable 返回按首次出现顺序
    /// 去重的新集合。
    ///
    /// # 错误
    /// null 输入或不支持的运行时类型返回与 Java 相同类别和消息的类型化错误。
    pub fn to_set<'a, T>(target: Option<SetTarget<'a, T>>) -> Result<SetValue<'a, T>, SetUtilsError>
    where
        T: Clone + Eq + Hash,
    {
        Validate::not_null(target.as_ref(), Some("Cannot convert null to set"))?;
        match target.expect("validated target") {
            SetTarget::Set(target) => Ok(SetValue::borrowed(target)),
            SetTarget::Array(target) => Ok(SetValue::owned(target.iter().cloned().collect())),
            SetTarget::Iterable(target) => Ok(SetValue::owned(target.collect())),
            SetTarget::PrimitiveArray(class_name) => Err(SetUtilsError::ClassCast {
                class_name: class_name.to_owned(),
            }),
            SetTarget::Unsupported(class_name) => Err(SetUtilsError::CannotConvert {
                class_name: class_name.to_owned(),
            }),
        }
    }

    /// 返回集合大小。
    ///
    /// 对应 Java: `SetUtils#size(Set)`。
    ///
    /// # 错误
    /// target 为 null 时返回 `Cannot get set size of null`。
    ///
    /// # 参数
    /// - `target`：待计算大小的集合；`None` 对应 Java null。
    ///
    /// # 返回
    /// Java `Set#size()` 对应的 `i32` 大小。
    pub fn size<T>(target: Option<&dyn SetView<T>>) -> Result<i32, ValidateError> {
        Validate::not_null(target, Some("Cannot get set size of null"))?;
        Ok(set_size(target.expect("validated target").len()))
    }

    /// 判断集合是否为 null 或没有元素。
    ///
    /// 对应 Java: `SetUtils#isEmpty(Set)`。
    ///
    /// # 参数
    /// - `target`：待判断集合；`None` 对应 Java null。
    ///
    /// # 返回
    /// target 为 null 或空集合时返回 `true`。
    #[must_use]
    pub fn is_empty<T>(target: Option<&dyn SetView<T>>) -> bool {
        target.is_none_or(SetView::is_empty)
    }

    /// 判断集合是否包含指定元素。
    ///
    /// 对应 Java: `SetUtils#contains(Set,Object)`。
    ///
    /// # 错误
    /// target 为 null 时返回上游精确参数错误。
    ///
    /// # 参数
    /// - `target`：目标集合；
    /// - `element`：待查找元素。
    ///
    /// # 返回
    /// 集合包含元素时返回 `true`。
    pub fn contains<T>(
        target: Option<&dyn SetView<T>>,
        element: &T,
    ) -> Result<bool, ValidateError> {
        Validate::not_null(target, Some("Cannot execute set contains: target is null"))?;
        Ok(target.expect("validated target").contains(element))
    }

    /// 判断集合是否包含数组中的全部元素。
    ///
    /// 对应 Java: `SetUtils#containsAll(Set,Object[])`。
    ///
    /// # 错误
    /// 严格按照 Java 顺序先校验 target，再校验 elements。
    ///
    /// # 参数
    /// - `target`：目标集合；
    /// - `elements`：待检查元素数组；`None` 对应 Java null。
    ///
    /// # 返回
    /// 所有元素均存在时返回 `true`。
    pub fn contains_all_array<T>(
        target: Option<&dyn SetView<T>>,
        elements: Option<&[T]>,
    ) -> Result<bool, ValidateError> {
        Validate::not_null(
            target,
            Some("Cannot execute set containsAll: target is null"),
        )?;
        Validate::not_null(
            elements,
            Some("Cannot execute set containsAll: elements is null"),
        )?;
        let target = target.expect("validated target");
        Ok(elements
            .expect("validated elements")
            .iter()
            .all(|element| target.contains(element)))
    }

    /// 判断集合是否包含 Collection 中的全部元素。
    ///
    /// 对应 Java: `SetUtils#containsAll(Set,Collection)`。
    ///
    /// # 错误
    /// 严格按照 Java 顺序先校验 target，再校验 elements；注意该重载的
    /// target 错误消息与数组重载不同，保持上游原文。
    ///
    /// # 参数
    /// - `target`：目标集合；
    /// - `elements`：待检查 Collection 迭代器；`None` 对应 Java null。
    ///
    /// # 返回
    /// 所有元素均存在时返回 `true`。
    pub fn contains_all_collection<'a, T, I>(
        target: Option<&dyn SetView<T>>,
        elements: Option<I>,
    ) -> Result<bool, ValidateError>
    where
        T: 'a,
        I: IntoIterator<Item = &'a T>,
    {
        Validate::not_null(target, Some("Cannot execute set contains: target is null"))?;
        Validate::not_null(
            elements.as_ref(),
            Some("Cannot execute set containsAll: elements is null"),
        )?;
        let target = target.expect("validated target");
        Ok(elements
            .expect("validated elements")
            .into_iter()
            .all(|element| target.contains(element)))
    }

    /// 创建包含一个元素的不可修改集合。
    ///
    /// 对应 Java: `SetUtils#singletonSet(Object)`。
    ///
    /// Java 允许 null 元素；Rust 可用 `Option<T>` 作为 `T` 保留该语义。
    ///
    /// # 参数
    /// - `element`：唯一元素。
    ///
    /// # 返回
    /// 不暴露修改入口的单元素集合。
    #[must_use]
    pub fn singleton_set<T>(element: T) -> SetValue<'static, T>
    where
        T: Eq + Hash,
    {
        let mut target = IndexSet::with_capacity(1);
        target.insert(element);
        SetValue::owned(target)
    }
}

fn set_size(size: usize) -> i32 {
    i32::try_from(size).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use indexmap::IndexSet;

    use super::{SetTarget, SetUtils, SetUtilsError, SetValue, SetView};
    use crate::util::ValidateError;

    fn ordered_set() -> IndexSet<Option<String>> {
        IndexSet::from([Some("one".to_owned()), Some("two".to_owned()), None])
    }

    #[test]
    fn converts_set_array_and_iterable_with_java_identity_and_order() {
        let source = ordered_set();
        let source_view: &dyn SetView<Option<String>> = &source;
        let borrowed = SetUtils::to_set(Some(SetTarget::Set(source_view))).unwrap();
        assert!(borrowed.is_borrowed_from(source_view));
        assert_eq!(
            borrowed.iter().cloned().collect::<Vec<_>>(),
            source.iter().cloned().collect::<Vec<_>>()
        );

        let array = [
            Some("two".to_owned()),
            Some("one".to_owned()),
            Some("two".to_owned()),
            None,
        ];
        let converted = SetUtils::to_set(Some(SetTarget::Array(&array))).unwrap();
        assert_eq!(
            converted.iter().cloned().collect::<Vec<_>>(),
            vec![Some("two".to_owned()), Some("one".to_owned()), None]
        );

        let iterable = vec![
            Some("a".to_owned()),
            Some("a".to_owned()),
            Some("b".to_owned()),
        ];
        let converted =
            SetUtils::to_set(Some(SetTarget::Iterable(Box::new(iterable.into_iter())))).unwrap();
        assert_eq!(
            converted.iter().cloned().collect::<Vec<_>>(),
            vec![Some("a".to_owned()), Some("b".to_owned())]
        );
    }

    #[test]
    fn preserves_conversion_errors() {
        assert_eq!(
            SetUtils::to_set(None::<SetTarget<'_, Option<String>>>)
                .err()
                .expect("null error"),
            SetUtilsError::Validation(ValidateError::IllegalArgument {
                message: Some("Cannot convert null to set".to_owned())
            })
        );
        assert_eq!(
            SetUtils::to_set(Some(SetTarget::<Option<String>>::Unsupported(
                "java.lang.Integer",
            )))
            .err()
            .expect("unsupported error"),
            SetUtilsError::CannotConvert {
                class_name: "java.lang.Integer".to_owned()
            }
        );
        assert_eq!(
            SetUtils::to_set(Some(SetTarget::<Option<String>>::PrimitiveArray("[I")))
                .err()
                .expect("primitive array error"),
            SetUtilsError::ClassCast {
                class_name: "[I".to_owned()
            }
        );
    }

    #[test]
    fn preserves_size_empty_contains_and_validation() {
        let source = ordered_set();
        let source_view: &dyn SetView<Option<String>> = &source;
        let empty = HashSet::<Option<String>>::new();
        let empty_view: &dyn SetView<Option<String>> = &empty;

        assert_eq!(SetUtils::size(Some(source_view)), Ok(3));
        assert_eq!(
            SetUtils::size(None::<&dyn SetView<Option<String>>>),
            Err(ValidateError::IllegalArgument {
                message: Some("Cannot get set size of null".to_owned())
            })
        );
        assert!(!SetUtils::is_empty(Some(source_view)));
        assert!(SetUtils::is_empty(Some(empty_view)));
        assert!(SetUtils::is_empty(None::<&dyn SetView<Option<String>>>));
        assert_eq!(super::set_size(usize::MAX), i32::MAX);

        assert_eq!(
            SetUtils::contains(Some(source_view), &Some("one".to_owned())),
            Ok(true)
        );
        assert_eq!(
            SetUtils::contains(Some(source_view), &Some("missing".to_owned())),
            Ok(false)
        );
        assert_eq!(SetUtils::contains(Some(source_view), &None), Ok(true));
        assert!(
            SetUtils::contains(
                None::<&dyn SetView<Option<String>>>,
                &Some("one".to_owned())
            )
            .is_err()
        );
    }

    #[test]
    fn preserves_contains_all_overloads_and_validation_order() {
        let source = ordered_set();
        let source_view: &dyn SetView<Option<String>> = &source;
        let present = [Some("one".to_owned()), None];
        let missing = [Some("one".to_owned()), Some("missing".to_owned())];

        assert_eq!(
            SetUtils::contains_all_array(Some(source_view), Some(&present)),
            Ok(true)
        );
        assert_eq!(
            SetUtils::contains_all_array(Some(source_view), Some(&missing)),
            Ok(false)
        );
        assert_eq!(
            SetUtils::contains_all_array(Some(source_view), Some(&[])),
            Ok(true)
        );
        assert_eq!(
            SetUtils::contains_all_collection(Some(source_view), Some(present.iter())),
            Ok(true)
        );
        assert_eq!(
            SetUtils::contains_all_collection(Some(source_view), Some(missing.iter())),
            Ok(false)
        );

        let array_target_error = SetUtils::contains_all_array(
            None::<&dyn SetView<Option<String>>>,
            None::<&[Option<String>]>,
        )
        .unwrap_err();
        assert_eq!(
            array_target_error.get_message(),
            Some("Cannot execute set containsAll: target is null")
        );
        let collection_target_error = SetUtils::contains_all_collection(
            None::<&dyn SetView<Option<String>>>,
            None::<std::slice::Iter<'_, Option<String>>>,
        )
        .unwrap_err();
        assert_eq!(
            collection_target_error.get_message(),
            Some("Cannot execute set contains: target is null")
        );
        assert!(
            SetUtils::contains_all_array(Some(source_view), None::<&[Option<String>]>).is_err()
        );
        assert!(
            SetUtils::contains_all_collection(
                Some(source_view),
                None::<std::slice::Iter<'_, Option<String>>>
            )
            .is_err()
        );
    }

    #[test]
    fn singleton_is_read_only_and_accepts_java_null_equivalent() {
        let singleton = SetUtils::singleton_set(None::<String>);
        assert_eq!(singleton.len(), 1);
        assert!(singleton.contains(&None));
        assert_eq!(singleton.iter().cloned().collect::<Vec<_>>(), vec![None]);
        assert!(!singleton.is_empty());
        assert!(!singleton.is_borrowed_from(&ordered_set()));

        let singleton_view: &dyn SetView<Option<String>> = &singleton;
        assert_eq!(singleton_view.len(), 1);
        assert!(singleton_view.contains(&None));
        assert_eq!(
            singleton_view.iter().cloned().collect::<Vec<_>>(),
            vec![None]
        );
        let _: &SetValue<'_, Option<String>> = &singleton;
    }

    #[test]
    fn supports_hash_and_tree_set_views_and_borrowed_java_set_operations() {
        let hash = HashSet::from([Some("two".to_owned()), Some("one".to_owned())]);
        let hash_view: &dyn SetView<Option<String>> = &hash;
        assert!(hash_view.contains(&Some("one".to_owned())));
        assert_eq!(hash_view.iter().count(), 2);

        let tree =
            std::collections::BTreeSet::from([Some("two".to_owned()), Some("one".to_owned())]);
        let tree_view: &dyn SetView<Option<String>> = &tree;
        assert_eq!(tree_view.len(), 2);
        assert!(tree_view.contains(&Some("two".to_owned())));

        let borrowed = SetUtils::to_set(Some(SetTarget::Set(tree_view))).unwrap();
        assert_eq!(borrowed.len(), 2);
        assert!(!borrowed.is_empty());
        assert!(borrowed.contains(&Some("one".to_owned())));
    }
}
