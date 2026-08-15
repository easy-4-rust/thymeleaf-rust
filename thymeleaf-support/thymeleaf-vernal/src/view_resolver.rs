//! Spring `ThymeleafViewResolver` 语义子集：视图名解析 + Model 桥 +
//! Locale 协商 + 缓存开关 + 前缀/后缀映射。

use std::any::Any;
use std::sync::Arc;

use vernal_web::{Model, RenderedView, View, ViewError, ViewResolver};

use thymeleaf::context::Context;
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::{Locale, NumberValue, Utf16String};
use thymeleaf::{ITemplateResolver, TemplateEngine};

/// 把逻辑视图名解析为 Thymeleaf 模板并渲染的视图解析器。
///
/// 对标 `org.thymeleaf.spring6.view.ThymeleafViewResolver` 的核心子集：
/// - **前缀/后缀**：视图名 `home` + 默认 `classpath:/templates/` 前缀与
///   `.html` 后缀 → 模板 `templates/home.html`（Spring
///   `spring.thymeleaf.prefix/suffix` 语义）。
/// - **Model 桥**：`Model` 的 `Arc<dyn Any>` 值转换为模板变量
///   （String/i64/f64/bool 直转，其余类型退化为占位文本——对应 Spring
///   侧对象经 `ThymeleafEvaluationContext` 进入模板的弱类型语义）。
/// - **Locale**：注入 `Context::set_locale`（缺省用进程默认）。
/// - **缓存**：`cacheable=false` 时每次渲染前失效该模板缓存（对标
///   `spring.thymeleaf.cache=false` 开发态语义）。
pub struct ThymeleafViewResolver {
    engine: Arc<TemplateEngine>,
    prefix: String,
    suffix: String,
    cacheable: bool,
}

impl ThymeleafViewResolver {
    /// 以引擎与模板解析器创建视图解析器（Spring 默认前缀/后缀）。
    ///
    /// # 参数
    /// - `engine`：模板引擎。
    /// - `resolver`：模板资源解析器（决定模板文本从何处装载；
    ///   `StringTemplateResolver` 下模板文本即模板名）。
    #[must_use]
    pub fn new(engine: Arc<TemplateEngine>, resolver: StringTemplateResolver) -> Self {
        engine
            .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
            .expect("resolver");
        Self {
            engine,
            prefix: "classpath:/templates/".to_owned(),
            suffix: ".html".to_owned(),
            cacheable: true,
        }
    }

    /// 设置视图名前缀（对标 `spring.thymeleaf.prefix`）。
    pub fn set_prefix(&mut self, prefix: impl Into<String>) {
        self.prefix = prefix.into();
    }

    /// 设置视图名后缀（对标 `spring.thymeleaf.suffix`）。
    pub fn set_suffix(&mut self, suffix: impl Into<String>) {
        self.suffix = suffix.into();
    }

    /// 设置模板缓存开关（对标 `spring.thymeleaf.cache`）。
    pub const fn set_cacheable(&mut self, cacheable: bool) {
        self.cacheable = cacheable;
    }

    /// 把逻辑视图名映射为模板名。
    ///
    /// `classpath:` scheme 归一化为资源相对路径（去掉 scheme 与前导
    /// `/`），其余前缀原样保留；视图名含 `/` 表示模板根下子路径。
    #[must_use]
    pub fn template_name_for(&self, view_name: &str) -> String {
        let prefix = self
            .prefix
            .strip_prefix("classpath:")
            .unwrap_or(&self.prefix)
            .trim_start_matches('/');
        format!("{prefix}{view_name}{}", self.suffix)
    }
}

impl ViewResolver for ThymeleafViewResolver {
    fn resolve_view_name(&self, view_name: &str, _locale: Option<&str>) -> Option<Arc<dyn View>> {
        Some(Arc::new(ResolvedThymeleafView {
            engine: Arc::clone(&self.engine),
            template: self.template_name_for(view_name),
            template_name_for_cache: Utf16String::from_rust_str(&self.template_name_for(view_name)),
            cacheable: self.cacheable,
        }))
    }
}

/// 已解析的 Thymeleaf 视图：持渲染所需的配置快照（满足 `View: 'static`）。
struct ResolvedThymeleafView {
    engine: Arc<TemplateEngine>,
    template: String,
    template_name_for_cache: Utf16String,
    cacheable: bool,
}

impl View for ResolvedThymeleafView {
    fn render(&self, model: &Model, locale: Option<&str>) -> Result<RenderedView, ViewError> {
        if !self.cacheable {
            self.engine
                .clear_template_cache_for(&self.template_name_for_cache)
                .map_err(|error| ViewError::new(error.to_string()))?;
        }

        let context = Context::new();
        if let Some(locale) = locale {
            let (language, country) = match locale.split_once('-') {
                Some((language, country)) => (language, country),
                None => (locale, ""),
            };
            context
                .set_locale(Some(Locale::new(
                    Utf16String::from_rust_str(language),
                    Utf16String::from_rust_str(country),
                )))
                .map_err(|error| ViewError::new(error.to_string()))?;
        }
        for name in model.attribute_names() {
            let value = model
                .get_attribute(name)
                .and_then(Option::as_ref)
                .map(clone_value_as_any);
            context.set_variable(
                Some(Utf16String::from_rust_str(name)),
                value.map(convert_model_value),
            );
        }

        let body = self
            .engine
            .process_template(&self.template, &context)
            .map_err(|error| ViewError::new(error.to_string()))?
            .to_string_lossy()
            .into_bytes();

        let mut headers = http::HeaderMap::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("text/html"),
        );
        Ok(RenderedView::new(
            http::StatusCode::OK,
            headers,
            bytes::Bytes::from(body),
        ))
    }
}

/// `Arc<dyn Any>` 克隆（值共享，不复制内容）。
fn clone_value_as_any(value: &Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> {
    Arc::clone(value)
}

/// `Model` 值 → 模板变量（弱类型桥，对应 Spring 对象进模板的语义）。
fn convert_model_value(value: Arc<dyn Any + Send + Sync>) -> Arc<TemplateValue> {
    if let Some(text) = value.downcast_ref::<String>() {
        return Arc::new(TemplateValue::string(Utf16String::from_rust_str(text)));
    }
    if let Some(number) = value.downcast_ref::<i64>() {
        return Arc::new(TemplateValue::Number(NumberValue::Long(*number)));
    }
    if let Some(number) = value.downcast_ref::<f64>() {
        return Arc::new(TemplateValue::Number(NumberValue::Double(*number)));
    }
    if let Some(flag) = value.downcast_ref::<bool>() {
        return Arc::new(TemplateValue::Boolean(*flag));
    }
    Arc::new(TemplateValue::string(Utf16String::from_rust_str("object")))
}
