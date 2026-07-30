use std::any::Any;
use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::util::{JavaLocale, JavaString};

use super::{
    Aggregates, Arrays, Bools, Calendars, Conversions, Dates, ExecutionInfo,
    IExpressionObjectFactory, Ids, Lists, Maps, Messages, Numbers, Objects, Sets,
    StandardExpressionResult, Strings, TemplateObject, TemplateValue, Temporals, Uris,
};

/// Standard Dialect 使用的表达式对象工厂。
///
/// 对应 Java: `org.thymeleaf.standard.expression.StandardExpressionObjectFactory`。
pub struct StandardExpressionObjectFactory;

impl StandardExpressionObjectFactory {
    /// 当前 Context。
    pub const CONTEXT_EXPRESSION_OBJECT_NAME: &'static str = "ctx";
    /// 求值根对象。
    pub const ROOT_EXPRESSION_OBJECT_NAME: &'static str = "root";
    /// OGNL 当前对象别名。
    pub const THIS_EXPRESSION_OBJECT_NAME: &'static str = "this";
    /// Context 变量 Map。
    pub const VARIABLES_EXPRESSION_OBJECT_NAME: &'static str = "vars";
    /// selection target。
    pub const SELECTION_TARGET_EXPRESSION_OBJECT_NAME: &'static str = "object";
    /// 当前 Locale。
    pub const LOCALE_EXPRESSION_OBJECT_NAME: &'static str = "locale";
    /// 已移除的 request 对象名。
    pub const REQUEST_EXPRESSION_OBJECT_NAME: &'static str = "request";
    /// 已移除的 response 对象名。
    pub const RESPONSE_EXPRESSION_OBJECT_NAME: &'static str = "response";
    /// 已移除的 session 对象名。
    pub const SESSION_EXPRESSION_OBJECT_NAME: &'static str = "session";
    /// 已移除的 servletContext 对象名。
    pub const SERVLET_CONTEXT_EXPRESSION_OBJECT_NAME: &'static str = "servletContext";
    /// 类型转换工具。
    pub const CONVERSIONS_EXPRESSION_OBJECT_NAME: &'static str = "conversions";
    /// URI 工具。
    pub const URIS_EXPRESSION_OBJECT_NAME: &'static str = "uris";
    /// java.time 工具。
    pub const TEMPORALS_EXPRESSION_OBJECT_NAME: &'static str = "temporals";
    /// Calendar 工具。
    pub const CALENDARS_EXPRESSION_OBJECT_NAME: &'static str = "calendars";
    /// Date 工具。
    pub const DATES_EXPRESSION_OBJECT_NAME: &'static str = "dates";
    /// Boolean 工具。
    pub const BOOLS_EXPRESSION_OBJECT_NAME: &'static str = "bools";
    /// Number 工具。
    pub const NUMBERS_EXPRESSION_OBJECT_NAME: &'static str = "numbers";
    /// Object 工具。
    pub const OBJECTS_EXPRESSION_OBJECT_NAME: &'static str = "objects";
    /// String 工具。
    pub const STRINGS_EXPRESSION_OBJECT_NAME: &'static str = "strings";
    /// Array 工具。
    pub const ARRAYS_EXPRESSION_OBJECT_NAME: &'static str = "arrays";
    /// List 工具。
    pub const LISTS_EXPRESSION_OBJECT_NAME: &'static str = "lists";
    /// Set 工具。
    pub const SETS_EXPRESSION_OBJECT_NAME: &'static str = "sets";
    /// Map 工具。
    pub const MAPS_EXPRESSION_OBJECT_NAME: &'static str = "maps";
    /// Aggregate 工具。
    pub const AGGREGATES_EXPRESSION_OBJECT_NAME: &'static str = "aggregates";
    /// Message 工具。
    pub const MESSAGES_EXPRESSION_OBJECT_NAME: &'static str = "messages";
    /// ID 序列工具。
    pub const IDS_EXPRESSION_OBJECT_NAME: &'static str = "ids";
    /// 模板执行信息。
    pub const EXECUTION_INFO_OBJECT_NAME: &'static str = "execInfo";

    /// 创建无状态工厂。
    pub const fn new() -> Self {
        Self
    }

    fn all_names() -> &'static [&'static str] {
        &[
            Self::CONTEXT_EXPRESSION_OBJECT_NAME,
            Self::ROOT_EXPRESSION_OBJECT_NAME,
            Self::VARIABLES_EXPRESSION_OBJECT_NAME,
            Self::SELECTION_TARGET_EXPRESSION_OBJECT_NAME,
            Self::LOCALE_EXPRESSION_OBJECT_NAME,
            Self::CONVERSIONS_EXPRESSION_OBJECT_NAME,
            Self::URIS_EXPRESSION_OBJECT_NAME,
            Self::TEMPORALS_EXPRESSION_OBJECT_NAME,
            Self::CALENDARS_EXPRESSION_OBJECT_NAME,
            Self::DATES_EXPRESSION_OBJECT_NAME,
            Self::BOOLS_EXPRESSION_OBJECT_NAME,
            Self::NUMBERS_EXPRESSION_OBJECT_NAME,
            Self::OBJECTS_EXPRESSION_OBJECT_NAME,
            Self::STRINGS_EXPRESSION_OBJECT_NAME,
            Self::ARRAYS_EXPRESSION_OBJECT_NAME,
            Self::LISTS_EXPRESSION_OBJECT_NAME,
            Self::SETS_EXPRESSION_OBJECT_NAME,
            Self::MAPS_EXPRESSION_OBJECT_NAME,
            Self::AGGREGATES_EXPRESSION_OBJECT_NAME,
            Self::MESSAGES_EXPRESSION_OBJECT_NAME,
            Self::IDS_EXPRESSION_OBJECT_NAME,
            Self::EXECUTION_INFO_OBJECT_NAME,
            Self::REQUEST_EXPRESSION_OBJECT_NAME,
            Self::RESPONSE_EXPRESSION_OBJECT_NAME,
            Self::SESSION_EXPRESSION_OBJECT_NAME,
            Self::SERVLET_CONTEXT_EXPRESSION_OBJECT_NAME,
        ]
    }
}

impl Default for StandardExpressionObjectFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl IExpressionObjectFactory for StandardExpressionObjectFactory {
    fn get_all_expression_object_names(&self) -> Vec<Option<JavaString>> {
        Self::all_names()
            .iter()
            .map(|name| Some(JavaString::from_rust_str(name)))
            .collect()
    }

    fn build_object(
        &self,
        context: Arc<dyn IExpressionContext>,
        expression_object_name: Option<&JavaString>,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let name = expression_object_name.map(JavaString::to_string_lossy);
        let Some(name) = name.as_deref() else {
            return Ok(None);
        };
        if matches!(
            name,
            Self::REQUEST_EXPRESSION_OBJECT_NAME
                | Self::RESPONSE_EXPRESSION_OBJECT_NAME
                | Self::SESSION_EXPRESSION_OBJECT_NAME
                | Self::SERVLET_CONTEXT_EXPRESSION_OBJECT_NAME
        ) {
            return Err(Box::new(TemplateProcessingException::new(Some(format!(
                "The '{}','{}','{}' and '{}' expression utility objects are no longer available by default for template expressions and their use is not recommended. In cases where they are really needed, they should be manually added as context variables.",
                Self::REQUEST_EXPRESSION_OBJECT_NAME,
                Self::SESSION_EXPRESSION_OBJECT_NAME,
                Self::SERVLET_CONTEXT_EXPRESSION_OBJECT_NAME,
                Self::RESPONSE_EXPRESSION_OBJECT_NAME
            )))));
        }
        if name == Self::SELECTION_TARGET_EXPRESSION_OBJECT_NAME {
            if let Some(value) = context
                .as_template_context()
                .filter(|template_context| template_context.has_selection_target())
                .and_then(crate::context::ITemplateContext::get_selection_target)
            {
                return Ok(Some(value));
            }
            return Ok(Some(object_value(ContextExpressionObject { context })));
        }
        if matches!(
            name,
            Self::ROOT_EXPRESSION_OBJECT_NAME
                | Self::VARIABLES_EXPRESSION_OBJECT_NAME
                | Self::CONTEXT_EXPRESSION_OBJECT_NAME
        ) {
            return Ok(Some(object_value(ContextExpressionObject { context })));
        }
        if name == Self::LOCALE_EXPRESSION_OBJECT_NAME {
            return Ok(Some(object_value(context.get_locale())));
        }
        let value = match name {
            Self::CONVERSIONS_EXPRESSION_OBJECT_NAME => {
                Conversions::new(Some(context)).ok().map(object_value)
            }
            Self::URIS_EXPRESSION_OBJECT_NAME => Some(object_value(Uris::new())),
            Self::BOOLS_EXPRESSION_OBJECT_NAME => Some(object_value(Bools::new())),
            Self::STRINGS_EXPRESSION_OBJECT_NAME => {
                Some(object_value(Strings::new(context.get_locale())))
            }
            Self::NUMBERS_EXPRESSION_OBJECT_NAME => {
                Some(object_value(Numbers::new(context.get_locale())))
            }
            Self::DATES_EXPRESSION_OBJECT_NAME => {
                Some(object_value(Dates::new(context.get_locale())))
            }
            Self::CALENDARS_EXPRESSION_OBJECT_NAME => {
                Some(object_value(Calendars::new(context.get_locale())))
            }
            Self::TEMPORALS_EXPRESSION_OBJECT_NAME => {
                Temporals::new(context.get_locale()).ok().map(object_value)
            }
            Self::OBJECTS_EXPRESSION_OBJECT_NAME => Some(object_value(Objects::new())),
            Self::ARRAYS_EXPRESSION_OBJECT_NAME => Some(object_value(Arrays::new())),
            Self::LISTS_EXPRESSION_OBJECT_NAME => Some(object_value(Lists::new())),
            Self::SETS_EXPRESSION_OBJECT_NAME => Some(object_value(Sets::new())),
            Self::MAPS_EXPRESSION_OBJECT_NAME => Some(object_value(Maps::new())),
            Self::AGGREGATES_EXPRESSION_OBJECT_NAME => Some(object_value(Aggregates::new())),
            Self::MESSAGES_EXPRESSION_OBJECT_NAME if context.as_template_context().is_some() => {
                Messages::new(Some(context)).ok().map(object_value)
            }
            Self::IDS_EXPRESSION_OBJECT_NAME if context.as_template_context().is_some() => {
                Ids::new(Some(context)).ok().map(object_value)
            }
            Self::EXECUTION_INFO_OBJECT_NAME if context.as_template_context().is_some() => {
                ExecutionInfo::new(Some(context)).ok().map(object_value)
            }
            _ => None,
        };
        Ok(value)
    }

    fn is_cacheable(&self, expression_object_name: Option<&JavaString>) -> bool {
        expression_object_name.is_some_and(|name| {
            name != &JavaString::from_rust_str(Self::SELECTION_TARGET_EXPRESSION_OBJECT_NAME)
        })
    }
}

fn object_value<T: TemplateObject + 'static>(value: T) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Object(Arc::new(value)))
}

struct ContextExpressionObject {
    context: Arc<dyn IExpressionContext>,
}

impl TemplateObject for ContextExpressionObject {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.context.IExpressionContext"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str("org.thymeleaf.context.IExpressionContext")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, super::TemplateObjectPropertyError>> {
        Some(Ok(self.context.get_variable(Some(property_name))))
    }
}

impl TemplateObject for JavaLocale {
    fn java_class_name(&self) -> &str {
        "java.util.Locale"
    }

    fn to_java_string(&self) -> JavaString {
        self.to_language_tag().clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, super::TemplateObjectPropertyError>> {
        let value = match property_name.to_string_lossy().as_str() {
            "language" => self.get_language(),
            "country" => self.get_country().clone(),
            "variant" => self.get_variant(),
            "languageTag" => self.to_language_tag().clone(),
            _ => return None,
        };
        Some(Ok(Some(Arc::new(TemplateValue::string(value)))))
    }

    fn java_invoke_method(
        &self,
        method_name: &JavaString,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, super::TemplateObjectMethodError>> {
        if !arguments.is_empty() {
            return None;
        }
        let value = match method_name.to_string_lossy().as_str() {
            "getLanguage" => self.get_language(),
            "getCountry" => self.get_country().clone(),
            "getVariant" => self.get_variant(),
            "toLanguageTag" => self.to_language_tag().clone(),
            "toString" => self.to_java_string(),
            _ => return None,
        };
        Some(Ok(Some(Arc::new(TemplateValue::string(value)))))
    }
}

macro_rules! stateless_template_object {
    ($type:ty, $class_name:literal) => {
        impl TemplateObject for $type {
            fn java_class_name(&self) -> &str {
                $class_name
            }

            fn to_java_string(&self) -> JavaString {
                JavaString::from_rust_str($class_name)
            }

            fn as_any(&self) -> &dyn Any {
                self
            }

            fn java_invoke_method(
                &self,
                method_name: &JavaString,
                arguments: &[Option<Arc<TemplateValue>>],
            ) -> Option<Result<Option<Arc<TemplateValue>>, super::TemplateObjectMethodError>> {
                Some(
                    super::standard_expression_object_invoker::invoke_stateless_expression_object(
                        self.as_any(),
                        $class_name,
                        method_name,
                        arguments,
                    ),
                )
            }

            fn java_get_property(
                &self,
                property_name: &JavaString,
            ) -> Option<Result<Option<Arc<TemplateValue>>, super::TemplateObjectPropertyError>>
            {
                super::standard_expression_object_invoker::get_standard_expression_object_property(
                    self.as_any(),
                    property_name,
                )
                .map(|value| Ok(Some(value)))
            }
        }
    };
}

stateless_template_object!(Aggregates, "org.thymeleaf.expression.Aggregates");
stateless_template_object!(Arrays, "org.thymeleaf.expression.Arrays");
stateless_template_object!(Bools, "org.thymeleaf.expression.Bools");
stateless_template_object!(Lists, "org.thymeleaf.expression.Lists");
stateless_template_object!(Maps, "org.thymeleaf.expression.Maps");
stateless_template_object!(Objects, "org.thymeleaf.expression.Objects");
stateless_template_object!(Sets, "org.thymeleaf.expression.Sets");
stateless_template_object!(Conversions, "org.thymeleaf.expression.Conversions");
stateless_template_object!(ExecutionInfo, "org.thymeleaf.expression.ExecutionInfo");
stateless_template_object!(Ids, "org.thymeleaf.expression.Ids");
stateless_template_object!(Messages, "org.thymeleaf.expression.Messages");
stateless_template_object!(Uris, "org.thymeleaf.expression.Uris");
