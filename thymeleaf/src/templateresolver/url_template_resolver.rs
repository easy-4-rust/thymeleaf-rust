use std::sync::Arc;

use crate::TemplateResolutionAttributes;
use crate::cache::{ICacheEntryValidity, NonCacheableCacheEntryValidity};
use crate::templateresource::UrlResourceConnectionHandler;
use crate::util::Utf16String;
use crate::{IEngineConfiguration, ITemplateResource, TemplateResourceError, UrlTemplateResource};

use super::{
    AbstractConfigurableTemplateResolver, ITemplateResolver, TemplateResolution,
    TemplateResolverError,
};

/// 从 URL 解析模板资源的可配置解析器。
///
/// 非法 URL 表示当前解析器不适用而返回 `None`；包含 `;jsessionid` 的模板名强制
/// 不缓存，避免同一模板因不同会话标识占据多个缓存条目。
///
/// 对应 Java: `org.thymeleaf.templateresolver.UrlTemplateResolver`。
pub struct UrlTemplateResolver {
    resolver: AbstractConfigurableTemplateResolver,
    connection_handler: Option<Arc<UrlResourceConnectionHandler>>,
}

impl UrlTemplateResolver {
    /// 创建使用标准可配置默认值的 URL 解析器。
    ///
    /// 对应 Java: `UrlTemplateResolver#UrlTemplateResolver()`。
    #[must_use]
    pub fn new() -> Self {
        Self {
            resolver: AbstractConfigurableTemplateResolver::new(
                "org.thymeleaf.templateresolver.UrlTemplateResolver",
            ),
            connection_handler: None,
        }
    }

    /// 设置非 file/HTTP/HTTPS 协议的连接处理器。
    ///
    /// Java 通过 JVM 全局 URL 协议处理器扩展自定义协议；Rust 把相同职责显式放在
    /// Resolver 实例上，并传递给它创建的每个相对资源。
    ///
    /// 对应 Java: JVM `URLStreamHandler` 扩展点的 Resolver 级 Rust 等价入口。
    pub fn set_connection_handler(
        &mut self,
        connection_handler: Option<Arc<UrlResourceConnectionHandler>>,
    ) {
        self.connection_handler = connection_handler;
    }

    /// 返回当前自定义 URL 协议处理器。
    ///
    /// 对应 Java: JVM `URLStreamHandler` 扩展状态的 Resolver 级 Rust 等价视图。
    #[must_use]
    pub fn get_connection_handler(&self) -> Option<&Arc<UrlResourceConnectionHandler>> {
        self.connection_handler.as_ref()
    }

    /// 计算 URL 模板的缓存有效性。
    ///
    /// 包含 `;jsessionid` 且能由 Java 正则 `.` 覆盖完整输入的模板强制不缓存，避免
    /// 同一资源因不同会话标识产生多个缓存条目；其余输入委托公共可配置策略。
    ///
    /// 对应 Java: `UrlTemplateResolver#computeValidity(...)`。
    #[must_use]
    pub fn compute_validity(&self, template: &Utf16String) -> Arc<dyn ICacheEntryValidity> {
        let text = template.to_string_lossy();
        let java_dot_matches_entire_template = !text.chars().any(|character| {
            matches!(
                character,
                '\n' | '\r' | '\u{0085}' | '\u{2028}' | '\u{2029}'
            )
        });
        if java_dot_matches_entire_template && text.to_lowercase().contains(";jsessionid") {
            return Arc::new(NonCacheableCacheEntryValidity::new());
        }
        self.resolver.compute_validity(template)
    }
}

impl Default for UrlTemplateResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for UrlTemplateResolver {
    type Target = AbstractConfigurableTemplateResolver;
    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl std::ops::DerefMut for UrlTemplateResolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resolver
    }
}

impl ITemplateResolver for UrlTemplateResolver {
    fn get_name(&self) -> Option<&Utf16String> {
        self.resolver.get_name()
    }

    fn get_order(&self) -> Option<i32> {
        self.resolver.get_order()
    }

    fn resolve_template(
        &self,
        _configuration: &dyn IEngineConfiguration,
        _owner_template: Option<&Utf16String>,
        template: &Utf16String,
        _template_resolution_attributes: Option<&TemplateResolutionAttributes>,
    ) -> Result<Option<TemplateResolution>, TemplateResolverError> {
        self.resolver.resolver().resolve_template(
            template,
            || {
                let resource_name = self.resolver.compute_resource_name(template);
                let resource_name = resource_name.to_string_lossy();
                let character_encoding = self
                    .resolver
                    .get_character_encoding()
                    .map(Utf16String::to_string_lossy);
                let resource = self.connection_handler.as_ref().map_or_else(
                    || {
                        UrlTemplateResource::new(
                            Some(&resource_name),
                            character_encoding.as_deref(),
                        )
                    },
                    |handler| {
                        UrlTemplateResource::with_connection_handler(
                            Some(&resource_name),
                            character_encoding.as_deref(),
                            Arc::clone(handler),
                        )
                    },
                );
                match resource {
                    Ok(resource) => Ok(Some(Arc::new(resource) as Arc<dyn ITemplateResource>)),
                    // Java 只把 MalformedURLException 解释为“该 Resolver 不适用”；
                    // 参数错误和模板输入错误仍须向调用方传播。
                    Err(TemplateResourceError::MalformedUrl { .. }) => Ok(None),
                    Err(error) => Err(TemplateResolverError::from(error)),
                }
            },
            || self.resolver.compute_template_mode(template),
            || self.compute_validity(template),
        )
    }
}
