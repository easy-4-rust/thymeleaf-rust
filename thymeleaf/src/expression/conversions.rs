use std::sync::{Arc, Weak};

use crate::context::IExpressionContext;
use crate::util::{JavaString, ValidateError};

use super::{
    JavaConversionResult, JavaConversionValue, JavaTargetClass, StandardConversionError,
    StandardExpressions, TemplateValue,
};

/// 在 Standard Expression 内执行类型转换。
///
/// 对应 Java: `org.thymeleaf.expression.Conversions`。
pub struct Conversions {
    /// Context 的弱引用避免被 ExpressionObjects 缓存后形成 Arc 引用环。
    context: Weak<dyn IExpressionContext>,
}

impl Conversions {
    /// 创建绑定表达式上下文的转换工具。
    /// 对应 Java 语义：`Conversions` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(context: Option<Arc<dyn IExpressionContext>>) -> Result<Self, ValidateError> {
        let context = context.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Context cannot be null".to_owned()),
        })?;
        Ok(Self {
            context: Arc::downgrade(&context),
        })
    }

    /// 按 Java 类名转换值；裸 `String` 解析为 `java.lang.String`。
    /// 对应 Java 语义：`Conversions` 的 `convert_by_class_name` 行为（Rust 侧辅助/私有路径）。
    pub fn convert_by_class_name<'a>(
        &'a self,
        target: Option<&'a TemplateValue>,
        class_name: Option<&JavaString>,
    ) -> Result<JavaConversionResult<'a>, StandardConversionError> {
        let class_name = class_name.ok_or_else(|| {
            StandardConversionError::Validation(ValidateError::IllegalArgument {
                message: Some("Class name cannot be null".to_owned()),
            })
        })?;
        let class_name = class_name.to_string_lossy();
        let target_class = if class_name == "String" || class_name == "java.lang.String" {
            JavaTargetClass::String
        } else if class_name.contains('.') {
            JavaTargetClass::Other(class_name)
        } else {
            JavaTargetClass::Other(format!("java.lang.{class_name}"))
        };
        self.convert(target, Some(&target_class))
    }

    /// 使用类型化目标类执行转换。
    /// 对应 Java: `Conversions#convert()`。
    pub fn convert<'a>(
        &'a self,
        target: Option<&'a TemplateValue>,
        target_class: Option<&JavaTargetClass>,
    ) -> Result<JavaConversionResult<'a>, StandardConversionError> {
        let context = self.context.upgrade().ok_or_else(|| {
            StandardConversionError::runtime(
                "org.thymeleaf.exceptions.TemplateProcessingException",
                "Expression context is no longer available",
            )
        })?;
        let service = StandardExpressions::get_conversion_service(context.get_configuration())
            .map_err(|error| {
                StandardConversionError::runtime(
                    "org.thymeleaf.exceptions.TemplateProcessingException",
                    error.to_string(),
                )
            })?;
        let value = match target {
            None | Some(TemplateValue::Null) => JavaConversionValue::Null,
            Some(TemplateValue::String(value) | TemplateValue::SafeHtml(value)) => {
                JavaConversionValue::String(value)
            }
            Some(value) => JavaConversionValue::Object(value),
        };
        service.convert(Some(context.as_any()), value, target_class)
    }
}
