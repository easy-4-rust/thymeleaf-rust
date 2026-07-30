use std::sync::{Arc, Weak};

use indexmap::IndexMap;

use crate::exceptions::TemplateProcessingException;
use crate::model::{
    AttributeValueQuotes, ICDATASection, ICloseElementTag, IComment, IDocType, IModel, IModelError,
    IModelFactory, IOpenElementTag, IProcessableElementTag, IProcessingInstruction,
    IStandaloneElementTag, ITemplateEvent, IText, IXMLDeclaration,
};
use crate::util::{JavaCharSequence, JavaString};
use crate::{IEngineConfiguration, TemplateMode};

use super::{
    Attribute, AttributeDefinitions, AttributeName, Attributes, CDATASection, CloseElementTag,
    Comment, DocType, ElementDefinitions, OpenElementTag, ProcessingInstruction,
    StandaloneElementTag, TemplateData, Text, XMLDeclaration, model::Model,
};

const DEFAULT_WHITE_SPACE: &str = " ";
const XML_DECLARATION_KEYWORD: &str = "xml";

/// 标准模板模型及事件工厂。
///
/// 工厂绑定同一引擎配置与模板模式，创建不可变事件和可变模型；标签属性修改始终
/// 派生新标签，并在删除不存在属性时保留原 `Arc` 身份。
/// 对应 Java: `org.thymeleaf.engine.StandardModelFactory`。
pub struct StandardModelFactory {
    configuration: Weak<dyn IEngineConfiguration>,
    element_definitions: Arc<ElementDefinitions>,
    attribute_definitions: Arc<AttributeDefinitions>,
    template_mode: TemplateMode,
}

impl StandardModelFactory {
    /// 创建绑定指定引擎配置与模板模式的标准模型工厂。
    pub fn new(
        configuration: Arc<dyn IEngineConfiguration>,
        template_mode: TemplateMode,
        element_definitions: Arc<ElementDefinitions>,
        attribute_definitions: Arc<AttributeDefinitions>,
    ) -> Self {
        Self {
            configuration: Arc::downgrade(&configuration),
            element_definitions,
            attribute_definitions,
            template_mode,
        }
    }

    fn attribute_definitions(&self) -> &AttributeDefinitions {
        self.attribute_definitions.as_ref()
    }

    fn configuration(&self) -> Arc<dyn IEngineConfiguration> {
        self.configuration
            .upgrade()
            .expect("StandardModelFactory cannot outlive its EngineConfiguration")
    }

    fn check_restricted_event_for_text_template_mode(
        &self,
        event_class: &str,
    ) -> Result<(), TemplateProcessingException> {
        if self.template_mode.is_text() {
            return Err(TemplateProcessingException::new(Some(format!(
                "Events of class {event_class} cannot be created in a text-type template mode ({})",
                self.template_mode
            ))));
        }
        Ok(())
    }

    fn build_attributes(
        &self,
        attributes: Option<&IndexMap<JavaString, Option<JavaString>>>,
        quotes: AttributeValueQuotes,
    ) -> Result<Option<Arc<Attributes>>, TemplateProcessingException> {
        let Some(attributes) = attributes else {
            return Ok(None);
        };
        if attributes.is_empty() {
            return Ok(None);
        }

        let mut built = Vec::with_capacity(attributes.len());
        for (name, value) in attributes {
            let definition = self
                .attribute_definitions()
                .for_name(Some(self.template_mode), Some(name))
                .map_err(processing_error)?;
            built.push(Arc::new(Attribute::new(
                definition,
                name.clone(),
                None,
                value.clone(),
                Some(quotes),
                None,
                -1,
                -1,
            )));
        }
        let white_spaces = (0..built.len())
            .map(|_| JavaString::from_rust_str(DEFAULT_WHITE_SPACE))
            .collect();
        Ok(Some(Attributes::new(Some(built), Some(white_spaces))))
    }
}

impl IModelFactory for StandardModelFactory {
    fn create_model(&self) -> Box<dyn IModel> {
        Box::new(Model::new(self.configuration(), self.template_mode))
    }

    fn create_model_with_event(
        &self,
        event: Arc<dyn ITemplateEvent>,
    ) -> Result<Box<dyn IModel>, IModelError> {
        let mut model = Model::new(self.configuration(), self.template_mode);
        model.add(Some(event))?;
        Ok(Box::new(model))
    }

    fn parse(
        &self,
        owner_template: &TemplateData,
        template: &JavaString,
    ) -> Result<Box<dyn IModel>, TemplateProcessingException> {
        self.configuration()
            .get_template_manager()
            .parse_string(
                owner_template,
                template,
                0,
                0,
                Some(self.template_mode),
                false,
            )
            .map_err(|error| {
                TemplateProcessingException::with_cause(
                    Some("Error while parsing model text".to_owned()),
                    error,
                )
            })
    }

    fn create_cdata_section(
        &self,
        content: JavaString,
    ) -> Result<Arc<dyn ICDATASection>, TemplateProcessingException> {
        self.check_restricted_event_for_text_template_mode("CDATASection")?;
        let content: Arc<dyn JavaCharSequence> = Arc::new(content);
        Ok(Arc::new(CDATASection::new(Some(content))))
    }

    fn create_comment(
        &self,
        content: JavaString,
    ) -> Result<Arc<dyn IComment>, TemplateProcessingException> {
        self.check_restricted_event_for_text_template_mode("Comment")?;
        let content: Arc<dyn JavaCharSequence> = Arc::new(content);
        Ok(Arc::new(Comment::new(Some(content))))
    }

    fn create_html5_doc_type(&self) -> Result<Arc<dyn IDocType>, TemplateProcessingException> {
        self.check_restricted_event_for_text_template_mode("DocType")?;
        DocType::new()
            .map(|event| Arc::new(event) as Arc<dyn IDocType>)
            .map_err(processing_error)
    }

    fn create_doc_type(
        &self,
        public_id: Option<JavaString>,
        system_id: Option<JavaString>,
    ) -> Result<Arc<dyn IDocType>, TemplateProcessingException> {
        self.check_restricted_event_for_text_template_mode("DocType")?;
        DocType::with_ids(public_id, system_id)
            .map(|event| Arc::new(event) as Arc<dyn IDocType>)
            .map_err(processing_error)
    }

    fn create_full_doc_type(
        &self,
        keyword: JavaString,
        element_name: JavaString,
        public_id: Option<JavaString>,
        system_id: Option<JavaString>,
        internal_subset: Option<JavaString>,
    ) -> Result<Arc<dyn IDocType>, TemplateProcessingException> {
        self.check_restricted_event_for_text_template_mode("DocType")?;
        DocType::with_components(
            Some(keyword),
            Some(element_name),
            public_id,
            system_id,
            internal_subset,
        )
        .map(|event| Arc::new(event) as Arc<dyn IDocType>)
        .map_err(processing_error)
    }

    fn create_processing_instruction(
        &self,
        target: JavaString,
        content: JavaString,
    ) -> Result<Arc<dyn IProcessingInstruction>, TemplateProcessingException> {
        self.check_restricted_event_for_text_template_mode("ProcessingInstruction")?;
        Ok(Arc::new(ProcessingInstruction::new(
            Some(target),
            Some(content),
        )))
    }

    fn create_text(&self, text: JavaString) -> Arc<dyn IText> {
        let text: Arc<dyn JavaCharSequence> = Arc::new(text);
        Arc::new(Text::new(Some(text)))
    }

    fn create_xml_declaration(
        &self,
        version: Option<JavaString>,
        encoding: Option<JavaString>,
        standalone: Option<JavaString>,
    ) -> Result<Arc<dyn IXMLDeclaration>, TemplateProcessingException> {
        self.check_restricted_event_for_text_template_mode("XMLDeclaration")?;
        Ok(Arc::new(XMLDeclaration::with_components(
            Some(JavaString::from_rust_str(XML_DECLARATION_KEYWORD)),
            version,
            encoding,
            standalone,
        )))
    }

    fn create_standalone_element_tag(
        &self,
        element_name: JavaString,
        attributes: Option<&IndexMap<JavaString, Option<JavaString>>>,
        attribute_value_quotes: AttributeValueQuotes,
        synthetic: bool,
        minimized: bool,
    ) -> Result<Arc<dyn IStandaloneElementTag>, TemplateProcessingException> {
        let definition = self
            .element_definitions
            .for_name(Some(self.template_mode), Some(&element_name))
            .map_err(processing_error)?;
        let attributes = self.build_attributes(attributes, attribute_value_quotes)?;
        StandaloneElementTag::new(
            self.template_mode,
            definition,
            element_name,
            attributes,
            synthetic,
            minimized,
        )
        .map(|tag| Arc::new(tag) as Arc<dyn IStandaloneElementTag>)
        .map_err(processing_error)
    }

    fn create_open_element_tag(
        &self,
        element_name: JavaString,
        attributes: Option<&IndexMap<JavaString, Option<JavaString>>>,
        attribute_value_quotes: AttributeValueQuotes,
        synthetic: bool,
    ) -> Result<Arc<dyn IOpenElementTag>, TemplateProcessingException> {
        let definition = self
            .element_definitions
            .for_name(Some(self.template_mode), Some(&element_name))
            .map_err(processing_error)?;
        let attributes = self.build_attributes(attributes, attribute_value_quotes)?;
        Ok(Arc::new(OpenElementTag::new(
            self.template_mode,
            definition,
            element_name,
            attributes,
            synthetic,
        )))
    }

    fn create_close_element_tag(
        &self,
        element_name: JavaString,
        synthetic: bool,
        unmatched: bool,
    ) -> Result<Arc<dyn ICloseElementTag>, TemplateProcessingException> {
        let definition = self
            .element_definitions
            .for_name(Some(self.template_mode), Some(&element_name))
            .map_err(processing_error)?;
        Ok(Arc::new(CloseElementTag::new(
            self.template_mode,
            definition,
            element_name,
            None,
            synthetic,
            unmatched,
        )))
    }

    fn set_attribute(
        &self,
        tag: Arc<dyn IProcessableElementTag>,
        attribute_name: JavaString,
        attribute_value: Option<JavaString>,
        attribute_value_quotes: Option<AttributeValueQuotes>,
    ) -> Result<Arc<dyn IProcessableElementTag>, TemplateProcessingException> {
        tag.with_attribute(
            self.attribute_definitions(),
            None,
            attribute_name,
            attribute_value,
            attribute_value_quotes,
        )
        .map_err(processing_error)
    }

    fn replace_attribute(
        &self,
        tag: Arc<dyn IProcessableElementTag>,
        old_attribute_name: &AttributeName,
        attribute_name: JavaString,
        attribute_value: Option<JavaString>,
        attribute_value_quotes: Option<AttributeValueQuotes>,
    ) -> Result<Arc<dyn IProcessableElementTag>, TemplateProcessingException> {
        tag.with_replaced_attribute(
            self.attribute_definitions(),
            old_attribute_name,
            None,
            attribute_name,
            attribute_value,
            attribute_value_quotes,
        )
        .map_err(processing_error)
    }

    fn remove_attribute(
        &self,
        tag: Arc<dyn IProcessableElementTag>,
        attribute_name: &AttributeName,
    ) -> Result<Arc<dyn IProcessableElementTag>, TemplateProcessingException> {
        Ok(tag.without_attribute(attribute_name))
    }
}

fn processing_error(error: impl std::fmt::Display) -> TemplateProcessingException {
    TemplateProcessingException::new(Some(error.to_string()))
}
