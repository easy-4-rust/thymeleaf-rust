use std::any::Any;
use std::fmt::{Display, Formatter};

use thiserror::Error;

use crate::util::{Utf16String, ValidateError};

/// Java 对象参与标准转换时提供的动态行为。
///
/// 这是 `java.lang.Object` 的紧耦合 Rust 适配，只承接默认转换服务实际调用的
/// `toString()` 行为。实现可以返回 Java null，也可以传播原始运行时异常。
/// 对应 Java 语义：`IStandardConversionService` 的 Rust 侧类型 `JavaConversionObject`。
pub trait JavaConversionObject: Any {
    /// 执行 Java `Object#toString()` 等价调用。
    ///
    /// # 返回
    /// 返回可空、借用或新建的 Java UTF-16 字符串，保留覆写方法返回既有
    /// String 实例时的引用身份。
    ///
    /// # 错误
    /// `toString()` 抛出的运行时异常必须按原类别和消息返回。
    fn java_to_string(&self) -> Result<Utf16StringConversionResult<'_>, StandardConversionError>;
}

/// 传给标准转换服务的 Java 运行时值分类。
///
/// Rust 需要显式区分 null、`java.lang.String` 与其他对象，才能保留
/// `AbstractStandardConversionService#convert` 的 `instanceof String` 快路径和
/// 字符串引用身份。
#[derive(Clone, Copy)]
/// 对应 Java 语义：`IStandardConversionService` 的 Rust 侧类型 `JavaConversionValue`。
pub enum JavaConversionValue<'a> {
    /// Java null。
    Null,
    /// `java.lang.String` 的借用。
    String(&'a Utf16String),
    /// 其他 Java 对象的动态借用。
    Object(&'a dyn JavaConversionObject),
}

/// Java `Class<T>` 目标类型适配。
///
/// 默认服务只特殊处理精确的 `String.class`，其他类型仅需保留
/// `Class#getName()` 文本以生成完全一致的不可用转换错误。
#[derive(Clone, Debug, Eq, PartialEq)]
/// 对应 Java 语义：`IStandardConversionService` 的 Rust 侧类型 `JavaTargetClass`。
pub enum JavaTargetClass {
    /// 精确的 `java.lang.String` 类。
    String,
    /// 其他目标类及其 Java 二进制名称。
    Other(String),
}

impl JavaTargetClass {
    /// 返回 Java `Class#getName()`。
    ///
    /// # 返回
    /// `java.lang.String` 或构造时保存的其他二进制类名。
    #[must_use]
    /// 对应 Java 语义：Java 接口/超类方法 `getName()` 的 Rust 移植（`IStandardConversionService` 继承路径）。
    pub fn get_name(&self) -> &str {
        match self {
            Self::String => "java.lang.String",
            Self::Other(name) => name,
        }
    }
}

/// `convertToString` 钩子的可空、借用或新建结果。
///
/// 借用分支供框架转换服务保留既有字符串身份；默认 `Object#toString()` 通常返回
/// 新字符串，因此使用拥有分支。
/// 对应 Java 语义：`IStandardConversionService` 的 Rust 侧类型 `Utf16StringConversionResult`。
pub enum Utf16StringConversionResult<'a> {
    /// Java null。
    Null,
    /// 借用的既有字符串。
    Borrowed(&'a Utf16String),
    /// 新创建或由转换器返回的字符串。
    Owned(Utf16String),
}

/// 标准转换服务的动态返回值。
///
/// Java 泛型 `<T>` 和 `Class<T>` 在运行时仍通过对象引用传递。该适配保留 null、
/// 字符串借用/拥有状态，以及扩展转换器返回其他类型时的借用或拥有对象。
/// 对应 Java 语义：`IStandardConversionService` 的 Rust 侧类型 `JavaConversionResult`。
pub enum JavaConversionResult<'a> {
    /// Java null。
    Null,
    /// 原字符串的同一引用。
    BorrowedString(&'a Utf16String),
    /// 新建字符串。
    OwnedString(Utf16String),
    /// 扩展转换器返回的既有对象引用。
    BorrowedObject(&'a dyn Any),
    /// 扩展转换器新建的对象。
    OwnedObject(Box<dyn Any>),
}

impl<'a> From<Utf16StringConversionResult<'a>> for JavaConversionResult<'a> {
    fn from(result: Utf16StringConversionResult<'a>) -> Self {
        match result {
            Utf16StringConversionResult::Null => Self::Null,
            Utf16StringConversionResult::Borrowed(value) => Self::BorrowedString(value),
            Utf16StringConversionResult::Owned(value) => Self::OwnedString(value),
        }
    }
}

/// 标准表达式转换时可观察的 Java 异常。
#[derive(Debug, Error, Eq, PartialEq)]
/// 对应 Java 语义：`IStandardConversionService` 的 Rust 侧类型 `StandardConversionError`。
pub enum StandardConversionError {
    /// `Validate.notNull(targetClass, ...)` 的参数错误。
    #[error(transparent)]
    Validation(#[from] ValidateError),
    /// 默认转换服务没有对应目标类转换。
    #[error("No available conversion for target class \"{target_class_name}\"")]
    NoAvailableConversion {
        /// Java `Class#getName()`。
        target_class_name: String,
    },
    /// 源对象 `toString()` 或扩展转换器抛出的运行时异常。
    #[error("{message}")]
    Runtime {
        /// 原 Java 异常类名。
        exception_class_name: String,
        /// 原 Java detail message；null 以空显示文本表达。
        message: String,
    },
}

impl StandardConversionError {
    /// 返回对应 Java 异常类名。
    ///
    /// # 返回
    /// 参数/不可用转换返回 `IllegalArgumentException`；运行时异常返回保存的类名。
    #[must_use]
    pub fn java_class_name(&self) -> &str {
        match self {
            Self::Validation(error) => error.java_class_name(),
            Self::NoAvailableConversion { .. } => "java.lang.IllegalArgumentException",
            Self::Runtime {
                exception_class_name,
                ..
            } => exception_class_name,
        }
    }

    /// 创建保留 Java 异常类别和消息的运行时错误。
    ///
    /// # 参数
    /// - `exception_class_name`：Java 异常类名；
    /// - `message`：可观察的 detail message，null 使用空字符串。
    ///
    /// # 返回
    /// 可由对象或扩展转换器传播的错误。
    #[must_use]
    /// 对应 Java 语义：`IStandardConversionService` 的 `runtime` 行为（Rust 侧辅助/私有路径）。
    pub fn runtime(exception_class_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Runtime {
            exception_class_name: exception_class_name.into(),
            message: message.into(),
        }
    }
}

/// 模板执行期间使用的标准表达式转换服务契约。
///
/// 转换机制与 Spring `ConversionService` 类似但保持框架中立。实现必须至少支持
/// 任意对象到字符串的转换，并且应当线程安全。
///
/// 对应 Java:
/// `org.thymeleaf.standard.expression.IStandardConversionService`。
pub trait IStandardConversionService: Send + Sync {
    /// 将对象转换到指定目标类。
    ///
    /// 对应 Java:
    /// `IStandardConversionService#convert(IExpressionContext,Object,Class<T>)`。
    ///
    /// # 参数
    /// - `context`：表达式上下文；默认实现允许 Java null，且不读取它；
    /// - `object`：待转换的可空 Java 运行时值；
    /// - `target_class`：目标类；`None` 对应 Java null。
    ///
    /// # 返回
    /// 转换后的可空、借用或拥有结果。
    ///
    /// # 错误
    /// 目标类为 null、没有可用转换，或源对象/扩展转换器抛出异常时返回对应类别。
    fn convert<'a>(
        &self,
        context: Option<&dyn Any>,
        object: JavaConversionValue<'a>,
        target_class: Option<&JavaTargetClass>,
    ) -> Result<JavaConversionResult<'a>, StandardConversionError>;
}

impl Display for JavaTargetClass {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.get_name())
    }
}
