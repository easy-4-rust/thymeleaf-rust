use std::sync::{Arc, OnceLock};

use crate::context::ITemplateContext;
use crate::expression::StandardExpressionResult;
use crate::model::{ICDATASection, IComment, IText};
use crate::util::{JavaCharSequence, Utf16String};

use super::IInliner;

static NO_OP_INLINER: OnceLock<Arc<NoOpInliner>> = OnceLock::new();

/// 不执行任何内联处理的共享单例。
///
/// 三种 inline 重载始终返回 Java null；正常引擎路径不会主动调用这些方法。
///
/// 构造能力保持私有，调用方应使用 [`NoOpInliner::instance`] 或
/// [`NoOpInliner::shared`]。
///
/// 对应 Java：`org.thymeleaf.inline.NoOpInliner`。
/// 来源文件：`lib/thymeleaf/src/main/java/org/thymeleaf/inline/NoOpInliner.java`。
///
/// # 起始版本
/// 自 Thymeleaf 3.0.0 起提供。
///
/// ```compile_fail
/// // Java 构造器是 private；Rust 也不能在 crate 外直接构造该对象。
/// let _ = thymeleaf::inline::NoOpInliner;
/// ```
#[non_exhaustive]
pub struct NoOpInliner;

impl NoOpInliner {
    /// 返回 Java `NoOpInliner.INSTANCE` 对应的唯一共享引用。
    #[must_use]
    /// 对应 Java 语义：`NoOpInliner` 的 `instance` 行为（Rust 侧辅助/私有路径）。
    pub fn instance() -> &'static Self {
        concrete_instance().as_ref()
    }

    /// 返回供 Engine Context 保存的共享 trait object。
    ///
    /// 每次调用都克隆同一个 [`Arc`]，从而使不同 `th:inline="none"` 处理结果保持
    /// Java 静态单例的引用身份。
    #[must_use]
    /// 对应 Java 语义：`NoOpInliner` 的 `shared` 行为（Rust 侧辅助/私有路径）。
    pub fn shared() -> Arc<dyn IInliner> {
        Arc::<NoOpInliner>::clone(concrete_instance())
    }

    /// 对可空文本参数执行 NoOp 内联。
    ///
    /// 对应 Java：`NoOpInliner#inline(ITemplateContext,IText)`。
    ///
    /// # 参数
    /// - `context`：可空模板上下文；该实现不会读取它。
    /// - `text`：可空文本事件；该实现不会读取它。
    ///
    /// # 返回值
    /// 始终返回 `None`，对应 Java `null`。
    pub fn inline_text_nullable(
        &self,
        _context: Option<&dyn ITemplateContext>,
        _text: Option<&dyn IText>,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        // 不执行任何操作；正常引擎处理链不应调用 NoOp 内联方法。
        Ok(None)
    }

    /// 对可空 CDATA section 参数执行 NoOp 内联。
    ///
    /// 对应 Java：`NoOpInliner#inline(ITemplateContext,ICDATASection)`。
    ///
    /// # 参数
    /// - `context`：可空模板上下文；该实现不会读取它。
    /// - `cdata_section`：可空 CDATA section 事件；该实现不会读取它。
    ///
    /// # 返回值
    /// 始终返回 `None`，对应 Java `null`。
    pub fn inline_cdata_section_nullable(
        &self,
        _context: Option<&dyn ITemplateContext>,
        _cdata_section: Option<&dyn ICDATASection>,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        // 不执行任何操作；正常引擎处理链不应调用 NoOp 内联方法。
        Ok(None)
    }

    /// 对可空注释参数执行 NoOp 内联。
    ///
    /// 对应 Java：`NoOpInliner#inline(ITemplateContext,IComment)`。
    ///
    /// # 参数
    /// - `context`：可空模板上下文；该实现不会读取它。
    /// - `comment`：可空注释事件；该实现不会读取它。
    ///
    /// # 返回值
    /// 始终返回 `None`，对应 Java `null`。
    pub fn inline_comment_nullable(
        &self,
        _context: Option<&dyn ITemplateContext>,
        _comment: Option<&dyn IComment>,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        // 不执行任何操作；正常引擎处理链不应调用 NoOp 内联方法。
        Ok(None)
    }
}

fn concrete_instance() -> &'static Arc<NoOpInliner> {
    NO_OP_INLINER.get_or_init(|| Arc::new(NoOpInliner))
}

impl IInliner for NoOpInliner {
    /// 返回固定名称 `NOOP`。
    fn get_name(&self) -> &Utf16String {
        static NAME: OnceLock<Utf16String> = OnceLock::new();
        NAME.get_or_init(|| Utf16String::from_rust_str("NOOP"))
    }

    /// 忽略非空文本参数并返回 `None`。
    fn inline_text(
        &self,
        context: &dyn ITemplateContext,
        text: &dyn IText,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        self.inline_text_nullable(Some(context), Some(text))
    }

    /// 忽略非空 CDATA section 参数并返回 `None`。
    fn inline_cdata_section(
        &self,
        context: &dyn ITemplateContext,
        cdata_section: &dyn ICDATASection,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        self.inline_cdata_section_nullable(Some(context), Some(cdata_section))
    }

    /// 忽略非空注释参数并返回 `None`。
    fn inline_comment(
        &self,
        context: &dyn ITemplateContext,
        comment: &dyn IComment,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        self.inline_comment_nullable(Some(context), Some(comment))
    }
}
