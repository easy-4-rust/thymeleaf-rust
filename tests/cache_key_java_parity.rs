//! 表达式与模板缓存键的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::collections::{BTreeSet, HashMap, hash_map::DefaultHasher};
use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use thymeleaf::cache::{
    ExpressionCacheKey, ExpressionCacheKeyError, TemplateCacheKey, TemplateCacheKeyError,
};
use thymeleaf::{
    TemplateMode, TemplateResolutionAttributeValue, TemplateResolutionAttributes,
    TemplateSelectorSet,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/cache_key_golden.txt");

struct FailingWriter {
    remaining_writes: usize,
}

impl Write for FailingWriter {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        if self.remaining_writes == 0 {
            return Err(std::fmt::Error);
        }
        self.remaining_writes -= 1;
        let _ = value;
        Ok(())
    }
}

#[test]
fn cache_key_objects_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    export_expression_cache_key(&mut output);
    export_template_cache_key(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn export_expression_cache_key(output: &mut String) {
    let basic =
        ExpressionCacheKey::new(Some("EXPRESSION"), Some("😀")).expect("valid expression key");
    emit(output, "expression.basic.type", basic.get_type());
    emit(
        output,
        "expression.basic.expression0",
        basic.get_expression0(),
    );
    emit(
        output,
        "expression.basic.expression1",
        basic.get_expression1().unwrap_or("null"),
    );
    emit(output, "expression.basic.string", &basic.to_string());
    emit(
        output,
        "expression.basic.hash",
        &basic.hash_code().to_string(),
    );

    let complete =
        ExpressionCacheKey::with_expression1(Some("PREPROCESS"), Some("${user}"), Some("*{name}"))
            .expect("valid expression key");
    emit(
        output,
        "expression.complete.expression1",
        complete.get_expression1().unwrap_or("null"),
    );
    emit(output, "expression.complete.string", &complete.to_string());
    emit(
        output,
        "expression.complete.hash",
        &complete.hash_code().to_string(),
    );

    let empty = ExpressionCacheKey::with_expression1(Some(""), Some(""), Some(""))
        .expect("empty strings are legal");
    emit(output, "expression.empty.string", &empty.to_string());
    emit(
        output,
        "expression.empty.hash",
        &empty.hash_code().to_string(),
    );

    emit_expression_failure(
        output,
        "expression.null_type",
        ExpressionCacheKey::new(None, Some("x")),
    );
    emit_expression_failure(
        output,
        "expression.null_expression0",
        ExpressionCacheKey::new(Some("T"), None),
    );

    let same =
        ExpressionCacheKey::with_expression1(Some("PREPROCESS"), Some("${user}"), Some("*{name}"))
            .expect("valid key");
    emit_bool(output, "expression.equals.same", complete == same);
    emit_bool(
        output,
        "expression.hash.same",
        complete.hash_code() == same.hash_code(),
    );
    emit_bool(
        output,
        "expression.equals.type",
        complete
            == ExpressionCacheKey::with_expression1(
                Some("OTHER"),
                Some("${user}"),
                Some("*{name}"),
            )
            .expect("valid key"),
    );
    emit_bool(
        output,
        "expression.equals.expression0",
        complete
            == ExpressionCacheKey::with_expression1(
                Some("PREPROCESS"),
                Some("other"),
                Some("*{name}"),
            )
            .expect("valid key"),
    );
    emit_bool(
        output,
        "expression.equals.expression1",
        complete
            == ExpressionCacheKey::with_expression1(
                Some("PREPROCESS"),
                Some("${user}"),
                Some("other"),
            )
            .expect("valid key"),
    );
    emit_bool(
        output,
        "expression.equals.null_expression1",
        complete
            == ExpressionCacheKey::new(Some("PREPROCESS"), Some("${user}")).expect("valid key"),
    );
    emit_bool(output, "expression.equals.other_type", false);
    emit_bool(output, "expression.equals.null", false);

    let collision0 = ExpressionCacheKey::new(Some("T"), Some("Aa")).expect("valid key");
    let collision1 = ExpressionCacheKey::new(Some("T"), Some("BB")).expect("valid key");
    emit_bool(
        output,
        "expression.collision.hash",
        collision0.hash_code() == collision1.hash_code(),
    );
    emit_bool(
        output,
        "expression.collision.equals",
        collision0 == collision1,
    );
}

fn export_template_cache_key(output: &mut String) {
    emit_template_failure(
        output,
        "template.null_template",
        TemplateCacheKey::new(None, None, None, 0, 0, None, None),
    );

    let plain = TemplateCacheKey::new(None, Some(""), None, i32::MIN, i32::MAX, None, None)
        .expect("valid key");
    emit(
        output,
        "template.plain.owner",
        plain.get_owner_template().unwrap_or("null"),
    );
    emit(output, "template.plain.template", plain.get_template());
    emit(
        output,
        "template.plain.selectors",
        if plain.get_template_selectors().is_some() {
            "<some>"
        } else {
            "null"
        },
    );
    emit(
        output,
        "template.plain.line",
        &plain.get_line_offset().to_string(),
    );
    emit(
        output,
        "template.plain.col",
        &plain.get_col_offset().to_string(),
    );
    emit(
        output,
        "template.plain.mode",
        plain
            .get_template_mode()
            .map(|mode| mode.to_string())
            .as_deref()
            .unwrap_or("null"),
    );
    emit(
        output,
        "template.plain.attributes",
        if plain.get_template_resolution_attributes().is_some() {
            "<some>"
        } else {
            "null"
        },
    );
    emit(output, "template.plain.string", &plain.to_string());

    let empty_selectors = Arc::new(BTreeSet::new());
    let empty_attributes = Arc::new(HashMap::new());
    let empty_collections = TemplateCacheKey::new(
        None,
        Some(""),
        Some(Arc::clone(&empty_selectors)),
        0,
        0,
        None,
        Some(Arc::clone(&empty_attributes)),
    )
    .expect("valid key");
    emit_bool(
        output,
        "template.empty.selectors_identity",
        std::ptr::eq(
            empty_collections
                .get_template_selectors()
                .expect("selectors"),
            empty_selectors.as_ref(),
        ),
    );
    emit_bool(
        output,
        "template.empty.attributes_identity",
        std::ptr::eq(
            empty_collections
                .get_template_resolution_attributes()
                .expect("attributes"),
            empty_attributes.as_ref(),
        ),
    );
    emit(
        output,
        "template.empty.string",
        &empty_collections.to_string(),
    );
    emit_bool(
        output,
        "template.empty.equals_null_collections",
        empty_collections
            == TemplateCacheKey::new(None, Some(""), None, 0, 0, None, None).expect("valid key"),
    );

    let full_selectors = selectors(&[
        Some("footer"),
        Some("article"),
        Some("\u{E000}"),
        Some("\u{10000}"),
    ]);
    let full_attributes = attributes("tenant", "acme");
    let full = TemplateCacheKey::new(
        Some("owner\nname"),
        Some("page\nname"),
        Some(Arc::clone(&full_selectors)),
        -2,
        7,
        Some(TemplateMode::XML),
        Some(Arc::clone(&full_attributes)),
    )
    .expect("valid key");
    for remaining_writes in 0..32 {
        let mut writer = FailingWriter { remaining_writes };
        let _ = write!(&mut writer, "{full}");
    }
    emit(
        output,
        "template.full.owner",
        full.get_owner_template().unwrap_or("null"),
    );
    emit(output, "template.full.template", full.get_template());
    emit_bool(
        output,
        "template.full.selectors_identity",
        std::ptr::eq(
            full.get_template_selectors().expect("selectors"),
            full_selectors.as_ref(),
        ),
    );
    emit(
        output,
        "template.full.line",
        &full.get_line_offset().to_string(),
    );
    emit(
        output,
        "template.full.col",
        &full.get_col_offset().to_string(),
    );
    emit(
        output,
        "template.full.mode",
        &full.get_template_mode().expect("mode").to_string(),
    );
    emit_bool(
        output,
        "template.full.attributes_identity",
        std::ptr::eq(
            full.get_template_resolution_attributes()
                .expect("attributes"),
            full_attributes.as_ref(),
        ),
    );
    emit(output, "template.full.string", &full.to_string());

    let same = TemplateCacheKey::new(
        Some("owner\nname"),
        Some("page\nname"),
        Some(selectors(&[
            Some("\u{10000}"),
            Some("\u{E000}"),
            Some("article"),
            Some("footer"),
        ])),
        -2,
        7,
        Some(TemplateMode::XML),
        Some(attributes("tenant", "acme")),
    )
    .expect("valid key");
    emit_bool(output, "template.equals.same", full == same);
    emit_bool(
        output,
        "template.hash.same",
        rust_hash(&full) == rust_hash(&same),
    );
    emit_bool(
        output,
        "template.equals.line",
        full == template_variant(
            Some("owner\nname"),
            "page\nname",
            Arc::clone(&full_selectors),
            -1,
            7,
            TemplateMode::XML,
            Arc::clone(&full_attributes),
        ),
    );
    emit_bool(
        output,
        "template.equals.col",
        full == template_variant(
            Some("owner\nname"),
            "page\nname",
            Arc::clone(&full_selectors),
            -2,
            8,
            TemplateMode::XML,
            Arc::clone(&full_attributes),
        ),
    );
    emit_bool(
        output,
        "template.equals.owner",
        full == template_variant(
            Some("other"),
            "page\nname",
            Arc::clone(&full_selectors),
            -2,
            7,
            TemplateMode::XML,
            Arc::clone(&full_attributes),
        ),
    );
    emit_bool(
        output,
        "template.equals.template",
        full == template_variant(
            Some("owner\nname"),
            "other",
            Arc::clone(&full_selectors),
            -2,
            7,
            TemplateMode::XML,
            Arc::clone(&full_attributes),
        ),
    );
    emit_bool(
        output,
        "template.equals.selectors",
        full == template_variant(
            Some("owner\nname"),
            "page\nname",
            selectors(&[Some("other")]),
            -2,
            7,
            TemplateMode::XML,
            Arc::clone(&full_attributes),
        ),
    );
    emit_bool(
        output,
        "template.equals.mode",
        full == template_variant(
            Some("owner\nname"),
            "page\nname",
            Arc::clone(&full_selectors),
            -2,
            7,
            TemplateMode::HTML,
            Arc::clone(&full_attributes),
        ),
    );
    emit_bool(
        output,
        "template.equals.attributes",
        full == template_variant(
            Some("owner\nname"),
            "page\nname",
            full_selectors,
            -2,
            7,
            TemplateMode::XML,
            attributes("tenant", "other"),
        ),
    );
    emit_bool(output, "template.equals.other_type", false);
    emit_bool(output, "template.equals.null", false);
}

fn template_variant(
    owner_template: Option<&str>,
    template: &str,
    template_selectors: Arc<TemplateSelectorSet>,
    line_offset: i32,
    col_offset: i32,
    template_mode: TemplateMode,
    template_resolution_attributes: Arc<TemplateResolutionAttributes>,
) -> TemplateCacheKey {
    TemplateCacheKey::new(
        owner_template,
        Some(template),
        Some(template_selectors),
        line_offset,
        col_offset,
        Some(template_mode),
        Some(template_resolution_attributes),
    )
    .expect("valid template variant")
}

fn selectors(values: &[Option<&str>]) -> Arc<TemplateSelectorSet> {
    Arc::new(
        values
            .iter()
            .map(|value| value.map(str::to_owned))
            .collect(),
    )
}

fn attributes(key: &str, value: &str) -> Arc<TemplateResolutionAttributes> {
    Arc::new(HashMap::from([(
        Some(key.to_owned()),
        TemplateResolutionAttributeValue::new(value.to_owned()),
    )]))
}

fn rust_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn emit_expression_failure(
    output: &mut String,
    key: &str,
    result: Result<ExpressionCacheKey, ExpressionCacheKeyError>,
) {
    match result {
        Ok(_) => emit_failure(output, key, "<none>", "<none>"),
        Err(error) => emit_failure(output, key, "IllegalArgumentException", &error.to_string()),
    }
}

fn emit_template_failure(
    output: &mut String,
    key: &str,
    result: Result<TemplateCacheKey, TemplateCacheKeyError>,
) {
    match result {
        Ok(_) => emit_failure(output, key, "<none>", "<none>"),
        Err(error) => emit_failure(output, key, "IllegalArgumentException", &error.to_string()),
    }
}

fn emit_failure(output: &mut String, key: &str, class: &str, message: &str) {
    emit(output, &format!("{key}.class"), class);
    emit(output, &format!("{key}.message"), message);
}

fn emit_bool(output: &mut String, key: &str, value: bool) {
    emit(output, key, &value.to_string());
}

fn emit(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push('=');
    output.push_str(&value.replace('\\', "\\\\").replace('\n', "\\n"));
    output.push('\n');
}
