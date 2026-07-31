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
    type_name: &'static str,
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
            type_name: std::any::type_name::<T>(),
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

    /// 返回配置日志使用的 Java `Object#toString()` 等价诊断文本。
    ///
    /// 常用标量和字符串按值输出；其他执行组件没有通用 Rust `Display` 合同，因此
    /// 返回其稳定具体类型名，避免把 trait-object 地址或调试内存表示写入日志。
    ///
    /// # 返回值
    ///
    /// 返回不包含对象地址的稳定诊断字符串。
    #[must_use]
    pub fn diagnostic_string(&self) -> String {
        macro_rules! scalar {
            ($type:ty) => {
                if let Some(value) = self.downcast_ref::<$type>() {
                    return value.to_string();
                }
            };
        }

        if let Some(value) = self.downcast_ref::<String>() {
            return value.clone();
        }
        if let Some(value) = self.downcast_ref::<crate::util::JavaString>() {
            return value.to_string_lossy();
        }
        scalar!(bool);
        scalar!(i8);
        scalar!(i16);
        scalar!(i32);
        scalar!(i64);
        scalar!(isize);
        scalar!(u8);
        scalar!(u16);
        scalar!(u32);
        scalar!(u64);
        scalar!(usize);
        scalar!(f32);
        scalar!(f64);
        scalar!(char);
        self.type_name.to_owned()
    }
}
