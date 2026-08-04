use std::any::Any;
use std::sync::{Arc, LazyLock, Weak};

use crate::context::IExpressionContext;
use crate::util::{Locale, Utf16String, ValidateError};

use super::{
    Aggregates, Arrays, Bools, Calendars, Conversions, Dates, ExecutionInfo, ExpressionObjectNames,
    IExpressionObjectFactory, Ids, Lists, Maps, Messages, Numbers, Objects, Sets,
    StandardExpressionResult, Strings, TemplateObject, TemplateValue, Temporals, Uris,
};

/// Standard Dialect 使用的表达式对象工厂。
///
/// 工厂声明 Standard 表达式可见的全部名称，并按 Context 能力构建对象。无状态工具
/// 在整个进程中保持单例身份；Locale、Context、selection 和模板执行相关对象按构建
/// 请求创建，再由 [`super::ExpressionObjects`] 决定是否在本次模板执行内缓存。
///
/// 对应 Java: `org.thymeleaf.standard.expression.StandardExpressionObjectFactory`。
pub struct StandardExpressionObjectFactory;

static ALL_EXPRESSION_OBJECT_NAMES: LazyLock<ExpressionObjectNames> = LazyLock::new(|| {
    StandardExpressionObjectFactory::all_names()
        .iter()
        .map(|name| Some(Utf16String::from_rust_str(name)))
        .collect::<Vec<_>>()
        .into()
});
static URIS_EXPRESSION_OBJECT: LazyLock<Arc<TemplateValue>> =
    LazyLock::new(|| object_value(Uris::new()));
static BOOLS_EXPRESSION_OBJECT: LazyLock<Arc<TemplateValue>> =
    LazyLock::new(|| object_value(Bools::new()));
static OBJECTS_EXPRESSION_OBJECT: LazyLock<Arc<TemplateValue>> =
    LazyLock::new(|| object_value(Objects::new()));
static ARRAYS_EXPRESSION_OBJECT: LazyLock<Arc<TemplateValue>> =
    LazyLock::new(|| object_value(Arrays::new()));
static LISTS_EXPRESSION_OBJECT: LazyLock<Arc<TemplateValue>> =
    LazyLock::new(|| object_value(Lists::new()));
static SETS_EXPRESSION_OBJECT: LazyLock<Arc<TemplateValue>> =
    LazyLock::new(|| object_value(Sets::new()));
static MAPS_EXPRESSION_OBJECT: LazyLock<Arc<TemplateValue>> =
    LazyLock::new(|| object_value(Maps::new()));
static AGGREGATES_EXPRESSION_OBJECT: LazyLock<Arc<TemplateValue>> =
    LazyLock::new(|| object_value(Aggregates::new()));

impl StandardExpressionObjectFactory {
    /// 当前表达式 Context。
    pub const CONTEXT_EXPRESSION_OBJECT_NAME: &'static str = "ctx";
    /// 求值根对象。
    pub const ROOT_EXPRESSION_OBJECT_NAME: &'static str = "root";
    /// OGNL 当前对象别名；该名称不由工厂构建，仅用于受限访问检查。
    pub const THIS_EXPRESSION_OBJECT_NAME: &'static str = "this";
    /// Context 变量 Map。
    pub const VARIABLES_EXPRESSION_OBJECT_NAME: &'static str = "vars";
    /// selection target。
    pub const SELECTION_TARGET_EXPRESSION_OBJECT_NAME: &'static str = "object";
    /// 当前 Locale。
    pub const LOCALE_EXPRESSION_OBJECT_NAME: &'static str = "locale";
    /// 已移除但为提供解释性错误而保留的 request 对象名。
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
    /// `java.time` 工具。自 Thymeleaf 3.1.0 起提供。
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

    /// 创建无状态、可在线程间共享的标准工厂。
    ///
    /// 对应 Java: `StandardExpressionObjectFactory#StandardExpressionObjectFactory()`。
    ///
    /// # 返回值
    ///
    /// 返回新的工厂值；所有实例共享名称集合与无状态表达式对象单例。
    #[must_use]
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
    /// 返回按 Java `LinkedHashSet` 顺序冻结的完整名称集合。
    ///
    /// 对应 Java: `StandardExpressionObjectFactory#getAllExpressionObjectNames()`。
    fn get_all_expression_object_names(&self) -> Option<ExpressionObjectNames> {
        Some(Arc::clone(&ALL_EXPRESSION_OBJECT_NAMES))
    }

    /// 根据名称和 Context 能力惰性构建标准表达式对象。
    ///
    /// 普通表达式 Context 无法构建依赖 `ITemplateContext` 的 `messages`、`ids` 和
    /// `execInfo`；selection 不存在时 `object` 回退为 Context。已移除的四个 Servlet
    /// 对象始终返回解释性参数错误。
    ///
    /// 对应 Java: `StandardExpressionObjectFactory#buildObject`。
    ///
    /// # 参数
    ///
    /// - `context`：当前模板执行的表达式 Context。
    /// - `expression_object_name`：待构建对象的可空名称。
    ///
    /// # 返回值
    ///
    /// 返回构建对象；未知名称或 Context 能力不足时返回 `None`。
    ///
    /// # 错误
    ///
    /// 访问已移除的 request、session、servletContext 或 response 名称时返回
    /// `ValidateError::IllegalArgument`。
    fn build_object(
        &self,
        context: Arc<dyn IExpressionContext>,
        expression_object_name: Option<&Utf16String>,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let name = expression_object_name.map(Utf16String::to_string_lossy);
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
            return Err(Box::new(ValidateError::IllegalArgument {
                message: Some(format!(
                    "The '{}','{}','{}' and '{}' expression utility objects are no longer available by default for template expressions and their use is not recommended. In cases where they are really needed, they should be manually added as context variables.",
                    Self::REQUEST_EXPRESSION_OBJECT_NAME,
                    Self::SESSION_EXPRESSION_OBJECT_NAME,
                    Self::SERVLET_CONTEXT_EXPRESSION_OBJECT_NAME,
                    Self::RESPONSE_EXPRESSION_OBJECT_NAME
                )),
            }));
        }
        if name == Self::SELECTION_TARGET_EXPRESSION_OBJECT_NAME {
            if let Some(value) = context
                .as_template_context()
                .filter(|template_context| template_context.has_selection_target())
                .and_then(crate::context::ITemplateContext::get_selection_target)
            {
                return Ok(Some(value));
            }
            return Ok(Some(object_value(ContextExpressionObject {
                context: Arc::downgrade(&context),
            })));
        }
        if matches!(
            name,
            Self::ROOT_EXPRESSION_OBJECT_NAME
                | Self::VARIABLES_EXPRESSION_OBJECT_NAME
                | Self::CONTEXT_EXPRESSION_OBJECT_NAME
        ) {
            return Ok(Some(object_value(ContextExpressionObject {
                context: Arc::downgrade(&context),
            })));
        }
        if name == Self::LOCALE_EXPRESSION_OBJECT_NAME {
            return Ok(Some(object_value(context.get_locale())));
        }
        let value = match name {
            Self::CONVERSIONS_EXPRESSION_OBJECT_NAME => {
                Conversions::new(Some(context)).ok().map(object_value)
            }
            Self::URIS_EXPRESSION_OBJECT_NAME => Some(Arc::clone(&URIS_EXPRESSION_OBJECT)),
            Self::BOOLS_EXPRESSION_OBJECT_NAME => Some(Arc::clone(&BOOLS_EXPRESSION_OBJECT)),
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
            Self::OBJECTS_EXPRESSION_OBJECT_NAME => Some(Arc::clone(&OBJECTS_EXPRESSION_OBJECT)),
            Self::ARRAYS_EXPRESSION_OBJECT_NAME => Some(Arc::clone(&ARRAYS_EXPRESSION_OBJECT)),
            Self::LISTS_EXPRESSION_OBJECT_NAME => Some(Arc::clone(&LISTS_EXPRESSION_OBJECT)),
            Self::SETS_EXPRESSION_OBJECT_NAME => Some(Arc::clone(&SETS_EXPRESSION_OBJECT)),
            Self::MAPS_EXPRESSION_OBJECT_NAME => Some(Arc::clone(&MAPS_EXPRESSION_OBJECT)),
            Self::AGGREGATES_EXPRESSION_OBJECT_NAME => {
                Some(Arc::clone(&AGGREGATES_EXPRESSION_OBJECT))
            }
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

    /// 判断对象是否能在一次模板执行内复用。
    ///
    /// 只有随 selection 栈变化的 `object` 不可缓存；可空或未知名称均遵循 Java
    /// 条件表达式结果。
    ///
    /// 对应 Java: `StandardExpressionObjectFactory#isCacheable(String)`。
    fn is_cacheable(&self, expression_object_name: Option<&Utf16String>) -> bool {
        expression_object_name.is_some_and(|name| {
            name != &Utf16String::from_rust_str(Self::SELECTION_TARGET_EXPRESSION_OBJECT_NAME)
        })
    }
}

fn object_value<T: TemplateObject + 'static>(value: T) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Object(Arc::new(value)))
}

struct ContextExpressionObject {
    /// 缓存对象属于 Context；反向引用必须是弱引用，避免 Context →
    /// ExpressionObjects → ContextExpressionObject → Context 的 Arc 环。
    context: Weak<dyn IExpressionContext>,
}

impl TemplateObject for ContextExpressionObject {
    fn class_name(&self) -> &str {
        "org.thymeleaf.context.IExpressionContext"
    }

    fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_rust_str("org.thymeleaf.context.IExpressionContext")
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn template_equals(&self, other: &dyn TemplateObject) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| Weak::ptr_eq(&self.context, &other.context))
    }

    fn get_property(
        &self,
        property_name: &Utf16String,
    ) -> Option<Result<Option<Arc<TemplateValue>>, super::TemplateObjectPropertyError>> {
        Some(Ok(self.context.upgrade().and_then(|context| {
            context.get_variable(Some(property_name))
        })))
    }
}

impl TemplateObject for Locale {
    fn class_name(&self) -> &str {
        "java.util.Locale"
    }

    fn to_utf16_string(&self) -> Utf16String {
        // Locale#toString 使用下划线连接 language/country/variant；
        // BCP-47 连字符形式只属于 Locale#toLanguageTag。
        Utf16String::from_rust_str(&self.to_string())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_property(
        &self,
        property_name: &Utf16String,
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

    fn invoke_method(
        &self,
        method_name: &Utf16String,
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
            "toString" => self.to_utf16_string(),
            _ => return None,
        };
        Some(Ok(Some(Arc::new(TemplateValue::string(value)))))
    }
}

macro_rules! stateless_template_object {
    ($type:ty, $class_name:literal) => {
        impl TemplateObject for $type {
            fn class_name(&self) -> &str {
                $class_name
            }

            fn to_utf16_string(&self) -> Utf16String {
                Utf16String::from_rust_str($class_name)
            }

            fn as_any(&self) -> &dyn Any {
                self
            }

            fn invoke_method(
                &self,
                method_name: &Utf16String,
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

            fn get_property(
                &self,
                property_name: &Utf16String,
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
