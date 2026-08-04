//! 固定 Java 上游的引擎配置、并发模型工厂和配置快照差分测试。

use std::any::TypeId;
use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, Barrier, Mutex};

use thymeleaf::cache::StandardCacheManager;
use thymeleaf::context::StandardEngineContextFactory;
use thymeleaf::decoupled::StandardDecoupledTemplateLogicResolver;
use thymeleaf::dialect::{IExecutionAttributeDialect, IExpressionObjectDialect, IProcessorDialect};
use thymeleaf::linkbuilder::{ILinkBuilder, StandardLinkBuilder};
use thymeleaf::messageresolver::{IMessageResolver, StandardMessageResolver};
use thymeleaf::standard::StandardDialect;
use thymeleaf::templateresolver::{ITemplateResolver, StringTemplateResolver};
use thymeleaf::util::Utf16String;
use thymeleaf::{
    DialectConfiguration, EngineConfiguration, IDialect, IEngineConfiguration, TemplateMode,
};

fn golden() -> BTreeMap<String, String> {
    include_str!("../../thymeleaf/tests/fixtures/engine_configuration_golden.txt")
        .lines()
        .map(|line| {
            let (key, value) = line.split_once('=').expect("golden key/value");
            (key.to_owned(), unescape(value))
        })
        .collect()
}

fn unescape(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            result.push(character);
            continue;
        }
        match characters.next().expect("escaped character") {
            'n' => result.push('\n'),
            'r' => result.push('\r'),
            '\\' => result.push('\\'),
            other => panic!("unexpected escape: {other}"),
        }
    }
    result
}

fn template_resolver(name: &str, order: Option<i32>) -> Arc<dyn ITemplateResolver> {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_name(Some(Utf16String::from_rust_str(name)));
    resolver.set_order(order);
    Arc::new(resolver)
}

fn message_resolver(name: &str, order: Option<i32>) -> Arc<dyn IMessageResolver> {
    let mut resolver = StandardMessageResolver::new();
    resolver.set_name(Some(Utf16String::from_rust_str(name)));
    resolver.set_order(order);
    Arc::new(resolver)
}

fn link_builder(name: &str, order: Option<i32>) -> Arc<dyn ILinkBuilder> {
    let mut builder = StandardLinkBuilder::new();
    builder.set_name(Some(Utf16String::from_rust_str(name)));
    builder.set_order(order);
    Arc::new(builder)
}

fn configuration(
    template_resolvers: Vec<Arc<dyn ITemplateResolver>>,
    message_resolvers: Vec<Arc<dyn IMessageResolver>>,
    link_builders: Vec<Arc<dyn ILinkBuilder>>,
) -> Arc<EngineConfiguration> {
    let dialect: Arc<dyn IDialect> = Arc::new(StandardDialect::new());
    EngineConfiguration::new(
        template_resolvers,
        message_resolvers,
        link_builders,
        vec![DialectConfiguration::new(Some(dialect)).expect("dialect configuration")],
        Some(Arc::new(StandardCacheManager::new())),
        Arc::new(StandardEngineContextFactory::new()),
        Arc::new(StandardDecoupledTemplateLogicResolver::new()),
    )
    .expect("engine configuration")
}

fn joined_names<T: ?Sized>(values: Vec<&T>, name: impl Fn(&T) -> Option<&Utf16String>) -> String {
    values
        .into_iter()
        .map(|value| name(value).map_or_else(String::new, Utf16String::to_string_lossy))
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn engine_configuration_matches_java_golden() {
    let fixture = golden();
    assert_eq!(
        fixture["baseline"],
        "10f9dd2eb8cbd98515ce14b149d115e0287d0add"
    );

    let configuration = configuration(
        vec![
            template_resolver("template-null", None),
            template_resolver("template-twenty-a", Some(20)),
            template_resolver("template-negative", Some(-1)),
            template_resolver("template-twenty-b", Some(20)),
        ],
        vec![
            message_resolver("message-null", None),
            message_resolver("message-five-a", Some(5)),
            message_resolver("message-min", Some(i32::MIN)),
            message_resolver("message-five-b", Some(5)),
        ],
        vec![
            link_builder("link-null", None),
            link_builder("link-max", Some(i32::MAX)),
            link_builder("link-zero-a", Some(0)),
            link_builder("link-zero-b", Some(0)),
        ],
    );

    assert_eq!(
        joined_names(configuration.get_template_resolvers(), |value| value
            .get_name()),
        fixture["order.template"]
    );
    assert_eq!(
        joined_names(configuration.get_message_resolvers(), |value| value
            .get_name()),
        fixture["order.message"]
    );
    assert_eq!(
        joined_names(configuration.get_link_builders(), |value| value.get_name()),
        fixture["order.link"]
    );

    let mut template_snapshot = configuration.get_template_resolvers();
    let mut message_snapshot = configuration.get_message_resolvers();
    let mut link_snapshot = configuration.get_link_builders();
    template_snapshot.clear();
    message_snapshot.clear();
    link_snapshot.clear();
    assert_eq!(
        configuration.get_template_resolvers().len().to_string(),
        fixture["snapshot.template.size"]
    );
    assert_eq!(
        configuration.get_message_resolvers().len().to_string(),
        fixture["snapshot.message.size"]
    );
    assert_eq!(
        configuration.get_link_builders().len().to_string(),
        fixture["snapshot.link.size"]
    );

    assert_eq!(
        configuration.get_dialects().len().to_string(),
        fixture["dialect.all"]
    );
    assert_eq!(
        configuration
            .get_dialects_of_type(TypeId::of::<StandardDialect>())
            .len()
            .to_string(),
        fixture["dialect.standard"]
    );
    assert_eq!(
        configuration
            .get_dialects_of_type(TypeId::of::<dyn IProcessorDialect>())
            .len()
            .to_string(),
        fixture["dialect.processor"]
    );
    assert_eq!(
        configuration
            .get_dialects_of_type(TypeId::of::<dyn IExpressionObjectDialect>())
            .len()
            .to_string(),
        fixture["dialect.expression"]
    );
    assert_eq!(
        configuration
            .get_dialects_of_type(TypeId::of::<dyn IExecutionAttributeDialect>())
            .len()
            .to_string(),
        fixture["dialect.execution"]
    );
    assert_eq!(
        configuration.is_standard_dialect_present().to_string(),
        fixture["dialect.present"]
    );
    assert_eq!(
        configuration
            .get_standard_dialect_prefix()
            .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy),
        fixture["dialect.prefix"]
    );
    assert_eq!(
        std::ptr::eq(
            configuration.get_element_definitions(),
            configuration.get_element_definitions(),
        )
        .to_string(),
        fixture["definitions.element.identity"]
    );
    assert_eq!(
        std::ptr::eq(
            configuration.get_attribute_definitions(),
            configuration.get_attribute_definitions(),
        )
        .to_string(),
        fixture["definitions.attribute.identity"]
    );

    let _manager = configuration.get_template_manager();
    assert_eq!("true", fixture["manager.present"]);
    let html_first = configuration.get_model_factory(TemplateMode::HTML);
    let html_second = configuration.get_model_factory(TemplateMode::HTML);
    let xml = configuration.get_model_factory(TemplateMode::XML);
    assert_eq!(
        std::ptr::eq(html_first, html_second).to_string(),
        fixture["model.same"]
    );
    assert_eq!(
        (!std::ptr::eq(html_first, xml)).to_string(),
        fixture["model.different_mode"]
    );

    for mode in [
        TemplateMode::HTML,
        TemplateMode::XML,
        TemplateMode::TEXT,
        TemplateMode::JAVASCRIPT,
        TemplateMode::CSS,
        TemplateMode::RAW,
    ] {
        assert_eq!(
            configuration.is_model_reshapeable(mode).to_string(),
            fixture[&format!("reshape.{mode}")]
        );
        let bucket = format!(
            "{},{},{},{},{},{},{},{},{},{}",
            configuration.get_template_boundaries_processors(mode).len(),
            configuration.get_cdata_section_processors(mode).len(),
            configuration.get_comment_processors(mode).len(),
            configuration.get_doc_type_processors(mode).len(),
            configuration.get_element_processors(mode).len(),
            configuration.get_text_processors(mode).len(),
            configuration
                .get_processing_instruction_processors(mode)
                .len(),
            configuration.get_xml_declaration_processors(mode).len(),
            configuration.get_pre_processors(mode).len(),
            configuration.get_post_processors(mode).len(),
        );
        assert_eq!(bucket, fixture[&format!("bucket.{mode}")]);
    }
}

#[test]
fn model_factory_is_initialized_once_under_real_concurrency() {
    let fixture = golden();
    let configuration = configuration(
        vec![template_resolver("template", None)],
        Vec::new(),
        Vec::new(),
    );
    let workers = 12;
    let barrier = Arc::new(Barrier::new(workers));
    let identities = Arc::new(Mutex::new(HashSet::new()));
    let handles = (0..workers)
        .map(|_| {
            let configuration = Arc::clone(&configuration);
            let barrier = Arc::clone(&barrier);
            let identities = Arc::clone(&identities);
            std::thread::spawn(move || {
                barrier.wait();
                let factory = configuration.get_model_factory(TemplateMode::JAVASCRIPT);
                let identity = std::ptr::from_ref(factory).cast::<()>() as usize;
                identities
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .insert(identity);
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().expect("model factory worker");
    }
    assert_eq!(
        identities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
            .to_string(),
        fixture["model.concurrent.identities"]
    );
}
