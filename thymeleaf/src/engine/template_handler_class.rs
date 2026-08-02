use std::error::Error;
use std::fmt;

use super::ITemplateHandler;

/// Handler 零参数构造失败时保留的动态原因。
pub type TemplateHandlerConstructorError = Box<dyn Error + Send + Sync + 'static>;

/// 创建一个全新 Handler 实例的零参数构造函数。
pub type TemplateHandlerConstructor =
    fn() -> Result<Box<dyn ITemplateHandler>, TemplateHandlerConstructorError>;

/// Rust 对 Java `Class<? extends ITemplateHandler>` 的类型安全等价表示。
///
/// 类型令牌把稳定类名与零参数构造函数绑定在同一个不可变值中，避免调用方把一个
/// Handler 的工厂与另一个 Handler 的类名错误组合。每次构造都必须产生一个独立实例；
/// 构造错误由模板处理链保留为原因。
///
/// 这是 Rust 运行时扩展，不计入 Thymeleaf Java 主对象分母。
#[derive(Clone, Copy)]
pub struct TemplateHandlerClass {
    name: &'static str,
    implements_template_handler: bool,
    constructor: Option<TemplateHandlerConstructor>,
}

impl TemplateHandlerClass {
    /// 创建 Handler 类型令牌。
    ///
    /// # 参数
    ///
    /// - `name`：稳定、完整的 Handler 类名；
    /// - `constructor`：每次调用都创建全新 Handler 的零参数构造函数。
    ///
    /// # 返回值
    ///
    /// 返回把类身份与构造行为绑定在一起的不可变类型令牌。
    #[must_use]
    pub const fn new(name: &'static str, constructor: TemplateHandlerConstructor) -> Self {
        Self {
            name,
            implements_template_handler: true,
            constructor: Some(constructor),
        }
    }

    /// 从 Java Class 的可观察元数据创建类型令牌。
    ///
    /// 该入口用于迁移第三方方言可能返回的非法 Class：`implements_template_handler`
    /// 对应 `ITemplateHandler.class.isAssignableFrom(handlerClass)`，
    /// `constructor` 为 `None` 对应缺少公开零参数构造器。正常 Rust 调用应优先使用
    /// [`TemplateHandlerClass::new`]。
    ///
    /// # 参数
    ///
    /// - `name`：Java Class 的完整名称；
    /// - `implements_template_handler`：是否可赋值给 `ITemplateHandler`；
    /// - `constructor`：可选的公开零参数构造能力。
    ///
    /// # 返回值
    ///
    /// 返回可表示合法或非法第三方 Java Class 状态的类型令牌。
    #[must_use]
    pub const fn from_java_class_metadata(
        name: &'static str,
        implements_template_handler: bool,
        constructor: Option<TemplateHandlerConstructor>,
    ) -> Self {
        Self {
            name,
            implements_template_handler,
            constructor,
        }
    }

    /// 返回稳定、完整的 Handler 类名。
    ///
    /// # 返回值
    ///
    /// 返回类型令牌创建时登记的类名。
    #[must_use]
    pub const fn get_name(&self) -> &'static str {
        self.name
    }

    /// 判断该 Java Class 是否实现 `ITemplateHandler`。
    ///
    /// 返回 Java `isAssignableFrom` 对应的布尔结果。
    #[must_use]
    pub const fn implements_template_handler(&self) -> bool {
        self.implements_template_handler
    }

    /// 判断该 Java Class 是否声明公开零参数构造器。
    ///
    /// 返回是否能够执行公开零参数构造。
    #[must_use]
    pub const fn has_zero_argument_constructor(&self) -> bool {
        self.constructor.is_some()
    }

    /// 使用零参数构造函数创建一个全新 Handler。
    ///
    /// # 返回值
    ///
    /// 构造成功时返回独立 Handler 实例。
    ///
    /// # 错误
    ///
    /// Handler 构造函数拒绝创建实例时返回其原始动态错误。
    pub fn new_instance(
        &self,
    ) -> Result<Box<dyn ITemplateHandler>, TemplateHandlerConstructorError> {
        let constructor = self.constructor.ok_or_else(|| {
            Box::new(MissingTemplateHandlerConstructor { name: self.name })
                as TemplateHandlerConstructorError
        })?;
        constructor()
    }
}

#[derive(Debug)]
struct MissingTemplateHandlerConstructor {
    name: &'static str,
}

impl fmt::Display for MissingTemplateHandlerConstructor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Handler class {} does not implement required zero-argument constructor",
            self.name
        )
    }
}

impl Error for MissingTemplateHandlerConstructor {}
