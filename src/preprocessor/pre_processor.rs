use crate::TemplateMode;

use super::{IPreProcessor, PreProcessorHandlerFactory};

/// 方言声明的模板预处理器配置。
///
/// Rust 以零参数工厂函数等价表示 Java `Class<? extends ITemplateHandler>`，每次调用
/// 都必须创建全新 handler。对应 Java: `org.thymeleaf.preprocessor.PreProcessor`。
pub struct PreProcessor {
    template_mode: TemplateMode,
    handler_factory: PreProcessorHandlerFactory,
    handler_class_name: &'static str,
    precedence: i32,
}

impl PreProcessor {
    /// 创建预处理器配置。
    ///
    /// # 参数
    ///
    /// - `template_mode`：唯一适用的模板模式。
    /// - `handler_factory`：创建全新 handler 的函数。
    /// - `handler_class_name`：Java `Class#getName()` 等价稳定名称。
    /// - `precedence`：方言内优先级。
    ///
    /// 对应 Java: `PreProcessor#PreProcessor`。
    #[must_use]
    pub const fn new(
        template_mode: TemplateMode,
        handler_factory: PreProcessorHandlerFactory,
        handler_class_name: &'static str,
        precedence: i32,
    ) -> Self {
        Self {
            template_mode,
            handler_factory,
            handler_class_name,
            precedence,
        }
    }
}

impl IPreProcessor for PreProcessor {
    fn get_template_mode(&self) -> TemplateMode {
        self.template_mode
    }

    fn get_precedence(&self) -> i32 {
        self.precedence
    }

    fn get_handler_factory(&self) -> PreProcessorHandlerFactory {
        self.handler_factory
    }

    fn get_handler_class_name(&self) -> &'static str {
        self.handler_class_name
    }
}
