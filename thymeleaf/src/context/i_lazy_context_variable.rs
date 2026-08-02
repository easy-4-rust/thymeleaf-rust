/// 模板上下文中的惰性变量合同。
///
/// 对应 Java: `org.thymeleaf.context.ILazyContextVariable<T>`。
///
/// 惰性求值只对上下文的一级变量生效，例如 `${lazy}`；嵌套在其他对象中的
/// `${container.lazy}` 不由 Thymeleaf 上下文解析器自动展开。
pub trait ILazyContextVariable<T> {
    /// 返回惰性变量的已解析值。
    ///
    /// 对应 Java: `ILazyContextVariable#getValue()`。
    ///
    /// # 返回
    ///
    /// 实现负责加载并保存的值；共享借用保持 Java 重复返回同一对象的身份语义。
    fn get_value(&self) -> &T;
}
