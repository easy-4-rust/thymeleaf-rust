use std::sync::Arc;

use crate::TemplateMode;
use crate::context::IExpressionContext;
use crate::engine::TemplateData;
use crate::util::{DateUtils, JavaDate, JavaString, ValidateError};

/// 暴露当前模板、顶层模板、模板栈及求值开始时间。
///
/// 对应 Java: `org.thymeleaf.expression.ExecutionInfo`。
pub struct ExecutionInfo {
    context: Arc<dyn IExpressionContext>,
    now: JavaDate,
}

impl ExecutionInfo {
    /// 创建执行信息快照；`now` 固定为构造瞬间。
    /// 对应 Java 语义：`ExecutionInfo` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(context: Option<Arc<dyn IExpressionContext>>) -> Result<Self, ValidateError> {
        let context = context.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Context cannot be null".to_owned()),
        })?;
        if context.as_template_context().is_none() {
            return Err(ValidateError::IllegalArgument {
                message: Some("Context must implement ITemplateContext".to_owned()),
            });
        }
        let now = DateUtils::create_now(None, Some(&context.get_locale()));
        Ok(Self { context, now })
    }

    /// 返回当前叶模板名称。
    /// 对应 Java: `ExecutionInfo#getTemplateName()`。
    pub fn get_template_name(&self) -> Option<JavaString> {
        self.template_context()
            .get_template_data()
            .get_template()
            .cloned()
    }

    /// 返回当前叶模板模式。
    /// 对应 Java: `ExecutionInfo#getTemplateMode()`。
    pub fn get_template_mode(&self) -> Option<TemplateMode> {
        self.template_context()
            .get_template_data()
            .get_template_mode()
    }

    /// 返回首次调用 TemplateEngine 的顶层模板名称。
    /// 对应 Java: `ExecutionInfo#getProcessedTemplateName()`。
    pub fn get_processed_template_name(&self) -> Option<JavaString> {
        self.template_context()
            .get_template_stack()
            .first()
            .and_then(|template_data| template_data.get_template().cloned())
    }

    /// 返回顶层模板模式。
    /// 对应 Java: `ExecutionInfo#getProcessedTemplateMode()`。
    pub fn get_processed_template_mode(&self) -> Option<TemplateMode> {
        self.template_context()
            .get_template_stack()
            .first()
            .and_then(|template_data| template_data.get_template_mode())
    }

    /// 返回从顶层到当前叶模板的名称快照。
    /// 对应 Java: `ExecutionInfo#getTemplateNames()`。
    pub fn get_template_names(&self) -> Vec<Option<JavaString>> {
        self.template_context()
            .get_template_stack()
            .into_iter()
            .map(|data| data.get_template().cloned())
            .collect()
    }

    /// 返回从顶层到当前叶模板的模式快照。
    /// 对应 Java: `ExecutionInfo#getTemplateModes()`。
    pub fn get_template_modes(&self) -> Vec<Option<TemplateMode>> {
        self.template_context()
            .get_template_stack()
            .into_iter()
            .map(|template_data| template_data.get_template_mode())
            .collect()
    }

    /// 返回 Context 当前模板栈的只读引用快照。
    /// 对应 Java: `ExecutionInfo#getTemplateStack()`。
    pub fn get_template_stack(&self) -> Vec<Arc<TemplateData>> {
        self.template_context().get_template_stack()
    }

    /// 返回创建本对象时捕获的当前时间。
    pub const fn get_now(&self) -> &JavaDate {
        &self.now
    }

    fn template_context(&self) -> &dyn crate::context::ITemplateContext {
        self.context
            .as_template_context()
            .expect("constructor verifies template context")
    }
}
