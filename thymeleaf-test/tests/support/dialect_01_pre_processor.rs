use std::sync::Arc;

use thymeleaf::context::ITemplateContext;
use thymeleaf::engine::{AbstractTemplateHandler, ITemplateHandler, TemplateHandlerHandle};
use thymeleaf::exceptions::{TemplateEngineException, TemplateProcessingException};
use thymeleaf::model::{
    ICDATASection, ICloseElementTag, IComment, IDocType, IOpenElementTag, IProcessingInstruction,
    IStandaloneElementTag, ITemplateEnd, ITemplateStart, IText, IXMLDeclaration,
};
use thymeleaf::util::Utf16String;

/// 为上游 pre/post processor 语料改写进入引擎前的全部模板事件。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.prepostprocessors.dialect.Dialect01PreProcessor`。
pub struct Dialect01PreProcessor {
    handler: AbstractTemplateHandler,
    processing_instructions: i32,
    open_element_tags: i32,
    standalone_element_tags: i32,
    texts: i32,
    comments: i32,
    cdata_sections: i32,
    doc_types: i32,
    xml_declarations: i32,
}

impl Dialect01PreProcessor {
    /// 创建计数器均为零、尚未接入处理链的预处理器。
    pub fn new() -> Self {
        Self {
            handler: AbstractTemplateHandler::new(),
            processing_instructions: 0,
            open_element_tags: 0,
            standalone_element_tags: 0,
            texts: 0,
            comments: 0,
            cdata_sections: 0,
            doc_types: 0,
            xml_declarations: 0,
        }
    }

    fn processing_error(
        error: impl std::error::Error + Send + Sync + 'static,
    ) -> Box<dyn TemplateEngineException> {
        Box::new(TemplateProcessingException::with_cause(
            Some("Could not create pre-processed template event".to_owned()),
            error,
        ))
    }

    fn handler_error(message: &str) -> Box<dyn TemplateEngineException> {
        Box::new(TemplateProcessingException::new(Some(message.to_owned())))
    }

    fn suffixed(value: Option<&Utf16String>, kind: &str, index: i32) -> Utf16String {
        let prefix =
            value.map_or_else(String::new, |value| format!("{} ", value.to_string_lossy()));
        Utf16String::from_rust_str(&format!("{prefix}({kind}:{index})"))
    }
}

impl Default for Dialect01PreProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ITemplateHandler for Dialect01PreProcessor {
    fn set_next(&mut self, next: Option<TemplateHandlerHandle>) {
        self.handler.set_next(next);
    }

    fn set_context(&mut self, context: Arc<dyn ITemplateContext>) {
        self.handler.set_context(context);
    }

    fn handle_template_start(
        &mut self,
        template_start: Arc<dyn ITemplateStart>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.handler.handle_template_start(template_start)
    }

    fn handle_template_end(
        &mut self,
        template_end: Arc<dyn ITemplateEnd>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.handler.handle_template_end(template_end)
    }

    fn handle_xml_declaration(
        &mut self,
        xml_declaration: Arc<dyn IXMLDeclaration>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let context = self
            .handler
            .get_context()
            .ok_or_else(|| Self::handler_error("PreProcessor context has not been set"))?;
        let encoding = Self::suffixed(xml_declaration.get_encoding(), "pre", self.xml_declarations);
        self.xml_declarations = self.xml_declarations.wrapping_add(1);
        let transformed = context
            .get_model_factory()
            .create_xml_declaration(
                xml_declaration.get_version().cloned(),
                Some(encoding),
                xml_declaration.get_standalone().cloned(),
            )
            .map_err(Self::processing_error)?;
        self.handler.handle_xml_declaration(transformed)
    }

    fn handle_doc_type(
        &mut self,
        doc_type: Arc<dyn IDocType>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let context = self
            .handler
            .get_context()
            .ok_or_else(|| Self::handler_error("PreProcessor context has not been set"))?;
        let internal_subset = Self::suffixed(doc_type.get_internal_subset(), "pre", self.doc_types);
        self.doc_types = self.doc_types.wrapping_add(1);
        let transformed = context
            .get_model_factory()
            .create_full_doc_type(
                doc_type
                    .get_keyword()
                    .cloned()
                    .unwrap_or_else(|| Utf16String::from_rust_str("DOCTYPE")),
                doc_type
                    .get_element_name()
                    .cloned()
                    .unwrap_or_else(|| Utf16String::from_rust_str("html")),
                doc_type.get_public_id().cloned(),
                doc_type.get_system_id().cloned(),
                Some(internal_subset),
            )
            .map_err(Self::processing_error)?;
        self.handler.handle_doc_type(transformed)
    }

    fn handle_cdata_section(
        &mut self,
        cdata_section: Arc<dyn ICDATASection>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let context = self
            .handler
            .get_context()
            .ok_or_else(|| Self::handler_error("PreProcessor context has not been set"))?;
        let content = cdata_section
            .get_content()
            .map_err(Self::processing_error)?;
        let transformed = context
            .get_model_factory()
            .create_cdata_section(Self::suffixed(content.as_ref(), "pre", self.cdata_sections))
            .map_err(Self::processing_error)?;
        self.cdata_sections = self.cdata_sections.wrapping_add(1);
        self.handler.handle_cdata_section(transformed)
    }

    fn handle_comment(
        &mut self,
        comment: Arc<dyn IComment>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let context = self
            .handler
            .get_context()
            .ok_or_else(|| Self::handler_error("PreProcessor context has not been set"))?;
        let content = comment.get_content().map_err(Self::processing_error)?;
        let transformed = context
            .get_model_factory()
            .create_comment(Self::suffixed(content.as_ref(), "pre", self.comments))
            .map_err(Self::processing_error)?;
        self.comments = self.comments.wrapping_add(1);
        self.handler.handle_comment(transformed)
    }

    fn handle_text(
        &mut self,
        text: Arc<dyn IText>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let context = self
            .handler
            .get_context()
            .ok_or_else(|| Self::handler_error("PreProcessor context has not been set"))?;
        let content = text.get_text().map_err(Self::processing_error)?;
        let transformed = context.get_model_factory().create_text(Self::suffixed(
            content.as_ref(),
            "pre",
            self.texts,
        ));
        self.texts = self.texts.wrapping_add(1);
        self.handler.handle_text(transformed)
    }

    fn handle_standalone_element(
        &mut self,
        tag: Arc<dyn IStandaloneElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let context = self
            .handler
            .get_context()
            .ok_or_else(|| Self::handler_error("PreProcessor context has not been set"))?;
        let processable = tag
            .clone()
            .into_processable_element_tag()
            .ok_or_else(|| Self::handler_error("Standalone tag is not processable"))?;
        let transformed = context
            .get_model_factory()
            .set_attribute(
                processable,
                Utf16String::from_rust_str("pre"),
                Some(Utf16String::from_rust_str(
                    &self.standalone_element_tags.to_string(),
                )),
                None,
            )
            .map_err(Self::processing_error)?
            .into_standalone_element_tag()
            .ok_or_else(|| Self::handler_error("Modified standalone tag changed its kind"))?;
        self.standalone_element_tags = self.standalone_element_tags.wrapping_add(1);
        self.handler.handle_standalone_element(transformed)
    }

    fn handle_open_element(
        &mut self,
        tag: Arc<dyn IOpenElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let context = self
            .handler
            .get_context()
            .ok_or_else(|| Self::handler_error("PreProcessor context has not been set"))?;
        let processable = tag
            .clone()
            .into_processable_element_tag()
            .ok_or_else(|| Self::handler_error("Open tag is not processable"))?;
        let transformed = context
            .get_model_factory()
            .set_attribute(
                processable,
                Utf16String::from_rust_str("pre"),
                Some(Utf16String::from_rust_str(
                    &self.open_element_tags.to_string(),
                )),
                None,
            )
            .map_err(Self::processing_error)?
            .into_open_element_tag()
            .ok_or_else(|| Self::handler_error("Modified open tag changed its kind"))?;
        self.open_element_tags = self.open_element_tags.wrapping_add(1);
        self.handler.handle_open_element(transformed)
    }

    fn handle_close_element(
        &mut self,
        tag: Arc<dyn ICloseElementTag>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        self.handler.handle_close_element(tag)
    }

    fn handle_processing_instruction(
        &mut self,
        instruction: Arc<dyn IProcessingInstruction>,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        let context = self
            .handler
            .get_context()
            .ok_or_else(|| Self::handler_error("PreProcessor context has not been set"))?;
        let content = Self::suffixed(
            instruction.get_content(),
            "pre",
            self.processing_instructions,
        );
        self.processing_instructions = self.processing_instructions.wrapping_add(1);
        let transformed = context
            .get_model_factory()
            .create_processing_instruction(
                instruction
                    .get_target()
                    .cloned()
                    .unwrap_or_else(|| Utf16String::from_rust_str("")),
                content,
            )
            .map_err(Self::processing_error)?;
        self.handler.handle_processing_instruction(transformed)
    }
}
