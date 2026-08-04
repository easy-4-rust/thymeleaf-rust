//! DialectSetConfiguration 与四个 Dialect 贡献 SPI 的固定 Java Golden 差分测试。

#[allow(dead_code, unused_imports)]
mod support;

use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use thymeleaf::context::{ExpressionContext, IExpressionContext};
use thymeleaf::dialect::{
    ExecutionAttributeMap, IDialect, IExecutionAttributeDialect, IExpressionObjectDialect,
    IPostProcessorDialect, IPreProcessorDialect, PostProcessorSet, PreProcessorSet,
};
use thymeleaf::engine::{
    AbstractTemplateHandler, AttributeDefinitions, ElementDefinitions, ITemplateHandler,
    TemplateHandlerClass, TemplateHandlerConstructorError,
};
use thymeleaf::expression::{
    ExpressionObjectNames, IExpressionObjectFactory, StandardExpressionResult, TemplateValue,
};
use thymeleaf::postprocessor::IPostProcessor;
use thymeleaf::preprocessor::IPreProcessor;
use thymeleaf::util::Utf16String;
use thymeleaf::{
    DialectConfiguration, DialectSetConfiguration, DialectSetConfigurationError,
    ExecutionAttributeValue, ITemplateEngine, StandardDialect, TemplateEngine, TemplateMode,
};

use support::{InteractionDialect01, ReplaceWithProcessableDialect};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/dialect_set_configuration_golden.txt");
const DIALECT_CLASS: &str = "org.thymeleaf.DialectSetConfigurationGolden$CapabilityDialect";
const A_PRE_CLASS: &str = "org.thymeleaf.DialectSetConfigurationGolden$APreProcessor";
const B_PRE_CLASS: &str = "org.thymeleaf.DialectSetConfigurationGolden$BPreProcessor";
const A_POST_CLASS: &str = "org.thymeleaf.DialectSetConfigurationGolden$APostProcessor";
const B_POST_CLASS: &str = "org.thymeleaf.DialectSetConfigurationGolden$BPostProcessor";
const PROBE_HANDLER_CLASS: &str = "org.thymeleaf.DialectSetConfigurationGolden$ProbeHandler";
const NO_CONSTRUCTOR_HANDLER_CLASS: &str =
    "org.thymeleaf.DialectSetConfigurationGolden$NoPublicConstructorHandler";

#[test]
fn dialect_set_configuration_and_contribution_spis_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    export_shape(&mut output);
    export_empty(&mut output);
    export_execution_attributes(&mut output);
    export_expression_factories(&mut output);
    export_pre_post_processors(&mut output);
    export_validation(&mut output);
    export_getter_validation(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

#[test]
fn dialect_aggregation_runtime_obligations_are_thread_safe_and_inject_definitions_once() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DialectSetConfiguration>();

    let pre = Arc::new(AwarePreProcessor::new());
    let post = Arc::new(AwarePostProcessor::new());
    let dialect: Arc<dyn IDialect> = Arc::new(CapabilityDialect::new(
        None,
        None,
        Some((
            0,
            Arc::new(AtomicUsize::new(0)),
            Some(vec![Some(pre.clone())]),
        )),
        Some((
            0,
            Arc::new(AtomicUsize::new(0)),
            Some(vec![Some(post.clone())]),
        )),
    ));
    let configuration = build(vec![dialect]).expect("valid aware dialect");

    assert_eq!(pre.element_calls.load(Ordering::SeqCst), 1);
    assert_eq!(pre.attribute_calls.load(Ordering::SeqCst), 1);
    assert_eq!(post.element_calls.load(Ordering::SeqCst), 1);
    assert_eq!(post.attribute_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        configuration
            .get_pre_processors(Some(TemplateMode::HTML))
            .expect("non-null mode")
            .len(),
        1
    );
    assert_eq!(
        configuration
            .get_post_processors(Some(TemplateMode::HTML))
            .expect("non-null mode")
            .len(),
        1
    );

    let shared = Arc::new(configuration);
    let mut workers = Vec::new();
    for _ in 0..8 {
        let shared = Arc::clone(&shared);
        workers.push(std::thread::spawn(move || {
            (
                shared.get_dialects().len(),
                shared.get_execution_attributes().len(),
                shared
                    .get_pre_processors(Some(TemplateMode::HTML))
                    .expect("non-null mode")
                    .len(),
            )
        }));
    }
    for worker in workers {
        assert_eq!(worker.join().expect("reader thread"), (1, 0, 1));
    }
}

#[test]
fn upstream_processor_computations_01_through_08_cover_all_processor_buckets_and_ordering() {
    let configuration = build(vec![
        Arc::new(StandardDialect::new()) as Arc<dyn IDialect>,
        Arc::new(InteractionDialect01::new()) as Arc<dyn IDialect>,
        Arc::new(ReplaceWithProcessableDialect::new()) as Arc<dyn IDialect>,
    ])
    .expect("all processor contributions are valid");

    let html = Some(TemplateMode::HTML);
    assert!(
        !configuration
            .get_template_boundaries_processors(html)
            .expect("HTML mode")
            .is_empty()
    );
    assert!(
        !configuration
            .get_cdata_section_processors(html)
            .expect("HTML mode")
            .is_empty()
    );
    assert!(
        !configuration
            .get_comment_processors(html)
            .expect("HTML mode")
            .is_empty()
    );
    assert!(
        !configuration
            .get_doc_type_processors(html)
            .expect("HTML mode")
            .is_empty()
    );
    assert!(
        !configuration
            .get_element_processors(html)
            .expect("HTML mode")
            .is_empty()
    );
    assert!(
        !configuration
            .get_processing_instruction_processors(html)
            .expect("HTML mode")
            .is_empty()
    );
    assert!(
        !configuration
            .get_text_processors(html)
            .expect("HTML mode")
            .is_empty()
    );
    assert!(
        !configuration
            .get_xml_declaration_processors(html)
            .expect("HTML mode")
            .is_empty()
    );

    for processors in [
        configuration
            .get_cdata_section_processors(html)
            .expect("HTML mode")
            .into_iter()
            .map(|processor| processor.get_precedence())
            .collect::<Vec<_>>(),
        configuration
            .get_comment_processors(html)
            .expect("HTML mode")
            .into_iter()
            .map(|processor| processor.get_precedence())
            .collect(),
        configuration
            .get_text_processors(html)
            .expect("HTML mode")
            .into_iter()
            .map(|processor| processor.get_precedence())
            .collect(),
    ] {
        assert!(
            processors.windows(2).all(|window| window[0] <= window[1]),
            "processors must retain Java precedence order: {processors:?}"
        );
    }

    assert!(
        !configuration
            .get_element_processors(Some(TemplateMode::XML))
            .expect("XML mode")
            .is_empty(),
        "StandardDialect must contribute XML element processors"
    );
    assert!(
        configuration
            .get_cdata_section_processors(Some(TemplateMode::RAW))
            .expect("RAW mode")
            .is_empty(),
        "an absent processor bucket must expose the shared empty-set semantics"
    );
}

fn export_shape(output: &mut String) {
    emit(output, "shape.execution", "getExecutionAttributes():Map");
    emit(
        output,
        "shape.expression",
        "getExpressionObjectFactory():IExpressionObjectFactory",
    );
    emit(
        output,
        "shape.pre",
        "getDialectPreProcessorPrecedence():int,getPreProcessors():Set",
    );
    emit(
        output,
        "shape.post",
        "getDialectPostProcessorPrecedence():int,getPostProcessors():Set",
    );
    emit(
        output,
        "shape.configuration.public",
        "build(Set):DialectSetConfiguration,getAttributeDefinitions():AttributeDefinitions,getCDATASectionProcessors(TemplateMode):Set,getCommentProcessors(TemplateMode):Set,getDialectConfigurations():Set,getDialects():Set,getDocTypeProcessors(TemplateMode):Set,getElementDefinitions():ElementDefinitions,getElementProcessors(TemplateMode):Set,getExecutionAttribute(String):Object,getExecutionAttributes():Map,getExpressionObjectFactory():IExpressionObjectFactory,getPostProcessors(TemplateMode):Set,getPreProcessors(TemplateMode):Set,getProcessingInstructionProcessors(TemplateMode):Set,getStandardDialectPrefix():String,getTemplateBoundariesProcessors(TemplateMode):Set,getTextProcessors(TemplateMode):Set,getXMLDeclarationProcessors(TemplateMode):Set,hasExecutionAttribute(String):boolean,isStandardDialectPresent():boolean",
    );
}

fn export_empty(output: &mut String) {
    let configuration =
        DialectSetConfiguration::build(Some(Vec::new())).expect("empty set is legal");
    emit(
        output,
        "empty.configurations",
        configuration.get_dialect_configurations().len(),
    );
    emit(output, "empty.dialects", configuration.get_dialects().len());
    emit(
        output,
        "empty.standard",
        configuration.is_standard_dialect_present(),
    );
    emit(
        output,
        "empty.prefix",
        configuration
            .get_standard_dialect_prefix()
            .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy),
    );
    emit(output, "empty.attributes", "{}");
    emit_names(
        output,
        "empty.expression.names",
        configuration
            .get_expression_object_factory()
            .get_all_expression_object_names(),
    );
    emit(
        output,
        "empty.pre.html",
        format_pre(
            configuration
                .get_pre_processors(Some(TemplateMode::HTML))
                .expect("non-null mode"),
        ),
    );
    emit(
        output,
        "empty.post.raw",
        format_post(
            configuration
                .get_post_processors(Some(TemplateMode::RAW))
                .expect("non-null mode"),
        ),
    );
    emit(
        output,
        "empty.configurations.mutable",
        "java.lang.UnsupportedOperationException:null|cause=null",
    );
    emit(
        output,
        "empty.attributes.mutable",
        "java.lang.UnsupportedOperationException:null|cause=null",
    );
}

fn export_execution_attributes(output: &mut String) {
    let first = Some(vec![
        (
            None,
            Some(Arc::new(ExecutionAttributeValue::new(
                "null-key".to_owned(),
            ))),
        ),
        (Some("null-value".to_owned()), None),
        (
            Some("alpha".to_owned()),
            Some(Arc::new(ExecutionAttributeValue::new(1_i32))),
        ),
    ]);
    let second = Some(vec![(
        Some("beta".to_owned()),
        Some(Arc::new(ExecutionAttributeValue::new(2_i32))),
    )]);
    let configuration = build(vec![
        dialect(first.clone(), None, None, None),
        dialect(None, None, None, None),
        dialect(second, None, None, None),
    ])
    .expect("valid attributes");
    emit(
        output,
        "attributes.entries",
        format_execution_attributes(configuration.get_execution_attributes()),
    );
    emit(
        output,
        "attributes.null.present",
        configuration.has_execution_attribute(None),
    );
    emit(
        output,
        "attributes.null.value",
        configuration
            .get_execution_attribute(Some(&Utf16String::from_rust_str("null-value")))
            .map_or_else(|| "null".to_owned(), |_| "<value>".to_owned()),
    );
    emit(
        output,
        "attributes.missing.present",
        configuration.has_execution_attribute(Some(&Utf16String::from_rust_str("missing"))),
    );

    let duplicate = Some(vec![(
        Some("alpha".to_owned()),
        Some(Arc::new(ExecutionAttributeValue::new(9_i32))),
    )]);
    emit_build_error(
        output,
        "attributes.conflict",
        build(vec![
            dialect(first.clone(), None, None, None),
            dialect(duplicate, None, None, None),
        ]),
    );
    let duplicate_null = Some(vec![(
        None,
        Some(Arc::new(ExecutionAttributeValue::new("again".to_owned()))),
    )]);
    emit_build_error(
        output,
        "attributes.conflict.null",
        build(vec![
            dialect(first, None, None, None),
            dialect(duplicate_null, None, None, None),
        ]),
    );
}

fn export_expression_factories(output: &mut String) {
    let first = Arc::new(ProbeFactory::new("A", Some(names(&["a", "shared"])), true));
    let second = Arc::new(ProbeFactory::new("B", Some(names(&["b", "shared"])), false));
    let aggregate = build(vec![
        dialect(None, Some(first.clone()), None, None),
        dialect(None, None, None, None),
        dialect(None, Some(second.clone()), None, None),
    ])
    .expect("valid factories");
    let factory = aggregate.get_expression_object_factory();
    emit_names(
        output,
        "expression.multi.names",
        factory.get_all_expression_object_names(),
    );
    let context = expression_context();
    emit_object(
        output,
        "expression.multi.shared",
        factory.build_object(
            Arc::clone(&context),
            Some(&Utf16String::from_rust_str("shared")),
        ),
    );
    emit_object(
        output,
        "expression.multi.a",
        factory.build_object(Arc::clone(&context), Some(&Utf16String::from_rust_str("a"))),
    );
    emit_object(
        output,
        "expression.multi.unknown",
        factory.build_object(
            Arc::clone(&context),
            Some(&Utf16String::from_rust_str("unknown")),
        ),
    );
    emit(
        output,
        "expression.multi.cache.shared",
        factory.is_cacheable(Some(&Utf16String::from_rust_str("shared"))),
    );
    emit(
        output,
        "expression.multi.calls",
        format!(
            "{}|{}",
            first.calls.load(Ordering::SeqCst),
            second.calls.load(Ordering::SeqCst)
        ),
    );

    let single = Arc::new(ProbeFactory::new("ONLY", Some(names(&["known"])), true));
    let single_aggregate = build(vec![dialect(None, Some(single.clone()), None, None)])
        .expect("single factory")
        .get_expression_object_factory();
    emit_names(
        output,
        "expression.single.names",
        single_aggregate.get_all_expression_object_names(),
    );
    emit_object(
        output,
        "expression.single.unknown",
        single_aggregate.build_object(
            Arc::clone(&context),
            Some(&Utf16String::from_rust_str("unknown")),
        ),
    );
    emit(
        output,
        "expression.single.cache.unknown",
        single_aggregate.is_cacheable(Some(&Utf16String::from_rust_str("unknown"))),
    );
    emit(
        output,
        "expression.single.calls",
        single.calls.load(Ordering::SeqCst),
    );
}

fn export_pre_post_processors(output: &mut String) {
    let pre_calls = Arc::new(AtomicUsize::new(0));
    let post_calls = Arc::new(AtomicUsize::new(0));
    let first_pre = Some(vec![
        Some(pre(
            B_PRE_CLASS,
            Some(TemplateMode::HTML),
            valid_handler(),
            20,
        )),
        Some(pre(
            A_PRE_CLASS,
            Some(TemplateMode::HTML),
            valid_handler(),
            20,
        )),
    ]);
    let second_pre = Some(vec![Some(pre(
        A_PRE_CLASS,
        Some(TemplateMode::HTML),
        valid_handler(),
        5,
    ))]);
    let first_post = Some(vec![
        Some(post(
            B_POST_CLASS,
            Some(TemplateMode::HTML),
            valid_handler(),
            20,
        )),
        Some(post(
            A_POST_CLASS,
            Some(TemplateMode::HTML),
            valid_handler(),
            20,
        )),
    ]);
    let second_post = Some(vec![Some(post(
        A_POST_CLASS,
        Some(TemplateMode::HTML),
        valid_handler(),
        5,
    ))]);
    let configuration = build(vec![
        Arc::new(CapabilityDialect::new(
            None,
            None,
            Some((-100, Arc::clone(&pre_calls), first_pre)),
            Some((-100, Arc::clone(&post_calls), first_post)),
        )),
        Arc::new(CapabilityDialect::new(
            None,
            None,
            Some((0, Arc::clone(&pre_calls), None)),
            Some((0, Arc::clone(&post_calls), None)),
        )),
        Arc::new(CapabilityDialect::new(
            None,
            None,
            Some((100, Arc::clone(&pre_calls), second_pre)),
            Some((100, Arc::clone(&post_calls), second_post)),
        )),
    ])
    .expect("valid pre/post contributions");
    emit(
        output,
        "pre.order",
        format_pre(
            configuration
                .get_pre_processors(Some(TemplateMode::HTML))
                .expect("non-null mode"),
        ),
    );
    emit(
        output,
        "post.order",
        format_post(
            configuration
                .get_post_processors(Some(TemplateMode::HTML))
                .expect("non-null mode"),
        ),
    );
    emit(
        output,
        "pre.dialect_precedence.calls",
        pre_calls.load(Ordering::SeqCst),
    );
    emit(
        output,
        "post.dialect_precedence.calls",
        post_calls.load(Ordering::SeqCst),
    );
    emit(
        output,
        "pre.empty.xml",
        format_pre(
            configuration
                .get_pre_processors(Some(TemplateMode::XML))
                .expect("non-null mode"),
        ),
    );
    emit(
        output,
        "post.empty.xml",
        format_post(
            configuration
                .get_post_processors(Some(TemplateMode::XML))
                .expect("non-null mode"),
        ),
    );
}

fn export_validation(output: &mut String) {
    emit_build_error(output, "build.null", DialectSetConfiguration::build(None));
    emit_build_error(
        output,
        "pre.null.entry",
        build(vec![Arc::new(CapabilityDialect::new(
            None,
            None,
            Some((0, counter(), Some(vec![None]))),
            None,
        ))]),
    );
    emit_build_error(
        output,
        "pre.null.mode",
        build(vec![pre_dialect(Some(vec![Some(pre(
            A_PRE_CLASS,
            None,
            valid_handler(),
            0,
        ))]))]),
    );
    emit_build_error(
        output,
        "pre.null.handler",
        build(vec![pre_dialect(Some(vec![Some(pre(
            A_PRE_CLASS,
            Some(TemplateMode::HTML),
            None,
            0,
        ))]))]),
    );
    emit_build_error(
        output,
        "pre.wrong.handler",
        build(vec![pre_dialect(Some(vec![Some(pre(
            A_PRE_CLASS,
            Some(TemplateMode::HTML),
            Some(TemplateHandlerClass::from_java_class_metadata(
                "java.lang.String",
                false,
                Some(new_probe_handler),
            )),
            0,
        ))]))]),
    );
    emit_build_error(
        output,
        "pre.no_zero_arg",
        build(vec![pre_dialect(Some(vec![Some(pre(
            A_PRE_CLASS,
            Some(TemplateMode::HTML),
            Some(TemplateHandlerClass::from_java_class_metadata(
                NO_CONSTRUCTOR_HANDLER_CLASS,
                true,
                None,
            )),
            0,
        ))]))]),
    );

    emit_build_error(
        output,
        "post.null.entry",
        build(vec![Arc::new(CapabilityDialect::new(
            None,
            None,
            None,
            Some((0, counter(), Some(vec![None]))),
        ))]),
    );
    emit_build_error(
        output,
        "post.null.mode",
        build(vec![post_dialect(Some(vec![Some(post(
            A_POST_CLASS,
            None,
            valid_handler(),
            0,
        ))]))]),
    );
    emit_build_error(
        output,
        "post.null.handler",
        build(vec![post_dialect(Some(vec![Some(post(
            A_POST_CLASS,
            Some(TemplateMode::HTML),
            None,
            0,
        ))]))]),
    );
    emit_build_error(
        output,
        "post.wrong.handler",
        build(vec![post_dialect(Some(vec![Some(post(
            A_POST_CLASS,
            Some(TemplateMode::HTML),
            Some(TemplateHandlerClass::from_java_class_metadata(
                "java.lang.String",
                false,
                Some(new_probe_handler),
            )),
            0,
        ))]))]),
    );
    emit_build_error(
        output,
        "post.no_zero_arg",
        build(vec![post_dialect(Some(vec![Some(post(
            A_POST_CLASS,
            Some(TemplateMode::HTML),
            Some(TemplateHandlerClass::from_java_class_metadata(
                NO_CONSTRUCTOR_HANDLER_CLASS,
                true,
                None,
            )),
            0,
        ))]))]),
    );
}

fn export_getter_validation(output: &mut String) {
    let configuration =
        DialectSetConfiguration::build(Some(Vec::new())).expect("empty set is legal");
    emit_validate_error(
        output,
        "getter.boundaries.null",
        configuration.get_template_boundaries_processors(None),
    );
    emit_validate_error(
        output,
        "getter.cdata.null",
        configuration.get_cdata_section_processors(None),
    );
    emit_validate_error(
        output,
        "getter.comment.null",
        configuration.get_comment_processors(None),
    );
    emit_validate_error(
        output,
        "getter.doctype.null",
        configuration.get_doc_type_processors(None),
    );
    emit_validate_error(
        output,
        "getter.element.null",
        configuration.get_element_processors(None),
    );
    emit_validate_error(
        output,
        "getter.instruction.null",
        configuration.get_processing_instruction_processors(None),
    );
    emit_validate_error(
        output,
        "getter.text.null",
        configuration.get_text_processors(None),
    );
    emit_validate_error(
        output,
        "getter.declaration.null",
        configuration.get_xml_declaration_processors(None),
    );
    emit_validate_error(
        output,
        "getter.pre.null",
        configuration.get_pre_processors(None),
    );
    emit_validate_error(
        output,
        "getter.post.null",
        configuration.get_post_processors(None),
    );
}

fn dialect(
    attributes: Option<ExecutionAttributeMap>,
    factory: Option<Arc<ProbeFactory>>,
    pre: Option<(i32, Arc<AtomicUsize>, Option<PreProcessorSet>)>,
    post: Option<(i32, Arc<AtomicUsize>, Option<PostProcessorSet>)>,
) -> Arc<dyn IDialect> {
    Arc::new(CapabilityDialect::new(
        attributes,
        factory.map(|value| value as Arc<dyn IExpressionObjectFactory>),
        pre,
        post,
    ))
}

fn pre_dialect(processors: Option<PreProcessorSet>) -> Arc<dyn IDialect> {
    Arc::new(CapabilityDialect::new(
        None,
        None,
        Some((0, counter(), processors)),
        None,
    ))
}

fn post_dialect(processors: Option<PostProcessorSet>) -> Arc<dyn IDialect> {
    Arc::new(CapabilityDialect::new(
        None,
        None,
        None,
        Some((0, counter(), processors)),
    ))
}

fn build(
    dialects: Vec<Arc<dyn IDialect>>,
) -> Result<DialectSetConfiguration, DialectSetConfigurationError> {
    let configurations = dialects
        .into_iter()
        .map(|dialect| DialectConfiguration::new(Some(dialect)).expect("non-null dialect"))
        .collect();
    DialectSetConfiguration::build(Some(configurations))
}

fn expression_context() -> Arc<dyn IExpressionContext> {
    let engine = TemplateEngine::new();
    let configuration = engine
        .get_configuration()
        .expect("default engine configuration");
    ExpressionContext::new(Some(configuration)).expect("non-null expression-context configuration")
}

fn names(values: &[&str]) -> Vec<Option<Utf16String>> {
    values
        .iter()
        .map(|value| Some(Utf16String::from_rust_str(value)))
        .collect()
}

fn counter() -> Arc<AtomicUsize> {
    Arc::new(AtomicUsize::new(0))
}

fn pre(
    class_name: &'static str,
    mode: Option<TemplateMode>,
    handler_class: Option<TemplateHandlerClass>,
    precedence: i32,
) -> Arc<dyn IPreProcessor> {
    Arc::new(ProbePreProcessor {
        class_name,
        mode,
        handler_class,
        precedence,
    })
}

fn post(
    class_name: &'static str,
    mode: Option<TemplateMode>,
    handler_class: Option<TemplateHandlerClass>,
    precedence: i32,
) -> Arc<dyn IPostProcessor> {
    Arc::new(ProbePostProcessor {
        class_name,
        mode,
        handler_class,
        precedence,
    })
}

fn valid_handler() -> Option<TemplateHandlerClass> {
    Some(TemplateHandlerClass::new(
        PROBE_HANDLER_CLASS,
        new_probe_handler,
    ))
}

fn new_probe_handler() -> Result<Box<dyn ITemplateHandler>, TemplateHandlerConstructorError> {
    Ok(Box::new(AbstractTemplateHandler::new()))
}

fn format_execution_attributes(
    attributes: &indexmap::IndexMap<Option<Utf16String>, Option<Arc<ExecutionAttributeValue>>>,
) -> String {
    let entries = attributes
        .iter()
        .map(|(key, value)| {
            let key = key
                .as_ref()
                .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy);
            let value = value.as_ref().map_or_else(
                || "null".to_owned(),
                |value| {
                    value
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| {
                            value
                                .downcast_ref::<i32>()
                                .map(std::string::ToString::to_string)
                        })
                        .unwrap_or_else(|| "<value>".to_owned())
                },
            );
            format!("{key}={value}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{entries}}}")
}

fn format_pre(processors: Vec<&dyn IPreProcessor>) -> String {
    processors
        .iter()
        .map(|processor| {
            let simple = processor
                .class_name()
                .rsplit('$')
                .next()
                .unwrap_or(processor.class_name());
            format!("{simple}:{}", processor.get_precedence())
        })
        .collect::<Vec<_>>()
        .join(",")
        .pipe(|entries| format!("[{entries}]"))
}

fn format_post(processors: Vec<&dyn IPostProcessor>) -> String {
    processors
        .iter()
        .map(|processor| {
            let simple = processor
                .class_name()
                .rsplit('$')
                .next()
                .unwrap_or(processor.class_name());
            format!("{simple}:{}", processor.get_precedence())
        })
        .collect::<Vec<_>>()
        .join(",")
        .pipe(|entries| format!("[{entries}]"))
}

fn emit_names(output: &mut String, key: &str, names: Option<ExpressionObjectNames>) {
    let value = names.map_or_else(
        || "null".to_owned(),
        |names| {
            format!(
                "[{}]",
                names
                    .iter()
                    .map(|name| name
                        .as_ref()
                        .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        },
    );
    emit(output, key, value);
}

fn emit_object(
    output: &mut String,
    key: &str,
    result: StandardExpressionResult<Option<Arc<TemplateValue>>>,
) {
    let value = result
        .expect("ProbeFactory does not fail")
        .and_then(|value| value.to_utf16_string())
        .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy());
    emit(output, key, value);
}

fn emit_build_error(
    output: &mut String,
    key: &str,
    result: Result<DialectSetConfiguration, DialectSetConfigurationError>,
) {
    match result {
        Ok(_) => emit(output, key, "NO_ERROR"),
        Err(error) => {
            let cause = match &error {
                DialectSetConfigurationError::Configuration(configuration) => {
                    configuration.source().map_or_else(
                        || "null".to_owned(),
                        |_| "java.lang.NoSuchMethodException".to_owned(),
                    )
                }
                DialectSetConfigurationError::IllegalArgument(_) => "null".to_owned(),
            };
            emit(
                output,
                key,
                format!("{}:{}|cause={cause}", error.class_name(), error),
            );
        }
    }
}

fn emit_validate_error<T>(
    output: &mut String,
    key: &str,
    result: Result<T, thymeleaf::util::ValidateError>,
) {
    match result {
        Ok(_) => emit(output, key, "NO_ERROR"),
        Err(error) => emit(
            output,
            key,
            format!("{}:{}|cause=null", error.class_name(), error),
        ),
    }
}

fn emit(output: &mut String, key: &str, value: impl ToString) {
    output.push_str(key);
    output.push('=');
    output.push_str(&value.to_string());
    output.push('\n');
}

trait Pipe: Sized {
    fn pipe<R>(self, operation: impl FnOnce(Self) -> R) -> R {
        operation(self)
    }
}

impl<T> Pipe for T {}

struct ProbeFactory {
    id: &'static str,
    names: Option<ExpressionObjectNames>,
    cacheable: bool,
    calls: AtomicUsize,
}

impl ProbeFactory {
    fn new(id: &'static str, names: Option<Vec<Option<Utf16String>>>, cacheable: bool) -> Self {
        Self {
            id,
            names: names.map(Into::into),
            cacheable,
            calls: AtomicUsize::new(0),
        }
    }
}

impl IExpressionObjectFactory for ProbeFactory {
    fn get_all_expression_object_names(&self) -> Option<ExpressionObjectNames> {
        self.names.clone()
    }

    fn build_object(
        &self,
        _context: Arc<dyn IExpressionContext>,
        expression_object_name: Option<&Utf16String>,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let name =
            expression_object_name.map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy);
        Ok(Some(Arc::new(TemplateValue::string(
            Utf16String::from_rust_str(&format!("{}:{name}", self.id)),
        ))))
    }

    fn is_cacheable(&self, _expression_object_name: Option<&Utf16String>) -> bool {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.cacheable
    }
}

struct CapabilityDialect {
    attributes: Option<ExecutionAttributeMap>,
    factory: Option<Arc<dyn IExpressionObjectFactory>>,
    pre: Option<(i32, Arc<AtomicUsize>, Option<PreProcessorSet>)>,
    post: Option<(i32, Arc<AtomicUsize>, Option<PostProcessorSet>)>,
}

impl CapabilityDialect {
    fn new(
        attributes: Option<ExecutionAttributeMap>,
        factory: Option<Arc<dyn IExpressionObjectFactory>>,
        pre: Option<(i32, Arc<AtomicUsize>, Option<PreProcessorSet>)>,
        post: Option<(i32, Arc<AtomicUsize>, Option<PostProcessorSet>)>,
    ) -> Self {
        Self {
            attributes,
            factory,
            pre,
            post,
        }
    }
}

impl IDialect for CapabilityDialect {
    fn class_name(&self) -> &'static str {
        DIALECT_CLASS
    }

    fn as_execution_attribute_dialect(&self) -> Option<&dyn IExecutionAttributeDialect> {
        Some(self)
    }

    fn as_expression_object_dialect(&self) -> Option<&dyn IExpressionObjectDialect> {
        Some(self)
    }

    fn as_pre_processor_dialect(&self) -> Option<&dyn IPreProcessorDialect> {
        Some(self)
    }

    fn as_post_processor_dialect(&self) -> Option<&dyn IPostProcessorDialect> {
        Some(self)
    }

    fn get_name(&self) -> Option<&str> {
        Some("capability")
    }
}

impl IExecutionAttributeDialect for CapabilityDialect {
    fn get_execution_attributes(&self) -> Option<ExecutionAttributeMap> {
        self.attributes.clone()
    }
}

impl IExpressionObjectDialect for CapabilityDialect {
    fn get_expression_object_factory(&self) -> Option<Arc<dyn IExpressionObjectFactory>> {
        self.factory.clone()
    }
}

impl IPreProcessorDialect for CapabilityDialect {
    fn get_dialect_pre_processor_precedence(&self) -> i32 {
        let Some((precedence, calls, _)) = &self.pre else {
            return 0;
        };
        calls.fetch_add(1, Ordering::SeqCst);
        *precedence
    }

    fn get_pre_processors(&self) -> Option<PreProcessorSet> {
        self.pre
            .as_ref()
            .and_then(|(_, _, processors)| processors.clone())
    }
}

impl IPostProcessorDialect for CapabilityDialect {
    fn get_dialect_post_processor_precedence(&self) -> i32 {
        let Some((precedence, calls, _)) = &self.post else {
            return 0;
        };
        calls.fetch_add(1, Ordering::SeqCst);
        *precedence
    }

    fn get_post_processors(&self) -> Option<PostProcessorSet> {
        self.post
            .as_ref()
            .and_then(|(_, _, processors)| processors.clone())
    }
}

struct ProbePreProcessor {
    class_name: &'static str,
    mode: Option<TemplateMode>,
    handler_class: Option<TemplateHandlerClass>,
    precedence: i32,
}

impl IPreProcessor for ProbePreProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.mode
    }

    fn get_precedence(&self) -> i32 {
        self.precedence
    }

    fn get_handler_class(&self) -> Option<&TemplateHandlerClass> {
        self.handler_class.as_ref()
    }

    fn class_name(&self) -> &'static str {
        self.class_name
    }
}

struct ProbePostProcessor {
    class_name: &'static str,
    mode: Option<TemplateMode>,
    handler_class: Option<TemplateHandlerClass>,
    precedence: i32,
}

impl IPostProcessor for ProbePostProcessor {
    fn get_template_mode(&self) -> Option<TemplateMode> {
        self.mode
    }

    fn get_precedence(&self) -> i32 {
        self.precedence
    }

    fn get_handler_class(&self) -> Option<&TemplateHandlerClass> {
        self.handler_class.as_ref()
    }

    fn class_name(&self) -> &'static str {
        self.class_name
    }
}

struct AwarePreProcessor {
    handler_class: TemplateHandlerClass,
    element_calls: AtomicUsize,
    attribute_calls: AtomicUsize,
    element_identity: Mutex<Option<usize>>,
    attribute_identity: Mutex<Option<usize>>,
}

impl AwarePreProcessor {
    fn new() -> Self {
        Self {
            handler_class: valid_handler().expect("valid Handler"),
            element_calls: AtomicUsize::new(0),
            attribute_calls: AtomicUsize::new(0),
            element_identity: Mutex::new(None),
            attribute_identity: Mutex::new(None),
        }
    }
}

impl IPreProcessor for AwarePreProcessor {
    fn is_attribute_definitions_aware(&self) -> bool {
        true
    }

    fn set_attribute_definitions(&self, definitions: Arc<AttributeDefinitions>) {
        self.attribute_calls.fetch_add(1, Ordering::SeqCst);
        *self.attribute_identity.lock().expect("attribute identity") =
            Some(Arc::as_ptr(&definitions) as usize);
    }

    fn is_element_definitions_aware(&self) -> bool {
        true
    }

    fn set_element_definitions(&self, definitions: Arc<ElementDefinitions>) {
        self.element_calls.fetch_add(1, Ordering::SeqCst);
        *self.element_identity.lock().expect("element identity") =
            Some(Arc::as_ptr(&definitions) as usize);
    }

    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(TemplateMode::HTML)
    }

    fn get_precedence(&self) -> i32 {
        0
    }

    fn get_handler_class(&self) -> Option<&TemplateHandlerClass> {
        Some(&self.handler_class)
    }
}

struct AwarePostProcessor {
    handler_class: TemplateHandlerClass,
    element_calls: AtomicUsize,
    attribute_calls: AtomicUsize,
}

impl AwarePostProcessor {
    fn new() -> Self {
        Self {
            handler_class: valid_handler().expect("valid Handler"),
            element_calls: AtomicUsize::new(0),
            attribute_calls: AtomicUsize::new(0),
        }
    }
}

impl IPostProcessor for AwarePostProcessor {
    fn is_attribute_definitions_aware(&self) -> bool {
        true
    }

    fn set_attribute_definitions(&self, _definitions: Arc<AttributeDefinitions>) {
        self.attribute_calls.fetch_add(1, Ordering::SeqCst);
    }

    fn is_element_definitions_aware(&self) -> bool {
        true
    }

    fn set_element_definitions(&self, _definitions: Arc<ElementDefinitions>) {
        self.element_calls.fetch_add(1, Ordering::SeqCst);
    }

    fn get_template_mode(&self) -> Option<TemplateMode> {
        Some(TemplateMode::HTML)
    }

    fn get_precedence(&self) -> i32 {
        0
    }

    fn get_handler_class(&self) -> Option<&TemplateHandlerClass> {
        Some(&self.handler_class)
    }
}
