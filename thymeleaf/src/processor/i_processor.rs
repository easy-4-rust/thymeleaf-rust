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
pub trait IProcessor: Send + Sync {
    /// 判断 Processor 是否实现 Java `IAttributeDefinitionsAware`。
    fn is_attribute_definitions_aware(&self) -> bool {
        false
    }

    /// 注入全局属性定义仓库。
    ///
    /// 对应 Java: `IAttributeDefinitionsAware#setAttributeDefinitions()`。Java 侧 awareness
    /// 是可选标记接口：未实现它的 Processor/PreProcessor/PostProcessor 不需要仓库，
    /// 此处空默认即等价于未实现该标记接口（no-op）。
    fn set_attribute_definitions(
        &self,
        _attribute_definitions: std::sync::Arc<crate::engine::AttributeDefinitions>,
    ) {
    }

    /// 判断 Processor 是否实现 Java `IElementDefinitionsAware`。
    fn is_element_definitions_aware(&self) -> bool {
        false
    }

    /// 注入全局元素定义仓库。
    ///
    /// 对应 Java: `IElementDefinitionsAware#setElementDefinitions()`；未实现该可选标记
    /// 接口的对象保持 no-op 默认，与 `IAttributeDefinitionsAware` 同机制。
    fn set_element_definitions(
        &self,
        _element_definitions: std::sync::Arc<crate::engine::ElementDefinitions>,
    ) {
    }

    /// 返回配置包装器保存的方言 precedence；普通 Processor 返回 `None`。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils.AbstractProcessorWrapper#getDialectPrecedence()`。
    fn get_dialect_precedence(&self) -> Option<i32> {
        None
    }

    /// 返回包装前的 Processor；普通 Processor 返回 `None`。
    ///
    /// 对应 Java: `ProcessorConfigurationUtils.AbstractProcessorWrapper#unwrap()`。
    fn get_wrapped_processor(&self) -> Option<&dyn IProcessor> {
        None
    }

    /// 将 Java `instanceof IElementProcessor` 暴露为对象安全能力查询。
    fn as_element_processor(&self) -> Option<&dyn crate::element::IElementProcessor> {
        None
    }

    /// 将 Java `instanceof ITextProcessor` 暴露为对象安全能力查询。
    fn as_text_processor(&self) -> Option<&dyn crate::text::ITextProcessor> {
        None
    }

    /// 将 Java `instanceof ICommentProcessor` 暴露为对象安全能力查询。
    fn as_comment_processor(&self) -> Option<&dyn crate::comment::ICommentProcessor> {
        None
    }

    /// 将 Java `instanceof ICDATASectionProcessor` 暴露为对象安全能力查询。
    fn as_cdata_section_processor(
        &self,
    ) -> Option<&dyn crate::cdatasection::ICDATASectionProcessor> {
        None
    }

    /// 将 Java `instanceof IDocTypeProcessor` 暴露为对象安全能力查询。
    fn as_doc_type_processor(&self) -> Option<&dyn crate::doctype::IDocTypeProcessor> {
        None
    }

    /// 将 Java `instanceof ITemplateBoundariesProcessor` 暴露为对象安全能力查询。
    fn as_template_boundaries_processor(
        &self,
    ) -> Option<&dyn crate::templateboundaries::ITemplateBoundariesProcessor> {
        None
    }

    /// 将 Java `instanceof IProcessingInstructionProcessor` 暴露为对象安全能力查询。
    fn as_processing_instruction_processor(
        &self,
    ) -> Option<&dyn crate::processinginstruction::IProcessingInstructionProcessor> {
        None
    }

    /// 将 Java `instanceof IXMLDeclarationProcessor` 暴露为对象安全能力查询。
    fn as_xml_declaration_processor(
        &self,
    ) -> Option<&dyn crate::xmldeclaration::IXMLDeclarationProcessor> {
        None
    }

    /// 返回 Java 风格的具体处理器类名，供稳定 precedence 排序打破平局。
    ///
    /// 具体迁移对象应覆盖为上游全限定类名；第三方实现默认使用 Rust 类型全名。
    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

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
    use std::sync::Mutex;

    use super::IProcessor;
    use crate::templatemode::TemplateMode;

    struct MutableProcessor {
        template_mode: Mutex<Option<TemplateMode>>,
        precedence: Mutex<i32>,
    }

    impl IProcessor for MutableProcessor {
        fn get_template_mode(&self) -> Option<TemplateMode> {
            *self.template_mode.lock().expect("template mode lock")
        }

        fn get_precedence(&self) -> i32 {
            *self.precedence.lock().expect("precedence lock")
        }
    }

    #[test]
    fn preserves_nullable_values_boundaries_and_dynamic_dispatch() {
        let processor = MutableProcessor {
            template_mode: Mutex::new(None),
            precedence: Mutex::new(i32::MIN),
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
            *processor.template_mode.lock().expect("template mode lock") = Some(template_mode);
            assert_eq!(contract.get_template_mode(), Some(template_mode));
        }

        *processor.precedence.lock().expect("precedence lock") = 0;
        assert_eq!(contract.get_precedence(), 0);
        *processor.precedence.lock().expect("precedence lock") = i32::MAX;
        assert_eq!(contract.get_precedence(), i32::MAX);
    }
}
