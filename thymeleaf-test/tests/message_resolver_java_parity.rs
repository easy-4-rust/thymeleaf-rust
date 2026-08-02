//! 消息解析器批次的固定 Java Golden 差分与 Rust origin 元数据义务测试。

use std::any::TypeId;
use std::collections::HashMap;
use std::io::{self, Cursor, Read};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use num_bigint::BigInt;
use thymeleaf::cache::{
    AlwaysValidCacheEntryValidity, ICacheEntryValidity, NonCacheableCacheEntryValidity,
};
use thymeleaf::context::{Context, EngineContext, IEngineContext};
use thymeleaf::engine::TemplateData;
use thymeleaf::expression::TemplateValue;
use thymeleaf::messageresolver::{IMessageResolver, StandardMessageResolver};
use thymeleaf::templateresource::TemplateResourceError;
use thymeleaf::util::{JavaBigDecimal, JavaDate, JavaLocale, JavaNumber, JavaString};
use thymeleaf::{ITemplateEngine, ITemplateResource, TemplateEngine};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/message_resolver_golden.txt");

#[test]
fn message_format_selected_contracts_match_java_golden() {
    let golden = golden_records();
    assert_eq!(golden.len(), 109, "固定 Java Golden 记录数发生漂移");
    assert_eq!(golden["java_baseline"], JAVA_BASELINE);
    let resolver = StandardMessageResolver::new();
    let us = locale("en-US", "US");
    let de = locale("de-DE", "DE");

    assert_format_without_parameters(
        &resolver,
        &golden,
        "format.open_brace_fast_path",
        &us,
        "left { open",
    );
    assert_format(
        &resolver,
        &golden,
        "format.indexed",
        &us,
        "{0} / {1}",
        &[string("A"), integer(12)],
    );
    assert_format_without_parameters(&resolver, &golden, "format.null_array", &us, "{0}");
    assert_format(
        &resolver,
        &golden,
        "format.explicit_null",
        &us,
        "{0}",
        &[None],
    );
    assert_format(
        &resolver,
        &golden,
        "format.default_date",
        &us,
        "{0}",
        &[date(0)],
    );
    assert_format(
        &resolver,
        &golden,
        "format.quote_literal",
        &us,
        "'{0}' {0}",
        &[string("A")],
    );
    assert_format(
        &resolver,
        &golden,
        "format.quote_double",
        &us,
        "L''amour {0}",
        &[string("A")],
    );
    assert_format(
        &resolver,
        &golden,
        "format.quote_unclosed",
        &us,
        "before '{0} after",
        &[string("A")],
    );
    assert_format(
        &resolver,
        &golden,
        "format.quote_nested",
        &us,
        "a''b '{' {0} '}'",
        &[string("A")],
    );
    assert_format(
        &resolver,
        &golden,
        "format.missing_parameter",
        &us,
        "{0}-{2}",
        &[string("A")],
    );
    let surrogate_parameter = Some(Arc::new(TemplateValue::string(JavaString::from_utf16(
        vec![u16::from(b'x'), 0xd800, u16::from(b'y')],
    ))));
    let formatted = resolver
        .format_message(&us, &java("{0}"), Some(&[surrogate_parameter]))
        .expect("surrogate parameter")
        .expect("formatter result");
    assert_eq!(
        utf16_hex(&formatted),
        golden["format.parameter_surrogate_hex"]
    );
    let surrogate_pattern = JavaString::from_utf16(vec![
        u16::from(b'x'),
        0xd800,
        u16::from(b'\''),
        u16::from(b'\''),
        u16::from(b'y'),
    ]);
    let formatted = resolver
        .format_message(&us, &surrogate_pattern, None)
        .expect("surrogate pattern")
        .expect("formatter result");
    assert_eq!(
        utf16_hex(&formatted),
        golden["format.pattern_surrogate_hex"]
    );
    assert_format(
        &resolver,
        &golden,
        "format.number.us",
        &us,
        "{0,number}",
        &[double(12345.5)],
    );
    assert_format(
        &resolver,
        &golden,
        "format.number.de",
        &de,
        "{0,number}",
        &[double(12345.5)],
    );
    assert_format(
        &resolver,
        &golden,
        "format.integer",
        &us,
        "{0,number,integer}",
        &[double(12345.6)],
    );
    assert_format(
        &resolver,
        &golden,
        "format.percent",
        &us,
        "{0,number,percent}",
        &[double(0.125)],
    );
    assert_format(
        &resolver,
        &golden,
        "format.currency",
        &us,
        "{0,number,currency}",
        &[double(12345.5)],
    );
    assert_format(
        &resolver,
        &golden,
        "format.number.custom",
        &us,
        "{0,number,#,##0.00}",
        &[double(12345.5)],
    );
    for (key, pattern, value) in [
        (
            "format.number.negative_default",
            "{0,number,#,##0.00}",
            -12345.5,
        ),
        (
            "format.number.negative_subpattern",
            "{0,number,#,##0.00;(#,##0.00)}",
            -12345.5,
        ),
        (
            "format.number.optional_fraction",
            "{0,number,0000.##}",
            12.5,
        ),
        ("format.number.percent_pattern", "{0,number,0.0%}", 0.125),
        ("format.number.permille_pattern", "{0,number,0.0‰}", 0.125),
        (
            "format.number.quoted_affix",
            "{0,number,'USD' #,##0.00 'net'}",
            12345.5,
        ),
        (
            "format.number.currency_pattern",
            "{0,number,¤ #,##0.00}",
            12345.5,
        ),
        ("format.number.scientific", "{0,number,0.###E0}", 12345.5),
        ("format.number.nan", "{0,number}", f64::NAN),
        (
            "format.number.positive_infinity",
            "{0,number}",
            f64::INFINITY,
        ),
        (
            "format.number.negative_infinity",
            "{0,number}",
            f64::NEG_INFINITY,
        ),
        ("format.number.round_half_even", "{0,number,0}", 2.5),
        ("format.number.round_half_even_odd", "{0,number,0}", 3.5),
    ] {
        assert_format(&resolver, &golden, key, &us, pattern, &[double(value)]);
    }
    assert_format(
        &resolver,
        &golden,
        "format.number.long_max",
        &us,
        "{0,number,integer}",
        &[long(i64::MAX)],
    );
    assert_format(
        &resolver,
        &golden,
        "format.number.big_integer",
        &us,
        "{0,number,integer}",
        &[big_integer("123456789012345678901234567890")],
    );
    assert_format(
        &resolver,
        &golden,
        "format.number.big_decimal",
        &us,
        "{0,number,#,##0.0000}",
        &[big_decimal("12345678901234567890.1250")],
    );
    assert_format(
        &resolver,
        &golden,
        "format.choice",
        &us,
        "{0,choice,0#none|1#one|1<{0,number,integer} items}",
        &[integer(3)],
    );
    for (key, pattern, value) in [
        ("format.choice.below_first", "{0,choice,1#one|2#two}", 0.0),
        (
            "format.choice.inclusive",
            "{0,choice,0#zero|1#one|1<more}",
            1.0,
        ),
        (
            "format.choice.exclusive",
            "{0,choice,0#zero|1#one|1<more}",
            1.0001,
        ),
        (
            "format.choice.infinity",
            "{0,choice,0#finite|∞#infinite}",
            f64::INFINITY,
        ),
        ("format.choice.quoted_pipe", "{0,choice,0#'a|b'|1#one}", 0.0),
    ] {
        assert_format(&resolver, &golden, key, &us, pattern, &[double(value)]);
    }
    let epoch = date(0);
    for (key, pattern) in [
        ("format.date.short", "{0,date,short}"),
        ("format.date.medium", "{0,date,medium}"),
        ("format.date.long", "{0,date,long}"),
        ("format.date.full", "{0,date,full}"),
        ("format.time.short", "{0,time,short}"),
        ("format.time.medium", "{0,time,medium}"),
        ("format.date.custom", "{0,date,yyyy-MM-dd HH:mm:ss}"),
    ] {
        assert_format(
            &resolver,
            &golden,
            key,
            &us,
            pattern,
            std::slice::from_ref(&epoch),
        );
    }
    for (key, locale, pattern) in [
        (
            "format.date.de_full",
            locale("de-DE", "DE"),
            "{0,date,full}",
        ),
        (
            "format.date.fr_long",
            locale("fr-FR", "FR"),
            "{0,date,long}",
        ),
        (
            "format.date.ja_short",
            locale("ja-JP", "JP"),
            "{0,date,short}",
        ),
        (
            "format.time.us_long",
            locale("en-US", "US"),
            "{0,time,long}",
        ),
        (
            "format.time.de_full",
            locale("de-DE", "DE"),
            "{0,time,full}",
        ),
        (
            "format.date.quoted_custom",
            locale("en-US", "US"),
            "{0,date,yyyy-MM-dd'T'HH:mm:ss XXX}",
        ),
    ] {
        assert_format(&resolver, &golden, key, &locale, pattern, &[date(0)]);
    }
    assert_format_without_parameters(&resolver, &golden, "format.unmatched_close", &us, "bad }");
    assert_format(
        &resolver,
        &golden,
        "format.unmatched_open",
        &us,
        "bad {0",
        &[string("A")],
    );

    let error = resolver
        .format_message(&us, &java("{x}"), Some(&[string("A")]))
        .expect_err("invalid argument number");
    assert_eq!(
        format!("java.lang.IllegalArgumentException|{error}"),
        golden["format.bad_index"]
    );
    let error = resolver
        .format_message(&us, &java("{0,unknown}"), Some(&[string("A")]))
        .expect_err("unknown format type");
    assert_eq!(
        format!("java.lang.IllegalArgumentException|{error}"),
        golden["format.bad_type"]
    );
    for (key, pattern, parameter, java_class) in [
        (
            "format.number.non_number",
            "{0,number}",
            string("A"),
            "java.lang.IllegalArgumentException",
        ),
        (
            "format.date.non_date",
            "{0,date}",
            string("A"),
            "java.lang.IllegalArgumentException",
        ),
        (
            "format.choice.bad",
            "{0,choice,bad}",
            integer(1),
            "java.lang.ArrayIndexOutOfBoundsException",
        ),
    ] {
        let error = resolver
            .format_message(&us, &java(pattern), Some(&[parameter]))
            .expect_err("invalid message format");
        assert_eq!(format!("{java_class}|{error}"), golden[key], "{key}");
    }
}

#[test]
fn template_properties_merge_order_unicode_and_errors_match_java_golden() {
    let golden = golden_records();
    let resolver = StandardMessageResolver::new();
    let resources = Arc::new(HashMap::from([
        (
            "home.properties".to_owned(),
            "base=base\nsame=base\nunicode=你好 😀\n"
                .as_bytes()
                .to_vec(),
        ),
        (
            "home_en.properties".to_owned(),
            b"language=en\nsame=language\n".to_vec(),
        ),
        (
            "home_en_US.properties".to_owned(),
            b"country=US\nsame=country\n".to_vec(),
        ),
        (
            "home_en_US-posix.properties".to_owned(),
            b"variant=posix\nsame=variant\n".to_vec(),
        ),
    ]));
    let requested = Arc::new(Mutex::new(Vec::new()));
    let resource = ProbeResource::root("home", Arc::clone(&resources), Arc::clone(&requested));
    let messages = resolver
        .resolve_messages_for_template(
            &java("template"),
            &resource,
            &JavaLocale::new(java("en-US-posix"), java("US")),
        )
        .expect("template messages");

    assert_eq!(
        requested.lock().expect("requested resources").join(","),
        golden["resource.requested"]
    );
    assert_eq!(messages.len().to_string(), golden["resource.size"]);
    for key in ["base", "language", "country", "variant", "same", "unicode"] {
        let record_key = format!("resource.{key}");
        assert_eq!(
            messages
                .get(&java(key))
                .expect("message key")
                .to_string_lossy(),
            golden[record_key.as_str()],
            "{key}"
        );
    }

    for (record, base_name) in [
        ("resource.null_base_size", None),
        ("resource.empty_base_size", Some("")),
    ] {
        let no_base = ProbeResource {
            base_name: base_name.map(str::to_owned),
            resources: Arc::clone(&resources),
            requested: Arc::new(Mutex::new(Vec::new())),
            selected: None,
        };
        let messages = resolver
            .resolve_messages_for_template(
                &java("template"),
                &no_base,
                &JavaLocale::new(java(""), java("US")),
            )
            .expect("base name short-circuits locale validation");
        assert_eq!(messages.len().to_string(), golden[record]);
    }

    let error = resolver
        .resolve_messages_for_template(
            &java("template"),
            &resource,
            &JavaLocale::new(java(""), java("US")),
        )
        .expect_err("locale without language");
    assert_eq!(
        format!(
            "org.thymeleaf.exceptions.TemplateProcessingException|{}",
            error
        ),
        golden["resource.locale_without_language"]
    );

    let variant_requested = Arc::new(Mutex::new(Vec::new()));
    let variant_resource = ProbeResource::root(
        "variant",
        Arc::new(HashMap::new()),
        Arc::clone(&variant_requested),
    );
    resolver
        .resolve_messages_for_template(
            &java("template"),
            &variant_resource,
            &JavaLocale::new(java("en-posix"), java("")),
        )
        .expect("variant without country");
    assert_eq!(
        variant_requested
            .lock()
            .expect("variant requests")
            .join(","),
        golden["resource.variant_without_country_requested"]
    );

    let syntax = ProbeResource::root(
        "syntax",
        Arc::new(HashMap::from([(
            "syntax.properties".to_owned(),
            concat!(
                "# comment\n",
                "! comment\n",
                "space key : spaced value  \n",
                "escaped\\ key\\:\\==escaped\\ value\\:\\=\n",
                "continued=first\\\n    second\\\n\tthird\n",
                "controls=tab\\tline\\nreturn\\rform\\fslash\\\\\n",
                "unicodeEscape=\\u4f60\\u597d\n",
                "duplicate=first\n",
                "duplicate=second\n",
                "emptyValue\n",
                "=emptyKey\n"
            )
            .as_bytes()
            .to_vec(),
        )])),
        Arc::new(Mutex::new(Vec::new())),
    );
    let syntax_messages = resolver
        .resolve_messages_for_template(&java("template"), &syntax, &locale("en-US", "US"))
        .expect("properties syntax");
    assert_eq!(
        syntax_messages.len().to_string(),
        golden["resource.syntax.size"]
    );
    for (record, key) in [
        ("resource.syntax.space", "space"),
        ("resource.syntax.escaped_key", "escaped key:="),
        ("resource.syntax.continued", "continued"),
        ("resource.syntax.unicode_escape", "unicodeEscape"),
        ("resource.syntax.duplicate", "duplicate"),
        ("resource.syntax.empty_value", "emptyValue"),
        ("resource.syntax.empty_key", ""),
    ] {
        assert_eq!(
            syntax_messages
                .get(&java(key))
                .expect("syntax key")
                .to_string_lossy(),
            golden[record],
            "{record}"
        );
    }
    assert_eq!(
        utf16_hex(
            syntax_messages
                .get(&java("controls"))
                .expect("controls syntax key")
        ),
        golden["resource.syntax.controls_hex"]
    );

    let malformed = ProbeResource::root(
        "bad",
        Arc::new(HashMap::from([(
            "bad.properties".to_owned(),
            b"bad=\\u12G4\n".to_vec(),
        )])),
        Arc::new(Mutex::new(Vec::new())),
    );
    let error = resolver
        .resolve_messages_for_template(&java("template"), &malformed, &locale("en-US", "US"))
        .expect_err("malformed unicode");
    assert_eq!(
        format!("org.thymeleaf.exceptions.TemplateInputException|{error}"),
        golden["resource.malformed_unicode"]
    );
}

#[test]
fn standard_resolver_defaults_absent_and_validation_match_java_golden() {
    let golden = golden_records();
    let resolver = Arc::new(StandardMessageResolver::new());
    resolver.set_default_messages(Some(&HashMap::from([
        (java("first"), java("one")),
        (java("same"), java("old")),
    ])));
    resolver.set_default_messages(Some(&HashMap::from([
        (java("second"), java("two")),
        (java("same"), java("new")),
    ])));
    resolver.set_default_messages(None);
    assert_eq!(
        resolver
            .get_default_messages()
            .read()
            .expect("default messages")
            .len()
            .to_string(),
        golden["defaults.size"]
    );
    for key in ["first", "second", "same"] {
        let record_key = format!("defaults.{key}");
        assert_eq!(
            resolver
                .get_default_messages()
                .read()
                .expect("default messages")
                .get(&java(key))
                .expect("default key")
                .to_string_lossy(),
            golden[record_key.as_str()]
        );
    }
    let error = resolver
        .add_default_message_nullable(None, Some(java("v")))
        .expect_err("null key");
    assert_eq!(
        format!("java.lang.IllegalArgumentException|{error}"),
        golden["defaults.key_null"]
    );
    let error = resolver
        .add_default_message_nullable(Some(java("k")), None)
        .expect_err("null value");
    assert_eq!(
        format!("java.lang.IllegalArgumentException|{error}"),
        golden["defaults.value_null"]
    );

    resolver
        .add_default_message(java("plain"), java("unchanged"))
        .expect("plain default");
    resolver
        .add_default_message(java("indexed"), java("Hello {0}"))
        .expect("indexed default");
    let engine = TemplateEngine::new();
    engine
        .set_message_resolver(Arc::clone(&resolver) as Arc<dyn IMessageResolver>)
        .expect("message resolver before initialization");
    let context = Context::with_locale(Some(locale("en-US", "US")));
    let rendered = engine
        .process_template(
            "<span th:text=\"#{plain}\">x</span>|<span th:text=\"#{indexed('Rust')}\">x</span>|<span th:text=\"#{missing}\">x</span>",
            &context,
        )
        .expect("message rendering")
        .to_string_lossy();
    assert_eq!(
        rendered,
        format!(
            "<span>{}</span>|<span>{}</span>|<span>{}</span>",
            golden["resolve.default.plain"],
            golden["resolve.default.indexed"],
            golden["absent.en_US"]
        )
    );
    let error = resolver
        .resolve_message_nullable(None, None, Some(&java("plain")), None)
        .expect_err("null context");
    assert_eq!(
        format!("java.lang.IllegalArgumentException|{error}"),
        golden["resolve.context_null"]
    );
    let error = resolver
        .create_absent_message_representation_nullable(None, None, None, None)
        .expect_err("null key is validated before context");
    assert_eq!(
        format!("java.lang.IllegalArgumentException|{error}"),
        golden["absent.key_null"]
    );
    let error = resolver
        .create_absent_message_representation_nullable(None, None, Some(&java("missing")), None)
        .expect_err("null context");
    assert_eq!(
        format!("java.lang.NullPointerException|{error}"),
        golden["absent.context_null"]
    );
}

#[test]
fn standard_resolver_composition_hooks_match_java_subclass_extension_points() {
    let golden = golden_records();
    let origin_calls = Arc::new(AtomicUsize::new(0));
    let format_calls = Arc::new(AtomicUsize::new(0));
    let absent_calls = Arc::new(AtomicUsize::new(0));
    let origin_probe = Arc::clone(&origin_calls);
    let format_probe = Arc::clone(&format_calls);
    let absent_probe = Arc::clone(&absent_calls);
    let resolver = Arc::new(
        StandardMessageResolver::new()
            .with_origin_messages_hook(move |_origin, _locale| {
                origin_probe.fetch_add(1, Ordering::SeqCst);
                HashMap::from([(java("origin"), java("origin-{0}"))])
            })
            .with_message_formatter_hook(move |locale, message, parameters| {
                format_probe.fetch_add(1, Ordering::SeqCst);
                let default = StandardMessageResolver::new()
                    .format_message(locale, message, parameters)?
                    .expect("default formatter result");
                Ok(Some(JavaString::from_utf16(
                    [
                        vec![u16::from(b'[')],
                        default.as_utf16().to_vec(),
                        vec![u16::from(b']')],
                    ]
                    .concat(),
                )))
            })
            .with_absent_message_hook(move |_context, _origin, key, _parameters| {
                absent_probe.fetch_add(1, Ordering::SeqCst);
                let key = key.expect("Java hook fixture receives key");
                Ok(Some(JavaString::from_rust_str(&format!(
                    "ABSENT:{}",
                    key.to_string_lossy()
                ))))
            }),
    );
    let engine = TemplateEngine::new();
    let engine_context = EngineContext::new(
        engine.get_configuration().expect("engine configuration"),
        TemplateData::new(None, None, None, None, None),
        None,
        locale("en-US", "US"),
        None,
    );
    let rendered = resolver
        .resolve_message(
            engine_context.as_ref(),
            Some(TypeId::of::<StandardMessageResolver>()),
            &java("origin"),
            Some(&[string("p")]),
        )
        .expect("origin hook")
        .expect("origin hook value")
        .to_string_lossy();
    assert_eq!(rendered, golden["hook.origin.value"]);
    let absent = resolver
        .create_absent_message_representation(
            engine_context.as_ref(),
            Some(TypeId::of::<StandardMessageResolver>()),
            &java("missing"),
            None,
        )
        .expect("absent hook")
        .expect("absent hook value");
    assert_eq!(absent.to_string_lossy(), golden["hook.absent.value"]);
    assert_eq!(
        origin_calls.load(Ordering::SeqCst).to_string(),
        golden["hook.origin.calls"]
    );
    assert_eq!(
        format_calls.load(Ordering::SeqCst).to_string(),
        golden["hook.format.calls"]
    );
    assert_eq!(
        absent_calls.load(Ordering::SeqCst).to_string(),
        golden["hook.absent.calls"]
    );
}

#[test]
fn standard_resolver_phase_order_and_template_cache_policy_follow_java_contract() {
    let load_calls = Arc::new(AtomicUsize::new(0));
    let load_probe = Arc::clone(&load_calls);
    let resolver = StandardMessageResolver::new().with_template_messages_hook(
        move |_template, _resource, _locale| {
            load_probe.fetch_add(1, Ordering::SeqCst);
            Ok(HashMap::from([(java("key"), java("template"))]))
        },
    );
    resolver
        .add_default_message(java("key"), java("default"))
        .expect("default message");

    let engine = TemplateEngine::new();
    let configuration = engine.get_configuration().expect("engine configuration");
    let resource: Arc<dyn ITemplateResource> = Arc::new(ProbeResource::root(
        "phase",
        Arc::new(HashMap::new()),
        Arc::new(Mutex::new(Vec::new())),
    ));
    let cacheable: Arc<dyn ICacheEntryValidity> = Arc::new(AlwaysValidCacheEntryValidity::new());
    let cacheable_data = TemplateData::new(
        Some(java("cacheable")),
        None,
        Some(Arc::clone(&resource)),
        None,
        Some(cacheable),
    );
    let context = EngineContext::new(
        Arc::clone(&configuration),
        cacheable_data,
        None,
        locale("en-US", "US"),
        None,
    );

    for _ in 0..2 {
        let resolved = resolver
            .resolve_message_with_phases(
                context.as_ref(),
                None,
                &java("key"),
                None,
                true,
                false,
                true,
            )
            .expect("cacheable template resolution")
            .expect("template message");
        assert_eq!(resolved, java("template"));
    }
    assert_eq!(
        load_calls.load(Ordering::SeqCst),
        1,
        "Java 仅缓存 cacheable 模板的 template + Locale 结果"
    );

    let default_only = resolver
        .resolve_message_with_phases(
            context.as_ref(),
            None,
            &java("key"),
            None,
            false,
            false,
            true,
        )
        .expect("default-only phase")
        .expect("default message");
    assert_eq!(default_only, java("default"));

    let non_cacheable: Arc<dyn ICacheEntryValidity> =
        Arc::new(NonCacheableCacheEntryValidity::new());
    context.set_template_data(Arc::new(TemplateData::new(
        Some(java("non-cacheable")),
        None,
        Some(resource),
        None,
        Some(non_cacheable),
    )));
    for _ in 0..2 {
        let resolved = resolver
            .resolve_message_with_phases(
                context.as_ref(),
                None,
                &java("key"),
                None,
                true,
                false,
                true,
            )
            .expect("non-cacheable template resolution")
            .expect("template message");
        assert_eq!(resolved, java("template"));
    }
    assert_eq!(
        load_calls.load(Ordering::SeqCst),
        3,
        "Java 对 non-cacheable 模板每次重新加载消息"
    );
}

fn assert_format(
    resolver: &StandardMessageResolver,
    golden: &HashMap<&str, &str>,
    key: &str,
    locale: &JavaLocale,
    pattern: &str,
    parameters: &[Option<Arc<TemplateValue>>],
) {
    let actual = resolver
        .format_message(locale, &java(pattern), Some(parameters))
        .expect("format message")
        .expect("formatter result");
    assert_eq!(actual.to_string_lossy(), golden[key], "{key}");
}

fn assert_format_without_parameters(
    resolver: &StandardMessageResolver,
    golden: &HashMap<&str, &str>,
    key: &str,
    locale: &JavaLocale,
    pattern: &str,
) {
    let actual = resolver
        .format_message(locale, &java(pattern), None)
        .expect("format message")
        .expect("formatter result");
    assert_eq!(actual.to_string_lossy(), golden[key], "{key}");
}

fn golden_records() -> HashMap<&'static str, &'static str> {
    JAVA_GOLDEN
        .lines()
        .map(|line| line.split_once('=').expect("golden key/value"))
        .collect()
}

fn locale(tag: &str, country: &str) -> JavaLocale {
    JavaLocale::new(java(tag), java(country))
}

fn java(value: &str) -> JavaString {
    JavaString::from_rust_str(value)
}

fn string(value: &str) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::string(java(value))))
}

fn integer(value: i32) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(value))))
}

fn double(value: f64) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::Number(JavaNumber::Double(value))))
}

fn long(value: i64) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::Number(JavaNumber::Long(value))))
}

fn big_integer(value: &str) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::Number(JavaNumber::BigInteger(
        value.parse::<BigInt>().expect("valid big integer"),
    ))))
}

fn big_decimal(value: &str) -> Option<Arc<TemplateValue>> {
    Some(Arc::new(TemplateValue::Number(JavaNumber::BigDecimal(
        JavaBigDecimal::parse(value).expect("valid big decimal"),
    ))))
}

fn date(epoch_millis: i64) -> Option<Arc<TemplateValue>> {
    let instant = chrono::DateTime::from_timestamp_millis(epoch_millis).expect("valid epoch");
    Some(Arc::new(TemplateValue::Object(Arc::new(JavaDate::date(
        instant,
    )))))
}

fn utf16_hex(value: &JavaString) -> String {
    value
        .as_utf16()
        .iter()
        .map(|unit| format!("{unit:04X}"))
        .collect::<Vec<_>>()
        .join(",")
}

struct ProbeResource {
    base_name: Option<String>,
    resources: Arc<HashMap<String, Vec<u8>>>,
    requested: Arc<Mutex<Vec<String>>>,
    selected: Option<String>,
}

impl ProbeResource {
    fn root(
        base_name: &str,
        resources: Arc<HashMap<String, Vec<u8>>>,
        requested: Arc<Mutex<Vec<String>>>,
    ) -> Self {
        Self {
            base_name: Some(base_name.to_owned()),
            resources,
            requested,
            selected: None,
        }
    }
}

impl ITemplateResource for ProbeResource {
    fn get_description(&self) -> String {
        self.selected
            .clone()
            .or_else(|| self.base_name.clone())
            .unwrap_or_default()
    }

    fn get_base_name(&self) -> Option<String> {
        self.base_name.clone()
    }

    fn exists(&self) -> bool {
        self.selected
            .as_ref()
            .is_none_or(|selected| self.resources.contains_key(selected))
    }

    fn reader(&self) -> io::Result<Box<dyn Read>> {
        let selected = self.selected.as_deref().unwrap_or("");
        let bytes = self
            .resources
            .get(selected)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, selected.to_owned()))?;
        Ok(Box::new(Cursor::new(bytes)))
    }

    fn relative(
        &self,
        relative_location: Option<&str>,
    ) -> Result<Box<dyn ITemplateResource>, TemplateResourceError> {
        let relative_location = relative_location.ok_or_else(|| {
            TemplateResourceError::InvalidArgument("relative location cannot be null".to_owned())
        })?;
        self.requested
            .lock()
            .expect("requested resources")
            .push(relative_location.to_owned());
        Ok(Box::new(Self {
            base_name: self.base_name.clone(),
            resources: Arc::clone(&self.resources),
            requested: Arc::clone(&self.requested),
            selected: Some(relative_location.to_owned()),
        }))
    }
}
