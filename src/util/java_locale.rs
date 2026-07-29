use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::JavaString;

static DEFAULT_LOCALE: OnceLock<RwLock<JavaLocale>> = OnceLock::new();

/// Thymeleaf 所需的 Java `Locale` 适配值。
///
/// 对应 Java: `java.util.Locale`，由 `org.thymeleaf.context.IContext` 暴露。
///
/// 保存 BCP-47 语言标签及 country；默认值可在进程内更新，复现
/// `Locale.getDefault()` / `Locale.setDefault()` 对后续 Context 构造的影响。
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct JavaLocale {
    language_tag: JavaString,
    country: JavaString,
}

impl JavaLocale {
    /// 从语言标签与 country 创建 Locale 适配。
    ///
    /// # 参数
    ///
    /// - `language_tag`：Java `Locale#toLanguageTag()` 对应文本。
    /// - `country`：Java `Locale#getCountry()` 对应文本。
    #[must_use]
    pub const fn new(language_tag: JavaString, country: JavaString) -> Self {
        Self {
            language_tag,
            country,
        }
    }

    /// 返回 BCP-47 语言标签。
    #[must_use]
    pub const fn to_language_tag(&self) -> &JavaString {
        &self.language_tag
    }

    /// 返回 ISO 3166 country 或空字符串。
    #[must_use]
    pub const fn get_country(&self) -> &JavaString {
        &self.country
    }

    /// 返回当前进程默认 Locale 的独立值快照。
    #[must_use]
    pub fn get_default() -> Self {
        read_recovering_poison(default_locale_lock()).clone()
    }

    /// 修改后续上下文构造使用的进程默认 Locale。
    ///
    /// # 参数
    ///
    /// - `locale`：替换 `Locale.getDefault()` 结果的值。
    pub fn set_default(locale: Self) {
        *write_recovering_poison(default_locale_lock()) = locale;
    }
}

fn default_locale_lock() -> &'static RwLock<JavaLocale> {
    DEFAULT_LOCALE.get_or_init(|| RwLock::new(locale_from_environment()))
}

fn locale_from_environment() -> JavaLocale {
    let raw = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
        .unwrap_or_else(|| "en-US".to_owned());
    let without_encoding = raw.split('.').next().unwrap_or("en-US");
    let normalized = without_encoding.replace('_', "-");
    let country = normalized
        .split('-')
        .nth(1)
        .filter(|part| part.len() == 2 || part.len() == 3)
        .unwrap_or("")
        .to_ascii_uppercase();
    JavaLocale::new(
        JavaString::from_rust_str(&normalized),
        JavaString::from_rust_str(&country),
    )
}

fn read_recovering_poison<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_recovering_poison<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
