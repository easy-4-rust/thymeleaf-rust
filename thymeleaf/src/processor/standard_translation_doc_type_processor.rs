use crate::TemplateMode;
use crate::context::ITemplateContext;
use crate::doctype::{AbstractDocTypeProcessor, IDocTypeProcessor, IDocTypeStructureHandler};
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::IDocType;
use crate::processor::IProcessor;
use crate::util::Utf16String;

type DocTypeCallback = Box<
    dyn Fn(
            &dyn ITemplateContext,
            &dyn IDocType,
            &mut dyn IDocTypeStructureHandler,
        ) -> Result<(), Box<dyn TemplateEngineException>>
        + Send
        + Sync,
>;

/// 将旧 Thymeleaf XHTML DTD system id 翻译为标准 W3C DTD 的 Processor。
///
/// 对应 Java:
/// `org.thymeleaf.standard.processor.StandardTranslationDocTypeProcessor`。
pub struct StandardTranslationDocTypeProcessor {
    processor: AbstractDocTypeProcessor<DocTypeCallback>,
}

impl StandardTranslationDocTypeProcessor {
    /// Java precedence。
    pub const PRECEDENCE: i32 = 1000;

    /// 创建 HTML DOCTYPE 翻译 Processor。
    /// 对应 Java 语义：`StandardTranslationDocTypeProcessor` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new() -> Result<Self, TemplateProcessingException> {
        let callback: DocTypeCallback = Box::new(|_context, doc_type, structure_handler| {
            if !doc_type
                .get_type()
                .is_some_and(|value| value.to_string_lossy().eq_ignore_ascii_case("SYSTEM"))
            {
                return Ok(());
            }
            let Some(system_id) = doc_type.get_system_id() else {
                return Ok(());
            };
            let Some((public_id, translated_system_id)) =
                translate_system_id(&system_id.to_string_lossy())
            else {
                return Ok(());
            };
            structure_handler.set_doc_type(
                doc_type
                    .get_keyword()
                    .cloned()
                    .unwrap_or_else(|| Utf16String::from_rust_str("DOCTYPE")),
                doc_type
                    .get_element_name()
                    .cloned()
                    .unwrap_or_else(|| Utf16String::from_rust_str("html")),
                Some(Utf16String::from_rust_str(public_id)),
                Some(Utf16String::from_rust_str(translated_system_id)),
                doc_type.get_internal_subset().cloned(),
            );
            Ok(())
        });
        Ok(Self {
            processor: AbstractDocTypeProcessor::new(
                Some(TemplateMode::HTML),
                Self::PRECEDENCE,
                "org.thymeleaf.standard.processor.StandardTranslationDocTypeProcessor",
                callback,
            )
            .map_err(|error| {
                TemplateProcessingException::with_cause(
                    Some("Could not create DOCTYPE translation processor".to_owned()),
                    error,
                )
            })?,
        })
    }
}

impl IProcessor for StandardTranslationDocTypeProcessor {
    fn as_doc_type_processor(&self) -> Option<&dyn IDocTypeProcessor> {
        Some(self)
    }

    fn java_class_name(&self) -> &'static str {
        self.processor.java_class_name()
    }
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.processor.get_template_mode()
    }
    fn get_precedence(&self) -> i32 {
        self.processor.get_precedence()
    }
}

impl IDocTypeProcessor for StandardTranslationDocTypeProcessor {
    fn process(
        &self,
        context: &dyn ITemplateContext,
        doc_type: &dyn IDocType,
        structure_handler: &mut dyn IDocTypeStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.processor.process(context, doc_type, structure_handler)
    }
}

fn translate_system_id(system_id: &str) -> Option<(&'static str, &'static str)> {
    let suffix = system_id.strip_prefix("http://www.thymeleaf.org/dtd/")?;
    let (public_id, translated_system_id) = if matches!(
        suffix,
        "xhtml1-strict-thymeleaf-1.dtd"
            | "xhtml1-strict-thymeleaf-2.dtd"
            | "xhtml1-strict-thymeleaf-3.dtd"
            | "xhtml1-strict-thymeleaf-4.dtd"
    ) {
        (
            "-//W3C//DTD XHTML 1.0 Strict//EN",
            "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd",
        )
    } else if matches!(
        suffix,
        "xhtml1-transitional-thymeleaf-1.dtd"
            | "xhtml1-transitional-thymeleaf-2.dtd"
            | "xhtml1-transitional-thymeleaf-3.dtd"
            | "xhtml1-transitional-thymeleaf-4.dtd"
    ) {
        (
            "-//W3C//DTD XHTML 1.0 Transitional//EN",
            "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd",
        )
    } else if matches!(
        suffix,
        "xhtml1-frameset-thymeleaf-1.dtd"
            | "xhtml1-frameset-thymeleaf-2.dtd"
            | "xhtml1-frameset-thymeleaf-3.dtd"
            | "xhtml1-frameset-thymeleaf-4.dtd"
    ) {
        (
            "-//W3C//DTD XHTML 1.0 Frameset//EN",
            "http://www.w3.org/TR/xhtml1/DTD/xhtml1-frameset.dtd",
        )
    } else if matches!(
        suffix,
        "xhtml11-thymeleaf-1.dtd"
            | "xhtml11-thymeleaf-2.dtd"
            | "xhtml11-thymeleaf-3.dtd"
            | "xhtml11-thymeleaf-4.dtd"
    ) {
        (
            "-//W3C//DTD XHTML 1.1//EN",
            "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd",
        )
    } else {
        return None;
    };
    Some((public_id, translated_system_id))
}
