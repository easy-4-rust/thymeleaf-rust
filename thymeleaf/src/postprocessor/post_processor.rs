use crate::TemplateMode;
use crate::engine::TemplateHandlerClass;
use crate::util::{Validate, ValidateError};

use super::IPostProcessor;

/// PostProcessor 的基础不可变实现。
///
/// 该实现足以覆盖方言注册 PostProcessor 的大多数场景。它保存唯一模板模式、
/// Handler 类型令牌和优先级；构造成功后这些值保持不变。
///
/// 对应 Java: `org.thymeleaf.postprocessor.PostProcessor`。
///
/// # 起始版本
///
/// 上游自 Thymeleaf 3.0.0 提供该对象。
pub struct PostProcessor {
    template_mode: TemplateMode,
    handler_class: TemplateHandlerClass,
    precedence: i32,
}

impl PostProcessor {
    /// 创建不可变的 PostProcessor 配置。
    ///
    /// 对应 Java:
    /// `PostProcessor#PostProcessor(TemplateMode, Class<? extends ITemplateHandler>, int)`。
    ///
    /// # 参数
    ///
    /// - `template_mode`：唯一适用的模板模式；`None` 表示空值；
    /// - `handler_class`：实现实际逻辑的 Handler 类型令牌；`None` 表示空值；
    /// - `precedence`：方言内优先级，保留完整有符号 32 位范围。
    ///
    /// # 返回值
    ///
    /// 校验成功后返回字段不可从外部修改的配置。
    ///
    /// # 错误
    ///
    /// 先校验 `template_mode`，再校验 `handler_class`。空值分别返回
    /// `Template mode cannot be null` 和 `Handler class cannot be null`。
    pub fn new(
        template_mode: Option<TemplateMode>,
        handler_class: Option<TemplateHandlerClass>,
        precedence: i32,
    ) -> Result<Self, ValidateError> {
        Validate::not_null(template_mode.as_ref(), Some("Template mode cannot be null"))?;
        Validate::not_null(handler_class.as_ref(), Some("Handler class cannot be null"))?;

        Ok(Self {
            template_mode: template_mode.expect("validated template mode"),
            handler_class: handler_class.expect("validated handler class"),
            precedence,
        })
    }

    /// 返回构造时指定的唯一模板模式。
    ///
    /// 对应 Java: `PostProcessor#getTemplateMode()`。
    ///
    /// # 返回值
    ///
    /// 返回不可变模板模式。
    #[must_use]
    pub const fn get_template_mode(&self) -> TemplateMode {
        self.template_mode
    }

    /// 返回构造时指定的 PostProcessor 优先级。
    ///
    /// 对应 Java: `PostProcessor#getPrecedence()`。
    ///
    /// # 返回值
    ///
    /// 返回不可变优先级。
    #[must_use]
    pub const fn get_precedence(&self) -> i32 {
        self.precedence
    }

    /// 返回构造时指定的 Handler 类型令牌。
    ///
    /// 对应 Java: `PostProcessor#getHandlerClass()`。
    ///
    /// # 返回值
    ///
    /// 重复调用返回同一个不可变类型令牌。
    #[must_use]
    pub const fn get_handler_class(&self) -> &TemplateHandlerClass {
        &self.handler_class
    }
}

impl IPostProcessor for PostProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(PostProcessor::get_template_mode(self))
    }

    fn get_precedence(&self) -> i32 {
        PostProcessor::get_precedence(self)
    }

    fn get_handler_class(&self) -> Option<&TemplateHandlerClass> {
        Some(PostProcessor::get_handler_class(self))
    }

    fn java_class_name(&self) -> &'static str {
        "org.thymeleaf.postprocessor.PostProcessor"
    }
}
