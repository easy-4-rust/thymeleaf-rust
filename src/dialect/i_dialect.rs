/// 所有 Thymeleaf 方言的基础契约。
///
/// 对应 Java: `org.thymeleaf.dialect.IDialect`。
///
/// 方言用于扩展模板处理能力；具体实现通常还会实现 Processor、前置处理、
/// 后置处理、表达式对象或执行属性等后续迁移的专用契约。本基础接口自身只公开
/// 方言名称。Java 接口没有声明名称一定非空，因此这里使用 `Option<&str>` 保留
/// 自定义实现返回 `null` 的可能性。
///
/// `Send + Sync` 是 Rust 并发渲染所需的宿主安全约束，不改变名称的可观察语义。
pub trait IDialect: Send + Sync {
    /// 返回方言名称。
    ///
    /// 对应 Java: `IDialect#getName()`。
    ///
    /// # 返回
    /// 方言名称；`None` 对应自定义 Java 实现返回 `null`。
    fn get_name(&self) -> Option<&str>;
}

#[cfg(test)]
mod tests {
    use super::IDialect;

    struct NullableNameDialect;

    impl IDialect for NullableNameDialect {
        fn get_name(&self) -> Option<&str> {
            None
        }
    }

    #[test]
    fn custom_dialect_can_preserve_a_java_null_name() {
        let dialect = NullableNameDialect;
        assert_eq!(dialect.get_name(), None);
    }
}
