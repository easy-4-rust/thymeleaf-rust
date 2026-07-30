use std::sync::Arc;

use indexmap::IndexSet;

use crate::cache::ICacheEntryValidity;
use crate::util::{JavaString, PatternSpec, PatternSpecError};
use crate::{ITemplateResource, TemplateMode};

use super::TemplateResolution;

/// 所有模板解析器共用的名称、顺序、模式过滤和解析流程。
///
/// 对应 Java: `org.thymeleaf.templateresolver.AbstractTemplateResolver`。
pub struct AbstractTemplateResolver {
    name: Option<JavaString>,
    order: Option<i32>,
    check_existence: bool,
    use_decoupled_logic: bool,
    resolvable_pattern_spec: PatternSpec,
}

impl AbstractTemplateResolver {
    /// 默认不检查资源存在性、不启用解耦逻辑。
    pub const DEFAULT_EXISTENCE_CHECK: bool = false;
    /// 默认不启用解耦逻辑。
    pub const DEFAULT_USE_DECOUPLED_LOGIC: bool = false;

    /// 使用具体 Java 类名创建抽象解析器状态。
    #[must_use]
    pub fn new(java_class_name: &str) -> Self {
        Self {
            name: Some(JavaString::from_rust_str(java_class_name)),
            order: None,
            check_existence: Self::DEFAULT_EXISTENCE_CHECK,
            use_decoupled_logic: Self::DEFAULT_USE_DECOUPLED_LOGIC,
            resolvable_pattern_spec: PatternSpec::new(),
        }
    }

    /// 返回解析器名称。
    #[must_use]
    pub fn get_name(&self) -> Option<&JavaString> {
        self.name.as_ref()
    }

    /// 设置解析器名称；`None` 保留 Java 可设置 null 的行为。
    pub fn set_name(&mut self, name: Option<JavaString>) {
        self.name = name;
    }

    /// 返回解析器链顺序。
    #[must_use]
    pub const fn get_order(&self) -> Option<i32> {
        self.order
    }

    /// 设置解析器链顺序。
    pub const fn set_order(&mut self, order: Option<i32>) {
        self.order = order;
    }

    /// 返回可解析模板模式规格。
    #[must_use]
    pub const fn get_resolvable_pattern_spec(&self) -> &PatternSpec {
        &self.resolvable_pattern_spec
    }

    /// 返回可解析模板模式集合。
    ///
    /// 对应 Java: `AbstractTemplateResolver#getResolvablePatterns()`，是
    /// `getResolvablePatternSpec().getPatterns()` 的便利视图。
    #[must_use]
    pub fn get_resolvable_patterns(&self) -> &IndexSet<Option<String>> {
        self.resolvable_pattern_spec.get_patterns()
    }

    /// 替换可解析模板模式。
    pub fn set_resolvable_patterns(
        &mut self,
        patterns: Option<&[Option<&str>]>,
    ) -> Result<(), PatternSpecError> {
        self.resolvable_pattern_spec.set_patterns(patterns)
    }

    /// 返回是否在产生解析结果前确认资源存在。
    #[must_use]
    pub const fn get_check_existence(&self) -> bool {
        self.check_existence
    }

    /// 设置资源存在性检查。
    pub const fn set_check_existence(&mut self, check_existence: bool) {
        self.check_existence = check_existence;
    }

    /// 返回是否查找解耦模板逻辑。
    #[must_use]
    pub const fn get_use_decoupled_logic(&self) -> bool {
        self.use_decoupled_logic
    }

    /// 设置是否查找解耦模板逻辑。
    pub const fn set_use_decoupled_logic(&mut self, use_decoupled_logic: bool) {
        self.use_decoupled_logic = use_decoupled_logic;
    }

    /// 执行抽象解析器的固定算法。
    ///
    /// 闭包依次对应 Java `computeTemplateResource`、`computeTemplateMode` 和
    /// `computeValidity`。模式不匹配、资源构造失败或存在性检查失败时返回 `None`。
    pub fn resolve_template<R, M, V>(
        &self,
        template: &JavaString,
        compute_template_resource: R,
        compute_template_mode: M,
        compute_validity: V,
    ) -> Option<TemplateResolution>
    where
        R: FnOnce() -> Option<Arc<dyn ITemplateResource>>,
        M: FnOnce() -> TemplateMode,
        V: FnOnce() -> Arc<dyn ICacheEntryValidity>,
    {
        if !self.compute_resolvable(template) {
            return None;
        }
        let template_resource = compute_template_resource()?;
        if self.check_existence && !template_resource.exists() {
            return None;
        }
        TemplateResolution::with_options(
            Some(template_resource),
            self.check_existence,
            Some(compute_template_mode()),
            self.use_decoupled_logic,
            Some(compute_validity()),
        )
        .ok()
    }

    /// 应用可解析模式判断模板名。
    #[must_use]
    pub fn compute_resolvable(&self, template: &JavaString) -> bool {
        self.resolvable_pattern_spec.is_empty()
            || self
                .resolvable_pattern_spec
                .matches(Some(&template.to_string_lossy()))
                .expect("validated resolver patterns cannot fail during matching")
    }
}
