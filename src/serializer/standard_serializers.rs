use std::sync::Arc;

use crate::IEngineConfiguration;
use crate::exceptions::TemplateProcessingException;
use crate::util::JavaString;

use super::{IStandardCSSSerializer, IStandardJavaScriptSerializer};

/// 从 Standard Dialect 执行属性取得序列化器。
///
/// 对应 Java: `org.thymeleaf.standard.serializer.StandardSerializers`。
pub struct StandardSerializers;

impl StandardSerializers {
    /// JavaScript Serializer 执行属性名称。
    pub const STANDARD_JAVASCRIPT_SERIALIZER_ATTRIBUTE_NAME: &'static str =
        "StandardJavaScriptSerializer";
    /// CSS Serializer 执行属性名称。
    pub const STANDARD_CSS_SERIALIZER_ATTRIBUTE_NAME: &'static str = "StandardCSSSerializer";

    /// 返回 Standard Dialect 注册的 JavaScript Serializer。
    pub fn get_java_script_serializer(
        configuration: &dyn IEngineConfiguration,
    ) -> Result<Arc<dyn IStandardJavaScriptSerializer>, TemplateProcessingException> {
        get_attribute::<Arc<dyn IStandardJavaScriptSerializer>>(
            configuration,
            Self::STANDARD_JAVASCRIPT_SERIALIZER_ATTRIBUTE_NAME,
            "No JavaScript Serializer has been registered as an execution argument. This is a requirement for using Standard serialization, and might happen if neither the Standard or the SpringStandard dialects have been added to the Template Engine and none of the specified dialects registers an attribute of type org.thymeleaf.standard.serializer.IStandardJavaScriptSerializer with name \"StandardJavaScriptSerializer\"",
        )
    }

    /// 返回 Standard Dialect 注册的 CSS Serializer。
    pub fn get_css_serializer(
        configuration: &dyn IEngineConfiguration,
    ) -> Result<Arc<dyn IStandardCSSSerializer>, TemplateProcessingException> {
        get_attribute::<Arc<dyn IStandardCSSSerializer>>(
            configuration,
            Self::STANDARD_CSS_SERIALIZER_ATTRIBUTE_NAME,
            "No CSS Serializer has been registered as an execution argument. This is a requirement for using Standard serialization, and might happen if neither the Standard or the SpringStandard dialects have been added to the Template Engine and none of the specified dialects registers an attribute of type org.thymeleaf.standard.serializer.IStandardCSSSerializer with name \"StandardCSSSerializer\"",
        )
    }
}

fn get_attribute<T>(
    configuration: &dyn IEngineConfiguration,
    name: &str,
    message: &str,
) -> Result<T, TemplateProcessingException>
where
    T: Clone + Send + Sync + 'static,
{
    configuration
        .get_execution_attributes()
        .get(&Some(JavaString::from_rust_str(name)))
        .and_then(Option::as_deref)
        .and_then(|attribute| attribute.downcast_ref::<T>())
        .cloned()
        .ok_or_else(|| TemplateProcessingException::new(Some(message.to_owned())))
}
