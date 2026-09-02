//! VALUE_ADD：`ProcessorConfigurationUtils` 覆盖缺口测试（2026-09-02）——风险：方言装饰契约
//! （unwrap 在 wrapped 时返回内层、unwrapped 时返回自身）各处理器类型 wrap/unwrap 往返未验证。
//!
//! Java 侧 `ProcessorConfigurationUtils` 无独立测试类；覆盖来自 `DialectSetConfiguration` 集成路径。
//! 以下测试以 VALUE_ADD 方式直接验证 wrap + unwrap 往返契约，补齐单元级覆盖缺口。

use std::sync::Arc;

use thymeleaf::cdatasection::{ICDATASectionProcessor, ICDATASectionStructureHandler};
use thymeleaf::comment::{ICommentProcessor, ICommentStructureHandler};
use thymeleaf::context::ITemplateContext;
use thymeleaf::doctype::{IDocTypeProcessor, IDocTypeStructureHandler};
use thymeleaf::exceptions::TemplateEngineException;
use thymeleaf::model::{
    ICDATASection, IComment, IDocType, IProcessingInstruction, ITemplateEnd, ITemplateStart,
    IText, IXMLDeclaration,
};
use thymeleaf::processinginstruction::{
    IProcessingInstructionProcessor, IProcessingInstructionStructureHandler,
};
use thymeleaf::templateboundaries::{
    ITemplateBoundariesProcessor, ITemplateBoundariesStructureHandler,
};
use thymeleaf::text::{ITextProcessor, ITextStructureHandler};
use thymeleaf::util::ProcessorConfigurationUtils;
use thymeleaf::xmldeclaration::{IXMLDeclarationProcessor, IXMLDeclarationStructureHandler};
use thymeleaf::{IProcessor, TemplateMode};

// ===========================================================================
// Mock processors that implement IProcessor with capability discovery
// ===========================================================================

struct MockTextProcessor(i32);

impl IProcessor for MockTextProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(TemplateMode::HTML)
    }
    fn get_precedence(&self) -> i32 {
        self.0
    }
    fn as_text_processor(&self) -> Option<&dyn ITextProcessor> {
        Some(self)
    }
}

impl ITextProcessor for MockTextProcessor {
    fn process(
        &self,
        _: &dyn ITemplateContext,
        _: &dyn IText,
        _: &mut dyn ITextStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
}

struct MockCommentProcessor(i32);

impl IProcessor for MockCommentProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(TemplateMode::HTML)
    }
    fn get_precedence(&self) -> i32 {
        self.0
    }
    fn as_comment_processor(&self) -> Option<&dyn ICommentProcessor> {
        Some(self)
    }
}

impl ICommentProcessor for MockCommentProcessor {
    fn process(
        &self,
        _: &dyn ITemplateContext,
        _: &dyn IComment,
        _: &mut dyn ICommentStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
}

struct MockCDATAProcessor(i32);

impl IProcessor for MockCDATAProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(TemplateMode::XML)
    }
    fn get_precedence(&self) -> i32 {
        self.0
    }
    fn as_cdata_section_processor(&self) -> Option<&dyn ICDATASectionProcessor> {
        Some(self)
    }
}

impl ICDATASectionProcessor for MockCDATAProcessor {
    fn process(
        &self,
        _: &dyn ITemplateContext,
        _: &dyn ICDATASection,
        _: &mut dyn ICDATASectionStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
}

struct MockDocTypeProcessor(i32);

impl IProcessor for MockDocTypeProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(TemplateMode::HTML)
    }
    fn get_precedence(&self) -> i32 {
        self.0
    }
    fn as_doc_type_processor(&self) -> Option<&dyn IDocTypeProcessor> {
        Some(self)
    }
}

impl IDocTypeProcessor for MockDocTypeProcessor {
    fn process(
        &self,
        _: &dyn ITemplateContext,
        _: &dyn IDocType,
        _: &mut dyn IDocTypeStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
}

struct MockPIProcessor(i32);

impl IProcessor for MockPIProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(TemplateMode::XML)
    }
    fn get_precedence(&self) -> i32 {
        self.0
    }
    fn as_processing_instruction_processor(
        &self,
    ) -> Option<&dyn IProcessingInstructionProcessor> {
        Some(self)
    }
}

impl IProcessingInstructionProcessor for MockPIProcessor {
    fn process(
        &self,
        _: &dyn ITemplateContext,
        _: &dyn IProcessingInstruction,
        _: &mut dyn IProcessingInstructionStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
}

struct MockBoundariesProcessor(i32);

impl IProcessor for MockBoundariesProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(TemplateMode::HTML)
    }
    fn get_precedence(&self) -> i32 {
        self.0
    }
    fn as_template_boundaries_processor(
        &self,
    ) -> Option<&dyn ITemplateBoundariesProcessor> {
        Some(self)
    }
}

impl ITemplateBoundariesProcessor for MockBoundariesProcessor {
    fn process_template_start(
        &self,
        _: &dyn ITemplateContext,
        _: &dyn ITemplateStart,
        _: &mut dyn ITemplateBoundariesStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
    fn process_template_end(
        &self,
        _: &dyn ITemplateContext,
        _: &dyn ITemplateEnd,
        _: &mut dyn ITemplateBoundariesStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
}

struct MockXMLDeclProcessor(i32);

impl IProcessor for MockXMLDeclProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(TemplateMode::XML)
    }
    fn get_precedence(&self) -> i32 {
        self.0
    }
    fn as_xml_declaration_processor(&self) -> Option<&dyn IXMLDeclarationProcessor> {
        Some(self)
    }
}

impl IXMLDeclarationProcessor for MockXMLDeclProcessor {
    fn process(
        &self,
        _: &dyn ITemplateContext,
        _: &dyn IXMLDeclaration,
        _: &mut dyn IXMLDeclarationStructureHandler,
    ) -> Result<(), Box<dyn TemplateEngineException>> {
        Ok(())
    }
}

// ===========================================================================
// unwrap_text: wrapped -> inner; bare -> self
// ===========================================================================

#[test]
fn unwrap_text_returns_inner_when_wrapped() {
    let raw: Arc<dyn IProcessor> = Arc::new(MockTextProcessor(100));
    let wrapped = ProcessorConfigurationUtils::wrap_text(Arc::clone(&raw), 200)
        .expect("text processor must be wrappable");
    let wrapped_ref: &dyn ITextProcessor = wrapped.as_ref();
    let unwrapped = ProcessorConfigurationUtils::unwrap_text(wrapped_ref);
    assert_eq!(unwrapped.get_precedence(), 100, "unwrapped must carry original precedence");
    assert!(unwrapped.get_dialect_precedence().is_none(), "unwrapped must not carry dialect precedence");
}

#[test]
fn unwrap_text_returns_self_when_not_wrapped() {
    let raw = MockTextProcessor(50);
    let raw_ref: &dyn ITextProcessor = &raw;
    let unwrapped = ProcessorConfigurationUtils::unwrap_text(raw_ref);
    assert_eq!(unwrapped.get_precedence(), 50, "bare unwrap must return self");
}

// ===========================================================================
// unwrap_comment: wrapped -> inner; bare -> self
// ===========================================================================

#[test]
fn unwrap_comment_returns_inner_when_wrapped() {
    let raw: Arc<dyn IProcessor> = Arc::new(MockCommentProcessor(110));
    let wrapped = ProcessorConfigurationUtils::wrap_comment(Arc::clone(&raw), 300)
        .expect("comment processor must be wrappable");
    let wrapped_ref: &dyn ICommentProcessor = wrapped.as_ref();
    let unwrapped = ProcessorConfigurationUtils::unwrap_comment(wrapped_ref);
    assert_eq!(unwrapped.get_precedence(), 110);
    assert!(unwrapped.get_dialect_precedence().is_none());
}

#[test]
fn unwrap_comment_returns_self_when_not_wrapped() {
    let raw = MockCommentProcessor(60);
    let raw_ref: &dyn ICommentProcessor = &raw;
    let unwrapped = ProcessorConfigurationUtils::unwrap_comment(raw_ref);
    assert_eq!(unwrapped.get_precedence(), 60);
}

// ===========================================================================
// unwrap_cdata_section: wrapped -> inner
// ===========================================================================

#[test]
fn unwrap_cdata_section_returns_inner_when_wrapped() {
    let raw: Arc<dyn IProcessor> = Arc::new(MockCDATAProcessor(120));
    let wrapped = ProcessorConfigurationUtils::wrap_cdata_section(Arc::clone(&raw), 400)
        .expect("cdata processor must be wrappable");
    let wrapped_ref: &dyn ICDATASectionProcessor = wrapped.as_ref();
    let unwrapped = ProcessorConfigurationUtils::unwrap_cdata_section(wrapped_ref);
    assert_eq!(unwrapped.get_precedence(), 120);
    assert!(unwrapped.get_dialect_precedence().is_none());
}

// ===========================================================================
// unwrap_doc_type: wrapped -> inner
// ===========================================================================

#[test]
fn unwrap_doc_type_returns_inner_when_wrapped() {
    let raw: Arc<dyn IProcessor> = Arc::new(MockDocTypeProcessor(130));
    let wrapped = ProcessorConfigurationUtils::wrap_doc_type(Arc::clone(&raw), 500)
        .expect("doc type processor must be wrappable");
    let wrapped_ref: &dyn IDocTypeProcessor = wrapped.as_ref();
    let unwrapped = ProcessorConfigurationUtils::unwrap_doc_type(wrapped_ref);
    assert_eq!(unwrapped.get_precedence(), 130);
    assert!(unwrapped.get_dialect_precedence().is_none());
}

// ===========================================================================
// unwrap_processing_instruction: wrapped -> inner
// ===========================================================================

#[test]
fn unwrap_processing_instruction_returns_inner_when_wrapped() {
    let raw: Arc<dyn IProcessor> = Arc::new(MockPIProcessor(140));
    let wrapped = ProcessorConfigurationUtils::wrap_processing_instruction(Arc::clone(&raw), 600)
        .expect("pi processor must be wrappable");
    let wrapped_ref: &dyn IProcessingInstructionProcessor = wrapped.as_ref();
    let unwrapped = ProcessorConfigurationUtils::unwrap_processing_instruction(wrapped_ref);
    assert_eq!(unwrapped.get_precedence(), 140);
    assert!(unwrapped.get_dialect_precedence().is_none());
}

// ===========================================================================
// unwrap_template_boundaries: wrapped -> inner
// ===========================================================================

#[test]
fn unwrap_template_boundaries_returns_inner_when_wrapped() {
    let raw: Arc<dyn IProcessor> = Arc::new(MockBoundariesProcessor(150));
    let wrapped = ProcessorConfigurationUtils::wrap_template_boundaries(Arc::clone(&raw), 700)
        .expect("boundaries processor must be wrappable");
    let wrapped_ref: &dyn ITemplateBoundariesProcessor = wrapped.as_ref();
    let unwrapped = ProcessorConfigurationUtils::unwrap_template_boundaries(wrapped_ref);
    assert_eq!(unwrapped.get_precedence(), 150);
    assert!(unwrapped.get_dialect_precedence().is_none());
}

// ===========================================================================
// unwrap_xml_declaration: wrapped -> inner
// ===========================================================================

#[test]
fn unwrap_xml_declaration_returns_inner_when_wrapped() {
    let raw: Arc<dyn IProcessor> = Arc::new(MockXMLDeclProcessor(160));
    let wrapped = ProcessorConfigurationUtils::wrap_xml_declaration(Arc::clone(&raw), 800)
        .expect("xml decl processor must be wrappable");
    let wrapped_ref: &dyn IXMLDeclarationProcessor = wrapped.as_ref();
    let unwrapped = ProcessorConfigurationUtils::unwrap_xml_declaration(wrapped_ref);
    assert_eq!(unwrapped.get_precedence(), 160);
    assert!(unwrapped.get_dialect_precedence().is_none());
}

// ===========================================================================
// wrap preserves dialect precedence on wrapper, original precedence unchanged
// ===========================================================================

#[test]
fn wrapped_processor_carries_dialect_precedence() {
    let raw: Arc<dyn IProcessor> = Arc::new(MockTextProcessor(100));
    let wrapped = ProcessorConfigurationUtils::wrap_text(Arc::clone(&raw), 999)
        .expect("text processor must be wrappable");
    assert_eq!(wrapped.get_dialect_precedence(), Some(999), "wrapper must carry dialect precedence");
    assert_eq!(wrapped.get_precedence(), 100, "wrapper must preserve original precedence");
}

// ===========================================================================
// wrap_cdata_section rejects non-cdata processor (capability mismatch)
// ===========================================================================

#[test]
fn wrap_cdata_section_rejects_non_cdata_processor() {
    let text_processor: Arc<dyn IProcessor> = Arc::new(MockTextProcessor(10));
    let result = ProcessorConfigurationUtils::wrap_cdata_section(Arc::clone(&text_processor), 1);
    assert!(result.is_err(), "wrapping a text processor as CDATA must fail");
}
