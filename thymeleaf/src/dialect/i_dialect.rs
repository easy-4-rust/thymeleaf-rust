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
pub trait IDialect: std::any::Any + Send + Sync {
    /// 返回用于复现 Java `getClass().getName()` 的稳定实现类名。
    ///
    /// 这是 Rust 动态类型适配入口，不增加 Java 接口方法；配置错误消息通过它保留
    /// 方言实现类身份，而不是误用面向用户的 [`IDialect::get_name`]。
    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// 返回具体方言的 Rust 运行时类型标识。
    fn dialect_type_id(&self) -> std::any::TypeId {
        std::any::Any::type_id(self)
    }

    /// 判断当前对象是否为 StandardDialect。
    ///
    /// 对应 Java 配置聚合阶段的 `dialect instanceof StandardDialect`。
    fn is_standard_dialect(&self) -> bool {
        false
    }

    /// 将 Java `instanceof IProcessorDialect` 暴露为对象安全能力查询。
    fn as_processor_dialect(&self) -> Option<&dyn super::IProcessorDialect> {
        None
    }

    /// 将 Java `instanceof IExecutionAttributeDialect` 暴露为对象安全能力查询。
    fn as_execution_attribute_dialect(&self) -> Option<&dyn super::IExecutionAttributeDialect> {
        None
    }

    /// 将 Java `instanceof IExpressionObjectDialect` 暴露为对象安全能力查询。
    fn as_expression_object_dialect(&self) -> Option<&dyn super::IExpressionObjectDialect> {
        None
    }

    /// 将 Java `instanceof IPreProcessorDialect` 暴露为对象安全能力查询。
    fn as_pre_processor_dialect(&self) -> Option<&dyn super::IPreProcessorDialect> {
        None
    }

    /// 将 Java `instanceof IPostProcessorDialect` 暴露为对象安全能力查询。
    fn as_post_processor_dialect(&self) -> Option<&dyn super::IPostProcessorDialect> {
        None
    }

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
