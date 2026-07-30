use std::any::Any;
use std::sync::Arc;

/// 引擎执行属性中的任意 Java `Object` 等价值。
///
/// 对应 Java: `IEngineConfiguration#getExecutionAttributes()` 返回的
/// `Map<String,Object>` 值。执行属性不仅包含模板数据，还会注册表达式解析器、
/// 变量求值器和转换服务，因此不能收窄为 `TemplateValue`。
#[derive(Clone)]
pub struct ExecutionAttributeValue {
    value: Arc<dyn Any + Send + Sync>,
}

impl ExecutionAttributeValue {
    /// 包装一个可在线程间共享的执行属性。
    ///
    /// # 参数
    /// - `value`：Java 执行属性对象的 Rust 等价值。
    ///
    /// # 返回
    /// 保留具体运行时类型、可供后续安全下转的共享包装。
    pub fn new<T>(value: T) -> Self
    where
        T: Any + Send + Sync,
    {
        Self {
            value: Arc::new(value),
        }
    }

    /// 按具体运行时类型读取执行属性。
    ///
    /// # 类型参数
    /// - `T`：注册时使用的精确 Rust 类型。
    ///
    /// # 返回
    /// 类型一致时返回同一对象的借用，否则返回 `None`。
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Any + Send + Sync,
    {
        self.value.downcast_ref::<T>()
    }
}
