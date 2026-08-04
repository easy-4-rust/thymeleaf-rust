use std::cell::RefCell;
use std::error::Error;
use std::fmt::Write as _;
use std::fmt::{Display, Formatter};
use std::io;
use std::sync::Arc;

use crate::IEngineConfiguration;
use crate::TemplateMode;
use crate::cdatasection::ICDATASectionStructureHandler;
use crate::comment::ICommentStructureHandler;
use crate::doctype::IDocTypeStructureHandler;
use crate::exceptions::{TemplateEngineException, TemplateProcessingException};
use crate::model::{IModel, IModelError, IModelVisitor, ITemplateEvent};
use crate::processinginstruction::IProcessingInstructionStructureHandler;
use crate::processor::AbstractProcessorAdapter;
use crate::templateboundaries::ITemplateBoundariesStructureHandler;
use crate::text::ITextStructureHandler;
use crate::util::{CharSequenceValue, TemplateWriter, Utf16String, ValidateError};
use crate::xmldeclaration::IXMLDeclarationStructureHandler;

use super::Text;
use super::cdata_section_structure_handler::CDATASectionStructureHandler;
use super::comment_structure_handler::CommentStructureHandler;
use super::doc_type_structure_handler::DocTypeStructureHandler;
use super::processing_instruction_structure_handler::ProcessingInstructionStructureHandler;
use super::template_boundaries_structure_handler::TemplateBoundariesStructureHandler;
use super::text_structure_handler::TextStructureHandler;
use super::xml_declaration_structure_handler::XMLDeclarationStructureHandler;

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../tests/fixtures/non_element_structure_handler_golden.txt");

#[test]
fn non_element_structure_handlers_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit_text(&mut output);
    emit_cdata(&mut output);
    emit_comment(&mut output);
    emit_doc_type(&mut output);
    emit_processing_instruction(&mut output);
    emit_xml_declaration(&mut output);
    emit_template_boundaries(&mut output);
    emit_abstract_processor_exceptions(&mut output);

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_text(output: &mut String) {
    let mut handler = TextStructureHandler::new();
    emit(output, "text.new", text_state(&handler));

    let sequence: Arc<dyn CharSequenceValue> = Arc::new(Utf16String::from_rust_str("alpha"));
    handler.set_text_sequence(Arc::clone(&sequence));
    let stored = handler
        .set_text_value
        .as_ref()
        .expect("setText stores the sequence");
    emit(
        output,
        "text.sequence",
        format!(
            "{},identity={}",
            text_state(&handler),
            Arc::ptr_eq(stored, &sequence)
        ),
    );

    handler.remove_text();
    emit_validate_error(output, "text.null", handler.set_text_nullable(None));
    emit(output, "text.null.state", text_state(&handler));

    handler.remove_text();
    emit_validate_error(
        output,
        "text.model.null",
        handler.replace_with_nullable(None, true),
    );
    emit(output, "text.model.null.state", text_state(&handler));
}

fn emit_cdata(output: &mut String) {
    let mut handler = CDATASectionStructureHandler::new();
    handler.remove_cdata_section();
    emit_validate_error(output, "cdata.null", handler.set_content_nullable(None));
    emit(
        output,
        "cdata.null.state",
        format!(
            "{},{},{}",
            handler.set_content, handler.replace_with_model, handler.remove_cdata_section
        ),
    );
    handler.remove_cdata_section();
    emit_validate_error(
        output,
        "cdata.model.null",
        handler.replace_with_nullable(None, false),
    );
    emit(
        output,
        "cdata.model.null.state",
        format!(
            "{},{},{}",
            handler.set_content, handler.replace_with_model, handler.remove_cdata_section
        ),
    );
}

fn emit_comment(output: &mut String) {
    let mut handler = CommentStructureHandler::new();
    handler.remove_comment();
    emit_validate_error(output, "comment.null", handler.set_content_nullable(None));
    emit(
        output,
        "comment.null.state",
        format!(
            "{},{},{}",
            handler.set_content, handler.replace_with_model, handler.remove_comment
        ),
    );
    handler.remove_comment();
    emit_validate_error(
        output,
        "comment.model.null",
        handler.replace_with_nullable(None, true),
    );
    emit(
        output,
        "comment.model.null.state",
        format!(
            "{},{},{}",
            handler.set_content, handler.replace_with_model, handler.remove_comment
        ),
    );
}

fn emit_doc_type(output: &mut String) {
    let mut handler = DocTypeStructureHandler::new();
    handler.remove_doc_type();
    emit_validate_error(
        output,
        "doctype.keyword.null",
        handler.set_doc_type_nullable(
            None,
            None,
            Some(Utf16String::from_rust_str("public")),
            Some(Utf16String::from_rust_str("system")),
            Some(Utf16String::from_rust_str("subset")),
        ),
    );
    emit(
        output,
        "doctype.keyword.null.state",
        doc_type_state(&handler),
    );

    handler.remove_doc_type();
    emit_validate_error(
        output,
        "doctype.element.null",
        handler.set_doc_type_nullable(
            Some(Utf16String::from_rust_str("DOCTYPE")),
            None,
            Some(Utf16String::from_rust_str("public")),
            Some(Utf16String::from_rust_str("system")),
            Some(Utf16String::from_rust_str("subset")),
        ),
    );
    emit(
        output,
        "doctype.element.null.state",
        doc_type_state(&handler),
    );

    handler
        .set_doc_type_nullable(
            Some(Utf16String::from_rust_str("DOCTYPE")),
            Some(Utf16String::from_rust_str("html")),
            None,
            None,
            None,
        )
        .expect("valid DOCTYPE");
    emit(
        output,
        "doctype.optional.null",
        format!(
            "{},keyword={},element={},public=null,system=null,subset=null",
            doc_type_state(&handler),
            utf16_string(handler.set_doc_type_keyword.as_ref()),
            utf16_string(handler.set_doc_type_element_name.as_ref())
        ),
    );
}

fn emit_processing_instruction(output: &mut String) {
    let mut handler = ProcessingInstructionStructureHandler::new();
    handler.remove_processing_instruction();
    emit_validate_error(
        output,
        "pi.target.null",
        handler.set_processing_instruction_nullable(None, None),
    );
    emit(
        output,
        "pi.target.null.state",
        processing_instruction_state(&handler),
    );

    handler.remove_processing_instruction();
    emit_validate_error(
        output,
        "pi.content.null",
        handler.set_processing_instruction_nullable(Some(Utf16String::from_rust_str("xml")), None),
    );
    emit(
        output,
        "pi.content.null.state",
        processing_instruction_state(&handler),
    );

    handler
        .set_processing_instruction_nullable(
            Some(Utf16String::from_rust_str("xml")),
            Some(Utf16String::from_rust_str("content")),
        )
        .expect("valid processing instruction");
    emit(
        output,
        "pi.valid",
        format!(
            "{},target={},content={}",
            processing_instruction_state(&handler),
            utf16_string(handler.set_processing_instruction_target.as_ref()),
            utf16_string(handler.set_processing_instruction_content.as_ref())
        ),
    );
}

fn emit_xml_declaration(output: &mut String) {
    let mut handler = XMLDeclarationStructureHandler::new();
    handler.remove_xml_declaration();
    emit_validate_error(
        output,
        "xml.keyword.null",
        handler.set_xml_declaration_nullable(
            None,
            Some(Utf16String::from_rust_str("1.0")),
            Some(Utf16String::from_rust_str("UTF-8")),
            Some(Utf16String::from_rust_str("yes")),
        ),
    );
    emit(
        output,
        "xml.keyword.null.state",
        xml_declaration_state(&handler),
    );

    handler
        .set_xml_declaration_nullable(Some(Utf16String::from_rust_str("xml")), None, None, None)
        .expect("valid XML declaration");
    emit(
        output,
        "xml.optional.null",
        format!(
            "{},keyword={},version=null,encoding=null,standalone=null",
            xml_declaration_state(&handler),
            utf16_string(handler.set_xml_declaration_keyword.as_ref())
        ),
    );
}

fn emit_template_boundaries(output: &mut String) {
    let mut handler = TemplateBoundariesStructureHandler::new();
    handler.set_local_variable(None, None);
    handler.set_local_variable(None, None);
    handler.remove_local_variable(None);
    handler.remove_local_variable(None);
    handler.set_selection_target(None);
    handler.set_inliner(None);
    emit(output, "boundary.null.context", boundary_state(&handler));

    let context_calls = RefCell::new(Vec::new());
    handler.apply_context_modifications_with(
        |_| context_calls.borrow_mut().push("setVariables".to_owned()),
        |name| {
            context_calls.borrow_mut().push(format!(
                "removeVariable:{}",
                name.map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy)
            ));
        },
        |selection_target| {
            context_calls.borrow_mut().push(format!(
                "setSelectionTarget:{}",
                if selection_target.is_none() {
                    "null"
                } else {
                    "value"
                }
            ));
        },
        |inliner| {
            context_calls.borrow_mut().push(format!(
                "setInliner:{}",
                if inliner.is_none() { "null" } else { "value" }
            ));
        },
    );
    emit(
        output,
        "boundary.apply.order",
        format!("[{}]", context_calls.borrow().join(", ")),
    );

    handler.insert_text(Utf16String::from_rust_str("before"), true);
    emit(
        output,
        "boundary.text",
        format!(
            "{},text={},processable={}",
            boundary_state(&handler),
            utf16_string(handler.insert_text_value.as_ref()),
            handler.insert_text_processable
        ),
    );

    emit_validate_error(
        output,
        "boundary.text.null",
        handler.insert_text_nullable(None, false),
    );
    emit(output, "boundary.text.null.state", boundary_state(&handler));

    let model: Arc<dyn IModel> = Arc::new(StoredOnlyModel);
    handler.insert_model(Arc::clone(&model), false);
    emit(
        output,
        "boundary.model",
        format!(
            "{},identity={},processable={}",
            boundary_state(&handler),
            Arc::ptr_eq(
                handler
                    .insert_model_value
                    .as_ref()
                    .expect("insertModel stores model"),
                &model
            ),
            handler.insert_model_processable
        ),
    );

    emit_validate_error(
        output,
        "boundary.model.null",
        handler.insert_model_nullable(None, true),
    );
    emit(
        output,
        "boundary.model.null.state",
        boundary_state(&handler),
    );

    handler.reset();
    emit(output, "boundary.reset", boundary_state(&handler));
}

fn emit_abstract_processor_exceptions(output: &mut String) {
    let adapter = AbstractProcessorAdapter::new(
        Some(TemplateMode::HTML),
        100,
        "org.thymeleaf.engine.NonElementStructureHandlerGolden$ThrowingTextProcessor",
        (),
    )
    .expect("valid abstract processor");

    let no_location_event = Text::new(Some(Arc::new(Utf16String::from_rust_str("x"))));
    let no_location = Box::new(TemplateProcessingException::new(Some("plain".to_owned())));
    let no_location_identity = (&*no_location as *const TemplateProcessingException).cast::<()>();
    let mut returned = adapter
        .execute(&no_location_event, |_| Err(no_location))
        .expect_err("processor must return the callback exception");
    let returned_identity = (returned
        .as_processing_exception_mut()
        .expect("processing exception")
        as *mut TemplateProcessingException
        as *const TemplateProcessingException)
        .cast::<()>();
    emit(
        output,
        "processor.tpe.noLocation",
        format!(
            "{},identity={}",
            processing_exception_state(&mut returned),
            returned_identity == no_location_identity
        ),
    );

    let located_event = Text::with_location(
        Some(Arc::new(Utf16String::from_rust_str("x"))),
        Some(Utf16String::from_rust_str("page.html")),
        7,
        11,
    );
    let enrich = Box::new(TemplateProcessingException::new(Some("enrich".to_owned())));
    let enrich_identity = (&*enrich as *const TemplateProcessingException).cast::<()>();
    let mut returned = adapter
        .execute(&located_event, |_| Err(enrich))
        .expect_err("processor must return the callback exception");
    let returned_identity = (returned
        .as_processing_exception_mut()
        .expect("processing exception")
        as *mut TemplateProcessingException
        as *const TemplateProcessingException)
        .cast::<()>();
    emit(
        output,
        "processor.tpe.enrich",
        format!(
            "{},identity={}",
            processing_exception_state(&mut returned),
            returned_identity == enrich_identity
        ),
    );

    let preserve = Box::new(TemplateProcessingException::with_location(
        Some("preserve".to_owned()),
        Some("own.html".to_owned()),
        3,
        4,
    ));
    let preserve_identity = (&*preserve as *const TemplateProcessingException).cast::<()>();
    let mut returned = adapter
        .execute(&located_event, |_| Err(preserve))
        .expect_err("processor must return the callback exception");
    let returned_identity = (returned
        .as_processing_exception_mut()
        .expect("processing exception")
        as *mut TemplateProcessingException
        as *const TemplateProcessingException)
        .cast::<()>();
    emit(
        output,
        "processor.tpe.preserve",
        format!(
            "{},identity={}",
            processing_exception_state(&mut returned),
            returned_identity == preserve_identity
        ),
    );

    let mut wrapped = adapter
        .execute(&located_event, |_| {
            Err(Box::new(IllegalStateCause) as Box<dyn TemplateEngineException>)
        })
        .expect_err("processor must wrap non-processing exceptions");
    assert_eq!(
        wrapped
            .source()
            .and_then(Error::source)
            .map(ToString::to_string)
            .as_deref(),
        Some("boom")
    );
    emit(
        output,
        "processor.wrap",
        format!(
            "{},cause=java.lang.IllegalStateException:boom",
            processing_exception_state(&mut wrapped)
        ),
    );
}

fn processing_exception_state(error: &mut Box<dyn TemplateEngineException>) -> String {
    let processing = error
        .as_processing_exception_mut()
        .expect("expected TemplateProcessingException");
    format!(
        "message={},template={},line={},col={}",
        processing.get_message(),
        processing.get_template_name().unwrap_or("null"),
        processing
            .get_line()
            .map_or_else(|| "null".to_owned(), |line| line.to_string()),
        processing
            .get_col()
            .map_or_else(|| "null".to_owned(), |col| col.to_string())
    )
}

fn text_state(handler: &TextStructureHandler) -> String {
    format!(
        "{},{},{}",
        handler.set_text, handler.replace_with_model, handler.remove_text
    )
}

fn doc_type_state(handler: &DocTypeStructureHandler) -> String {
    format!(
        "{},{},{}",
        handler.set_doc_type, handler.replace_with_model, handler.remove_doc_type
    )
}

fn processing_instruction_state(handler: &ProcessingInstructionStructureHandler) -> String {
    format!(
        "{},{},{}",
        handler.set_processing_instruction,
        handler.replace_with_model,
        handler.remove_processing_instruction
    )
}

fn xml_declaration_state(handler: &XMLDeclarationStructureHandler) -> String {
    format!(
        "{},{},{}",
        handler.set_xml_declaration, handler.replace_with_model, handler.remove_xml_declaration
    )
}

fn boundary_state(handler: &TemplateBoundariesStructureHandler) -> String {
    format!(
        "text={},model={},set={},setSize={},setNull={},remove={},removeSize={},removeNull={},selection={},selectionNull={},inliner={},inlinerNull={}",
        handler.insert_text,
        handler.insert_model,
        handler.set_local_variable,
        handler.added_local_variables.len(),
        handler.added_local_variables.contains_key(&None),
        handler.remove_local_variable,
        handler.removed_local_variable_names.len(),
        handler.removed_local_variable_names.contains(&None),
        handler.set_selection_target,
        handler.selection_target_object.is_none(),
        handler.set_inliner,
        handler.set_inliner_value.is_none()
    )
}

fn emit_validate_error(output: &mut String, key: &str, result: Result<(), ValidateError>) {
    match result {
        Ok(()) => emit(output, key, "NONE"),
        Err(error) => emit(
            output,
            key,
            format!("{}:{}", error.java_class_name(), error),
        ),
    }
}

fn utf16_string(value: Option<&Utf16String>) -> String {
    value.map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy)
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("write golden output");
}

/// 只用于验证 StructureHandler 保存模型共享身份；任何模型操作均不应发生。
struct StoredOnlyModel;

#[derive(Debug)]
struct IllegalStateCause;

impl Display for IllegalStateCause {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("boom")
    }
}

impl Error for IllegalStateCause {}

impl TemplateEngineException for IllegalStateCause {}

impl IModel for StoredOnlyModel {
    fn get_configuration(&self) -> &dyn IEngineConfiguration {
        panic!("stored-only model must not be queried")
    }

    fn get_template_mode(&self) -> TemplateMode {
        panic!("stored-only model must not be queried")
    }

    fn size(&self) -> usize {
        panic!("stored-only model must not be queried")
    }

    fn get(&self, _pos: usize) -> Arc<dyn ITemplateEvent> {
        panic!("stored-only model must not be queried")
    }

    fn add(&mut self, _event: Option<Arc<dyn ITemplateEvent>>) -> Result<(), IModelError> {
        panic!("stored-only model must not be mutated")
    }

    fn insert(
        &mut self,
        _pos: usize,
        _event: Option<Arc<dyn ITemplateEvent>>,
    ) -> Result<(), IModelError> {
        panic!("stored-only model must not be mutated")
    }

    fn replace(
        &mut self,
        _pos: usize,
        _event: Option<Arc<dyn ITemplateEvent>>,
    ) -> Result<(), IModelError> {
        panic!("stored-only model must not be mutated")
    }

    fn add_model(&mut self, _model: Option<&dyn IModel>) -> Result<(), IModelError> {
        panic!("stored-only model must not be mutated")
    }

    fn insert_model(
        &mut self,
        _pos: usize,
        _model: Option<&dyn IModel>,
    ) -> Result<(), IModelError> {
        panic!("stored-only model must not be mutated")
    }

    fn remove(&mut self, _pos: usize) -> Result<(), IModelError> {
        panic!("stored-only model must not be mutated")
    }

    fn reset(&mut self) -> Result<(), IModelError> {
        panic!("stored-only model must not be mutated")
    }

    fn clone_model(&self) -> Box<dyn IModel> {
        panic!("stored-only model must not be cloned")
    }

    fn accept(&self, _visitor: &mut dyn IModelVisitor) {
        panic!("stored-only model must not be visited")
    }

    fn write(&self, _writer: &mut dyn TemplateWriter) -> io::Result<()> {
        panic!("stored-only model must not be written")
    }
}
