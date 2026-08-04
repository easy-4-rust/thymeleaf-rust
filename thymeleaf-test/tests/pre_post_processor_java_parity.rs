//! PreProcessor/PostProcessor 配置对象的固定 Java Golden 差分与 Rust 运行时义务。

use std::cmp::Ordering;
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::{Arc, Barrier};

use thymeleaf::context::Context;
use thymeleaf::dialect::{AbstractDialect, IDialect, IPostProcessorDialect, IPreProcessorDialect};
use thymeleaf::engine::{
    AbstractTemplateHandler, ITemplateHandler, TemplateHandlerClass,
    TemplateHandlerConstructorError,
};
use thymeleaf::postprocessor::{IPostProcessor, PostProcessor};
use thymeleaf::preprocessor::{IPreProcessor, PreProcessor};
use thymeleaf::util::{ProcessorComparators, ProcessorConfigurationUtils, ValidateError};
use thymeleaf::{TemplateEngine, TemplateMode};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/pre_post_processor_golden.txt");
const PROBE_HANDLER_NAME: &str = "PrePostProcessorGolden$ProbeHandler";
const THROWING_HANDLER_NAME: &str = "PrePostProcessorGolden$ThrowingHandler";

static HANDLER_SEQUENCE: AtomicUsize = AtomicUsize::new(0);
static CONCURRENT_HANDLER_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

#[test]
fn pre_and_post_processor_contracts_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "java_baseline", JAVA_BASELINE);
    emit(&mut output, "shape.pre.class.final", true);
    emit(&mut output, "shape.post.class.final", true);
    emit(
        &mut output,
        "shape.pre.interface.methods",
        "getHandlerClass():Class,getPrecedence():int,getTemplateMode():TemplateMode",
    );
    emit(
        &mut output,
        "shape.post.interface.methods",
        "getHandlerClass():Class,getPrecedence():int,getTemplateMode():TemplateMode",
    );
    emit(
        &mut output,
        "shape.pre.constructor",
        "public(TemplateMode+Class+int)",
    );
    emit(
        &mut output,
        "shape.post.constructor",
        "public(TemplateMode+Class+int)",
    );

    emit_validate_error(
        &mut output,
        "pre.validation.both_null",
        require_error(
            PreProcessor::new(None, None, 7),
            "mode validation must run first",
        ),
    );
    emit_validate_error(
        &mut output,
        "pre.validation.handler_null",
        require_error(
            PreProcessor::new(Some(TemplateMode::HTML), None, 7),
            "null Handler class must fail",
        ),
    );
    emit_validate_error(
        &mut output,
        "post.validation.both_null",
        require_error(
            PostProcessor::new(None, None, 7),
            "mode validation must run first",
        ),
    );
    emit_validate_error(
        &mut output,
        "post.validation.handler_null",
        require_error(
            PostProcessor::new(Some(TemplateMode::HTML), None, 7),
            "null Handler class must fail",
        ),
    );

    let cases = [
        (TemplateMode::HTML, i32::MIN),
        (TemplateMode::XML, -1),
        (TemplateMode::TEXT, 0),
        (TemplateMode::JAVASCRIPT, 1),
        (TemplateMode::CSS, 1000),
        (TemplateMode::RAW, i32::MAX),
    ];
    for (mode, precedence) in cases {
        let pre = pre_processor(mode, precedence);
        let post = post_processor(mode, precedence);
        emit(
            &mut output,
            &format!("pre.state.{}", mode_name(mode)),
            state(
                pre.get_template_mode(),
                pre.get_precedence(),
                pre.get_handler_class(),
                std::ptr::eq(pre.get_handler_class(), pre.get_handler_class()),
            ),
        );
        emit(
            &mut output,
            &format!("post.state.{}", mode_name(mode)),
            state(
                post.get_template_mode(),
                post.get_precedence(),
                post.get_handler_class(),
                std::ptr::eq(post.get_handler_class(), post.get_handler_class()),
            ),
        );
    }

    let dynamic_pre = CustomPreProcessor::new(
        TemplateMode::CSS,
        probe_handler_class(),
        -17,
        "PrePostProcessorGolden$CustomPreProcessor",
    );
    let dynamic_post = CustomPostProcessor::new(
        TemplateMode::JAVASCRIPT,
        probe_handler_class(),
        23,
        "PrePostProcessorGolden$CustomPostProcessor",
    );
    let pre_contract: &dyn IPreProcessor = &dynamic_pre;
    let post_contract: &dyn IPostProcessor = &dynamic_post;
    emit(
        &mut output,
        "dynamic.pre",
        state(
            pre_contract.get_template_mode().expect("custom mode"),
            pre_contract.get_precedence(),
            pre_contract
                .get_handler_class()
                .expect("custom Handler class"),
            true,
        ),
    );
    emit(
        &mut output,
        "dynamic.post",
        state(
            post_contract.get_template_mode().expect("custom mode"),
            post_contract.get_precedence(),
            post_contract
                .get_handler_class()
                .expect("custom Handler class"),
            true,
        ),
    );

    let pre_low = pre_processor(TemplateMode::HTML, -1);
    let pre_high = pre_processor(TemplateMode::HTML, 1);
    let post_low = post_processor(TemplateMode::HTML, -1);
    let post_high = post_processor(TemplateMode::HTML, 1);
    emit(
        &mut output,
        "ordering.pre.self",
        ordering_sign(ProcessorComparators::compare_pre_processors(
            &pre_low, &pre_low,
        )),
    );
    emit(
        &mut output,
        "ordering.pre.precedence",
        ordering_sign(ProcessorComparators::compare_pre_processors(
            &pre_low, &pre_high,
        )),
    );
    emit(
        &mut output,
        "ordering.post.self",
        ordering_sign(ProcessorComparators::compare_post_processors(
            &post_low, &post_low,
        )),
    );
    emit(
        &mut output,
        "ordering.post.precedence",
        ordering_sign(ProcessorComparators::compare_post_processors(
            &post_low, &post_high,
        )),
    );

    let pre_a: Arc<dyn IPreProcessor> = Arc::new(CustomPreProcessor::new(
        TemplateMode::HTML,
        probe_handler_class(),
        0,
        "PrePostProcessorGolden$APreProcessor",
    ));
    let pre_b: Arc<dyn IPreProcessor> = Arc::new(CustomPreProcessor::new(
        TemplateMode::HTML,
        probe_handler_class(),
        0,
        "PrePostProcessorGolden$BPreProcessor",
    ));
    let post_a: Arc<dyn IPostProcessor> = Arc::new(CustomPostProcessor::new(
        TemplateMode::HTML,
        probe_handler_class(),
        0,
        "PrePostProcessorGolden$APostProcessor",
    ));
    let post_b: Arc<dyn IPostProcessor> = Arc::new(CustomPostProcessor::new(
        TemplateMode::HTML,
        probe_handler_class(),
        0,
        "PrePostProcessorGolden$BPostProcessor",
    ));
    emit(
        &mut output,
        "ordering.pre.implementation_class",
        ordering_sign(ProcessorComparators::compare_pre_processors(
            pre_a.as_ref(),
            pre_b.as_ref(),
        )),
    );
    emit(
        &mut output,
        "ordering.post.implementation_class",
        ordering_sign(ProcessorComparators::compare_post_processors(
            post_a.as_ref(),
            post_b.as_ref(),
        )),
    );

    let wrapped_pre_low = ProcessorConfigurationUtils::wrap_pre_processor(Arc::clone(&pre_a), -10);
    let wrapped_pre_high = ProcessorConfigurationUtils::wrap_pre_processor(Arc::clone(&pre_b), 10);
    let wrapped_post_low =
        ProcessorConfigurationUtils::wrap_post_processor(Arc::clone(&post_a), -10);
    let wrapped_post_high =
        ProcessorConfigurationUtils::wrap_post_processor(Arc::clone(&post_b), 10);
    emit(
        &mut output,
        "ordering.pre.wrapped_dialect",
        ordering_sign(ProcessorComparators::compare_pre_processors(
            wrapped_pre_low.as_ref(),
            wrapped_pre_high.as_ref(),
        )),
    );
    emit(
        &mut output,
        "ordering.post.wrapped_dialect",
        ordering_sign(ProcessorComparators::compare_post_processors(
            wrapped_post_low.as_ref(),
            wrapped_post_high.as_ref(),
        )),
    );
    emit(
        &mut output,
        "ordering.pre.unwrap.identity",
        std::ptr::eq(
            ProcessorConfigurationUtils::unwrap_pre_processor(wrapped_pre_low.as_ref()),
            pre_a.as_ref(),
        ),
    );
    emit(
        &mut output,
        "ordering.post.unwrap.identity",
        std::ptr::eq(
            ProcessorConfigurationUtils::unwrap_post_processor(wrapped_post_low.as_ref()),
            post_a.as_ref(),
        ),
    );

    HANDLER_SEQUENCE.store(0, AtomicOrdering::SeqCst);
    let pre = pre_processor(TemplateMode::HTML, 0);
    let first = pre
        .get_handler_class()
        .new_instance()
        .expect("first Handler instance");
    let second = pre
        .get_handler_class()
        .new_instance()
        .expect("second Handler instance");
    emit(
        &mut output,
        "handler.class.name",
        pre.get_handler_class().get_name(),
    );
    emit(
        &mut output,
        "handler.instances.distinct",
        handler_address(first.as_ref()) != handler_address(second.as_ref()),
    );
    emit(
        &mut output,
        "handler.instances.sequence",
        format!("1,{}", HANDLER_SEQUENCE.load(AtomicOrdering::SeqCst)),
    );

    emit(&mut output, "handler.failure.pre", engine_failure(true));
    emit(&mut output, "handler.failure.post", engine_failure(false));

    assert_eq!(output, JAVA_GOLDEN);
}

#[test]
fn handler_type_token_and_processor_configs_are_send_sync_and_thread_shareable() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<TemplateHandlerClass>();
    assert_send_sync::<PreProcessor>();
    assert_send_sync::<PostProcessor>();

    CONCURRENT_HANDLER_SEQUENCE.store(0, AtomicOrdering::SeqCst);
    let class = concurrent_probe_handler_class();
    let barrier = Arc::new(Barrier::new(8));
    let mut workers = Vec::new();
    for _ in 0..8 {
        let barrier = Arc::clone(&barrier);
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            let handler = class
                .new_instance()
                .expect("shared class token constructs on every thread");
            assert_ne!(handler_address(handler.as_ref()), 0);
            class.get_name()
        }));
    }
    for worker in workers {
        assert_eq!(
            worker.join().expect("constructor worker"),
            PROBE_HANDLER_NAME
        );
    }
    assert_eq!(CONCURRENT_HANDLER_SEQUENCE.load(AtomicOrdering::SeqCst), 8);
}

fn pre_processor(mode: TemplateMode, precedence: i32) -> PreProcessor {
    PreProcessor::new(Some(mode), Some(probe_handler_class()), precedence)
        .expect("valid PreProcessor")
}

fn post_processor(mode: TemplateMode, precedence: i32) -> PostProcessor {
    PostProcessor::new(Some(mode), Some(probe_handler_class()), precedence)
        .expect("valid PostProcessor")
}

fn probe_handler_class() -> TemplateHandlerClass {
    TemplateHandlerClass::new(PROBE_HANDLER_NAME, new_probe_handler)
}

fn throwing_handler_class() -> TemplateHandlerClass {
    TemplateHandlerClass::new(THROWING_HANDLER_NAME, new_throwing_handler)
}

fn concurrent_probe_handler_class() -> TemplateHandlerClass {
    TemplateHandlerClass::new(PROBE_HANDLER_NAME, new_concurrent_probe_handler)
}

fn new_probe_handler() -> Result<Box<dyn ITemplateHandler>, TemplateHandlerConstructorError> {
    HANDLER_SEQUENCE.fetch_add(1, AtomicOrdering::SeqCst);
    Ok(Box::new(AbstractTemplateHandler::new()))
}

fn new_throwing_handler() -> Result<Box<dyn ITemplateHandler>, TemplateHandlerConstructorError> {
    Err(Box::new(HandlerBoom))
}

fn new_concurrent_probe_handler()
-> Result<Box<dyn ITemplateHandler>, TemplateHandlerConstructorError> {
    CONCURRENT_HANDLER_SEQUENCE.fetch_add(1, AtomicOrdering::SeqCst);
    Ok(Box::new(AbstractTemplateHandler::new()))
}

#[derive(Debug)]
struct HandlerBoom;

impl std::fmt::Display for HandlerBoom {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("handler boom")
    }
}

impl Error for HandlerBoom {}

struct CustomPreProcessor {
    mode: TemplateMode,
    handler_class: TemplateHandlerClass,
    precedence: i32,
    class_name: &'static str,
}

impl CustomPreProcessor {
    const fn new(
        mode: TemplateMode,
        handler_class: TemplateHandlerClass,
        precedence: i32,
        class_name: &'static str,
    ) -> Self {
        Self {
            mode,
            handler_class,
            precedence,
            class_name,
        }
    }
}

impl IPreProcessor for CustomPreProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(self.mode)
    }

    fn get_precedence(&self) -> i32 {
        self.precedence
    }

    fn get_handler_class(&self) -> Option<&TemplateHandlerClass> {
        Some(&self.handler_class)
    }

    fn class_name(&self) -> &'static str {
        self.class_name
    }
}

struct CustomPostProcessor {
    mode: TemplateMode,
    handler_class: TemplateHandlerClass,
    precedence: i32,
    class_name: &'static str,
}

impl CustomPostProcessor {
    const fn new(
        mode: TemplateMode,
        handler_class: TemplateHandlerClass,
        precedence: i32,
        class_name: &'static str,
    ) -> Self {
        Self {
            mode,
            handler_class,
            precedence,
            class_name,
        }
    }
}

impl IPostProcessor for CustomPostProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(self.mode)
    }

    fn get_precedence(&self) -> i32 {
        self.precedence
    }

    fn get_handler_class(&self) -> Option<&TemplateHandlerClass> {
        Some(&self.handler_class)
    }

    fn class_name(&self) -> &'static str {
        self.class_name
    }
}

struct FailingPreDialect {
    dialect: AbstractDialect,
}

impl FailingPreDialect {
    fn new() -> Self {
        Self {
            dialect: AbstractDialect::new(Some("ThrowingPre"))
                .expect("fixed dialect name is valid"),
        }
    }
}

impl IDialect for FailingPreDialect {
    fn as_pre_processor_dialect(&self) -> Option<&dyn IPreProcessorDialect> {
        Some(self)
    }

    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IPreProcessorDialect for FailingPreDialect {
    fn get_dialect_pre_processor_precedence(&self) -> i32 {
        0
    }

    fn get_pre_processors(&self) -> Option<Vec<Option<Arc<dyn IPreProcessor>>>> {
        Some(vec![Some(Arc::new(
            PreProcessor::new(Some(TemplateMode::HTML), Some(throwing_handler_class()), 0)
                .expect("fixed failing PreProcessor configuration is valid"),
        ))])
    }
}

struct FailingPostDialect {
    dialect: AbstractDialect,
}

impl FailingPostDialect {
    fn new() -> Self {
        Self {
            dialect: AbstractDialect::new(Some("ThrowingPost"))
                .expect("fixed dialect name is valid"),
        }
    }
}

impl IDialect for FailingPostDialect {
    fn as_post_processor_dialect(&self) -> Option<&dyn IPostProcessorDialect> {
        Some(self)
    }

    fn get_name(&self) -> Option<&str> {
        Some(self.dialect.get_name())
    }
}

impl IPostProcessorDialect for FailingPostDialect {
    fn get_dialect_post_processor_precedence(&self) -> i32 {
        0
    }

    fn get_post_processors(&self) -> Option<Vec<Option<Arc<dyn IPostProcessor>>>> {
        Some(vec![Some(Arc::new(
            PostProcessor::new(Some(TemplateMode::HTML), Some(throwing_handler_class()), 0)
                .expect("fixed failing PostProcessor configuration is valid"),
        ))])
    }
}

fn engine_failure(pre: bool) -> String {
    let engine = TemplateEngine::new();
    if pre {
        engine
            .add_dialect(Arc::new(FailingPreDialect::new()) as Arc<dyn IDialect>)
            .expect("add failing pre dialect");
    } else {
        engine
            .add_dialect(Arc::new(FailingPostDialect::new()) as Arc<dyn IDialect>)
            .expect("add failing post dialect");
    }
    let error = engine
        .process_template(
            if pre { "<p>pre</p>" } else { "<p>post</p>" },
            &Context::new(),
        )
        .expect_err("Handler construction must fail");
    let expected_message = if pre {
        "An exception happened during the creation of a new instance of pre-processor java.lang.Class"
    } else {
        "An exception happened during the creation of a new instance of post-processor java.lang.Class"
    };
    assert_eq!(error.to_string(), expected_message);

    let constructor_cause = error.source().expect("outer error keeps constructor cause");
    assert_eq!(constructor_cause.to_string(), "handler boom");
    let root = constructor_cause
        .source()
        .expect("Rust adapter keeps original Handler error");
    assert!(root.downcast_ref::<HandlerBoom>().is_some());
    assert_eq!(root.to_string(), "handler boom");

    format!(
        "org.thymeleaf.exceptions.TemplateProcessingException:{expected_message}|cause=java.lang.IllegalStateException:{}",
        root
    )
}

fn state(
    mode: TemplateMode,
    precedence: i32,
    handler_class: &TemplateHandlerClass,
    stable_identity: bool,
) -> String {
    format!(
        "{}|{precedence}|{}|{stable_identity}",
        mode_name(mode),
        handler_class.get_name()
    )
}

const fn mode_name(mode: TemplateMode) -> &'static str {
    match mode {
        TemplateMode::HTML => "HTML",
        TemplateMode::XML => "XML",
        TemplateMode::TEXT => "TEXT",
        TemplateMode::JAVASCRIPT => "JAVASCRIPT",
        TemplateMode::CSS => "CSS",
        TemplateMode::RAW => "RAW",
    }
}

const fn ordering_sign(ordering: Ordering) -> i32 {
    match ordering {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

fn handler_address(handler: &dyn ITemplateHandler) -> usize {
    std::ptr::from_ref(handler).cast::<()>() as usize
}

fn emit_validate_error(output: &mut String, key: &str, error: ValidateError) {
    emit(
        output,
        key,
        format!(
            "{}:{}",
            error.class_name(),
            error.get_message().unwrap_or("null")
        ),
    );
}

fn require_error<T>(result: Result<T, ValidateError>, message: &str) -> ValidateError {
    match result {
        Ok(_) => panic!("{message}"),
        Err(error) => error,
    }
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    output.push_str(key);
    output.push('=');
    output.push_str(&value.to_string());
    output.push('\n');
}
