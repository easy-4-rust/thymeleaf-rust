use crate::util::JavaString;

/// 所有引擎模板事件共享的可空模板位置。
///
/// 对应 Java: `org.thymeleaf.engine.AbstractTemplateEvent`。
pub struct AbstractTemplateEvent {
    template_name: Option<JavaString>,
    line: i32,
    col: i32,
}

impl AbstractTemplateEvent {
    /// 创建没有位置信息的事件基类。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            template_name: None,
            line: -1,
            col: -1,
        }
    }

    /// 创建带上游原始位置字段的事件基类。
    #[must_use]
    pub fn with_location(template_name: Option<JavaString>, line: i32, col: i32) -> Self {
        Self {
            template_name,
            line,
            col,
        }
    }

    /// 复制另一个事件的原始位置字段。
    #[must_use]
    pub fn copy_of(original: &Self) -> Self {
        Self {
            template_name: original.template_name.clone(),
            line: original.line,
            col: original.col,
        }
    }

    /// 仅当模板名非 null 且行列均不为 `-1` 时返回 `true`。
    #[must_use]
    pub const fn has_location(&self) -> bool {
        self.template_name.is_some() && self.line != -1 && self.col != -1
    }

    /// 返回可空模板名。
    #[must_use]
    pub const fn get_template_name(&self) -> Option<&JavaString> {
        self.template_name.as_ref()
    }

    /// 返回原始行号。
    #[must_use]
    pub const fn get_line(&self) -> i32 {
        self.line
    }

    /// 返回原始列号。
    #[must_use]
    pub const fn get_col(&self) -> i32 {
        self.col
    }
}

impl Default for AbstractTemplateEvent {
    fn default() -> Self {
        Self::new()
    }
}
