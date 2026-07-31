//! 模板 Resolver 批次的固定 Java Golden 差分与 Rust 扩展义务测试。

use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use indexmap::IndexMap;
use thymeleaf::cache::ICacheEntryValidity;
use thymeleaf::context::Context;
use thymeleaf::expression::TemplateValue;
use thymeleaf::templateresource::UrlResourceConnectionHandler;
use thymeleaf::util::JavaString;
use thymeleaf::web::IWebApplication;
use thymeleaf::{
    AbstractConfigurableTemplateResolver, ClassLoaderTemplateResolver, DefaultTemplateResolver,
    FileTemplateResolver, IEngineConfiguration, ITemplateEngine, ITemplateResolver,
    ITemplateResource, StringTemplateResolver, TemplateEngine, TemplateMode, TemplateResolution,
    TemplateResolutionAttributes, TemplateResolverError, TemplateResourceError,
    UrlTemplateResolver, WebApplicationTemplateResolver,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/template_resolver_golden.txt");

#[test]
fn template_resolver_batch_matches_java_golden() {
    let configuration = initialized_configuration();
    let mut output = String::new();
    emit(&mut output, "java_baseline", JAVA_BASELINE);
    export_abstract_resolver(&mut output, configuration.as_ref());
    export_configurable_resolver(&mut output);
    export_default_resolver(&mut output, configuration.as_ref());
    export_string_resolver(&mut output, configuration.as_ref());
    export_file_resolver(&mut output, configuration.as_ref());
    export_class_loader_resolver(&mut output, configuration.as_ref());
    export_url_resolver(&mut output, configuration.as_ref());
    export_web_resolver(&mut output, configuration.as_ref());
    assert_eq!(output, JAVA_GOLDEN);
}

#[test]
fn url_resolver_passes_custom_protocol_handler_to_resources() {
    let configuration = initialized_configuration();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let observed = Arc::clone(&calls);
    let handler: Arc<UrlResourceConnectionHandler> = Arc::new(move |url| {
        observed
            .lock()
            .expect("protocol call lock")
            .push(url.to_string());
        Ok(Box::new(Cursor::new(b"custom-body".to_vec())))
    });
    let mut resolver = UrlTemplateResolver::new();
    resolver.set_connection_handler(Some(Arc::clone(&handler)));
    assert!(Arc::ptr_eq(
        resolver
            .get_connection_handler()
            .expect("configured handler"),
        &handler
    ));

    let resolution = resolver
        .resolve_template(
            configuration.as_ref(),
            None,
            &java("custom://host/templates/main.html"),
            None,
        )
        .expect("custom protocol resolution")
        .expect("custom protocol applies");
    let mut reader = resolution
        .get_template_resource()
        .reader()
        .expect("custom protocol reader");
    let mut body = String::new();
    reader.read_to_string(&mut body).expect("read custom body");
    assert_eq!(body, "custom-body");
    assert_eq!(
        calls.lock().expect("protocol calls").as_slice(),
        ["custom://host/templates/main.html"]
    );
}

fn initialized_configuration() -> Arc<dyn IEngineConfiguration> {
    let engine = TemplateEngine::new();
    engine
        .process_template("resolver-init", &Context::new())
        .expect("initialize template engine");
    engine
        .get_configuration()
        .expect("initialized engine configuration")
}

fn export_abstract_resolver(output: &mut String, configuration: &dyn IEngineConfiguration) {
    let mut resolver = ProbeResolver::new();
    emit_optional_java(output, "abstract.name.default", resolver.get_name());
    emit_optional_i32(output, "abstract.order.default", resolver.get_order());
    emit_bool(
        output,
        "abstract.resolvable.empty",
        resolver.get_resolvable_pattern_spec().is_empty(),
    );
    emit_bool(
        output,
        "abstract.check.default",
        resolver.get_check_existence(),
    );
    emit_bool(
        output,
        "abstract.decoupled.default",
        resolver.get_use_decoupled_logic(),
    );

    resolver.set_name(None);
    resolver.set_order(Some(-2));
    resolver.set_use_decoupled_logic(true);
    emit_optional_java(output, "abstract.name.null", resolver.get_name());
    emit_optional_i32(output, "abstract.order.negative", resolver.get_order());
    emit_bool(
        output,
        "abstract.decoupled.true",
        resolver.get_use_decoupled_logic(),
    );

    let error = resolver
        .resolve_template_nullable(None, None, Some(&java("name")), None)
        .err()
        .expect("null configuration");
    emit_resolver_failure(output, "abstract.resolve.null_configuration", &error);
    let error = resolver
        .resolve_template_nullable(Some(configuration), None, None, None)
        .err()
        .expect("null template");
    emit_resolver_failure(output, "abstract.resolve.null_template", &error);

    resolver.reset_calls();
    resolver
        .set_resolvable_patterns(Some(&[Some("admin/*")]))
        .expect("resolvable pattern");
    let rejected = resolver
        .resolve_template(configuration, None, &java("public/x"), None)
        .expect("pattern rejection");
    emit_none_resolution(output, "abstract.resolve.pattern_rejected", rejected);
    emit_usize(
        output,
        "abstract.resolve.pattern_rejected.resource_calls",
        resolver.resource_calls(),
    );

    resolver.reset_calls();
    resolver.return_null.store(true, Ordering::SeqCst);
    let no_resource = resolver
        .resolve_template(configuration, None, &java("admin/x"), None)
        .expect("null resource");
    emit_none_resolution(output, "abstract.resolve.null_resource", no_resource);
    emit_usize(
        output,
        "abstract.resolve.null_resource.mode_calls",
        resolver.mode_calls(),
    );
    emit_usize(
        output,
        "abstract.resolve.null_resource.validity_calls",
        resolver.validity_calls(),
    );

    resolver.return_null.store(false, Ordering::SeqCst);
    resolver.resource_exists.store(false, Ordering::SeqCst);
    resolver.set_check_existence(true);
    resolver.reset_calls();
    let missing = resolver
        .resolve_template(configuration, None, &java("admin/x"), None)
        .expect("missing resource");
    emit_none_resolution(output, "abstract.resolve.missing", missing);
    emit_usize(
        output,
        "abstract.resolve.missing.resource_calls",
        resolver.resource_calls(),
    );
    emit_usize(
        output,
        "abstract.resolve.missing.mode_calls",
        resolver.mode_calls(),
    );
    emit_usize(
        output,
        "abstract.resolve.missing.validity_calls",
        resolver.validity_calls(),
    );

    resolver.set_check_existence(false);
    resolver.reset_calls();
    let unchecked = resolver
        .resolve_template(configuration, None, &java("admin/x"), None)
        .expect("unchecked resource")
        .expect("resolution");
    emit_bool(output, "abstract.resolve.unchecked.present", true);
    emit_bool(
        output,
        "abstract.resolve.unchecked.verified",
        unchecked.is_template_resource_existence_verified(),
    );
    emit_usize(
        output,
        "abstract.resolve.unchecked.mode_calls",
        resolver.mode_calls(),
    );
    emit_usize(
        output,
        "abstract.resolve.unchecked.validity_calls",
        resolver.validity_calls(),
    );
}

fn export_configurable_resolver(output: &mut String) {
    let mut resolver =
        AbstractConfigurableTemplateResolver::new("TemplateResolverGolden$ProbeResolver");
    emit_optional_java(output, "config.prefix.default", resolver.get_prefix());
    emit_optional_java(output, "config.suffix.default", resolver.get_suffix());
    emit_bool(
        output,
        "config.force_suffix.default",
        resolver.get_force_suffix(),
    );
    emit_optional_java(
        output,
        "config.encoding.default",
        resolver.get_character_encoding(),
    );
    emit_mode(output, "config.mode.default", resolver.get_template_mode());
    emit_bool(
        output,
        "config.force_mode.default",
        resolver.get_force_template_mode(),
    );
    emit_bool(output, "config.cacheable.default", resolver.is_cacheable());
    emit_optional_i64(output, "config.ttl.default", resolver.get_cache_ttl_ms());
    emit_usize(
        output,
        "config.aliases.default_size",
        resolver.get_template_aliases().len(),
    );

    resolver.set_prefix(Some(java("/views/")));
    resolver.set_suffix(Some(java(".html")));
    emit_java(
        output,
        "config.resource.basic",
        &resolver.compute_resource_name(&java("page")),
    );
    emit_java(
        output,
        "config.resource.known_extension",
        &resolver.compute_resource_name(&java("page.xml")),
    );
    resolver.set_force_suffix(true);
    emit_java(
        output,
        "config.resource.force_suffix",
        &resolver.compute_resource_name(&java("page.xml")),
    );
    resolver.set_prefix(Some(java("\t \u{3000}")));
    resolver.set_suffix(Some(java("\t \u{3000}")));
    emit_java(
        output,
        "config.resource.blank_affixes",
        &resolver.compute_resource_name(&java("page")),
    );

    resolver.set_prefix(None);
    resolver.set_suffix(None);
    resolver.set_force_suffix(false);
    let first = HashMap::from([(java("short"), java("first"))]);
    resolver.set_template_aliases(Some(&first));
    let second = HashMap::from([
        (java("short"), java("override")),
        (java("other"), java("second")),
    ]);
    resolver.set_template_aliases(Some(&second));
    resolver.set_template_aliases(None);
    emit_usize(
        output,
        "config.aliases.merge_size",
        resolver.get_template_aliases().len(),
    );
    emit_java(
        output,
        "config.aliases.override",
        &resolver.compute_resource_name(&java("short")),
    );
    emit_java(
        output,
        "config.aliases.preserve",
        &resolver.compute_resource_name(&java("other")),
    );
    let error = resolver
        .add_template_alias_nullable(None, Some(java("value")))
        .expect_err("null alias");
    emit_resolver_failure(output, "config.alias.null", &error);
    let error = resolver
        .add_template_alias_nullable(Some(java("x")), None)
        .expect_err("null template alias value");
    emit_resolver_failure(output, "config.alias.value_null", &error);

    let isolated = JavaString::from_utf16(vec![u16::from(b'p'), 0xD800, u16::from(b'x')]);
    resolver.set_prefix(Some(JavaString::from_utf16(vec![0xD801])));
    resolver.set_suffix(Some(JavaString::from_utf16(vec![0xD802])));
    resolver.set_force_suffix(true);
    emit(
        output,
        "config.resource.utf16",
        &code_units(&resolver.compute_resource_name(&isolated)),
    );

    let error = resolver
        .set_template_mode_nullable(None)
        .expect_err("null enum mode");
    emit_resolver_failure(output, "config.mode.enum_null", &error);
    let error = resolver
        .set_template_mode_name(None)
        .expect_err("null string mode");
    emit_resolver_failure(output, "config.mode.string_null", &error);

    let mut modes =
        AbstractConfigurableTemplateResolver::new("TemplateResolverGolden$ProbeResolver");
    modes
        .set_xml_template_mode_patterns(Some(&[Some("*.data")]))
        .expect("XML patterns");
    modes
        .set_html_template_mode_patterns(Some(&[Some("*.data")]))
        .expect("HTML patterns");
    emit_mode(
        output,
        "config.mode.pattern_precedence",
        modes.compute_template_mode(&java("sample.data")),
    );
    emit_mode(
        output,
        "config.mode.auto_text",
        modes.compute_template_mode(&java("sample.txt")),
    );
    modes.set_template_mode(TemplateMode::CSS);
    modes.set_force_template_mode(true);
    emit_mode(
        output,
        "config.mode.forced",
        modes.compute_template_mode(&java("sample.html")),
    );

    let mut validity =
        AbstractConfigurableTemplateResolver::new("TemplateResolverGolden$ProbeResolver");
    validity
        .set_cacheable_patterns(Some(&[Some("*.both")]))
        .expect("cacheable patterns");
    validity
        .set_non_cacheable_patterns(Some(&[Some("*.both"), Some("*.none")]))
        .expect("non-cacheable patterns");
    emit_validity(
        output,
        "config.validity.both",
        validity.compute_validity(&java("x.both")).as_ref(),
        None,
    );
    emit_validity(
        output,
        "config.validity.non_cacheable",
        validity.compute_validity(&java("x.none")).as_ref(),
        None,
    );
    validity.set_cacheable(false);
    emit_validity(
        output,
        "config.validity.default_false",
        validity.compute_validity(&java("x.other")).as_ref(),
        None,
    );
    validity.set_cacheable(true);
    validity.set_cache_ttl_ms(Some(-7));
    emit_validity(
        output,
        "config.validity.ttl",
        validity.compute_validity(&java("x.other")).as_ref(),
        Some(-7),
    );
}

fn export_default_resolver(output: &mut String, configuration: &dyn IEngineConfiguration) {
    let mut resolver = DefaultTemplateResolver::new();
    emit_optional_java(output, "default.name", resolver.get_name());
    emit_mode(output, "default.mode", resolver.get_template_mode());
    emit_optional_java(output, "default.template", resolver.get_template());
    let first = resolver
        .resolve_template(configuration, None, &java("ignored"), None)
        .expect("default resolution")
        .expect("default applies");
    emit(
        output,
        "default.reader.empty",
        &read_resource(first.get_template_resource()),
    );
    emit_validity(output, "default.validity", first.get_validity(), None);

    resolver.set_template(Some(java("fixed")));
    resolver.set_template_mode(TemplateMode::TEXT);
    let dynamic: &dyn ITemplateResolver = &resolver;
    let fixed = dynamic
        .resolve_template(
            configuration,
            Some(&java("owner")),
            &java("ignored-again"),
            None,
        )
        .expect("dynamic default resolution")
        .expect("dynamic default applies");
    emit(
        output,
        "default.dynamic.reader",
        &read_resource(fixed.get_template_resource()),
    );
    emit_mode(output, "default.dynamic.mode", fixed.get_template_mode());

    let error = resolver
        .set_template_mode_nullable(None)
        .expect_err("null enum mode");
    emit_resolver_failure(output, "default.mode.enum_null", &error);
    let error = resolver
        .set_template_mode_name(None)
        .expect_err("null string mode");
    emit_resolver_failure(output, "default.mode.string_null", &error);
    resolver.set_template(None);
    let error = resolver
        .resolve_template(configuration, None, &java("ignored"), None)
        .err()
        .expect("null fixed template");
    emit_resolver_failure(output, "default.template.null_resolution", &error);
}

fn export_string_resolver(output: &mut String, configuration: &dyn IEngineConfiguration) {
    let mut resolver = StringTemplateResolver::new();
    emit_optional_java(output, "string_resolver.name", resolver.get_name());
    emit_mode(output, "string_resolver.mode", resolver.get_template_mode());
    emit_bool(output, "string_resolver.cacheable", resolver.is_cacheable());
    emit_optional_i64(output, "string_resolver.ttl", resolver.get_cache_ttl_ms());
    let contents = java("<p>你好 😀</p>");
    let first = resolver
        .resolve_template(configuration, None, &contents, None)
        .expect("string resolution")
        .expect("string resolver always applies");
    emit(
        output,
        "string_resolver.reader",
        &read_resource(first.get_template_resource()),
    );
    emit_validity(
        output,
        "string_resolver.validity.default",
        first.get_validity(),
        None,
    );

    resolver.set_cacheable(true);
    let always = resolver
        .resolve_template(configuration, None, &java("x"), None)
        .expect("always validity")
        .expect("string resolution");
    emit_validity(
        output,
        "string_resolver.validity.always",
        always.get_validity(),
        None,
    );
    resolver.set_cache_ttl_ms(Some(-5));
    let ttl = resolver
        .resolve_template(configuration, None, &java("x"), None)
        .expect("TTL validity")
        .expect("string resolution");
    emit_validity(
        output,
        "string_resolver.validity.ttl",
        ttl.get_validity(),
        Some(-5),
    );
    resolver
        .set_use_decoupled_logic(false)
        .expect("false decoupled flag");
    let error = resolver
        .set_use_decoupled_logic(true)
        .expect_err("string resolver rejects decoupled logic");
    emit_failure(
        output,
        "string_resolver.decoupled.true",
        "org.thymeleaf.exceptions.ConfigurationException",
        error.get_message().unwrap_or("null"),
    );
    let error = resolver
        .set_template_mode_nullable(None)
        .expect_err("null enum mode");
    emit_resolver_failure(output, "string_resolver.mode.enum_null", &error);
    let error = resolver
        .set_template_mode_name(None)
        .expect_err("null string mode");
    emit_resolver_failure(output, "string_resolver.mode.string_null", &error);
}

fn export_file_resolver(output: &mut String, configuration: &dyn IEngineConfiguration) {
    let directory = temporary_directory("file-resolver");
    fs::write(directory.join("main.txt"), b"file-body").expect("write file resolver fixture");
    let mut resolver = FileTemplateResolver::new();
    resolver.set_prefix(Some(java(&format!("{}/", directory.to_string_lossy()))));
    resolver.set_suffix(Some(java(".txt")));
    resolver.set_check_existence(true);
    let resolution = resolver
        .resolve_template(configuration, None, &java("main"), None)
        .expect("file resolution")
        .expect("file exists");
    emit_bool(output, "file_resolver.present", true);
    emit_optional_str(
        output,
        "file_resolver.base_name",
        resolution
            .get_template_resource()
            .get_base_name()
            .as_deref(),
    );
    emit(
        output,
        "file_resolver.reader",
        &read_resource(resolution.get_template_resource()),
    );
    emit_bool(
        output,
        "file_resolver.verified",
        resolution.is_template_resource_existence_verified(),
    );
    fs::remove_dir_all(directory).expect("remove file resolver fixture");

    let error = FileTemplateResolver::new()
        .resolve_template(configuration, None, &java(""), None)
        .err()
        .expect("empty file resource name");
    emit_resolver_failure(output, "file_resolver.empty_template", &error);
}

fn export_class_loader_resolver(output: &mut String, configuration: &dyn IEngineConfiguration) {
    let root = temporary_directory("class-resolver");
    let templates = root.join("templates");
    fs::create_dir_all(&templates).expect("create class resolver fixture");
    fs::write(templates.join("main.txt"), b"class-body").expect("write class resolver fixture");
    let mut resolver = ClassLoaderTemplateResolver::with_search_roots(vec![root.clone()]);
    resolver.set_prefix(Some(java("templates/")));
    resolver.set_suffix(Some(java(".txt")));
    resolver.set_check_existence(true);
    let resolution = resolver
        .resolve_template(configuration, None, &java("main"), None)
        .expect("class-loader resolution")
        .expect("class-loader resource exists");
    emit_optional_java(output, "class_resolver.name", resolver.get_name());
    emit_bool(output, "class_resolver.present", true);
    emit_optional_str(
        output,
        "class_resolver.base_name",
        resolution
            .get_template_resource()
            .get_base_name()
            .as_deref(),
    );
    emit(
        output,
        "class_resolver.reader",
        &read_resource(resolution.get_template_resource()),
    );
    emit_bool(
        output,
        "class_resolver.verified",
        resolution.is_template_resource_existence_verified(),
    );
    fs::remove_dir_all(root).expect("remove class resolver fixture");

    let error = ClassLoaderTemplateResolver::new()
        .resolve_template(configuration, None, &java(""), None)
        .err()
        .expect("empty class-loader resource name");
    emit_resolver_failure(output, "class_resolver.empty_template", &error);
}

fn export_url_resolver(output: &mut String, configuration: &dyn IEngineConfiguration) {
    let resolver = UrlTemplateResolver::new();
    emit_optional_java(output, "url_resolver.name", resolver.get_name());
    let malformed = resolver
        .resolve_template(configuration, None, &java("not-a-url"), None)
        .expect("malformed URL means not applicable");
    emit_none_resolution(output, "url_resolver.malformed", malformed);
    let error = resolver
        .resolve_template(configuration, None, &java(""), None)
        .err()
        .expect("empty URL is an argument error");
    emit_resolver_failure(output, "url_resolver.empty_template", &error);

    let jsession = resolver
        .resolve_template(
            configuration,
            None,
            &java("HTTP://example.test/a;JSESSIONID=1"),
            None,
        )
        .expect("session URL")
        .expect("valid session URL");
    emit_validity(
        output,
        "url_resolver.jsessionid",
        jsession.get_validity(),
        None,
    );
    let newline = resolver.compute_validity(&java("http://example.test/a;jsessionid=1\nx"));
    emit_validity(
        output,
        "url_resolver.jsessionid_newline",
        newline.as_ref(),
        None,
    );
}

fn export_web_resolver(output: &mut String, configuration: &dyn IEngineConfiguration) {
    let error = WebApplicationTemplateResolver::try_new(None)
        .err()
        .expect("null web application");
    emit_resolver_failure(output, "web_resolver.null_application", &error);
    let application: Arc<dyn IWebApplication> = Arc::new(TestWebApplication);
    let mut resolver = WebApplicationTemplateResolver::new(Arc::clone(&application));
    resolver.set_prefix(Some(java("templates/")));
    resolver.set_suffix(Some(java(".html")));
    let resolution = resolver
        .resolve_template(configuration, None, &java("main"), None)
        .expect("web resolution")
        .expect("web resolver applies");
    emit_optional_java(output, "web_resolver.name", resolver.get_name());
    emit(
        output,
        "web_resolver.description",
        &resolution.get_template_resource().get_description(),
    );
    emit_bool(
        output,
        "web_resolver.verified",
        resolution.is_template_resource_existence_verified(),
    );
    let error = WebApplicationTemplateResolver::new(application)
        .resolve_template(configuration, None, &java(""), None)
        .err()
        .expect("empty web resource name");
    emit_resolver_failure(output, "web_resolver.empty_template", &error);
}

struct ProbeResolver {
    resolver: AbstractConfigurableTemplateResolver,
    return_null: AtomicBool,
    resource_exists: AtomicBool,
    resource_calls: AtomicUsize,
    mode_calls: AtomicUsize,
    validity_calls: AtomicUsize,
}

impl ProbeResolver {
    fn new() -> Self {
        Self {
            resolver: AbstractConfigurableTemplateResolver::new(
                "TemplateResolverGolden$ProbeResolver",
            ),
            return_null: AtomicBool::new(false),
            resource_exists: AtomicBool::new(true),
            resource_calls: AtomicUsize::new(0),
            mode_calls: AtomicUsize::new(0),
            validity_calls: AtomicUsize::new(0),
        }
    }

    fn reset_calls(&self) {
        self.resource_calls.store(0, Ordering::SeqCst);
        self.mode_calls.store(0, Ordering::SeqCst);
        self.validity_calls.store(0, Ordering::SeqCst);
    }

    fn resource_calls(&self) -> usize {
        self.resource_calls.load(Ordering::SeqCst)
    }

    fn mode_calls(&self) -> usize {
        self.mode_calls.load(Ordering::SeqCst)
    }

    fn validity_calls(&self) -> usize {
        self.validity_calls.load(Ordering::SeqCst)
    }
}

impl std::ops::Deref for ProbeResolver {
    type Target = AbstractConfigurableTemplateResolver;

    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl std::ops::DerefMut for ProbeResolver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resolver
    }
}

impl ITemplateResolver for ProbeResolver {
    fn get_name(&self) -> Option<&JavaString> {
        self.resolver.get_name()
    }

    fn get_order(&self) -> Option<i32> {
        self.resolver.get_order()
    }

    fn resolve_template(
        &self,
        _configuration: &dyn IEngineConfiguration,
        _owner_template: Option<&JavaString>,
        template: &JavaString,
        _template_resolution_attributes: Option<&TemplateResolutionAttributes>,
    ) -> Result<Option<TemplateResolution>, TemplateResolverError> {
        self.resolver.resolver().resolve_template(
            template,
            || {
                self.resource_calls.fetch_add(1, Ordering::SeqCst);
                if self.return_null.load(Ordering::SeqCst) {
                    return Ok(None);
                }
                Ok(Some(Arc::new(ProbeResource {
                    description: self.resolver.compute_resource_name(template),
                    exists: self.resource_exists.load(Ordering::SeqCst),
                }) as Arc<dyn ITemplateResource>))
            },
            || {
                self.mode_calls.fetch_add(1, Ordering::SeqCst);
                self.resolver.compute_template_mode(template)
            },
            || {
                self.validity_calls.fetch_add(1, Ordering::SeqCst);
                self.resolver.compute_validity(template)
            },
        )
    }
}

struct ProbeResource {
    description: JavaString,
    exists: bool,
}

impl ITemplateResource for ProbeResource {
    fn get_description(&self) -> String {
        self.description.to_string_lossy()
    }

    fn get_base_name(&self) -> Option<String> {
        Some(self.description.to_string_lossy())
    }

    fn exists(&self) -> bool {
        self.exists
    }

    fn reader(&self) -> std::io::Result<Box<dyn Read>> {
        Ok(Box::new(Cursor::new(
            self.description.to_string_lossy().into_bytes(),
        )))
    }

    fn relative(
        &self,
        relative_location: Option<&str>,
    ) -> Result<Box<dyn ITemplateResource>, TemplateResourceError> {
        Ok(Box::new(Self {
            description: java(relative_location.unwrap_or("null")),
            exists: self.exists,
        }))
    }
}

struct TestWebApplication;

impl IWebApplication for TestWebApplication {
    fn contains_attribute(&self, _name: Option<&JavaString>) -> bool {
        false
    }

    fn get_attribute_count(&self) -> i32 {
        0
    }

    fn get_all_attribute_names(&self) -> Vec<Option<JavaString>> {
        Vec::new()
    }

    fn get_attribute_map(&self) -> IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>> {
        IndexMap::new()
    }

    fn get_attribute_value(&self, _name: Option<&JavaString>) -> Option<Arc<TemplateValue>> {
        None
    }

    fn set_attribute_value(&self, _name: Option<JavaString>, _value: Option<Arc<TemplateValue>>) {}

    fn remove_attribute(&self, _name: Option<&JavaString>) {}

    fn resource_exists(&self, _path: Option<&JavaString>) -> bool {
        false
    }

    fn get_resource_as_stream(&self, _path: Option<&JavaString>) -> Option<Box<dyn Read + Send>> {
        None
    }
}

fn emit_validity(
    output: &mut String,
    key: &str,
    validity: &dyn ICacheEntryValidity,
    ttl: Option<i64>,
) {
    let kind = if !validity.is_cacheable() {
        "NonCacheableCacheEntryValidity"
    } else if ttl.is_some() {
        assert!(!validity.is_cache_still_valid());
        "TTLCacheEntryValidity"
    } else {
        assert!(validity.is_cache_still_valid());
        "AlwaysValidCacheEntryValidity"
    };
    emit(output, &format!("{key}.type"), kind);
    if let Some(ttl) = ttl {
        emit(output, &format!("{key}.ttl"), &ttl.to_string());
    }
}

fn read_resource(resource: &dyn ITemplateResource) -> String {
    let mut reader = resource.reader().expect("template resource reader");
    let mut body = String::new();
    reader
        .read_to_string(&mut body)
        .expect("read template body");
    body
}

fn emit_resolver_failure(output: &mut String, key: &str, error: &TemplateResolverError) {
    let java_type = match error {
        TemplateResolverError::InvalidArgument(_) => "java.lang.IllegalArgumentException",
        TemplateResolverError::Resource(TemplateResourceError::InvalidArgument(_)) => {
            "java.lang.IllegalArgumentException"
        }
        TemplateResolverError::Resource(TemplateResourceError::MalformedUrl { .. }) => {
            "java.net.MalformedURLException"
        }
        TemplateResolverError::Resource(TemplateResourceError::Input(_)) => {
            "org.thymeleaf.exceptions.TemplateInputException"
        }
        TemplateResolverError::Resolution(_) => "java.lang.IllegalArgumentException",
    };
    emit_failure(output, key, java_type, &error.to_string());
}

fn emit_failure(output: &mut String, key: &str, java_type: &str, message: &str) {
    emit(output, &format!("{key}.type"), java_type);
    emit(output, &format!("{key}.message"), message);
}

fn emit_none_resolution(output: &mut String, key: &str, resolution: Option<TemplateResolution>) {
    assert!(resolution.is_none());
    emit(output, key, "null");
}

fn emit_optional_java(output: &mut String, key: &str, value: Option<&JavaString>) {
    emit(
        output,
        key,
        value
            .map_or("null".to_owned(), JavaString::to_string_lossy)
            .as_str(),
    );
}

fn emit_optional_str(output: &mut String, key: &str, value: Option<&str>) {
    emit(output, key, value.unwrap_or("null"));
}

fn emit_optional_i32(output: &mut String, key: &str, value: Option<i32>) {
    emit(
        output,
        key,
        &value.map_or_else(|| "null".to_owned(), |value| value.to_string()),
    );
}

fn emit_optional_i64(output: &mut String, key: &str, value: Option<i64>) {
    emit(
        output,
        key,
        &value.map_or_else(|| "null".to_owned(), |value| value.to_string()),
    );
}

fn emit_usize(output: &mut String, key: &str, value: usize) {
    emit(output, key, &value.to_string());
}

fn emit_bool(output: &mut String, key: &str, value: bool) {
    emit(output, key, if value { "true" } else { "false" });
}

fn emit_mode(output: &mut String, key: &str, value: TemplateMode) {
    emit(output, key, &value.to_string());
}

fn emit_java(output: &mut String, key: &str, value: &JavaString) {
    emit(output, key, &value.to_string_lossy());
}

fn emit(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push('=');
    output.push_str(value);
    output.push('\n');
}

fn code_units(value: &JavaString) -> String {
    value
        .as_utf16()
        .iter()
        .map(|unit| format!("{unit:04X}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn java(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn temporary_directory(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "thymeleaf-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    if directory.exists() {
        fs::remove_dir_all(&directory).expect("clear stale temporary directory");
    }
    fs::create_dir_all(&directory).expect("create temporary directory");
    directory
}
