use std::sync::Arc;

use crate::IEngineConfiguration;
use crate::exceptions::TemplateProcessingException;
use crate::util::JavaString;

use super::{
    IStandardConversionService, IStandardExpressionParser, IStandardVariableExpressionEvaluator,
    StandardExpressionResult,
};

/// Standard Expression 注册服务的类型安全访问入口。
///
/// 对应 Java: `org.thymeleaf.standard.expression.StandardExpressions`。
pub struct StandardExpressions;

impl StandardExpressions {
    /// 变量表达式求值器的执行属性名称。
    pub const STANDARD_VARIABLE_EXPRESSION_EVALUATOR_ATTRIBUTE_NAME: &'static str =
        "StandardVariableExpressionEvaluator";
    /// Standard Expression Parser 的执行属性名称。
    pub const STANDARD_EXPRESSION_PARSER_ATTRIBUTE_NAME: &'static str = "StandardExpressionParser";
    /// Standard Conversion Service 的执行属性名称。
    pub const STANDARD_CONVERSION_SERVICE_ATTRIBUTE_NAME: &'static str =
        "StandardConversionService";

    /// 取得当前 Standard Dialect 注册的表达式解析器。
    ///
    /// # 参数
    /// - `configuration`：当前模板执行配置。
    ///
    /// # 错误
    /// 属性不存在、为 Java null 或运行时类型不正确时返回模板处理错误。
    pub fn get_expression_parser(
        configuration: &dyn IEngineConfiguration,
    ) -> StandardExpressionResult<Arc<dyn IStandardExpressionParser>> {
        get_registered_attribute::<Arc<dyn IStandardExpressionParser>>(
            configuration,
            Self::STANDARD_EXPRESSION_PARSER_ATTRIBUTE_NAME,
            "No Standard Expression Parser has been registered as an execution argument. \
             This is a requirement for using Standard Expressions, and might happen if neither \
             the Standard or the SpringStandard dialects have been added to the Template Engine \
             and none of the specified dialects registers an attribute of type \
             org.thymeleaf.standard.expression.IStandardExpressionParser with name \
             \"StandardExpressionParser\"",
        )
    }

    /// 取得当前 Standard Dialect 注册的变量表达式求值器。
    ///
    /// # 参数
    /// - `configuration`：当前模板执行配置。
    ///
    /// # 错误
    /// 属性不存在、为 Java null 或运行时类型不正确时返回模板处理错误。
    pub fn get_variable_expression_evaluator(
        configuration: &dyn IEngineConfiguration,
    ) -> StandardExpressionResult<Arc<dyn IStandardVariableExpressionEvaluator>> {
        get_registered_attribute::<Arc<dyn IStandardVariableExpressionEvaluator>>(
            configuration,
            Self::STANDARD_VARIABLE_EXPRESSION_EVALUATOR_ATTRIBUTE_NAME,
            "No Standard Variable Expression Evaluator has been registered as an execution \
             argument. This is a requirement for using Standard Expressions, and might happen \
             if neither the Standard or the SpringStandard dialects have been added to the \
             Template Engine and none of the specified dialects registers an attribute of type \
             org.thymeleaf.standard.expression.IStandardVariableExpressionEvaluator with name \
             \"StandardVariableExpressionEvaluator\"",
        )
    }

    /// 取得当前 Standard Dialect 注册的转换服务。
    ///
    /// # 参数
    /// - `configuration`：当前模板执行配置。
    ///
    /// # 错误
    /// 属性不存在、为 Java null 或运行时类型不正确时返回模板处理错误。
    pub fn get_conversion_service(
        configuration: &dyn IEngineConfiguration,
    ) -> StandardExpressionResult<Arc<dyn IStandardConversionService>> {
        get_registered_attribute::<Arc<dyn IStandardConversionService>>(
            configuration,
            Self::STANDARD_CONVERSION_SERVICE_ATTRIBUTE_NAME,
            "No Standard Conversion Service has been registered as an execution argument. This \
             is a requirement for using Standard Expressions, and might happen if neither the \
             Standard or the SpringStandard dialects have been added to the Template Engine and \
             none of the specified dialects registers an attribute of type \
             org.thymeleaf.standard.expression.IStandardConversionService with name \
             \"StandardConversionService\"",
        )
    }
}

fn get_registered_attribute<T>(
    configuration: &dyn IEngineConfiguration,
    name: &str,
    error_message: &str,
) -> StandardExpressionResult<T>
where
    T: Clone + Send + Sync + 'static,
{
    let name = JavaString::from_rust_str(name);
    configuration
        .get_execution_attributes()
        .get(&Some(name))
        .and_then(Option::as_deref)
        .and_then(|attribute| attribute.downcast_ref::<T>())
        .cloned()
        .ok_or_else(|| {
            Box::new(TemplateProcessingException::new(Some(
                error_message.to_owned(),
            ))) as crate::expression::StandardExpressionError
        })
}
