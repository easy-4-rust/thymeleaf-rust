use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use thiserror::Error;

use crate::template_spec::format_attributes;
use crate::util::LoggingUtils;
use crate::{TemplateMode, TemplateResolutionAttributes, TemplateSelectorSet};

/// 模板缓存使用的不可变复合键。
///
/// 对应 Java: `org.thymeleaf.cache.TemplateCacheKey`。
///
/// 键完整包含 owner template、模板名、selectors、行列偏移、强制模板模式和模板解析
/// 属性。selectors 与解析属性按上游约定应当已经由 `TemplateSpec` 变为非空、自然排序
/// 且不可修改的集合；Rust 使用 `Arc` 共享同一只读集合身份，并在构造时预计算哈希。
pub struct TemplateCacheKey {
    owner_template: Option<String>,
    template: String,
    template_selectors: Option<Arc<TemplateSelectorSet>>,
    line_offset: i32,
    col_offset: i32,
    template_mode: Option<TemplateMode>,
    template_resolution_attributes: Option<Arc<TemplateResolutionAttributes>>,
    hash_code: u64,
}

impl TemplateCacheKey {
    /// 创建模板缓存复合键。
    ///
    /// 对应 Java:
    /// `TemplateCacheKey#TemplateCacheKey(String, String, Set<String>, int, int,
    /// TemplateMode, Map<String,Object>)`。
    ///
    /// # 参数
    /// - `owner_template`：Java 参数 `ownerTemplate`；独立模板时为 `None`。
    /// - `template`：Java 参数 `template`；唯一必填字段，空字符串允许。
    /// - `template_selectors`：Java 参数 `templateSelectors`；`None` 表示整个模板。
    /// - `line_offset`：Java 参数 `lineOffset`。
    /// - `col_offset`：Java 参数 `colOffset`。
    /// - `template_mode`：Java 参数 `templateMode`；可以为 `None`。
    /// - `template_resolution_attributes`：Java 参数
    ///   `templateResolutionAttributes`；可以为 `None`。
    ///
    /// # 错误
    /// `template` 为 `None` 时返回 `TemplateCacheKeyError::TemplateCannotBeNull`。
    pub fn new(
        owner_template: Option<&str>,
        template: Option<&str>,
        template_selectors: Option<Arc<TemplateSelectorSet>>,
        line_offset: i32,
        col_offset: i32,
        template_mode: Option<TemplateMode>,
        template_resolution_attributes: Option<Arc<TemplateResolutionAttributes>>,
    ) -> Result<Self, TemplateCacheKeyError> {
        let template = template.ok_or(TemplateCacheKeyError::TemplateCannotBeNull)?;
        let mut key = Self {
            owner_template: owner_template.map(str::to_owned),
            template: template.to_owned(),
            template_selectors,
            line_offset,
            col_offset,
            template_mode,
            template_resolution_attributes,
            hash_code: 0,
        };
        key.hash_code = key.compute_hash_code();
        Ok(key)
    }

    /// 返回拥有当前模板的外层模板。
    ///
    /// 对应 Java: `TemplateCacheKey#getOwnerTemplate()`。
    ///
    /// # 返回
    /// owner template；独立模板返回 `None`。
    #[must_use]
    pub fn get_owner_template(&self) -> Option<&str> {
        self.owner_template.as_deref()
    }

    /// 返回模板名或字符串模板内容。
    ///
    /// 对应 Java: `TemplateCacheKey#getTemplate()`。
    ///
    /// # 返回
    /// 构造时传入的非空模板字符串。
    #[must_use]
    pub fn get_template(&self) -> &str {
        &self.template
    }

    /// 返回模板 selectors 的同一只读集合。
    ///
    /// 对应 Java: `TemplateCacheKey#getTemplateSelectors()`。
    ///
    /// # 返回
    /// selectors 集合；选择整个模板时返回 `None`。
    #[must_use]
    pub fn get_template_selectors(&self) -> Option<&TemplateSelectorSet> {
        self.template_selectors.as_deref()
    }

    /// 返回解析字符串模板时使用的行偏移。
    ///
    /// 对应 Java: `TemplateCacheKey#getLineOffset()`。
    ///
    /// # 返回
    /// 原始 Java `int` 等价偏移。
    #[must_use]
    pub const fn get_line_offset(&self) -> i32 {
        self.line_offset
    }

    /// 返回解析字符串模板时使用的列偏移。
    ///
    /// 对应 Java: `TemplateCacheKey#getColOffset()`。
    ///
    /// # 返回
    /// 原始 Java `int` 等价偏移。
    #[must_use]
    pub const fn get_col_offset(&self) -> i32 {
        self.col_offset
    }

    /// 返回缓存键中强制使用的模板模式。
    ///
    /// 对应 Java: `TemplateCacheKey#getTemplateMode()`。
    ///
    /// # 返回
    /// 模板模式；未强制时返回 `None`。
    #[must_use]
    pub const fn get_template_mode(&self) -> Option<TemplateMode> {
        self.template_mode
    }

    /// 返回模板解析属性的同一只读映射。
    ///
    /// 对应 Java: `TemplateCacheKey#getTemplateResolutionAttributes()`。
    ///
    /// # 返回
    /// 解析属性映射；未指定时返回 `None`。
    #[must_use]
    pub fn get_template_resolution_attributes(&self) -> Option<&TemplateResolutionAttributes> {
        self.template_resolution_attributes.as_deref()
    }

    fn compute_hash_code(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.owner_template.hash(&mut hasher);
        self.template.hash(&mut hasher);
        self.template_selectors.hash(&mut hasher);
        self.line_offset.hash(&mut hasher);
        self.col_offset.hash(&mut hasher);
        self.template_mode.hash(&mut hasher);
        hash_attributes(&self.template_resolution_attributes, &mut hasher);
        hasher.finish()
    }
}

impl PartialEq for TemplateCacheKey {
    fn eq(&self, other: &Self) -> bool {
        if std::ptr::eq(self, other) {
            return true;
        }
        if self.hash_code != other.hash_code {
            return false;
        }
        if self.line_offset != other.line_offset {
            return false;
        }
        if self.col_offset != other.col_offset {
            return false;
        }
        if self.owner_template != other.owner_template {
            return false;
        }
        if self.template != other.template {
            return false;
        }
        if self.template_selectors != other.template_selectors {
            return false;
        }
        if self.template_mode != other.template_mode {
            return false;
        }
        self.template_resolution_attributes == other.template_resolution_attributes
    }
}

impl Eq for TemplateCacheKey {}

impl Hash for TemplateCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash_code);
    }
}

impl Display for TemplateCacheKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(
            &LoggingUtils::loggify_str(Some(&self.template)).expect("non-null template"),
        )?;
        if let Some(owner_template) = &self.owner_template {
            write!(
                formatter,
                "@({};{},{})",
                LoggingUtils::loggify_str(Some(owner_template)).expect("non-null owner template"),
                self.line_offset,
                self.col_offset
            )?;
        }
        if let Some(template_selectors) = &self.template_selectors {
            write!(formatter, "::{}", format_selectors(template_selectors))?;
        }
        if let Some(template_mode) = self.template_mode {
            write!(formatter, " @{template_mode}")?;
        }
        if let Some(attributes) = &self.template_resolution_attributes {
            write!(formatter, " ({})", format_attributes(attributes))?;
        }
        Ok(())
    }
}

/// 创建 `TemplateCacheKey` 时的参数校验错误。
///
/// 对应 Java: `org.thymeleaf.cache.TemplateCacheKey` 构造器抛出的
/// `IllegalArgumentException`。该类型是 Rust 类型化错误扩展，不计入 Java
/// 对象迁移分子。
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum TemplateCacheKeyError {
    /// 模板对应 Java `null`。
    #[error("Template cannot be null")]
    TemplateCannotBeNull,
}

fn hash_attributes<H: Hasher>(
    attributes: &Option<Arc<TemplateResolutionAttributes>>,
    state: &mut H,
) {
    let Some(attributes) = attributes else {
        0_u8.hash(state);
        return;
    };
    1_u8.hash(state);
    let mut entry_hashes = attributes
        .iter()
        .map(|entry| {
            let mut hasher = DefaultHasher::new();
            entry.hash(&mut hasher);
            hasher.finish()
        })
        .collect::<Vec<_>>();
    entry_hashes.sort_unstable();
    entry_hashes.hash(state);
}

fn format_selectors(template_selectors: &TemplateSelectorSet) -> String {
    let mut selectors = template_selectors.iter().collect::<Vec<_>>();
    selectors.sort_by(|left, right| compare_selectors(left, right));
    let selectors = selectors
        .into_iter()
        .map(|selector| selector.as_deref().unwrap_or("null"))
        .collect::<Vec<_>>();
    format!("[{}]", selectors.join(", "))
}

fn compare_selectors(left: &Option<String>, right: &Option<String>) -> std::cmp::Ordering {
    match (left, right) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(left), Some(right)) => left.encode_utf16().cmp(right.encode_utf16()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashMap, hash_map::DefaultHasher};
    use std::fmt::Write;
    use std::hash::{Hash, Hasher};
    use std::sync::Arc;

    use super::{TemplateCacheKey, TemplateCacheKeyError, compare_selectors};
    use crate::{
        TemplateMode, TemplateResolutionAttributeValue, TemplateResolutionAttributes,
        TemplateSelectorSet,
    };

    fn rust_hash(value: &TemplateCacheKey) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    struct FailingWriter {
        remaining_writes: usize,
    }

    impl Write for FailingWriter {
        fn write_str(&mut self, value: &str) -> std::fmt::Result {
            if self.remaining_writes == 0 {
                return Err(std::fmt::Error);
            }
            self.remaining_writes -= 1;
            let _ = value;
            Ok(())
        }
    }

    fn selectors(values: &[Option<&str>]) -> Arc<TemplateSelectorSet> {
        Arc::new(
            values
                .iter()
                .map(|value| value.map(str::to_owned))
                .collect(),
        )
    }

    fn attributes() -> Arc<TemplateResolutionAttributes> {
        Arc::new(HashMap::from([(
            Some("tenant".to_owned()),
            TemplateResolutionAttributeValue::new("acme".to_owned()),
        )]))
    }

    fn full_key() -> TemplateCacheKey {
        TemplateCacheKey::new(
            Some("owner"),
            Some("page"),
            Some(selectors(&[Some("footer"), Some("article")])),
            2,
            3,
            Some(TemplateMode::HTML),
            Some(attributes()),
        )
        .expect("valid template cache key")
    }

    #[test]
    fn validates_template_and_preserves_null_and_empty_distinctions() {
        assert_eq!(
            TemplateCacheKey::new(None, None, None, 0, 0, None, None).err(),
            Some(TemplateCacheKeyError::TemplateCannotBeNull)
        );
        assert_eq!(
            TemplateCacheKeyError::TemplateCannotBeNull.to_string(),
            "Template cannot be null"
        );

        let empty_selectors = Arc::new(BTreeSet::new());
        let empty_attributes = Arc::new(HashMap::new());
        let key = TemplateCacheKey::new(
            None,
            Some(""),
            Some(Arc::clone(&empty_selectors)),
            i32::MIN,
            i32::MAX,
            None,
            Some(Arc::clone(&empty_attributes)),
        )
        .expect("empty collections and template are legal");
        assert_eq!(key.get_owner_template(), None);
        assert_eq!(key.get_template(), "");
        assert!(std::ptr::eq(
            key.get_template_selectors().expect("selectors"),
            empty_selectors.as_ref()
        ));
        assert_eq!(key.get_line_offset(), i32::MIN);
        assert_eq!(key.get_col_offset(), i32::MAX);
        assert_eq!(key.get_template_mode(), None);
        assert!(std::ptr::eq(
            key.get_template_resolution_attributes()
                .expect("attributes"),
            empty_attributes.as_ref()
        ));
        assert_eq!(key.to_string(), "::[] ({})");
    }

    #[test]
    fn display_preserves_owner_offsets_utf16_selectors_mode_and_attributes() {
        let key = TemplateCacheKey::new(
            Some("owner\nname"),
            Some("page\nname"),
            Some(selectors(&[Some("\u{E000}"), Some("\u{10000}"), None])),
            -2,
            7,
            Some(TemplateMode::XML),
            Some(attributes()),
        )
        .expect("valid key");

        assert_eq!(
            key.to_string(),
            "page name@(owner name;-2,7)::[null, 𐀀, \u{E000}] @XML ({tenant=acme})"
        );
    }

    #[test]
    fn equality_checks_identity_cached_hash_and_every_field() {
        let left = full_key();
        assert!(left == left);

        let different_hash =
            TemplateCacheKey::new(None, Some("other"), None, 0, 0, None, None).expect("valid key");
        assert!(left != different_hash);

        let mut different_line = full_key();
        different_line.line_offset = 8;
        different_line.hash_code = left.hash_code;
        assert!(left != different_line);

        let mut different_col = full_key();
        different_col.col_offset = 8;
        different_col.hash_code = left.hash_code;
        assert!(left != different_col);

        let mut different_owner = full_key();
        different_owner.owner_template = Some("other".to_owned());
        different_owner.hash_code = left.hash_code;
        assert!(left != different_owner);

        let mut different_template = full_key();
        different_template.template = "other".to_owned();
        different_template.hash_code = left.hash_code;
        assert!(left != different_template);

        let mut different_selectors = full_key();
        different_selectors.template_selectors = Some(selectors(&[Some("other")]));
        different_selectors.hash_code = left.hash_code;
        assert!(left != different_selectors);

        let mut different_mode = full_key();
        different_mode.template_mode = Some(TemplateMode::XML);
        different_mode.hash_code = left.hash_code;
        assert!(left != different_mode);

        let mut different_attributes = full_key();
        different_attributes.template_resolution_attributes =
            Some(Arc::new(TemplateResolutionAttributes::new()));
        different_attributes.hash_code = left.hash_code;
        assert!(left != different_attributes);

        let equal = full_key();
        assert!(left == equal);
        assert_eq!(rust_hash(&left), rust_hash(&equal));
    }

    #[test]
    fn selector_comparison_covers_all_java_utf16_ordering_branches() {
        let none = None;
        let supplementary = Some("\u{10000}".to_owned());
        let private_use = Some("\u{E000}".to_owned());

        assert_eq!(compare_selectors(&none, &none), std::cmp::Ordering::Equal);
        assert_eq!(
            compare_selectors(&none, &supplementary),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_selectors(&supplementary, &none),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_selectors(&supplementary, &private_use),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn display_propagates_formatter_failures_from_each_segment() {
        let key = full_key();

        for remaining_writes in 0..32 {
            let mut writer = FailingWriter { remaining_writes };
            let _ = write!(&mut writer, "{key}");
        }
    }
}
