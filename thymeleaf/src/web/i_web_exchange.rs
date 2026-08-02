use std::sync::Arc;

use indexmap::IndexMap;

use crate::expression::TemplateValue;
use crate::util::{JavaLocale, JavaString};

use super::{IWebApplication, IWebRequest, IWebSession};

/// 一次 Web 请求/响应交换合同。
///
/// 对应 Java: `org.thymeleaf.web.IWebExchange`。
pub trait IWebExchange: Send + Sync {
    /// 返回 Servlet Web exchange capability。
    ///
    /// 对应 Java: `IWebExchange instanceof IServletWebExchange`。核心 crate 保持 Web
    /// 框架中立，因此默认实现返回 `None`；仅 Servlet 宿主适配器可以覆盖此方法，声明
    /// 自身同时具备 Java Servlet exchange 语义。
    ///
    /// # 返回值
    ///
    /// `Some` 表示可安全作为 Java `IServletWebExchange` 使用；`None` 表示普通中立
    /// Web exchange，不能被 `Contexts#getServletWebExchange` 强制转换。
    fn as_servlet_web_exchange(&self) -> Option<&dyn IWebExchange> {
        None
    }

    /// 返回请求对象。
    fn get_request(&self) -> &dyn IWebRequest;
    /// 返回可空会话对象。
    fn get_session(&self) -> Option<&dyn IWebSession>;
    /// 返回应用对象。
    fn get_application(&self) -> &dyn IWebApplication;
    /// 判断会话对象存在且其底层会话已经建立。
    fn has_session(&self) -> bool {
        self.get_session().is_some_and(IWebSession::exists)
    }
    /// 返回宿主 Principal 等价动态对象。
    fn get_principal(&self) -> Option<Arc<TemplateValue>>;
    /// 返回请求 Locale。
    fn get_locale(&self) -> Option<JavaLocale>;
    /// 返回响应内容类型。
    fn get_content_type(&self) -> Option<JavaString>;
    /// 返回字符编码。
    fn get_character_encoding(&self) -> Option<JavaString>;
    /// 判断 exchange 属性是否存在。
    fn contains_attribute(&self, name: Option<&JavaString>) -> bool;
    /// 返回 exchange 属性数量。
    fn get_attribute_count(&self) -> i32;
    /// 返回 exchange 属性名称快照。
    fn get_all_attribute_names(&self) -> Vec<Option<JavaString>>;
    /// 返回 exchange 属性 Map 快照。
    fn get_attribute_map(&self) -> IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>;
    /// 返回 exchange 属性值。
    fn get_attribute_value(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>>;
    /// 新增或替换 exchange 属性。
    fn set_attribute_value(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>);
    /// 删除 exchange 属性。
    fn remove_attribute(&self, name: Option<&JavaString>);
    /// 执行宿主 URL 重写。
    fn transform_url(&self, url: Option<&JavaString>) -> Option<JavaString>;
}
