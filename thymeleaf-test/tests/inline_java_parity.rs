//! 基础 Inliner SPI 与 NoOp 单例的固定 Java Golden 差分和 Rust 共享义务。

use std::any::{Any, TypeId};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, OnceLock};

use indexmap::IndexMap;
use thymeleaf::context::{
    IContext, IContextVariableNames, IExpressionContext, ITemplateContext, IdentifierSequences,
};
use thymeleaf::engine::{CDATASection, Comment, TemplateData, Text};
use thymeleaf::expression::{IExpressionObjects, StandardExpressionResult, TemplateValue};
use thymeleaf::inline::{IInliner, NoOpInliner};
use thymeleaf::messageresolver::MessageResolutionResult;
use thymeleaf::model::{ICDATASection, IComment, IModelFactory, IProcessableElementTag, IText};
use thymeleaf::util::{JavaCharSequence, JavaLocale, JavaString};
use thymeleaf::{
    IEngineConfiguration, TemplateMode, TemplateProcessingException, TemplateResolutionAttributes,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/inline_golden.txt");

#[test]
fn inline_contract_and_no_op_singleton_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "java_baseline", JAVA_BASELINE);
    emit(&mut output, "shape.class.final", true);
    emit(&mut output, "shape.constructor.count", 1);
    emit(&mut output, "shape.constructor.private", true);
    emit(&mut output, "shape.instance.public_static_final", true);
    emit(
        &mut output,
        "shape.interface.methods",
        "getName():String,inline(ITemplateContext+ICDATASection):CharSequence,inline(ITemplateContext+IComment):CharSequence,inline(ITemplateContext+IText):CharSequence",
    );

    let instance = NoOpInliner::instance();
    emit(
        &mut output,
        "noop.instance.same",
        std::ptr::eq(instance, NoOpInliner::instance()),
    );
    emit_java(&mut output, "noop.name", instance.get_name());
    emit_optional_sequence(
        &mut output,
        "noop.null.text",
        instance
            .inline_text_nullable(None, None)
            .expect("NoOp null text"),
    );
    emit_optional_sequence(
        &mut output,
        "noop.null.cdata",
        instance
            .inline_cdata_section_nullable(None, None)
            .expect("NoOp null CDATA"),
    );
    emit_optional_sequence(
        &mut output,
        "noop.null.comment",
        instance
            .inline_comment_nullable(None, None)
            .expect("NoOp null comment"),
    );

    let context = PanicTemplateContext;
    let text = Text::new(Some(java_sequence("text")));
    let cdata = CDATASection::new(Some(java_sequence("cdata")));
    let comment = Comment::new(Some(java_sequence("comment")));
    emit_optional_sequence(
        &mut output,
        "noop.non_null.text",
        instance
            .inline_text(&context, &text)
            .expect("NoOp non-null text"),
    );
    emit_optional_sequence(
        &mut output,
        "noop.non_null.cdata",
        instance
            .inline_cdata_section(&context, &cdata)
            .expect("NoOp non-null CDATA"),
    );
    emit_optional_sequence(
        &mut output,
        "noop.non_null.comment",
        instance
            .inline_comment(&context, &comment)
            .expect("NoOp non-null comment"),
    );

    let probe = ProbeInliner::default();
    let dynamic: &dyn IInliner = &probe;
    emit_java(&mut output, "probe.name", dynamic.get_name());
    emit_optional_sequence(
        &mut output,
        "probe.text",
        dynamic
            .inline_text(&context, &text)
            .expect("probe text dispatch"),
    );
    emit_optional_sequence(
        &mut output,
        "probe.cdata",
        dynamic
            .inline_cdata_section(&context, &cdata)
            .expect("probe CDATA dispatch"),
    );
    emit_optional_sequence(
        &mut output,
        "probe.comment",
        dynamic
            .inline_comment(&context, &comment)
            .expect("probe comment dispatch"),
    );
    emit(
        &mut output,
        "probe.calls",
        format!(
            "{},{},{}",
            probe.text_calls.load(Ordering::SeqCst),
            probe.cdata_calls.load(Ordering::SeqCst),
            probe.comment_calls.load(Ordering::SeqCst)
        ),
    );

    let (identity_count, name_count) = concurrent_singleton_observation();
    emit(&mut output, "concurrent.identity_count", identity_count);
    emit(&mut output, "concurrent.name_count", name_count);

    assert_eq!(output, JAVA_GOLDEN);
}

#[test]
fn no_op_shared_arc_preserves_the_java_singleton_identity() {
    let first = NoOpInliner::shared();
    let second = NoOpInliner::shared();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(
        Arc::as_ptr(&first) as *const (),
        NoOpInliner::instance() as *const NoOpInliner as *const ()
    );
}

fn concurrent_singleton_observation() -> (usize, usize) {
    let barrier = Arc::new(Barrier::new(8));
    let mut threads = Vec::new();
    for _ in 0..8 {
        let barrier = Arc::clone(&barrier);
        threads.push(std::thread::spawn(move || {
            barrier.wait();
            let instance = NoOpInliner::shared();
            (
                Arc::as_ptr(&instance) as *const () as usize,
                instance.get_name().to_string_lossy(),
            )
        }));
    }

    let mut identities = HashSet::new();
    let mut name_count = 0;
    for thread in threads {
        let (identity, name) = thread.join().expect("NoOp worker");
        identities.insert(identity);
        if name == "NOOP" {
            name_count += 1;
        }
    }
    (identities.len(), name_count)
}

#[derive(Default)]
struct ProbeInliner {
    text_calls: AtomicUsize,
    cdata_calls: AtomicUsize,
    comment_calls: AtomicUsize,
}

impl IInliner for ProbeInliner {
    fn get_name(&self) -> &JavaString {
        static NAME: OnceLock<JavaString> = OnceLock::new();
        NAME.get_or_init(|| java("PROBE"))
    }

    fn inline_text(
        &self,
        _context: &dyn ITemplateContext,
        _text: &dyn IText,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        self.text_calls.fetch_add(1, Ordering::SeqCst);
        Ok(Some(Box::new(java("TEXT"))))
    }

    fn inline_cdata_section(
        &self,
        _context: &dyn ITemplateContext,
        _cdata_section: &dyn ICDATASection,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        self.cdata_calls.fetch_add(1, Ordering::SeqCst);
        Ok(Some(Box::new(java("CDATA"))))
    }

    fn inline_comment(
        &self,
        _context: &dyn ITemplateContext,
        _comment: &dyn IComment,
    ) -> StandardExpressionResult<Option<Box<dyn JavaCharSequence>>> {
        self.comment_calls.fetch_add(1, Ordering::SeqCst);
        Ok(Some(Box::new(java("COMMENT"))))
    }
}

struct PanicTemplateContext;

impl IContext for PanicTemplateContext {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_locale(&self) -> JavaLocale {
        panic!("NoOp/Probe must not read the context locale")
    }

    fn contains_variable(&self, _name: Option<&JavaString>) -> bool {
        panic!("NoOp/Probe must not read context variables")
    }

    fn get_variable_names(&self) -> Arc<dyn IContextVariableNames + '_> {
        panic!("NoOp/Probe must not read context variable names")
    }

    fn get_variable(&self, _name: Option<&JavaString>) -> Option<Arc<TemplateValue>> {
        panic!("NoOp/Probe must not read context variables")
    }
}

impl IExpressionContext for PanicTemplateContext {
    fn get_configuration(&self) -> &dyn IEngineConfiguration {
        panic!("NoOp/Probe must not read the engine configuration")
    }

    fn get_configuration_arc(&self) -> Arc<dyn IEngineConfiguration> {
        panic!("NoOp/Probe must not clone the engine configuration")
    }

    fn get_expression_objects(&self) -> &dyn IExpressionObjects {
        panic!("NoOp/Probe must not read expression objects")
    }
}

impl ITemplateContext for PanicTemplateContext {
    fn get_template_data(&self) -> Arc<TemplateData> {
        panic!("NoOp/Probe must not read template data")
    }

    fn get_template_mode(&self) -> TemplateMode {
        panic!("NoOp/Probe must not read template mode")
    }

    fn get_template_stack(&self) -> Vec<Arc<TemplateData>> {
        panic!("NoOp/Probe must not read template stack")
    }

    fn get_element_stack(&self) -> Vec<Arc<dyn IProcessableElementTag>> {
        panic!("NoOp/Probe must not read element stack")
    }

    fn get_template_resolution_attributes(&self) -> Option<&TemplateResolutionAttributes> {
        panic!("NoOp/Probe must not read resolution attributes")
    }

    fn get_model_factory(&self) -> &dyn IModelFactory {
        panic!("NoOp/Probe must not read model factory")
    }

    fn has_selection_target(&self) -> bool {
        panic!("NoOp/Probe must not read selection target")
    }

    fn get_selection_target(&self) -> Option<Arc<TemplateValue>> {
        panic!("NoOp/Probe must not read selection target")
    }

    fn get_inliner(&self) -> Option<Arc<dyn IInliner>> {
        panic!("NoOp/Probe must not recursively read the current inliner")
    }

    fn get_message(
        &self,
        _origin: Option<TypeId>,
        _key: &JavaString,
        _message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
        _use_absent_message_representation: bool,
    ) -> MessageResolutionResult<Option<JavaString>> {
        panic!("NoOp/Probe must not resolve messages")
    }

    fn build_link(
        &self,
        _base: Option<&JavaString>,
        _parameters: Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
    ) -> Result<JavaString, TemplateProcessingException> {
        panic!("NoOp/Probe must not build links")
    }

    fn get_identifier_sequences(&self) -> &IdentifierSequences {
        panic!("NoOp/Probe must not read identifier sequences")
    }
}

fn java(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn java_sequence(value: &str) -> Arc<dyn JavaCharSequence> {
    Arc::new(java(value))
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    output.push_str(key);
    output.push('=');
    output.push_str(&value.to_string());
    output.push('\n');
}

fn emit_java(output: &mut String, key: &str, value: &JavaString) {
    emit(output, key, value.to_string_lossy());
}

fn emit_optional_sequence(
    output: &mut String,
    key: &str,
    value: Option<Box<dyn JavaCharSequence>>,
) {
    match value {
        Some(value) => emit(
            output,
            key,
            value
                .java_to_string()
                .expect("inliner result must stringify")
                .to_string_lossy(),
        ),
        None => emit(output, key, "null"),
    }
}
