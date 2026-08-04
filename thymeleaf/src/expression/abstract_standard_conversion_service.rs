use std::any::Any;

use crate::util::Validate;

use super::{
    ConversionObject, ConversionResult, ConversionValue, IStandardConversionService,
    StandardConversionError, TargetClass, Utf16StringConversionResult,
};

/// 标准转换服务实现使用的抽象基类契约。
///
/// 本 trait 复现 Java 抽象类的模板方法：公开 `convert` 分派是不可覆写的，
/// 子类型只能定制 `convertToString` 与 `convertOther`。字符串目标的 null 和
/// 已有 String 快路径在钩子前完成。
///
/// 对应 Java:
/// `org.thymeleaf.standard.expression.AbstractStandardConversionService`。
pub trait AbstractStandardConversionService: Send + Sync {
    /// 将非 null、非 String 对象转换为字符串。
    ///
    /// 对应 Java:
    /// `AbstractStandardConversionService#convertToString(IExpressionContext,Object)`。
    ///
    /// # 参数
    /// - `context`：原样传给扩展实现的可空表达式上下文；
    /// - `object`：非 null 且运行时类型不是 String 的对象。
    ///
    /// # 返回
    /// 默认调用对象的 `toString()`，保留可空结果。
    ///
    /// # 错误
    /// 对象 `toString()` 抛出的运行时异常原样传播。
    fn convert_to_string<'a>(
        &self,
        _context: Option<&dyn Any>,
        object: &'a dyn ConversionObject,
    ) -> Result<Utf16StringConversionResult<'a>, StandardConversionError> {
        object.java_to_string()
    }

    /// 执行字符串之外的目标类型转换。
    ///
    /// 对应 Java:
    /// `AbstractStandardConversionService#convertOther(IExpressionContext,Object,Class<T>)`。
    ///
    /// # 参数
    /// - `context`：可空表达式上下文；
    /// - `object`：待转换的可空值；
    /// - `target_class`：非 null、非 String 目标类。
    ///
    /// # 返回
    /// 扩展实现转换后的动态结果。
    ///
    /// # 错误
    /// 默认实现始终返回包含目标 Java 类名的 `IllegalArgumentException` 等价错误。
    fn convert_other<'a>(
        &self,
        _context: Option<&dyn Any>,
        _object: ConversionValue<'a>,
        target_class: &TargetClass,
    ) -> Result<ConversionResult<'a>, StandardConversionError> {
        Err(StandardConversionError::NoAvailableConversion {
            target_class_name: target_class.get_name().to_owned(),
        })
    }
}

impl<T> IStandardConversionService for T
where
    T: AbstractStandardConversionService,
{
    fn convert<'a>(
        &self,
        context: Option<&dyn Any>,
        object: ConversionValue<'a>,
        target_class: Option<&TargetClass>,
    ) -> Result<ConversionResult<'a>, StandardConversionError> {
        Validate::not_null(target_class, Some("Target class cannot be null"))?;
        let target_class = target_class.expect("validated target class");

        // Java 对精确 String.class 执行最常见的快路径，null 与现有 String
        // 均不会调用可覆写钩子。
        if target_class == &TargetClass::String {
            return match object {
                ConversionValue::Null => Ok(ConversionResult::Null),
                ConversionValue::String(value) => Ok(ConversionResult::BorrowedString(value)),
                ConversionValue::Object(value) => {
                    self.convert_to_string(context, value).map(Into::into)
                }
            };
        }

        self.convert_other(context, object, target_class)
    }
}
