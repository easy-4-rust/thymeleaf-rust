use crate::util::JavaString;

/// 保存消息解析器名称和执行顺序的抽象基类状态。
///
/// 对应 Java: `org.thymeleaf.messageresolver.AbstractMessageResolver`。
///
/// Java 类型是抽象类，只实现 `IMessageResolver#getName` 与 `getOrder` 的公共
/// 状态，不提供消息解析逻辑。Rust 使用组合代替继承，由具体解析器持有本对象。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbstractMessageResolver {
    name: Option<JavaString>,
    order: Option<i32>,
}

impl AbstractMessageResolver {
    /// 创建默认名称为具体 Java 类名、顺序为 null 的解析器。
    ///
    /// `java_class_name` 对应 Java 构造期间 `this.getClass().getName()` 的结果，
    /// 因而由具体 Rust 解析器传入其 Java 对象全限定名。
    ///
    /// # 参数
    ///
    /// - `java_class_name`：具体解析器对应的 Java 全限定类名。
    ///
    /// # 返回值
    ///
    /// 名称已初始化、顺序仍为 `None` 的公共解析器状态。
    #[must_use]
    pub fn new(java_class_name: &str) -> Self {
        Self {
            name: Some(JavaString::from_rust_str(java_class_name)),
            order: None,
        }
    }

    /// 返回可空的解析器名称。
    ///
    /// 对应 Java: `AbstractMessageResolver#getName()`。
    ///
    /// # 返回值
    ///
    /// 当前名称；`None` 对应 Java `null`。
    #[must_use]
    pub const fn get_name(&self) -> Option<&JavaString> {
        self.name.as_ref()
    }

    /// 设置可空解析器名称。
    ///
    /// 对应 Java: `AbstractMessageResolver#setName(String)`。
    ///
    /// # 参数
    ///
    /// - `name`：新名称；允许用 `None` 保留 Java 可空语义。
    pub fn set_name(&mut self, name: Option<JavaString>) {
        self.name = name;
    }

    /// 返回解析器在消息解析器链中的可空顺序。
    ///
    /// 对应 Java: `AbstractMessageResolver#getOrder()`。
    ///
    /// # 返回值
    ///
    /// 当前顺序；`None` 对应 Java `null`。
    #[must_use]
    pub const fn get_order(&self) -> Option<i32> {
        self.order
    }

    /// 设置可空链式顺序。
    ///
    /// 对应 Java: `AbstractMessageResolver#setOrder(Integer)`。
    ///
    /// # 参数
    ///
    /// - `order`：新的可空执行顺序。
    pub fn set_order(&mut self, order: Option<i32>) {
        self.order = order;
    }
}

#[cfg(test)]
mod tests {
    use crate::util::JavaString;

    use super::AbstractMessageResolver;

    #[test]
    fn preserves_java_name_and_order_state() {
        let mut resolver = AbstractMessageResolver::new("example.ChildResolver");
        assert_eq!(
            resolver.get_name(),
            Some(&JavaString::from_rust_str("example.ChildResolver"))
        );
        assert_eq!(resolver.get_order(), None);

        resolver.set_name(None);
        resolver.set_order(Some(-7));
        assert_eq!(resolver.get_name(), None);
        assert_eq!(resolver.get_order(), Some(-7));
    }
}
