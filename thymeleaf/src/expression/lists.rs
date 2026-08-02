use crate::util::{
    JavaComparable, JavaComparator, JavaList, ListTarget, ListUtils, ListUtilsError, ListView,
    ValidateError,
};

/// Thymeleaf 标准表达式中的列表操作对象。
///
/// 对应 Java: `org.thymeleaf.expression.Lists`。
///
/// 该无状态对象通常以 `#lists` 暴露，全部方法委托给 [`ListUtils`]。
#[derive(Debug, Default, Clone, Copy)]
pub struct Lists;

impl Lists {
    /// 创建列表表达式对象。
    ///
    /// 对应 Java: `Lists#Lists()`。
    ///
    /// # 返回
    /// 新的无状态 `#lists` 对象。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 将动态目标转换为列表。
    ///
    /// 对应 Java: `Lists#toList(Object)`。
    ///
    /// # 参数
    /// - `target`：动态目标；`None` 对应 Java null。
    ///
    /// # 返回
    /// 已有列表借用或新 ArrayList 等价值。
    ///
    /// # 错误
    /// 传播 [`ListUtils::to_list`] 的转换错误。
    pub fn to_list<'a, T>(
        &self,
        target: Option<ListTarget<'a, T>>,
    ) -> Result<JavaList<'a, T>, ListUtilsError>
    where
        T: Clone,
    {
        ListUtils::to_list(target)
    }

    /// 返回列表大小。
    ///
    /// 对应 Java: `Lists#size(List)`。
    ///
    /// # 参数
    /// - `target`：目标列表。
    ///
    /// # 返回
    /// 列表大小。
    ///
    /// # 错误
    /// target 为 null 时返回精确参数错误。
    pub fn size<T>(&self, target: Option<&dyn ListView<T>>) -> Result<i32, ValidateError> {
        ListUtils::size(target)
    }

    /// 判断列表是否为 null 或为空。
    ///
    /// 对应 Java: `Lists#isEmpty(List)`。
    ///
    /// # 参数
    /// - `target`：目标列表。
    ///
    /// # 返回
    /// null 或空列表返回 `true`。
    #[must_use]
    pub fn is_empty<T>(&self, target: Option<&dyn ListView<T>>) -> bool {
        ListUtils::is_empty(target)
    }

    /// 判断列表是否包含元素。
    ///
    /// 对应 Java: `Lists#contains(List,Object)`。
    ///
    /// # 参数
    /// - `target`：目标列表；
    /// - `element`：请求元素。
    ///
    /// # 返回
    /// 元素存在时返回 `true`。
    ///
    /// # 错误
    /// target 为 null 时返回精确参数错误。
    pub fn contains<T>(
        &self,
        target: Option<&dyn ListView<T>>,
        element: &T,
    ) -> Result<bool, ValidateError>
    where
        T: PartialEq,
    {
        ListUtils::contains(target, element)
    }

    /// 判断列表是否包含数组中的全部元素。
    ///
    /// 对应 Java: `Lists#containsAll(List,Object[])`。
    ///
    /// # 参数
    /// - `target`：目标列表；
    /// - `elements`：请求元素数组。
    ///
    /// # 返回
    /// 全部存在时返回 `true`。
    ///
    /// # 错误
    /// target 或 elements 为 null 时返回精确参数错误。
    pub fn contains_all_array<T>(
        &self,
        target: Option<&dyn ListView<T>>,
        elements: Option<&[T]>,
    ) -> Result<bool, ValidateError>
    where
        T: PartialEq,
    {
        ListUtils::contains_all_array(target, elements)
    }

    /// 判断列表是否包含 Collection 中的全部元素。
    ///
    /// 对应 Java: `Lists#containsAll(List,Collection)`。
    ///
    /// # 参数
    /// - `target`：目标列表；
    /// - `elements`：请求 Collection 迭代器。
    ///
    /// # 返回
    /// 全部存在时返回 `true`。
    ///
    /// # 错误
    /// target 或 elements 为 null 时返回精确参数错误。
    pub fn contains_all_collection<'a, T, I>(
        &self,
        target: Option<&dyn ListView<T>>,
        elements: Option<I>,
    ) -> Result<bool, ValidateError>
    where
        T: PartialEq + 'a,
        I: IntoIterator<Item = &'a T>,
    {
        ListUtils::contains_all_collection(target, elements)
    }

    /// 按自然顺序稳定排序列表副本。
    ///
    /// 对应 Java: `Lists#sort(List<T>)`。
    ///
    /// # 参数
    /// - `list`：待排序列表。
    ///
    /// # 返回
    /// 同类型构造或 ArrayList 回退的新列表。
    ///
    /// # 错误
    /// 传播自然排序错误。
    pub fn sort<T>(
        &self,
        list: Option<&dyn ListView<T>>,
    ) -> Result<JavaList<'static, T>, ListUtilsError>
    where
        T: Clone + JavaComparable + 'static,
    {
        ListUtils::sort(list)
    }

    /// 使用 nullable Comparator 稳定排序列表副本。
    ///
    /// 对应 Java: `Lists#sort(List<T>,Comparator<? super T>)`。
    ///
    /// # 参数
    /// - `list`：待排序列表；
    /// - `comparator`：Comparator；`None` 对应 Java null。
    ///
    /// # 返回
    /// 同类型构造或 ArrayList 回退的新列表。
    ///
    /// # 错误
    /// 传播 Comparator、自然排序或列表实现错误。
    pub fn sort_with_comparator<T>(
        &self,
        list: Option<&dyn ListView<T>>,
        comparator: Option<&mut dyn JavaComparator<T>>,
    ) -> Result<JavaList<'static, T>, ListUtilsError>
    where
        T: Clone + JavaComparable + 'static,
    {
        ListUtils::sort_with_comparator(list, comparator)
    }

    /// 使用非 null Comparator 排序非 Comparable 元素。
    ///
    /// # 参数
    /// - `list`：待排序列表；
    /// - `comparator`：非 null Comparator。
    ///
    /// # 返回
    /// 新的稳定排序列表。
    ///
    /// # 错误
    /// 传播 Comparator 或列表实现错误。
    /// 对应 Java 语义：`Lists` 的 `sort_with_required_comparator` 行为（Rust 侧辅助/私有路径）。
    pub fn sort_with_required_comparator<T>(
        &self,
        list: Option<&dyn ListView<T>>,
        comparator: &mut dyn JavaComparator<T>,
    ) -> Result<JavaList<'static, T>, ListUtilsError>
    where
        T: Clone + 'static,
    {
        ListUtils::sort_with_required_comparator(list, comparator)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::Lists;
    use crate::util::{JavaListType, ListTarget, ListView};

    #[test]
    fn delegates_every_java_operation_to_list_utils() {
        let lists = Lists::new();
        let source = vec![Some("b".to_owned()), Some("a".to_owned()), None];
        let view: &dyn ListView<Option<String>> = &source;
        let converted = lists
            .to_list(Some(ListTarget::List(view)))
            .expect("convert");
        assert!(converted.is_borrowed_from(view));
        assert_eq!(lists.size(Some(view)), Ok(3));
        assert!(!lists.is_empty(Some(view)));
        assert_eq!(lists.contains(Some(view), &None), Ok(true));
        assert_eq!(
            lists.contains_all_array(Some(view), Some(&[Some("a".to_owned()), None])),
            Ok(true)
        );
        assert_eq!(
            lists.contains_all_collection(Some(view), Some([Some("b".to_owned()), None].iter())),
            Ok(true)
        );

        let sortable = vec!["b".to_owned(), "a".to_owned()];
        let sortable_view: &dyn ListView<String> = &sortable;
        assert_eq!(
            lists.sort(Some(sortable_view)).unwrap().get(0),
            Some(&"a".to_owned())
        );
        let mut descending = |left: &String, right: &String| Ok(right.cmp(left));
        assert_eq!(
            lists
                .sort_with_comparator(Some(sortable_view), Some(&mut descending))
                .unwrap()
                .get(0),
            Some(&"b".to_owned())
        );
        let mut same = |_left: &String, _right: &String| Ok(Ordering::Equal);
        assert_eq!(
            lists
                .sort_with_required_comparator(Some(sortable_view), &mut same)
                .unwrap()
                .list_type(),
            JavaListType::ArrayList
        );
        assert!(Lists.is_empty(None::<&dyn ListView<Option<String>>>));
    }
}
