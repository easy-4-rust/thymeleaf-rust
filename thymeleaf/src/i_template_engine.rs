use std::sync::Arc;

use crate::context::IContext;
use crate::exceptions::TemplateEngineException;
use crate::util::{JavaWriter, Utf16String};
use crate::{IEngineConfiguration, IThrottledTemplateProcessor, TemplateSpec};

/// 模板引擎统一操作结果。
pub type TemplateEngineResult<T> =
    Result<T, Box<dyn TemplateEngineException + Send + Sync + 'static>>;

/// Thymeleaf 模板引擎公共合同。
///
/// 对应 Java: `org.thymeleaf.ITemplateEngine`。
pub trait ITemplateEngine: Send + Sync {
    /// 初始化引擎并返回此后保持不变的配置对象。
    fn get_configuration(&self) -> TemplateEngineResult<Arc<dyn IEngineConfiguration>>;

    /// 处理模板规格并把完整结果返回为 Java UTF-16 字符串。
    fn process(
        &self,
        template_spec: &TemplateSpec,
        context: &dyn IContext,
    ) -> TemplateEngineResult<Utf16String>;

    /// 处理模板规格并把结果写入调用方 Writer，结束时刷新 Writer。
    fn process_to_writer(
        &self,
        template_spec: &TemplateSpec,
        context: &dyn IContext,
        writer: Box<dyn JavaWriter>,
    ) -> TemplateEngineResult<()>;

    /// 准备由调用方按背压节奏驱动的模板处理器。
    fn process_throttled(
        &self,
        template_spec: &TemplateSpec,
        context: &dyn IContext,
    ) -> TemplateEngineResult<Box<dyn IThrottledTemplateProcessor>>;

    /// 使用模板名称返回完整输出。
    ///
    /// 对应 Java: `ITemplateEngine#process(String, IContext)`。
    fn process_template(
        &self,
        template: &str,
        context: &dyn IContext,
    ) -> TemplateEngineResult<Utf16String> {
        self.process(&template_spec(template, None)?, context)
    }

    /// 使用模板名称及片段选择器返回完整输出。
    ///
    /// 对应 Java: `ITemplateEngine#process(String, Set<String>, IContext)`。
    fn process_template_with_selectors(
        &self,
        template: &str,
        template_selectors: &crate::TemplateSelectorSet,
        context: &dyn IContext,
    ) -> TemplateEngineResult<Utf16String> {
        self.process(&template_spec(template, Some(template_selectors))?, context)
    }

    /// 使用模板名称把完整输出写入调用方 Writer，并在执行结束时刷新 Writer。
    ///
    /// `template` 是模板名或模板正文，具体含义由 Resolver 决定；`context` 提供表达式变量；
    /// `writer` 接收渲染结果。成功返回 `()`，解析、处理或输出失败时保留具体引擎错误类型。
    /// 对应 Java: `ITemplateEngine#process(String, IContext, Writer)`。
    fn process_template_to_writer(
        &self,
        template: &str,
        context: &dyn IContext,
        writer: Box<dyn JavaWriter>,
    ) -> TemplateEngineResult<()> {
        self.process_to_writer(&template_spec(template, None)?, context, writer)
    }

    /// 使用模板名称和片段选择器把输出写入调用方 Writer，并在执行结束时刷新 Writer。
    ///
    /// `template_selectors` 仅在支持选择器的模板模式中生效。成功返回 `()`；解析、处理
    /// 或输出失败时保留具体引擎错误类型。
    /// 对应 Java: `ITemplateEngine#process(String, Set<String>, IContext, Writer)`。
    fn process_template_with_selectors_to_writer(
        &self,
        template: &str,
        template_selectors: &crate::TemplateSelectorSet,
        context: &dyn IContext,
        writer: Box<dyn JavaWriter>,
    ) -> TemplateEngineResult<()> {
        self.process_to_writer(
            &template_spec(template, Some(template_selectors))?,
            context,
            writer,
        )
    }

    /// 使用模板名称创建节流处理器。
    ///
    /// 对应 Java: `ITemplateEngine#processThrottled(String, IContext)`。
    fn process_throttled_template(
        &self,
        template: &str,
        context: &dyn IContext,
    ) -> TemplateEngineResult<Box<dyn IThrottledTemplateProcessor>> {
        self.process_throttled(&template_spec(template, None)?, context)
    }

    /// 使用模板名称和片段选择器创建由调用方按背压节奏驱动的处理器。
    ///
    /// 对应 Java: `ITemplateEngine#processThrottled(String, Set<String>, IContext)`。
    fn process_throttled_template_with_selectors(
        &self,
        template: &str,
        template_selectors: &crate::TemplateSelectorSet,
        context: &dyn IContext,
    ) -> TemplateEngineResult<Box<dyn IThrottledTemplateProcessor>> {
        self.process_throttled(&template_spec(template, Some(template_selectors))?, context)
    }
}

fn template_spec(
    template: &str,
    selectors: Option<&crate::TemplateSelectorSet>,
) -> TemplateEngineResult<TemplateSpec> {
    TemplateSpec::with_selectors_and_template_mode(Some(template), selectors, None, None).map_err(
        |error| {
            Box::new(crate::TemplateProcessingException::with_cause(
                Some("Invalid template specification".to_owned()),
                error,
            )) as Box<dyn TemplateEngineException + Send + Sync>
        },
    )
}
