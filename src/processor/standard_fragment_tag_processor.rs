use crate::TemplateMode;
use crate::element::AbstractElementTagProcessor;
use crate::util::JavaString;

use super::{StandardElementCallback, delegate_standard_element_tag_processor};

/// 标记可选择 Fragment 并在正常渲染时删除声明属性的 Processor。
/// 对应 Java: `org.thymeleaf.standard.processor.StandardFragmentTagProcessor`。
pub struct StandardFragmentTagProcessor {
    processor: AbstractElementTagProcessor<StandardElementCallback>,
}

impl StandardFragmentTagProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1500;
    /// 属性名。
    pub const ATTR_NAME: &'static str = "fragment";

    /// 创建 Processor。
    pub fn new(
        template_mode: TemplateMode,
        dialect_prefix: Option<JavaString>,
    ) -> Result<Self, crate::exceptions::TemplateProcessingException> {
        let callback_prefix = dialect_prefix.clone();
        let callback: StandardElementCallback =
            Box::new(move |_context, _tag, structure_handler| {
                structure_handler.remove_attribute_with_prefix(
                    callback_prefix.clone(),
                    JavaString::from_rust_str(Self::ATTR_NAME),
                );
                Ok(())
            });
        Ok(Self {
            processor: AbstractElementTagProcessor::new(
                Some(template_mode),
                dialect_prefix,
                None,
                false,
                Some(JavaString::from_rust_str(Self::ATTR_NAME)),
                true,
                Self::PRECEDENCE,
                "org.thymeleaf.standard.processor.StandardFragmentTagProcessor",
                callback,
            )?,
        })
    }
}

delegate_standard_element_tag_processor!(StandardFragmentTagProcessor, processor);
