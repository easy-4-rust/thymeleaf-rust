use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::expression::{TemplateObject, TemplateObjectPropertyError, TemplateValue};
use crate::util::{JavaNumber, JavaString};

/// 迭代状态访问时产生的 Java 运行时错误。
///
/// 对应 Java: `org.thymeleaf.engine.IterationStatusVar` 中对可空 `Integer size`
/// 自动拆箱时可能抛出的异常。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IterationStatusVarError {
    /// `isLast()` 对 null `size` 执行自动拆箱。
    NullSize,
}

impl IterationStatusVarError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        "java.lang.NullPointerException"
    }
}

impl Display for IterationStatusVarError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(
            "Cannot invoke \"java.lang.Integer.intValue()\" because \"this.size\" is null",
        )
    }
}

impl Error for IterationStatusVarError {}

struct IterationStatusState {
    index: i32,
    size: Option<i32>,
    current: Option<Arc<TemplateValue>>,
}

/// 模板循环中暴露给表达式的可变迭代状态对象。
///
/// 每次循环复用同一对象身份，只更新 index/current；表达式通过 JavaBean 属性实时
/// 读取最新状态。对应 Java: `org.thymeleaf.engine.IterationStatusVar`。
pub struct IterationStatusVar {
    state: RwLock<IterationStatusState>,
}

impl IterationStatusVar {
    /// 创建 index 为零、指定可空总大小且 current 为 null 的状态。
    ///
    /// 对应 Java 包内构造与字段初始化。
    #[must_use]
    pub fn new(size: Option<i32>) -> Self {
        Self {
            state: RwLock::new(IterationStatusState {
                index: 0,
                size,
                current: None,
            }),
        }
    }

    /// 返回从零开始的当前索引。
    #[must_use]
    pub fn get_index(&self) -> i32 {
        read_state(&self.state).index
    }

    /// 返回从一开始的当前计数。
    #[must_use]
    pub fn get_count(&self) -> i32 {
        self.get_index().wrapping_add(1)
    }

    /// 判断当前状态是否具有预先确定的总大小。
    #[must_use]
    pub fn has_size(&self) -> bool {
        read_state(&self.state).size.is_some()
    }

    /// 返回可空总大小。
    #[must_use]
    pub fn get_size(&self) -> Option<i32> {
        read_state(&self.state).size
    }

    /// 返回可空当前迭代对象并保持对象身份。
    #[must_use]
    pub fn get_current(&self) -> Option<Arc<TemplateValue>> {
        read_state(&self.state).current.clone()
    }

    /// 判断当前一基计数是否为偶数。
    #[must_use]
    pub fn is_even(&self) -> bool {
        self.get_count() % 2 == 0
    }

    /// 判断当前一基计数是否为奇数。
    #[must_use]
    pub fn is_odd(&self) -> bool {
        !self.is_even()
    }

    /// 判断当前元素是否为首个元素。
    #[must_use]
    pub fn is_first(&self) -> bool {
        self.get_index() == 0
    }

    /// 判断当前元素是否为最后一个元素。
    ///
    /// # 错误
    ///
    /// `size` 未知时保留 Java 自动拆箱产生的 `NullPointerException`。
    pub fn is_last(&self) -> Result<bool, IterationStatusVarError> {
        let state = read_state(&self.state);
        state
            .size
            .map(|size| state.index == size.wrapping_sub(1))
            .ok_or(IterationStatusVarError::NullSize)
    }

    /// 按 Java `toString()` 布局生成状态文本。
    #[must_use]
    pub fn to_java_string(&self) -> JavaString {
        let state = read_state(&self.state);
        let size = state
            .size
            .map_or_else(|| "null".to_owned(), |value| value.to_string());
        let current = state
            .current
            .as_deref()
            .and_then(TemplateValue::to_java_string)
            .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy());
        JavaString::from_rust_str(&format!(
            "{{index = {}, count = {}, size = {size}, current = {current}}}",
            state.index,
            state.index.wrapping_add(1)
        ))
    }

    pub(super) fn set_current(&self, current: Option<Arc<TemplateValue>>) {
        write_state(&self.state).current = current;
    }

    pub(super) fn increment_index(&self) {
        let mut state = write_state(&self.state);
        state.index = state.index.wrapping_add(1);
    }
}

impl TemplateObject for IterationStatusVar {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.engine.IterationStatusVar"
    }

    fn to_java_string(&self) -> JavaString {
        Self::to_java_string(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, TemplateObjectPropertyError>> {
        let value = match property_name.to_string_lossy().as_str() {
            "index" => Some(integer_value(self.get_index())),
            "count" => Some(integer_value(self.get_count())),
            "size" => self.get_size().map(integer_value),
            "current" => self.get_current(),
            "even" => Some(boolean_value(self.is_even())),
            "odd" => Some(boolean_value(self.is_odd())),
            "first" => Some(boolean_value(self.is_first())),
            "last" => match self.is_last() {
                Ok(value) => Some(boolean_value(value)),
                Err(error) => return Some(Err(Box::new(error))),
            },
            _ => return None,
        };
        Some(Ok(value))
    }
}

fn integer_value(value: i32) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Number(JavaNumber::Integer(value)))
}

fn boolean_value(value: bool) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Boolean(value))
}

fn read_state(lock: &RwLock<IterationStatusState>) -> RwLockReadGuard<'_, IterationStatusState> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_state(lock: &RwLock<IterationStatusState>) -> RwLockWriteGuard<'_, IterationStatusState> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::IterationStatusVar;
    use crate::expression::{TemplateObject, TemplateValue};
    use crate::util::JavaString;

    #[test]
    fn status_machine_overflow_and_java_bean_properties_match_java_golden() {
        let unknown = IterationStatusVar::new(None);
        assert_golden("unknown", &record(&unknown));
        assert_eq!(
            unknown
                .is_last()
                .expect_err("unknown Java size must fail")
                .to_string(),
            unknown_last_error()
        );
        assert_property(&unknown, "index", Some("0"));
        assert_property(&unknown, "count", Some("1"));
        assert_property(&unknown, "size", None);
        assert_property(&unknown, "current", None);
        assert_property(&unknown, "even", Some("false"));
        assert_property(&unknown, "odd", Some("true"));
        assert_property(&unknown, "first", Some("true"));
        let last = unknown
            .java_get_property(&JavaString::from_rust_str("last"))
            .expect("last property accessor")
            .expect_err("last property must preserve Java null-unboxing error");
        assert_eq!(last.to_string(), unknown_last_error());
        assert!(
            unknown
                .java_get_property(&JavaString::from_rust_str("missing"))
                .is_none()
        );

        let known = IterationStatusVar::new(Some(3));
        known.set_current(Some(Arc::new(TemplateValue::string(
            JavaString::from_rust_str("value"),
        ))));
        assert_golden("known0", &record(&known));
        assert_property(&known, "last", Some("false"));
        known.increment_index();
        assert_golden("known1", &record(&known));
        known.increment_index();
        known.set_current(None);
        assert_golden("known2", &record(&known));
        assert_property(&known, "last", Some("true"));

        let overflow = IterationStatusVar::new(Some(i32::MIN));
        {
            let mut state = super::write_state(&overflow.state);
            state.index = i32::MAX;
        }
        assert_golden("overflow", &record(&overflow));
        assert_property(&overflow, "count", Some("-2147483648"));
        assert_property(&overflow, "last", Some("true"));
    }

    fn record(status: &IterationStatusVar) -> String {
        let values = format!(
            "{},{},{},{},{},{},{},{}",
            status.get_index(),
            status.get_count(),
            status.has_size(),
            nullable(status.get_size().map(|value| value.to_string())),
            nullable(
                status
                    .get_current()
                    .and_then(|value| value.to_java_string())
                    .map(|value| value.to_string_lossy())
            ),
            status.is_even(),
            status.is_odd(),
            status.is_first(),
        );
        let last = status
            .is_last()
            .map(|value| value.to_string())
            .unwrap_or_else(|error| format!("{}:{}", error.java_class_name(), error));
        format!(
            "{values},last={last},text={}",
            status.to_java_string().to_string_lossy()
        )
    }

    fn assert_golden(key: &str, actual: &str) {
        let expected = include_str!("../../tests/fixtures/iteration_status_var_golden.txt")
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}=")))
            .expect("Java Golden record");
        assert_eq!(actual, expected, "Java Golden key {key}");
    }

    fn assert_property(status: &IterationStatusVar, property: &str, expected: Option<&str>) {
        let actual = status
            .java_get_property(&JavaString::from_rust_str(property))
            .expect("known JavaBean property")
            .expect("known property must not fail")
            .and_then(|value| value.to_java_string())
            .map(|value| value.to_string_lossy());
        assert_eq!(actual.as_deref(), expected, "property {property}");
    }

    fn nullable(value: Option<String>) -> String {
        value.unwrap_or_else(|| "null".to_owned())
    }

    fn unknown_last_error() -> String {
        include_str!("../../tests/fixtures/iteration_status_var_golden.txt")
            .lines()
            .find_map(|line| line.strip_prefix("unknown="))
            .and_then(|line| line.split(",last=").nth(1))
            .and_then(|line| line.split(",text=").next())
            .expect("Java Golden null-size error")
            .split_once(':')
            .map(|(_, message)| message.to_owned())
            .expect("Java class and exception message")
    }
}
