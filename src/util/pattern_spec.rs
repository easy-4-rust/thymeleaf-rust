use std::error::Error;
use std::fmt::{Display, Formatter};

use indexmap::IndexSet;

use super::{PatternUtils, PatternUtilsError, StringPattern, Validate, ValidateError};

/// Thymeleaf 模式集合操作失败。
///
/// 对应 Java: `org.thymeleaf.util.PatternSpec` 调用链产生的
/// `IllegalArgumentException`、`NullPointerException` 和
/// `PatternSyntaxException`。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternSpecError {
    /// `addPattern` 的非空校验失败。
    Validation(ValidateError),
    /// 模式转换、编译或 matcher 输入失败。
    Pattern(PatternUtilsError),
}

impl PatternSpecError {
    /// 返回对应的 Java 异常类名。
    ///
    /// # 返回
    /// 委托给 [`ValidateError`] 或 [`PatternUtilsError`] 的稳定类名。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::Validation(error) => error.java_class_name(),
            Self::Pattern(error) => error.java_class_name(),
        }
    }

    /// 返回底层错误消息。
    ///
    /// # 返回
    /// 显式校验或模式语法消息；隐式 null 错误返回 `None`。
    #[must_use]
    pub fn get_message(&self) -> Option<&str> {
        match self {
            Self::Validation(error) => error.get_message(),
            Self::Pattern(error) => error.get_message(),
        }
    }
}

impl Display for PatternSpecError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(error) => Display::fmt(error, formatter),
            Self::Pattern(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for PatternSpecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Validation(error) => Some(error),
            Self::Pattern(error) => Some(error),
        }
    }
}

impl From<ValidateError> for PatternSpecError {
    fn from(error: ValidateError) -> Self {
        Self::Validation(error)
    }
}

impl From<PatternUtilsError> for PatternSpecError {
    fn from(error: PatternUtilsError) -> Self {
        Self::Pattern(error)
    }
}

/// 按插入顺序保存并匹配模板名称模式的可变规格。
///
/// 对应 Java: `org.thymeleaf.util.PatternSpec`。
///
/// 该对象通常由模板解析器用于可解析模板、模板模式和缓存策略选择。Rust 使用
/// [`IndexSet`] 保留 `LinkedHashSet` 的去重和插入顺序，并用独立编译模式向量保留
/// Java `Pattern` 的引用身份：重复 `addPattern` 不增加公开字符串，却仍会新增一个
/// 编译实例。编译失败时不回滚此前字符串或已编译前缀，保持上游部分变更语义。
#[derive(Clone, Debug, Default)]
pub struct PatternSpec {
    pattern_strs: IndexSet<Option<String>>,
    patterns: Vec<StringPattern>,
}

impl PatternSpec {
    /// 创建空模式规格。
    ///
    /// 对应 Java: `PatternSpec#PatternSpec()`。
    ///
    /// # 返回
    /// 没有公开模式或编译模式的新对象。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 判断当前是否没有已编译模式。
    ///
    /// 对应 Java: `PatternSpec#isEmpty()`。
    ///
    /// # 返回
    /// 编译模式为空时返回 `true`。发生部分编译失败后，公开字符串集合可能非空而本
    /// 方法仍返回 `true`，与 Java 字段判定一致。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// 返回不可变的有序模式字符串集合。
    ///
    /// 对应 Java: `PatternSpec#getPatterns()`。
    ///
    /// # 返回
    /// 对内部 `IndexSet` 的共享引用；`Option<String>` 保留 Java Set 可能包含 null
    /// 的部分失败状态。共享借用在编译期阻止调用方修改内部集合。
    #[must_use]
    pub fn get_patterns(&self) -> &IndexSet<Option<String>> {
        &self.pattern_strs
    }

    /// 替换全部模式。
    ///
    /// 对应 Java: `PatternSpec#setPatterns(Set<String>)`。
    ///
    /// # 参数
    /// - `new_patterns`：按 Java Set 迭代顺序给出的模式；`None` 对应 null Set，
    ///   内部重复值按 `LinkedHashSet` 规则去重，元素 `None` 对应 null。
    ///
    /// # 错误
    /// null 元素或无效正则返回对应错误。与 Java 一致，方法先复制全部字符串，再按
    /// 顺序编译；失败时保留完整字符串集合和成功编译的前缀。
    pub fn set_patterns(
        &mut self,
        new_patterns: Option<&[Option<&str>]>,
    ) -> Result<(), PatternSpecError> {
        self.pattern_strs.clear();
        self.patterns.clear();
        let Some(new_patterns) = new_patterns else {
            return Ok(());
        };

        for pattern in new_patterns {
            self.pattern_strs.insert(pattern.map(ToOwned::to_owned));
        }

        for pattern in &self.pattern_strs {
            self.patterns
                .push(PatternUtils::str_pattern_to_pattern(pattern.as_deref())?);
        }
        Ok(())
    }

    /// 增加一个模式。
    ///
    /// 对应 Java: `PatternSpec#addPattern(String)`。
    ///
    /// # 参数
    /// - `pattern`：非 null、非空且不能全为 Java whitespace 的模式字符串。
    ///
    /// # 错误
    /// 空模式返回精确 `IllegalArgumentException`；正则无效时返回语法错误。字符串
    /// 在编译前加入公开集合，因此语法错误不回滚；重复字符串仍编译并追加独立实例。
    pub fn add_pattern(&mut self, pattern: Option<&str>) -> Result<(), PatternSpecError> {
        Validate::not_empty_str(pattern, Some("Pattern cannot be null or empty"))?;
        let pattern = pattern.expect("validated pattern");
        self.pattern_strs.insert(Some(pattern.to_owned()));
        self.patterns
            .push(PatternUtils::str_pattern_to_pattern(Some(pattern))?);
        Ok(())
    }

    /// 清除全部公开字符串和已编译模式。
    ///
    /// 对应 Java: `PatternSpec#clearPatterns()`。
    pub fn clear_patterns(&mut self) {
        self.pattern_strs.clear();
        self.patterns.clear();
    }

    /// 判断模板名称是否匹配任一模式。
    ///
    /// 对应 Java: `PatternSpec#matches(String)`。
    ///
    /// # 参数
    /// - `template_name`：待匹配模板名称；`None` 对应 Java null。
    ///
    /// # 返回
    /// 按插入/编译顺序遇到首个匹配立即返回 `true`；无模式时返回 `false`。
    ///
    /// # 错误
    /// 仅当至少存在一个编译模式且 template_name 为 `None` 时，保留 Java matcher
    /// 产生的 `NullPointerException`。空规格对 null 输入直接返回 `false`。
    pub fn matches(&self, template_name: Option<&str>) -> Result<bool, PatternSpecError> {
        for pattern in &self.patterns {
            if pattern.matches(template_name)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[cfg(test)]
    fn compiled_pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fmt::Write;

    use super::PatternSpec;

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write_str(&mut self, _value: &str) -> std::fmt::Result {
            Err(std::fmt::Error)
        }
    }

    #[test]
    fn new_get_clear_and_null_set_preserve_empty_contracts() {
        let mut spec = PatternSpec::new();
        assert!(spec.is_empty());
        assert!(spec.get_patterns().is_empty());
        assert_eq!(spec.matches(None), Ok(false));
        assert_eq!(spec.set_patterns(None), Ok(()));
        spec.clear_patterns();
        assert!(spec.is_empty());
    }

    #[test]
    fn sets_deduplicated_ordered_patterns_and_matches_in_full() {
        let mut spec = PatternSpec::new();
        spec.set_patterns(Some(&[Some("*.html"), Some("admin/*"), Some("*.html")]))
            .expect("patterns");
        assert_eq!(
            spec.get_patterns().iter().collect::<Vec<_>>(),
            vec![&Some("*.html".to_owned()), &Some("admin/*".to_owned())]
        );
        assert_eq!(spec.compiled_pattern_count(), 2);
        assert_eq!(spec.matches(Some("index.html")), Ok(true));
        assert_eq!(spec.matches(Some("admin/users")), Ok(true));
        assert_eq!(spec.matches(Some("index.htm")), Ok(false));
    }

    #[test]
    fn repeated_add_keeps_one_public_string_and_multiple_pattern_identities() {
        let mut spec = PatternSpec::new();
        spec.add_pattern(Some("*.html")).expect("first");
        spec.add_pattern(Some("*.html")).expect("second");
        assert_eq!(spec.get_patterns().len(), 1);
        assert_eq!(spec.compiled_pattern_count(), 2);
        assert_eq!(spec.matches(Some("view.html")), Ok(true));
        spec.clear_patterns();
        assert_eq!(spec.compiled_pattern_count(), 0);
    }

    #[test]
    fn add_validates_before_mutation_and_preserves_failed_compile_string() {
        let mut spec = PatternSpec::new();
        for invalid in [None, Some(""), Some("\u{2008}")] {
            let error = spec.add_pattern(invalid).expect_err("validation");
            assert_eq!(
                error.java_class_name(),
                "java.lang.IllegalArgumentException"
            );
            assert_eq!(error.get_message(), Some("Pattern cannot be null or empty"));
            assert_eq!(error.to_string(), "Pattern cannot be null or empty");
            assert!(error.source().is_some());
            assert!(write!(&mut FailingWriter, "{error}").is_err());
        }
        assert!(spec.get_patterns().is_empty());

        let syntax = spec.add_pattern(Some("{")).expect_err("syntax");
        assert_eq!(
            syntax.java_class_name(),
            "java.util.regex.PatternSyntaxException"
        );
        assert_eq!(spec.get_patterns().len(), 1);
        assert!(spec.is_empty());
        assert!(!syntax.to_string().is_empty());
        assert!(syntax.source().is_some());
        assert!(write!(&mut FailingWriter, "{syntax}").is_err());
    }

    #[test]
    fn set_failure_keeps_all_strings_and_compiled_prefix_without_rollback() {
        let mut spec = PatternSpec::new();
        let error = spec
            .set_patterns(Some(&[Some("*.html"), Some("{"), Some("*.txt")]))
            .expect_err("syntax");
        assert_eq!(
            error.java_class_name(),
            "java.util.regex.PatternSyntaxException"
        );
        assert_eq!(spec.get_patterns().len(), 3);
        assert_eq!(spec.compiled_pattern_count(), 1);
        assert_eq!(spec.matches(Some("view.html")), Ok(true));
        assert_eq!(spec.matches(Some("view.txt")), Ok(false));

        let null = spec
            .set_patterns(Some(&[Some("*.html"), None, Some("*.txt")]))
            .expect_err("null");
        assert_eq!(null.java_class_name(), "java.lang.NullPointerException");
        assert_eq!(spec.get_patterns().len(), 3);
        assert_eq!(spec.compiled_pattern_count(), 1);
    }

    #[test]
    fn null_template_errors_only_when_a_compiled_pattern_is_visited() {
        let mut spec = PatternSpec::new();
        assert_eq!(spec.matches(None), Ok(false));
        spec.add_pattern(Some("*")).expect("pattern");
        let error = spec.matches(None).expect_err("null");
        assert_eq!(error.java_class_name(), "java.lang.NullPointerException");
        assert_eq!(error.get_message(), None);
        assert!(error.source().is_some());
    }
}
