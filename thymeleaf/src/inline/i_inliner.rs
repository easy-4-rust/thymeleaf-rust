use crate::context::ITemplateContext;
use crate::expression::StandardExpressionResult;
use crate::model::{ICDATASection, IComment, IText};
use crate::util::{JavaCharSequence, Utf16String};

/// 文本类模板节点的内联处理合同。
///
/// 内联器负责处理文本、CDATA section 和注释节点中出现的逻辑，而不是元素上的
/// 逻辑，例如 `[[${...}]]` 输出表达式以及 JavaScript 内联制品。
///
/// Rust trait object 会被 Engine Context 和 Processor 共享，因此实现还必须满足
/// `Send + Sync`。
///
/// 对应 Java：`org.thymeleaf.inline.IInliner`。
/// 来源文件：`lib/thymeleaf/src/main/java/org/thymeleaf/inline/IInliner.java`。
///
/// # 起始版本
/// 自 Thymeleaf 3.0.0 起提供。
pub trait IInliner: Send + Sync {
    /// 返回可识别的内联器名称。
    ///
    /// 对应 Java：`IInliner#getName()`。
    ///
    /// # 返回值
    /// 返回用于诊断和配置识别的名称。
    fn get_name(&self) -> &Utf16String;

    /// 对文本节点执行内联。
    ///
    /// 对应 Java：`IInliner#inline(ITemplateContext,IText)`。
    ///
    /// # 参数
    /// - `context`：当前模板处理上下文。
    /// - `text`：需要内联的文本事件。
    ///
    /// # 返回值
    /// 返回修改后的字符序列；不需要修改时可以返回原事件对应内容或 `None`。
    ///
    /// # 错误
    /// 返回具体实现进行表达式求值、模板处理或参数校验时产生的错误。
    fn inline_text(
        &self,
        context: &dyn ITemplateContext,
        text: &dyn IText,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>>;

    /// 对 CDATA section 节点执行内联。
    ///
    /// 对应 Java：`IInliner#inline(ITemplateContext,ICDATASection)`。
    ///
    /// # 参数
    /// - `context`：当前模板处理上下文。
    /// - `cdata_section`：需要内联的 CDATA section 事件。
    ///
    /// # 返回值
    /// 返回修改后的字符序列；不需要修改时可以返回原事件对应内容或 `None`。
    ///
    /// # 错误
    /// 返回具体实现进行表达式求值、模板处理或参数校验时产生的错误。
    fn inline_cdata_section(
        &self,
        context: &dyn ITemplateContext,
        cdata_section: &dyn ICDATASection,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>>;

    /// 对注释节点执行内联。
    ///
    /// 对应 Java：`IInliner#inline(ITemplateContext,IComment)`。
    ///
    /// # 参数
    /// - `context`：当前模板处理上下文。
    /// - `comment`：需要内联的注释事件。
    ///
    /// # 返回值
    /// 返回修改后的字符序列；不需要修改时可以返回原事件对应内容或 `None`。
    ///
    /// # 错误
    /// 返回具体实现进行表达式求值、模板处理或参数校验时产生的错误。
    fn inline_comment(
        &self,
        context: &dyn ITemplateContext,
        comment: &dyn IComment,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>>;
}
