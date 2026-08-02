//! `AbstractCacheManager`、`ICacheManager` 与 `StandardCacheManager` 的
//! Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::any::Any;
use std::fmt::Write;
use std::sync::Arc;

use thymeleaf::cache::{ExpressionCacheKey, ICacheManager, StandardCacheManager};
use thymeleaf::util::JavaString;

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/standard_cache_manager_golden.txt");

#[test]
fn standard_cache_manager_matches_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    let defaults = StandardCacheManager::new();
    emit_java_string(
        &mut output,
        "default.template.name",
        defaults.get_template_cache_name(),
    );
    emit(
        &mut output,
        "default.template.initial",
        defaults.get_template_cache_initial_size(),
    );
    emit(
        &mut output,
        "default.template.max",
        defaults.get_template_cache_max_size(),
    );
    emit(
        &mut output,
        "default.template.soft",
        defaults.get_template_cache_use_soft_references(),
    );
    emit_java_string_option(
        &mut output,
        "default.template.logger",
        defaults.get_template_cache_logger_name(),
    );
    emit(
        &mut output,
        "default.template.checker",
        if defaults.get_template_cache_validity_checker().is_some() {
            "org.thymeleaf.cache.StandardParsedTemplateEntryValidator"
        } else {
            "null"
        },
    );
    emit_java_string(
        &mut output,
        "default.expression.name",
        defaults.get_expression_cache_name(),
    );
    emit(
        &mut output,
        "default.expression.initial",
        defaults.get_expression_cache_initial_size(),
    );
    emit(
        &mut output,
        "default.expression.max",
        defaults.get_expression_cache_max_size(),
    );
    emit(
        &mut output,
        "default.expression.soft",
        defaults.get_expression_cache_use_soft_references(),
    );
    emit_java_string_option(
        &mut output,
        "default.expression.logger",
        defaults.get_expression_cache_logger_name(),
    );
    emit(
        &mut output,
        "default.expression.checker",
        if defaults.get_expression_cache_validity_checker().is_some() {
            "present"
        } else {
            "null"
        },
    );
    let specific_names = defaults
        .get_all_specific_cache_names()
        .expect("Java StandardCacheManager returns an empty non-null list");
    emit(
        &mut output,
        "default.specific.names",
        format!(
            "[{}]",
            specific_names
                .iter()
                .map(JavaString::to_string_lossy)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    );
    emit(
        &mut output,
        "default.specific.cache",
        if defaults
            .get_specific_cache::<String, String>(&JavaString::from_rust_str("missing"))
            .is_some()
        {
            "present"
        } else {
            "null"
        },
    );

    let template0 = defaults
        .get_template_cache()
        .expect("default template cache");
    let template1 = defaults
        .get_template_cache()
        .expect("same default template cache");
    emit(
        &mut output,
        "lazy.template.same",
        std::ptr::eq(template0, template1),
    );
    let expression0 = defaults
        .get_expression_cache()
        .expect("default expression cache");
    let expression1 = defaults
        .get_expression_cache()
        .expect("same default expression cache");
    emit(
        &mut output,
        "lazy.expression.same",
        std::ptr::eq(expression0, expression1),
    );

    let mut configured = StandardCacheManager::new();
    configured.set_template_cache_name(JavaString::from_rust_str("T"));
    configured.set_template_cache_initial_size(7);
    configured.set_template_cache_max_size(-2);
    configured.set_template_cache_use_soft_references(false);
    configured.set_template_cache_logger_name(Some(JavaString::from_rust_str("template.logger")));
    configured.set_template_cache_validity_checker(None);
    configured.set_template_cache_enable_counters(true);
    configured.set_expression_cache_name(JavaString::from_rust_str("E"));
    configured.set_expression_cache_initial_size(9);
    configured.set_expression_cache_max_size(11);
    configured.set_expression_cache_use_soft_references(false);
    configured
        .set_expression_cache_logger_name(Some(JavaString::from_rust_str("expression.logger")));
    configured.set_expression_cache_validity_checker(None);
    configured.set_expression_cache_enable_counters(true);
    emit_java_string(
        &mut output,
        "configured.template.name",
        configured.get_template_cache_name(),
    );
    emit(
        &mut output,
        "configured.template.initial",
        configured.get_template_cache_initial_size(),
    );
    emit(
        &mut output,
        "configured.template.max",
        configured.get_template_cache_max_size(),
    );
    emit(
        &mut output,
        "configured.template.soft",
        configured.get_template_cache_use_soft_references(),
    );
    emit_java_string_option(
        &mut output,
        "configured.template.logger",
        configured.get_template_cache_logger_name(),
    );
    emit(
        &mut output,
        "configured.template.checker",
        if configured.get_template_cache_validity_checker().is_some() {
            "present"
        } else {
            "null"
        },
    );
    emit_java_string(
        &mut output,
        "configured.expression.name",
        configured.get_expression_cache_name(),
    );
    emit(
        &mut output,
        "configured.expression.initial",
        configured.get_expression_cache_initial_size(),
    );
    emit(
        &mut output,
        "configured.expression.max",
        configured.get_expression_cache_max_size(),
    );
    emit(
        &mut output,
        "configured.expression.soft",
        configured.get_expression_cache_use_soft_references(),
    );
    emit_java_string_option(
        &mut output,
        "configured.expression.logger",
        configured.get_expression_cache_logger_name(),
    );
    emit(
        &mut output,
        "configured.expression.checker",
        if configured.get_expression_cache_validity_checker().is_some() {
            "present"
        } else {
            "null"
        },
    );

    let mut disabled = StandardCacheManager::new();
    disabled.set_template_cache_max_size(0);
    disabled.set_expression_cache_max_size(0);
    emit(
        &mut output,
        "disabled.template.first",
        option_presence(disabled.get_template_cache()),
    );
    emit(
        &mut output,
        "disabled.expression.first",
        option_presence(disabled.get_expression_cache()),
    );
    disabled.set_template_cache_max_size(1);
    disabled.set_expression_cache_max_size(1);
    emit(
        &mut output,
        "disabled.template.sticky",
        option_presence(disabled.get_template_cache()),
    );
    emit(
        &mut output,
        "disabled.expression.sticky",
        option_presence(disabled.get_expression_cache()),
    );

    let mut mutation = StandardCacheManager::new();
    mutation.set_expression_cache_name(JavaString::from_rust_str("before"));
    let mutation0 = mutation
        .get_expression_cache()
        .expect("expression cache before mutation")
        as *const dyn thymeleaf::cache::ICache<ExpressionCacheKey, dyn Any + Send + Sync>
        as *const () as usize;
    mutation.set_expression_cache_name(JavaString::from_rust_str("after"));
    let mutation1 = mutation
        .get_expression_cache()
        .expect("expression cache after mutation")
        as *const dyn thymeleaf::cache::ICache<ExpressionCacheKey, dyn Any + Send + Sync>
        as *const () as usize;
    emit_java_string(
        &mut output,
        "mutation.getter",
        mutation.get_expression_cache_name(),
    );
    emit(&mut output, "mutation.cache.same", mutation0 == mutation1);

    let key = ExpressionCacheKey::new(Some("type"), Some("expression")).expect("cache key");
    let cache = mutation
        .get_expression_cache()
        .expect("initialized expression cache");
    let value: Arc<dyn Any + Send + Sync> = Arc::new("value".to_owned());
    cache.put(key, value);
    emit(&mut output, "clear.before", cache.key_set().len());
    mutation.clear_all_caches();
    let cache = mutation
        .get_expression_cache()
        .expect("same expression cache after clear");
    emit(&mut output, "clear.after", cache.key_set().len());

    assert_eq!(output, JAVA_GOLDEN);
}

fn option_presence<T: ?Sized>(value: Option<&T>) -> &'static str {
    if value.is_some() { "present" } else { "null" }
}

fn emit_java_string_option(output: &mut String, key: &str, value: Option<&JavaString>) {
    match value {
        Some(value) => emit_java_string(output, key, value),
        None => emit(output, key, "null"),
    }
}

fn emit_java_string(output: &mut String, key: &str, value: &JavaString) {
    emit(output, key, value.to_string_lossy());
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    writeln!(output, "{key}={value}").expect("string output");
}
