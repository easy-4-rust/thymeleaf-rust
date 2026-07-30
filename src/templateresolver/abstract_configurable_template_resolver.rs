use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexSet;

use crate::TemplateMode;
use crate::cache::{
    AlwaysValidCacheEntryValidity, ICacheEntryValidity, NonCacheableCacheEntryValidity,
    TTLCacheEntryValidity,
};
use crate::util::{ContentTypeUtils, JavaString, PatternSpec, PatternSpecError};

use super::AbstractTemplateResolver;

/// 带资源名、模板模式和缓存策略配置的模板解析器公共状态。
///
/// 对应 Java: `org.thymeleaf.templateresolver.AbstractConfigurableTemplateResolver`。
pub struct AbstractConfigurableTemplateResolver {
    resolver: AbstractTemplateResolver,
    prefix: Option<JavaString>,
    suffix: Option<JavaString>,
    force_suffix: bool,
    character_encoding: Option<JavaString>,
    template_mode: TemplateMode,
    force_template_mode: bool,
    cacheable: bool,
    cache_ttl_ms: Option<i64>,
    template_aliases: HashMap<JavaString, JavaString>,
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

    /// 创建具体可配置解析器的公共状态。
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
            cache_ttl_ms: None,
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
    pub fn resolver_mut(&mut self) -> &mut AbstractTemplateResolver {
        &mut self.resolver
    }

    /// 返回资源名前缀。
    #[must_use]
    pub fn get_prefix(&self) -> Option<&JavaString> {
        self.prefix.as_ref()
    }

    /// 设置资源名前缀。
    pub fn set_prefix(&mut self, prefix: Option<JavaString>) {
        self.prefix = prefix;
    }

    /// 返回资源名后缀。
    #[must_use]
    pub fn get_suffix(&self) -> Option<&JavaString> {
        self.suffix.as_ref()
    }

    /// 设置资源名后缀。
    pub fn set_suffix(&mut self, suffix: Option<JavaString>) {
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
    #[must_use]
    pub fn get_character_encoding(&self) -> Option<&JavaString> {
        self.character_encoding.as_ref()
    }

    /// 设置读取资源使用的字符编码。
    pub fn set_character_encoding(&mut self, character_encoding: Option<JavaString>) {
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
    pub const fn get_template_aliases(&self) -> &HashMap<JavaString, JavaString> {
        &self.template_aliases
    }

    /// 原子替换模板别名表。
    pub fn set_template_aliases(&mut self, template_aliases: HashMap<JavaString, JavaString>) {
        self.template_aliases = template_aliases;
    }

    /// 增加或替换一个模板别名。
    pub fn add_template_alias(&mut self, alias: JavaString, template_name: JavaString) {
        self.template_aliases.insert(alias, template_name);
    }

    /// 删除全部模板别名。
    pub fn clear_template_aliases(&mut self) {
        self.template_aliases.clear();
    }

    /// 返回 XML 模式规格。
    pub const fn get_xml_template_mode_pattern_spec(&self) -> &PatternSpec {
        &self.xml_template_mode_pattern_spec
    }
    /// 返回 XML 模式模板字符串集合。
    pub fn get_xml_template_mode_patterns(&self) -> &IndexSet<Option<String>> {
        self.xml_template_mode_pattern_spec.get_patterns()
    }
    /// 设置 XML 模式模板集合。
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
    pub fn get_html_template_mode_patterns(&self) -> &IndexSet<Option<String>> {
        self.html_template_mode_pattern_spec.get_patterns()
    }
    /// 设置 HTML 模式模板集合。
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
    pub fn get_text_template_mode_patterns(&self) -> &IndexSet<Option<String>> {
        self.text_template_mode_pattern_spec.get_patterns()
    }
    /// 设置 TEXT 模式模板集合。
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
    pub fn get_java_script_template_mode_patterns(&self) -> &IndexSet<Option<String>> {
        self.java_script_template_mode_pattern_spec.get_patterns()
    }
    /// 设置 JAVASCRIPT 模式模板集合。
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
    pub fn get_css_template_mode_patterns(&self) -> &IndexSet<Option<String>> {
        self.css_template_mode_pattern_spec.get_patterns()
    }
    /// 设置 CSS 模式模板集合。
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
    pub fn get_raw_template_mode_patterns(&self) -> &IndexSet<Option<String>> {
        self.raw_template_mode_pattern_spec.get_patterns()
    }
    /// 设置 RAW 模式模板集合。
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
    pub fn get_cacheable_patterns(&self) -> &IndexSet<Option<String>> {
        self.cacheable_pattern_spec.get_patterns()
    }
    /// 设置强制可缓存模板集合。
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
    pub fn get_non_cacheable_patterns(&self) -> &IndexSet<Option<String>> {
        self.non_cacheable_pattern_spec.get_patterns()
    }
    /// 设置强制不可缓存模板集合。
    pub fn set_non_cacheable_patterns(
        &mut self,
        patterns: Option<&[Option<&str>]>,
    ) -> Result<(), PatternSpecError> {
        self.non_cacheable_pattern_spec.set_patterns(patterns)
    }

    /// 依次应用别名、前缀和后缀生成资源名。
    #[must_use]
    pub fn compute_resource_name(&self, template: &JavaString) -> JavaString {
        let unaliased_name = self
            .template_aliases
            .get(template)
            .cloned()
            .unwrap_or_else(|| template.clone());
        let unaliased_text = unaliased_name.to_string_lossy();
        let prefix = self
            .prefix
            .as_ref()
            .filter(|value| !is_empty_or_whitespace(value))
            .map(JavaString::to_string_lossy);
        let suffix = self
            .suffix
            .as_ref()
            .filter(|value| !is_empty_or_whitespace(value))
            .map(JavaString::to_string_lossy);
        let should_apply_suffix = suffix.is_some()
            && (self.force_suffix
                || !ContentTypeUtils::has_recognized_file_extension(Some(&unaliased_text)));
        let mut result = String::new();
        if let Some(prefix) = prefix {
            result.push_str(&prefix);
        }
        result.push_str(&unaliased_text);
        if should_apply_suffix {
            result.push_str(suffix.as_deref().expect("suffix checked"));
        }
        JavaString::from_rust_str(&result)
    }

    /// 按模式规格、资源扩展名和默认值计算模板模式。
    #[must_use]
    pub fn compute_template_mode(&self, template: &JavaString) -> TemplateMode {
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
    #[must_use]
    pub fn compute_validity(&self, template: &JavaString) -> Arc<dyn ICacheEntryValidity> {
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

fn is_empty_or_whitespace(value: &JavaString) -> bool {
    value.as_utf16().iter().all(|character| {
        matches!(
            *character,
            0x0009..=0x000D | 0x001C..=0x0020 | 0x1680 | 0x2000..=0x2006 | 0x2008..=0x200A
                | 0x2028 | 0x2029 | 0x205F | 0x3000
        )
    })
}
