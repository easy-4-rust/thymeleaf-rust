use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::util::JavaString;

static HTML_INSTANCE_NO_PREFIX: OnceLock<Arc<TemplateFragmentMarkupReferenceResolver>> =
    OnceLock::new();
static XML_INSTANCE_NO_PREFIX: OnceLock<Arc<TemplateFragmentMarkupReferenceResolver>> =
    OnceLock::new();
static HTML_INSTANCES_BY_PREFIX: OnceLock<
    RwLock<HashMap<String, Arc<TemplateFragmentMarkupReferenceResolver>>>,
> = OnceLock::new();
static XML_INSTANCES_BY_PREFIX: OnceLock<
    RwLock<HashMap<String, Arc<TemplateFragmentMarkupReferenceResolver>>>,
> = OnceLock::new();

const HTML_FORMAT_WITHOUT_PREFIX: &str = "/[ref='%s' or data-ref='%s' or fragment='%s' or data-fragment='%s' or fragment^='%s(' or data-fragment^='%s(' or fragment^='%s (' or data-fragment^='%s (']";
const XML_FORMAT_WITHOUT_PREFIX: &str =
    "/[ref='%s' or fragment='%s' or fragment^='%s(' or fragment^='%s (']";

/// 把 Thymeleaf fragment 引用解析为标记 selector，并按引用缓存结果。
///
/// HTML 前缀按 Java 规则转为小写，XML 前缀保持大小写；无前缀及每个有前缀实例均
/// 被全局复用。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.markup.TemplateFragmentMarkupReferenceResolver`。
pub struct TemplateFragmentMarkupReferenceResolver {
    selectors_by_reference: RwLock<HashMap<JavaString, JavaString>>,
    resolver_format: String,
    placeholder_count: usize,
    html: bool,
    standard_dialect_prefix: Option<String>,
}

impl TemplateFragmentMarkupReferenceResolver {
    /// 返回 HTML 或 XML 指定 Standard Dialect 前缀对应的共享 resolver。
    ///
    /// 对应 Java: `TemplateFragmentMarkupReferenceResolver#forPrefix`。
    #[must_use]
    pub fn for_prefix(html: bool, standard_dialect_prefix: Option<&JavaString>) -> Arc<Self> {
        let prefix = standard_dialect_prefix.map(JavaString::to_string_lossy);
        if prefix.as_deref().is_none_or(str::is_empty) {
            return if html {
                HTML_INSTANCE_NO_PREFIX
                    .get_or_init(|| Arc::new(Self::new(true, None)))
                    .clone()
            } else {
                XML_INSTANCE_NO_PREFIX
                    .get_or_init(|| Arc::new(Self::new(false, None)))
                    .clone()
            };
        }
        let prefix = if html {
            prefix.expect("checked").to_lowercase()
        } else {
            prefix.expect("checked")
        };
        let repository = if html {
            HTML_INSTANCES_BY_PREFIX.get_or_init(|| RwLock::new(HashMap::with_capacity(3)))
        } else {
            XML_INSTANCES_BY_PREFIX.get_or_init(|| RwLock::new(HashMap::with_capacity(3)))
        };
        if let Some(resolver) = read_lock(repository).get(&prefix) {
            return resolver.clone();
        }
        let resolver = Arc::new(Self::new(html, Some(&prefix)));
        write_lock(repository)
            .entry(prefix)
            .or_insert_with(|| resolver.clone())
            .clone()
    }

    /// 把 fragment 引用解析为 selector；重复引用返回缓存值的相同内容。
    ///
    /// 对应 Java:
    /// `TemplateFragmentMarkupReferenceResolver#resolveSelectorFromReference`。
    #[must_use]
    pub fn resolve_selector_from_reference(&self, reference: &JavaString) -> JavaString {
        if let Some(selector) = read_lock(&self.selectors_by_reference).get(reference) {
            return selector.clone();
        }
        let reference = reference.to_string_lossy();
        let mut selector = self.resolver_format.clone();
        for _ in 0..self.placeholder_count {
            selector = selector.replacen("%s", &reference, 1);
        }
        let selector = JavaString::from_rust_str(&selector);
        write_lock(&self.selectors_by_reference)
            .entry(JavaString::from_rust_str(&reference))
            .or_insert_with(|| selector.clone())
            .clone()
    }

    /// 对应 Java 语义：`TemplateFragmentMarkupReferenceResolver` 的 `reference_attribute_names` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn reference_attribute_names(&self) -> Vec<String> {
        match (self.html, self.standard_dialect_prefix.as_deref()) {
            (true, None) => ["ref", "data-ref", "fragment", "data-fragment"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            (false, None) => ["ref", "fragment"].into_iter().map(str::to_owned).collect(),
            (true, Some(prefix)) => [
                format!("{prefix}:ref"),
                format!("data-{prefix}-ref"),
                format!("{prefix}:fragment"),
                format!("data-{prefix}-fragment"),
            ]
            .into_iter()
            .collect(),
            (false, Some(prefix)) => [format!("{prefix}:ref"), format!("{prefix}:fragment")]
                .into_iter()
                .collect(),
        }
    }

    fn new(html: bool, standard_dialect_prefix: Option<&str>) -> Self {
        let (resolver_format, placeholder_count) = match (html, standard_dialect_prefix) {
            (true, None) => (HTML_FORMAT_WITHOUT_PREFIX.to_owned(), 8),
            (false, None) => (XML_FORMAT_WITHOUT_PREFIX.to_owned(), 4),
            (true, Some(prefix)) => (
                format!(
                    "/[{prefix}:ref='%s' or data-{prefix}-ref='%s' or {prefix}:fragment='%s' or data-{prefix}-fragment='%s' or {prefix}:fragment^='%s(' or data-{prefix}-fragment^='%s(' or {prefix}:fragment^='%s (' or data-{prefix}-fragment^='%s (']"
                ),
                8,
            ),
            (false, Some(prefix)) => (
                format!(
                    "/[{prefix}:ref='%s' or {prefix}:fragment='%s' or {prefix}:fragment^='%s(' or {prefix}:fragment^='%s (']"
                ),
                4,
            ),
        };
        Self {
            selectors_by_reference: RwLock::new(HashMap::with_capacity(20)),
            resolver_format,
            placeholder_count,
            html,
            standard_dialect_prefix: standard_dialect_prefix.map(str::to_owned),
        }
    }
}

fn read_lock<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_lock<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
