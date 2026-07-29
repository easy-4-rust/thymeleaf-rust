use crate::templatemode::TemplateMode;

/// 所有 Processor 方言处理器都必须实现的基础契约。
///
/// 对应 Java: `org.thymeleaf.processor.IProcessor`。
///
/// 该 trait 本身只描述处理器适用的模板模式与优先级。具体处理器应继续实现元素、
/// 文本、注释、CDATA、DOCTYPE、处理指令、模板边界或 XML 声明等子契约。
///
/// Java 接口没有声明模板模式非空；任意自定义实现都可以返回 `null`，非空校验直到
/// `AbstractProcessor` 构造时才执行。因此这里使用 `Option<TemplateMode>` 精确保留
/// 接口层的可观察取值范围。
pub trait IProcessor {
    /// 返回当前处理器适用的模板模式。
    ///
    /// 对应 Java: `IProcessor#getTemplateMode()`。
    ///
    /// # 返回
    ///
    /// `Some(TemplateMode)` 表示具体模式，`None` 对应自定义 Java 实现返回 `null`。
    fn get_template_mode(&self) -> Option<TemplateMode>;

    /// 返回当前处理器的执行优先级。
    ///
    /// 对应 Java: `IProcessor#getPrecedence()`。
    ///
    /// # 返回
    ///
    /// 完整 Java `int` 取值范围内的优先级；数值越小的处理器由上层排序器越早执行。
    fn get_precedence(&self) -> i32;
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::IProcessor;
    use crate::templatemode::TemplateMode;

    struct MutableProcessor {
        template_mode: Cell<Option<TemplateMode>>,
        precedence: Cell<i32>,
    }

    impl IProcessor for MutableProcessor {
        fn get_template_mode(&self) -> Option<TemplateMode> {
            self.template_mode.get()
        }

        fn get_precedence(&self) -> i32 {
            self.precedence.get()
        }
    }

    #[test]
    fn preserves_nullable_values_boundaries_and_dynamic_dispatch() {
        let processor = MutableProcessor {
            template_mode: Cell::new(None),
            precedence: Cell::new(i32::MIN),
        };
        let contract: &dyn IProcessor = &processor;

        assert_eq!(contract.get_template_mode(), None);
        assert_eq!(contract.get_precedence(), i32::MIN);

        for template_mode in [
            TemplateMode::HTML,
            TemplateMode::XML,
            TemplateMode::TEXT,
            TemplateMode::JAVASCRIPT,
            TemplateMode::CSS,
            TemplateMode::RAW,
        ] {
            processor.template_mode.set(Some(template_mode));
            assert_eq!(contract.get_template_mode(), Some(template_mode));
        }

        processor.precedence.set(0);
        assert_eq!(contract.get_precedence(), 0);
        processor.precedence.set(i32::MAX);
        assert_eq!(contract.get_precedence(), i32::MAX);
    }
}
