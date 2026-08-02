use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// 身份计数器构造失败。
///
/// 对应 Java: `java.util.IdentityHashMap(int)` 在
/// `org.thymeleaf.util.IdentityCounter#IdentityCounter(int)` 构造链中抛出的
/// `IllegalArgumentException`。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityCounterError {
    message: String,
}

impl IdentityCounterError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl Display for IdentityCounterError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for IdentityCounterError {}

/// 按引用身份记录对象是否已经出现的专用集合。
///
/// 对应 Java: `org.thymeleaf.util.IdentityCounter<T>`。
///
/// 与 Java `IdentityHashMap` 相同，两个值即使内容相等，只要不是同一对象引用，就被视为
/// 不同键。Rust 使用 `Rc::ptr_eq` 表达引用身份，并保存 `Rc` 强引用，确保已记录对象
/// 在计数器存活期间不会释放。JavaDoc 明确声明本对象非线程安全；`Rc` 使这一约束在
/// Rust 类型系统中得到保留。`None` 对应 Java `null`，也可以被记录。
pub struct IdentityCounter<T: ?Sized> {
    counted: HashMap<Option<IdentityKey<T>>, ()>,
}

impl<T: ?Sized> IdentityCounter<T> {
    /// 创建具有预期最大大小提示的身份计数器。
    ///
    /// 对应 Java: `IdentityCounter#IdentityCounter(int)`。
    ///
    /// # 参数
    /// - `expected_max_size`：传给 Java `IdentityHashMap(int)` 的容量提示。
    ///
    /// # 错误
    /// 负数时返回保留 Java 消息的类型化错误。
    pub fn new(expected_max_size: i32) -> Result<Self, IdentityCounterError> {
        if expected_max_size < 0 {
            return Err(IdentityCounterError::new(format!(
                "expectedMaxSize is negative: {expected_max_size}"
            )));
        }
        // Java 的参数只是容量提示，不属于业务状态。常用容量立即预留；极大提示按需
        // 增长，以保持 Java 对 Integer.MAX_VALUE 延迟分配并成功构造的行为。
        let initial_capacity = usize::try_from(expected_max_size)
            .unwrap_or_default()
            .min(1_024);
        Ok(Self {
            counted: HashMap::with_capacity(initial_capacity),
        })
    }

    /// 按引用身份记录一个对象。
    ///
    /// 对应 Java: `IdentityCounter#count(Object)`。
    ///
    /// # 参数
    /// - `object`：要记录的共享对象；`None` 对应 Java `null`。
    pub fn count(&mut self, object: Option<Rc<T>>) {
        self.counted.insert(object.map(IdentityKey), ());
    }

    /// 判断同一对象引用是否已经被记录。
    ///
    /// 对应 Java: `IdentityCounter#isAlreadyCounted(Object)`。
    ///
    /// # 参数
    /// - `object`：要按身份查询的对象；`None` 对应 Java `null`。
    ///
    /// # 返回
    /// 相同 `Rc` 分配身份或 `null` 已经出现时返回 `true`。值相等但分配不同返回
    /// `false`。
    #[must_use]
    pub fn is_already_counted(&self, object: Option<&Rc<T>>) -> bool {
        self.counted
            .contains_key(&object.map(|value| IdentityKey(Rc::clone(value))))
    }
}

struct IdentityKey<T: ?Sized>(Rc<T>);

impl<T: ?Sized> PartialEq for IdentityKey<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: ?Sized> Eq for IdentityKey<T> {}

impl<T: ?Sized> Hash for IdentityKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Rc::as_ptr(&self.0), state);
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::IdentityCounter;

    #[test]
    fn rejects_java_identity_hash_map_capacity_boundaries() {
        assert_eq!(
            IdentityCounter::<String>::new(-1)
                .err()
                .expect("negative error")
                .to_string(),
            "expectedMaxSize is negative: -1"
        );
        assert!(IdentityCounter::<String>::new(0).is_ok());
        assert!(IdentityCounter::<String>::new(i32::MAX).is_ok());
    }

    #[test]
    fn distinguishes_equal_values_by_reference_identity() {
        let first = Rc::new("same".to_owned());
        let equal_but_distinct = Rc::new("same".to_owned());
        let first_alias = Rc::clone(&first);
        let mut counter = IdentityCounter::new(2).expect("counter");

        assert!(!counter.is_already_counted(Some(&first)));
        counter.count(Some(Rc::clone(&first)));
        assert!(counter.is_already_counted(Some(&first)));
        assert!(counter.is_already_counted(Some(&first_alias)));
        assert!(!counter.is_already_counted(Some(&equal_but_distinct)));

        counter.count(Some(equal_but_distinct));
        counter.count(Some(first));
        assert_eq!(counter.counted.len(), 2);
    }

    #[test]
    fn counts_java_null_once_and_keeps_non_null_separate() {
        let value = Rc::new("value".to_owned());
        let mut counter = IdentityCounter::new(1).expect("counter");

        assert!(!counter.is_already_counted(None));
        counter.count(None);
        counter.count(None);
        assert!(counter.is_already_counted(None));
        assert!(!counter.is_already_counted(Some(&value)));
        counter.count(Some(Rc::clone(&value)));
        assert!(counter.is_already_counted(Some(&value)));
        assert_eq!(counter.counted.len(), 2);
    }
}
