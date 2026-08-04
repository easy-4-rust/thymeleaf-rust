use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexSet;

use crate::cache::{
    AlwaysValidCacheEntryValidity, ICacheEntryValidity, NonCacheableCacheEntryValidity,
    TTLCacheEntryValidity,
};
use crate::util::{ContentTypeUtils, PatternSpec, PatternSpecError, Utf16String};
use crate::{TemplateMode, TemplateModeParseError};

use super::{AbstractTemplateResolver, TemplateResolverError};

/// 带资源名、模板模式和缓存策略配置的模板解析器公共状态。
///
/// 对应 Java: `org.thymeleaf.templateresolver.AbstractConfigurableTemplateResolver`。
pub struct AbstractConfigurableTemplateResolver {
    resolver: AbstractTemplateResolver,
    prefix: Option<Utf16String>,
    suffix: Option<Utf16String>,
    force_suffix: bool,
    character_encoding: Option<Utf16String>,
    template_mode: TemplateMode,
    force_template_mode: bool,
    cacheable: bool,
    cache_ttl_ms: Option<i64>,
    template_aliases: HashMap<Utf16String, Utf16String>,
    xml_template_mode_pattern_spec: PatternSpec,
    html_template_mode_pattern_spec: PatternSpec,
    text_template_mode_pattern_spec: PatternSpec,
    java_script_template_mode_pattern_spec: PatternSpec,
    css_template_mode_pattern_spec: PatternSpec,
    raw_template_mode_pattern_spec: PatternSpec,
    cacheable_pattern_spec: PatternSpec,
    non_cacheable_pattern_spec: PatternSpec,
}

impl AbstractConfigurableTemplateResolver {
    /// 默认模板模式。
    pub const DEFAULT_TEMPLATE_MODE: TemplateMode = TemplateMode::HTML;
    /// 默认允许缓存。
    pub const DEFAULT_CACHEABLE: bool = true;
    /// 默认缓存 TTL；`None` 表示由缓存容量淘汰。
    pub const DEFAULT_CACHE_TTL_MS: Option<i64> = None;

    /// 创建具体可配置解析器的公共状态。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#AbstractConfigurableTemplateResolver()`。
    #[must_use]
    pub fn new(java_class_name: &str) -> Self {
        Self {
            resolver: AbstractTemplateResolver::new(java_class_name),
            prefix: None,
            suffix: None,
            force_suffix: false,
            character_encoding: None,
            template_mode: Self::DEFAULT_TEMPLATE_MODE,
            force_template_mode: false,
            cacheable: Self::DEFAULT_CACHEABLE,
            cache_ttl_ms: Self::DEFAULT_CACHE_TTL_MS,
            template_aliases: HashMap::with_capacity(8),
            xml_template_mode_pattern_spec: PatternSpec::new(),
            html_template_mode_pattern_spec: PatternSpec::new(),
            text_template_mode_pattern_spec: PatternSpec::new(),
            java_script_template_mode_pattern_spec: PatternSpec::new(),
            css_template_mode_pattern_spec: PatternSpec::new(),
            raw_template_mode_pattern_spec: PatternSpec::new(),
            cacheable_pattern_spec: PatternSpec::new(),
            non_cacheable_pattern_spec: PatternSpec::new(),
        }
    }

    /// 返回基础解析器状态。
    #[must_use]
    pub const fn resolver(&self) -> &AbstractTemplateResolver {
        &self.resolver
    }

    /// 返回可变基础解析器状态，供具体类的 Java setter 委托。
    ///
    /// 对应 Java: 具体 Resolver 对 `AbstractTemplateResolver` 公共状态的继承访问。
    pub fn resolver_mut(&mut self) -> &mut AbstractTemplateResolver {
        &mut self.resolver
    }

    /// 返回资源名前缀。对应 Java: `AbstractConfigurableTemplateResolver#getPrefix()`。
    #[must_use]
    pub fn get_prefix(&self) -> Option<&Utf16String> {
        self.prefix.as_ref()
    }

    /// 设置资源名前缀。对应 Java: `AbstractConfigurableTemplateResolver#setPrefix(String)`。
    pub fn set_prefix(&mut self, prefix: Option<Utf16String>) {
        self.prefix = prefix;
    }

    /// 返回资源名后缀。对应 Java: `AbstractConfigurableTemplateResolver#getSuffix()`。
    #[must_use]
    pub fn get_suffix(&self) -> Option<&Utf16String> {
        self.suffix.as_ref()
    }

    /// 设置资源名后缀。对应 Java: `AbstractConfigurableTemplateResolver#setSuffix(String)`。
    pub fn set_suffix(&mut self, suffix: Option<Utf16String>) {
        self.suffix = suffix;
    }

    /// 返回是否强制附加后缀。
    #[must_use]
    pub const fn get_force_suffix(&self) -> bool {
        self.force_suffix
    }

    /// 设置是否强制附加后缀。
    pub const fn set_force_suffix(&mut self, force_suffix: bool) {
        self.force_suffix = force_suffix;
    }

    /// 返回读取资源使用的字符编码。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#getCharacterEncoding()`。
    #[must_use]
    pub fn get_character_encoding(&self) -> Option<&Utf16String> {
        self.character_encoding.as_ref()
    }

    /// 设置读取资源使用的字符编码。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#setCharacterEncoding(String)`。
    pub fn set_character_encoding(&mut self, character_encoding: Option<Utf16String>) {
        self.character_encoding = character_encoding;
    }

    /// 返回默认模板模式。
    #[must_use]
    pub const fn get_template_mode(&self) -> TemplateMode {
        self.template_mode
    }

    /// 设置默认模板模式。
    pub const fn set_template_mode(&mut self, template_mode: TemplateMode) {
        self.template_mode = template_mode;
    }

    /// 使用可空枚举设置默认模板模式。
    ///
    /// # 参数
    /// - `template_mode`：模板模式；`None` 对应 Java `null`。
    ///
    /// # 错误
    /// 模式缺失时返回 Java setter 的精确参数错误。
    pub fn set_template_mode_nullable(
        &mut self,
        template_mode: Option<TemplateMode>,
    ) -> Result<(), TemplateResolverError> {
        self.template_mode = template_mode.ok_or_else(|| {
            TemplateResolverError::InvalidArgument(
                "Cannot set a null template mode value".to_owned(),
            )
        })?;
        Ok(())
    }

    /// 使用文本设置默认模板模式。
    ///
    /// 该入口保留 Java 为兼容旧配置系统提供的字符串 setter。名称按
    /// [`TemplateMode::parse`] 的规则解析。
    ///
    /// # 参数
    /// - `template_mode`：模板模式名称；`None` 对应 Java `null`。
    ///
    /// # 错误
    /// 缺失名称使用 setter 自身的 Java 校验消息；空白名称和其他解析错误保留
    /// `TemplateMode::parse` 的错误消息。
    pub fn set_template_mode_name(
        &mut self,
        template_mode: Option<&str>,
    ) -> Result<(), TemplateResolverError> {
        let template_mode = template_mode.ok_or_else(|| {
            TemplateResolverError::InvalidArgument(
                "Cannot set a null template mode value".to_owned(),
            )
        })?;
        self.template_mode =
            TemplateMode::parse(Some(template_mode)).map_err(|error: TemplateModeParseError| {
                TemplateResolverError::InvalidArgument(error.to_string())
            })?;
        Ok(())
    }

    /// 返回是否禁止根据文件扩展名自动推断模板模式。
    #[must_use]
    pub const fn get_force_template_mode(&self) -> bool {
        self.force_template_mode
    }

    /// 设置是否强制使用配置模板模式。
    pub const fn set_force_template_mode(&mut self, force_template_mode: bool) {
        self.force_template_mode = force_template_mode;
    }

    /// 返回默认缓存开关。
    #[must_use]
    pub const fn is_cacheable(&self) -> bool {
        self.cacheable
    }

    /// 设置默认缓存开关。
    pub const fn set_cacheable(&mut self, cacheable: bool) {
        self.cacheable = cacheable;
    }

    /// 返回缓存 TTL 毫秒值。
    #[must_use]
    pub const fn get_cache_ttl_ms(&self) -> Option<i64> {
        self.cache_ttl_ms
    }

    /// 设置缓存 TTL 毫秒值；`None` 表示直到容量淘汰。
    pub const fn set_cache_ttl_ms(&mut self, cache_ttl_ms: Option<i64>) {
        self.cache_ttl_ms = cache_ttl_ms;
    }

    /// 返回模板别名表。
    #[must_use]
    pub const fn get_template_aliases(&self) -> &HashMap<Utf16String, Utf16String> {
        &self.template_aliases
    }

    /// 把一组模板别名合并到当前别名表。
    ///
    /// Java 方法使用 `putAll`，因此不会先清空已有别名；`None` 对应 Java `null`
    /// 并保持当前表不变。相同 alias 的新值覆盖旧值。
    pub fn set_template_aliases(
        &mut self,
        template_aliases: Option<&HashMap<Utf16String, Utf16String>>,
    ) {
        if let Some(template_aliases) = template_aliases {
            self.template_aliases.extend(
                template_aliases
                    .iter()
                    .map(|(alias, template)| (alias.clone(), template.clone())),
            );
        }
    }

    /// 增加或替换一个模板别名。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#addTemplateAlias(String,String)`。
    pub fn add_template_alias(&mut self, alias: Utf16String, template_name: Utf16String) {
        self.template_aliases.insert(alias, template_name);
    }

    /// 使用 Java 可空参数增加或替换一个模板别名。
    ///
    /// # 参数
    /// - `alias`：别名；`None` 对应 Java `null`。
    /// - `template_name`：别名指向的模板名；`None` 对应 Java `null`。
    ///
    /// # 错误
    /// 先校验 alias，再校验模板名，保留 Java 的精确消息和失败顺序。
    pub fn add_template_alias_nullable(
        &mut self,
        alias: Option<Utf16String>,
        template_name: Option<Utf16String>,
    ) -> Result<(), TemplateResolverError> {
        let alias = alias.ok_or_else(|| {
            TemplateResolverError::InvalidArgument("Alias cannot be null".to_owned())
        })?;
        let template_name = template_name.ok_or_else(|| {
            TemplateResolverError::InvalidArgument("Template name cannot be null".to_owned())
        })?;
        self.add_template_alias(alias, template_name);
        Ok(())
    }

    /// 删除全部模板别名。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#clearTemplateAliases()`。
    pub fn clear_template_aliases(&mut self) {
        self.template_aliases.clear();
    }

    /// 返回 XML 模式规格。
    pub const fn get_xml_template_mode_pattern_spec(&self) -> &PatternSpec {
        &self.xml_template_mode_pattern_spec
    }
    /// 返回 XML 模式模板字符串集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#getXmlTemplateModePatterns()`。
    pub fn get_xml_template_mode_patterns(&self) -> &IndexSet<Option<String>> {
        self.xml_template_mode_pattern_spec.get_patterns()
    }
    /// 设置 XML 模式模板集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#setXmlTemplateModePatterns(Set)`。
    pub fn set_xml_template_mode_patterns(
        &mut self,
        patterns: Option<&[Option<&str>]>,
    ) -> Result<(), PatternSpecError> {
        self.xml_template_mode_pattern_spec.set_patterns(patterns)
    }
    /// 返回 HTML 模式规格。
    pub const fn get_html_template_mode_pattern_spec(&self) -> &PatternSpec {
        &self.html_template_mode_pattern_spec
    }
    /// 返回 HTML 模式模板字符串集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#getHtmlTemplateModePatterns()`。
    pub fn get_html_template_mode_patterns(&self) -> &IndexSet<Option<String>> {
        self.html_template_mode_pattern_spec.get_patterns()
    }
    /// 设置 HTML 模式模板集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#setHtmlTemplateModePatterns(Set)`。
    pub fn set_html_template_mode_patterns(
        &mut self,
        patterns: Option<&[Option<&str>]>,
    ) -> Result<(), PatternSpecError> {
        self.html_template_mode_pattern_spec.set_patterns(patterns)
    }
    /// 返回 TEXT 模式规格。
    pub const fn get_text_template_mode_pattern_spec(&self) -> &PatternSpec {
        &self.text_template_mode_pattern_spec
    }
    /// 返回 TEXT 模式模板字符串集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#getTextTemplateModePatterns()`。
    pub fn get_text_template_mode_patterns(&self) -> &IndexSet<Option<String>> {
        self.text_template_mode_pattern_spec.get_patterns()
    }
    /// 设置 TEXT 模式模板集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#setTextTemplateModePatterns(Set)`。
    pub fn set_text_template_mode_patterns(
        &mut self,
        patterns: Option<&[Option<&str>]>,
    ) -> Result<(), PatternSpecError> {
        self.text_template_mode_pattern_spec.set_patterns(patterns)
    }
    /// 返回 JAVASCRIPT 模式规格。
    pub const fn get_java_script_template_mode_pattern_spec(&self) -> &PatternSpec {
        &self.java_script_template_mode_pattern_spec
    }
    /// 返回 JAVASCRIPT 模式模板字符串集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#getJavaScriptTemplateModePatterns()`。
    pub fn get_java_script_template_mode_patterns(&self) -> &IndexSet<Option<String>> {
        self.java_script_template_mode_pattern_spec.get_patterns()
    }
    /// 设置 JAVASCRIPT 模式模板集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#setJavaScriptTemplateModePatterns(Set)`。
    pub fn set_java_script_template_mode_patterns(
        &mut self,
        patterns: Option<&[Option<&str>]>,
    ) -> Result<(), PatternSpecError> {
        self.java_script_template_mode_pattern_spec
            .set_patterns(patterns)
    }
    /// 返回 CSS 模式规格。
    pub const fn get_css_template_mode_pattern_spec(&self) -> &PatternSpec {
        &self.css_template_mode_pattern_spec
    }
    /// 返回 CSS 模式模板字符串集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#getCssTemplateModePatterns()`。
    pub fn get_css_template_mode_patterns(&self) -> &IndexSet<Option<String>> {
        self.css_template_mode_pattern_spec.get_patterns()
    }
    /// 设置 CSS 模式模板集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#setCssTemplateModePatterns(Set)`。
    pub fn set_css_template_mode_patterns(
        &mut self,
        patterns: Option<&[Option<&str>]>,
    ) -> Result<(), PatternSpecError> {
        self.css_template_mode_pattern_spec.set_patterns(patterns)
    }
    /// 返回 RAW 模式规格。
    pub const fn get_raw_template_mode_pattern_spec(&self) -> &PatternSpec {
        &self.raw_template_mode_pattern_spec
    }
    /// 返回 RAW 模式模板字符串集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#getRawTemplateModePatterns()`。
    pub fn get_raw_template_mode_patterns(&self) -> &IndexSet<Option<String>> {
        self.raw_template_mode_pattern_spec.get_patterns()
    }
    /// 设置 RAW 模式模板集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#setRawTemplateModePatterns(Set)`。
    pub fn set_raw_template_mode_patterns(
        &mut self,
        patterns: Option<&[Option<&str>]>,
    ) -> Result<(), PatternSpecError> {
        self.raw_template_mode_pattern_spec.set_patterns(patterns)
    }
    /// 返回强制可缓存模式规格。
    pub const fn get_cacheable_pattern_spec(&self) -> &PatternSpec {
        &self.cacheable_pattern_spec
    }
    /// 返回强制缓存模板字符串集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#getCacheablePatterns()`。
    pub fn get_cacheable_patterns(&self) -> &IndexSet<Option<String>> {
        self.cacheable_pattern_spec.get_patterns()
    }
    /// 设置强制可缓存模板集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#setCacheablePatterns(Set)`。
    pub fn set_cacheable_patterns(
        &mut self,
        patterns: Option<&[Option<&str>]>,
    ) -> Result<(), PatternSpecError> {
        self.cacheable_pattern_spec.set_patterns(patterns)
    }
    /// 返回强制不可缓存模式规格。
    pub const fn get_non_cacheable_pattern_spec(&self) -> &PatternSpec {
        &self.non_cacheable_pattern_spec
    }
    /// 返回强制不缓存模板字符串集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#getNonCacheablePatterns()`。
    pub fn get_non_cacheable_patterns(&self) -> &IndexSet<Option<String>> {
        self.non_cacheable_pattern_spec.get_patterns()
    }
    /// 设置强制不可缓存模板集合。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#setNonCacheablePatterns(Set)`。
    pub fn set_non_cacheable_patterns(
        &mut self,
        patterns: Option<&[Option<&str>]>,
    ) -> Result<(), PatternSpecError> {
        self.non_cacheable_pattern_spec.set_patterns(patterns)
    }

    /// 依次应用别名、前缀和后缀生成资源名。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#computeResourceName(...)`。
    #[must_use]
    pub fn compute_resource_name(&self, template: &Utf16String) -> Utf16String {
        let unaliased_name = self
            .template_aliases
            .get(template)
            .cloned()
            .unwrap_or_else(|| template.clone());
        let unaliased_text = unaliased_name.to_string_lossy();
        let prefix = self
            .prefix
            .as_ref()
            .filter(|value| !is_empty_or_whitespace(value));
        let suffix = self
            .suffix
            .as_ref()
            .filter(|value| !is_empty_or_whitespace(value));
        let should_apply_suffix = suffix.is_some()
            && (self.force_suffix
                || !ContentTypeUtils::has_recognized_file_extension(Some(&unaliased_text)));
        let mut result = Vec::new();
        if let Some(prefix) = prefix {
            result.extend_from_slice(prefix.as_utf16());
        }
        result.extend_from_slice(unaliased_name.as_utf16());
        if should_apply_suffix {
            result.extend_from_slice(suffix.expect("suffix checked").as_utf16());
        }
        Utf16String::from_utf16(result)
    }

    /// 按模式规格、资源扩展名和默认值计算模板模式。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#computeTemplateMode(...)`。
    #[must_use]
    pub fn compute_template_mode(&self, template: &Utf16String) -> TemplateMode {
        let text = template.to_string_lossy();
        for (spec, template_mode) in [
            (&self.xml_template_mode_pattern_spec, TemplateMode::XML),
            (&self.html_template_mode_pattern_spec, TemplateMode::HTML),
            (&self.text_template_mode_pattern_spec, TemplateMode::TEXT),
            (
                &self.java_script_template_mode_pattern_spec,
                TemplateMode::JAVASCRIPT,
            ),
            (&self.css_template_mode_pattern_spec, TemplateMode::CSS),
            (&self.raw_template_mode_pattern_spec, TemplateMode::RAW),
        ] {
            if spec
                .matches(Some(&text))
                .expect("validated template mode patterns")
            {
                return template_mode;
            }
        }
        if !self.force_template_mode {
            let resource_name = self.compute_resource_name(template);
            if let Some(template_mode) = ContentTypeUtils::compute_template_mode_for_template_name(
                Some(&resource_name.to_string_lossy()),
            ) {
                return template_mode;
            }
        }
        self.template_mode
    }

    /// 按可缓存模式、不可缓存模式、默认开关和 TTL 计算缓存有效性。
    ///
    /// 对应 Java: `AbstractConfigurableTemplateResolver#computeValidity(...)`。
    #[must_use]
    pub fn compute_validity(&self, template: &Utf16String) -> Arc<dyn ICacheEntryValidity> {
        let text = template.to_string_lossy();
        let cacheable = if self
            .cacheable_pattern_spec
            .matches(Some(&text))
            .expect("validated cacheable patterns")
        {
            true
        } else if self
            .non_cacheable_pattern_spec
            .matches(Some(&text))
            .expect("validated non-cacheable patterns")
        {
            false
        } else {
            self.cacheable
        };
        if !cacheable {
            return Arc::new(NonCacheableCacheEntryValidity::new());
        }
        self.cache_ttl_ms.map_or_else(
            || Arc::new(AlwaysValidCacheEntryValidity::new()) as Arc<dyn ICacheEntryValidity>,
            |ttl| Arc::new(TTLCacheEntryValidity::new(ttl)),
        )
    }
}

impl std::ops::Deref for AbstractConfigurableTemplateResolver {
    type Target = AbstractTemplateResolver;

    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl std::ops::DerefMut for AbstractConfigurableTemplateResolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resolver
    }
}

fn is_empty_or_whitespace(value: &Utf16String) -> bool {
    value.as_utf16().iter().all(|character| {
        matches!(
            *character,
            0x0009..=0x000D | 0x001C..=0x0020 | 0x1680 | 0x2000..=0x2006 | 0x2008..=0x200A
                | 0x2028 | 0x2029 | 0x205F | 0x3000
        )
    })
}
