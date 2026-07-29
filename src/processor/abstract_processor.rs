use crate::processor::IProcessor;
use crate::templatemode::TemplateMode;
use crate::util::{Validate, ValidateError};

/// 所有 Thymeleaf Processor 的基础状态实现。
///
/// 对应 Java: `org.thymeleaf.processor.AbstractProcessor`。
///
/// Java 抽象基类只保存构造时校验为非空、之后不可变的模板模式和优先级，并为
/// [`IProcessor`] 提供 final getter。Rust 不模拟类继承；具体 Processor 可以组合
/// 本对象，并把基础 trait 调用委托给它。
///
/// Thymeleaf 2.x 也存在同名类，但上游在 3.0 中完全重新实现；本对象只对齐固定的
/// Thymeleaf 3.1.5 基线。
pub struct AbstractProcessor {
    precedence: i32,
    template_mode: TemplateMode,
}

impl AbstractProcessor {
    /// 创建具有指定模板模式和优先级的 Processor 基础状态。
    ///
    /// 对应 Java: `AbstractProcessor#AbstractProcessor(TemplateMode, int)`。
    ///
    /// # 参数
    ///
    /// - `template_mode`：Java 参数 `templateMode`；`None` 对应 Java `null`；
    /// - `precedence`：Java 参数 `precedence`，保留完整有符号 32 位取值范围。
    ///
    /// # 返回
    ///
    /// 校验成功后返回字段不可从外部修改的基础状态。
    ///
    /// # 错误
    ///
    /// `template_mode` 为 `None` 时返回
    /// `ValidateError::IllegalArgument("Template mode cannot be null")`，对应 Java
    /// 构造器调用 `Validate.notNull` 抛出的 `IllegalArgumentException`。
    pub fn new(
        template_mode: Option<TemplateMode>,
        precedence: i32,
    ) -> Result<Self, ValidateError> {
        Validate::not_null(template_mode.as_ref(), Some("Template mode cannot be null"))?;

        Ok(Self {
            precedence,
            template_mode: template_mode.expect("validated template mode"),
        })
    }

    /// 返回构造时指定的非空模板模式。
    ///
    /// 对应 Java: `AbstractProcessor#getTemplateMode()`。
    ///
    /// # 返回
    ///
    /// 不可变的模板模式；具体实现不能覆盖或改写该值。
    #[must_use]
    pub const fn get_template_mode(&self) -> TemplateMode {
        self.template_mode
    }

    /// 返回构造时指定的 Processor 优先级。
    ///
    /// 对应 Java: `AbstractProcessor#getPrecedence()`。
    ///
    /// # 返回
    ///
    /// 不可变的 Java `int` 对应值。
    #[must_use]
    pub const fn get_precedence(&self) -> i32 {
        self.precedence
    }
}

impl IProcessor for AbstractProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(AbstractProcessor::get_template_mode(self))
    }

    fn get_precedence(&self) -> i32 {
        AbstractProcessor::get_precedence(self)
    }
}

#[cfg(test)]
mod tests {
    use super::AbstractProcessor;
    use crate::processor::IProcessor;
    use crate::templatemode::TemplateMode;
    use crate::util::ValidateError;

    #[test]
    fn rejects_null_template_mode_with_exact_validate_error() {
        let error = AbstractProcessor::new(None, 123)
            .err()
            .expect("null template mode must fail");

        assert_eq!(
            error,
            ValidateError::IllegalArgument {
                message: Some("Template mode cannot be null".to_owned()),
            }
        );
        assert_eq!(
            error.java_class_name(),
            "java.lang.IllegalArgumentException"
        );
        assert_eq!(error.to_string(), "Template mode cannot be null");
    }

    #[test]
    fn preserves_all_modes_precedence_boundaries_and_dynamic_contract() {
        let cases = [
            (TemplateMode::HTML, i32::MIN),
            (TemplateMode::XML, -1),
            (TemplateMode::TEXT, 0),
            (TemplateMode::JAVASCRIPT, 1),
            (TemplateMode::CSS, 1_000),
            (TemplateMode::RAW, i32::MAX),
        ];

        for (template_mode, precedence) in cases {
            let processor = AbstractProcessor::new(Some(template_mode), precedence)
                .expect("non-null mode is valid");
            let contract: &dyn IProcessor = &processor;

            assert_eq!(processor.get_template_mode(), template_mode);
            assert_eq!(processor.get_precedence(), precedence);
            assert_eq!(contract.get_template_mode(), Some(template_mode));
            assert_eq!(contract.get_precedence(), precedence);

            // Java 字段和 getter 均为 final；重复读取必须保持构造值。
            assert_eq!(processor.get_template_mode(), template_mode);
            assert_eq!(processor.get_precedence(), precedence);
        }
    }
}
