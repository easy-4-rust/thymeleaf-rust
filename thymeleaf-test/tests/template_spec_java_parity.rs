//! Thymeleaf 3.1.5 `TemplateSpec` 的 Java/Rust Golden 差分测试。

use std::any::Any;
use std::collections::HashMap;

use thymeleaf::{
    TemplateMode, TemplateResolutionAttributeValue, TemplateResolutionAttributes,
    TemplateSelectorSet, TemplateSpec, TemplateSpecError,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/template_spec_golden.txt");

#[test]
fn template_spec_matches_java_golden() {
    assert_eq!(rust_template_spec_golden(), JAVA_GOLDEN);
}

fn rust_template_spec_golden() -> String {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit_constructor_shapes(&mut output);
    emit_content_types(&mut output);
    emit_selector_validation(&mut output);
    emit_attribute_semantics(&mut output);
    emit_equality_semantics(&mut output);
    emit_display_semantics(&mut output);
    output
}

fn emit_constructor_shapes(output: &mut String) {
    emit_constructor_error(
        output,
        "constructor.null_template",
        TemplateSpec::with_template_mode(None, Some(TemplateMode::HTML)),
    );

    let mode = TemplateSpec::with_template_mode(Some("index"), Some(TemplateMode::XML)).unwrap();
    emit_spec(output, "constructor.mode", &mode);

    let content = TemplateSpec::with_output_content_type(Some("index"), Some("text/html")).unwrap();
    emit_spec(output, "constructor.content", &content);

    let attributes = single_attribute("tenant", "acme");
    let resolved =
        TemplateSpec::with_resolution_attributes(Some("index"), Some(&attributes)).unwrap();
    emit_spec(output, "constructor.attributes", &resolved);

    let selectors = selector_set(&[Some("main")]);
    let selected_mode = TemplateSpec::with_selectors_and_template_mode(
        Some("index"),
        Some(&selectors),
        Some(TemplateMode::RAW),
        Some(&attributes),
    )
    .unwrap();
    emit_spec(output, "constructor.selected_mode", &selected_mode);

    let selected_content = TemplateSpec::with_selectors_and_output_content_type(
        Some("index"),
        Some(&selectors),
        Some("text/css"),
        Some(&attributes),
    )
    .unwrap();
    emit_spec(output, "constructor.selected_content", &selected_content);
}

fn emit_content_types(output: &mut String) {
    let content_types = [
        "text/html",
        "application/xhtml+xml",
        "application/xml",
        "text/xml",
        "application/rss+xml",
        "application/atom+xml",
        "application/javascript",
        "application/x-javascript",
        "application/ecmascript",
        "text/javascript",
        "text/ecmascript",
        "application/json",
        "text/css",
        "text/plain",
        "text/event-stream",
        "application/octet-stream",
        "",
        " \t",
        "; TEXT/HTML ;; Charset=UTF-8",
    ];
    for (index, content_type) in content_types.into_iter().enumerate() {
        let spec =
            TemplateSpec::with_output_content_type(Some("index"), Some(content_type)).unwrap();
        emit_spec(output, &format!("content.{index}"), &spec);
    }
    emit_constructor_error(
        output,
        "content.malformed",
        TemplateSpec::with_output_content_type(Some("index"), Some(";;;")),
    );
}

fn emit_selector_validation(output: &mut String) {
    let null_selectors = TemplateSpec::with_selectors_and_template_mode(
        Some("index"),
        None,
        Some(TemplateMode::HTML),
        None,
    )
    .unwrap();
    emit_spec(output, "selectors.null", &null_selectors);

    let empty = TemplateSelectorSet::new();
    let empty_selectors = TemplateSpec::with_selectors_and_template_mode(
        Some("index"),
        Some(&empty),
        Some(TemplateMode::HTML),
        None,
    )
    .unwrap();
    emit_spec(output, "selectors.empty", &empty_selectors);

    let ordered = selector_set(&[Some("footer"), Some("article")]);
    let ordered_spec = TemplateSpec::with_selectors_and_template_mode(
        Some("index"),
        Some(&ordered),
        Some(TemplateMode::HTML),
        None,
    )
    .unwrap();
    emit_spec(output, "selectors.ordered", &ordered_spec);

    let utf16_order = selector_set(&[Some("\u{E000}"), Some("\u{10000}")]);
    let utf16_order_spec = TemplateSpec::with_selectors_and_template_mode(
        Some("index"),
        Some(&utf16_order),
        Some(TemplateMode::HTML),
        None,
    )
    .unwrap();
    emit_spec(output, "selectors.utf16_order", &utf16_order_spec);

    let null_element = selector_set(&[None]);
    emit_constructor_error(
        output,
        "selectors.null_element",
        TemplateSpec::with_selectors_and_template_mode(
            Some("index"),
            Some(&null_element),
            Some(TemplateMode::HTML),
            None,
        ),
    );

    for invalid in ["", " \n"] {
        let invalid_set = selector_set(&[Some(invalid)]);
        emit_constructor_error(
            output,
            &format!("selectors.invalid.{}", invalid.len()),
            TemplateSpec::with_selectors_and_template_mode(
                Some("index"),
                Some(&invalid_set),
                Some(TemplateMode::HTML),
                None,
            ),
        );
    }

    let non_breaking_space = selector_set(&[Some("\u{00A0}")]);
    emit_constructor_error(
        output,
        "selectors.nbsp",
        TemplateSpec::with_selectors_and_template_mode(
            Some("index"),
            Some(&non_breaking_space),
            Some(TemplateMode::HTML),
            None,
        ),
    );

    let em_space = selector_set(&[Some("\u{2003}")]);
    emit_constructor_error(
        output,
        "selectors.em_space",
        TemplateSpec::with_selectors_and_template_mode(
            Some("index"),
            Some(&em_space),
            Some(TemplateMode::HTML),
            None,
        ),
    );
}

fn emit_attribute_semantics(output: &mut String) {
    let empty = TemplateResolutionAttributes::new();
    let empty_spec = TemplateSpec::with_resolution_attributes(Some("index"), Some(&empty)).unwrap();
    emit_spec(output, "attributes.empty", &empty_spec);

    let mut source = HashMap::from([
        (
            Some("tenant".to_owned()),
            TemplateResolutionAttributeValue::new("acme".to_owned()),
        ),
        (
            Some("attempt".to_owned()),
            TemplateResolutionAttributeValue::new(3_i32),
        ),
        (None, TemplateResolutionAttributeValue::null()),
    ]);
    let copied = TemplateSpec::with_resolution_attributes(Some("index"), Some(&source)).unwrap();
    source.clear();
    let copied_attributes = copied.get_template_resolution_attributes().unwrap();
    emit(
        output,
        "attributes.copied.size",
        &copied_attributes.len().to_string(),
    );
    emit(
        output,
        "attributes.copied.tenant",
        &copied_attributes
            .get(&Some("tenant".to_owned()))
            .unwrap()
            .to_string(),
    );
    emit(
        output,
        "attributes.copied.attempt",
        &copied_attributes
            .get(&Some("attempt".to_owned()))
            .unwrap()
            .to_string(),
    );
    emit(
        output,
        "attributes.copied.null_key",
        &copied_attributes.contains_key(&None).to_string(),
    );
    emit(
        output,
        "attributes.copied.null_value",
        &copied_attributes.get(&None).unwrap().to_string(),
    );
    // Rust 通过只暴露共享引用在编译期提供比 Java 包装映射更强的不可修改保证。
    emit(
        output,
        "attributes.unmodifiable",
        "UnsupportedOperationException:null",
    );
}

fn emit_equality_semantics(output: &mut String) {
    let no_content = TemplateSpec::with_template_mode(Some("index"), None).unwrap();
    let same_no_content = TemplateSpec::with_template_mode(Some("index"), None).unwrap();
    emit(
        output,
        "equals.identity_without_content",
        &no_content
            .equals_java(Some(&no_content))
            .unwrap()
            .to_string(),
    );
    emit(
        output,
        "equals.null",
        &no_content.equals_java(None).unwrap().to_string(),
    );
    emit(
        output,
        "equals.other_type",
        &no_content
            .equals_java(Some(&"index" as &dyn Any))
            .unwrap()
            .to_string(),
    );
    emit_equals_result(
        output,
        "equals.null_content_bug",
        no_content.equals_java(Some(&same_no_content)),
    );

    let base = TemplateSpec::with_output_content_type(Some("index"), Some("text/html")).unwrap();
    let same = TemplateSpec::with_output_content_type(Some("index"), Some("text/html")).unwrap();
    emit(
        output,
        "equals.same",
        &base.equals_java(Some(&same)).unwrap().to_string(),
    );
    emit(output, "equals.same_hash", &(base == same).to_string());
    let different_template =
        TemplateSpec::with_output_content_type(Some("other"), Some("text/html")).unwrap();
    emit(
        output,
        "equals.template",
        &base
            .equals_java(Some(&different_template))
            .unwrap()
            .to_string(),
    );

    let selectors = selector_set(&[Some("main")]);
    let different_selectors = TemplateSpec::with_selectors_and_output_content_type(
        Some("index"),
        Some(&selectors),
        Some("text/html"),
        None,
    )
    .unwrap();
    emit(
        output,
        "equals.selectors",
        &base
            .equals_java(Some(&different_selectors))
            .unwrap()
            .to_string(),
    );
    let different_mode =
        TemplateSpec::with_template_mode(Some("index"), Some(TemplateMode::XML)).unwrap();
    emit(
        output,
        "equals.mode",
        &no_content
            .equals_java(Some(&different_mode))
            .unwrap()
            .to_string(),
    );
    let different_content =
        TemplateSpec::with_output_content_type(Some("index"), Some("application/xhtml+xml"))
            .unwrap();
    emit(
        output,
        "equals.content",
        &base
            .equals_java(Some(&different_content))
            .unwrap()
            .to_string(),
    );
    let attributes = single_attribute("tenant", "acme");
    let different_attributes = TemplateSpec::with_selectors_and_output_content_type(
        Some("index"),
        None,
        Some("text/html"),
        Some(&attributes),
    )
    .unwrap();
    emit(
        output,
        "equals.attributes",
        &base
            .equals_java(Some(&different_attributes))
            .unwrap()
            .to_string(),
    );
}

fn emit_display_semantics(output: &mut String) {
    let selectors = selector_set(&[Some("footer"), Some("article")]);
    let attributes = single_attribute("tenant", "acme");
    let complete = TemplateSpec::with_selectors_and_output_content_type(
        Some("home\npage"),
        Some(&selectors),
        Some("text/html;charset=UTF-8"),
        Some(&attributes),
    )
    .unwrap();
    emit(output, "display.complete", &complete.to_string());

    let short = "x".repeat(120);
    let short_spec = TemplateSpec::with_template_mode(Some(&short), None).unwrap();
    emit(output, "display.short", &short_spec.to_string());

    let long_name = format!("{}\n{}z", "a".repeat(34), "b".repeat(90));
    let long_spec = TemplateSpec::with_template_mode(Some(&long_name), None).unwrap();
    emit(output, "display.long", &long_spec.to_string());
}

fn emit_spec(output: &mut String, key: &str, spec: &TemplateSpec) {
    emit(output, &format!("{key}.template"), spec.get_template());
    emit(
        output,
        &format!("{key}.has_selectors"),
        &spec.has_template_selectors().to_string(),
    );
    emit(
        output,
        &format!("{key}.selectors"),
        &format_selectors(spec.get_template_selectors()),
    );
    emit(
        output,
        &format!("{key}.has_mode"),
        &spec.has_template_mode().to_string(),
    );
    emit(
        output,
        &format!("{key}.mode"),
        &spec
            .get_template_mode()
            .map_or_else(|| "null".to_owned(), |mode| mode.to_string()),
    );
    emit(
        output,
        &format!("{key}.has_attributes"),
        &spec.has_template_resolution_attributes().to_string(),
    );
    emit(
        output,
        &format!("{key}.attributes"),
        &format_attributes(spec.get_template_resolution_attributes()),
    );
    emit(
        output,
        &format!("{key}.content_type"),
        spec.get_output_content_type().unwrap_or("null"),
    );
    emit(
        output,
        &format!("{key}.sse"),
        &spec.is_output_sse().to_string(),
    );
}

fn emit_constructor_error(
    output: &mut String,
    key: &str,
    result: Result<TemplateSpec, TemplateSpecError>,
) {
    match result {
        Ok(_) => emit(output, key, "NO_EXCEPTION"),
        Err(error) => {
            let class = match error {
                TemplateSpecError::TemplateCannotBeNull
                | TemplateSpecError::ModeAndContentTypeConflict
                | TemplateSpecError::NullOrEmptyTemplateSelector => "IllegalArgumentException",
                TemplateSpecError::MalformedOutputContentType => "ArrayIndexOutOfBoundsException",
                TemplateSpecError::EqualsNullOutputContentType => "NullPointerException",
            };
            emit(output, key, &format!("{class}:{error}"));
        }
    }
}

fn emit_equals_result(output: &mut String, key: &str, result: Result<bool, TemplateSpecError>) {
    match result {
        Ok(value) => emit(output, key, &value.to_string()),
        Err(error) => emit(output, key, &format!("NullPointerException:{error}")),
    }
}

fn selector_set(values: &[Option<&str>]) -> TemplateSelectorSet {
    values
        .iter()
        .map(|value| value.map(str::to_owned))
        .collect()
}

fn single_attribute(key: &str, value: &str) -> TemplateResolutionAttributes {
    HashMap::from([(
        Some(key.to_owned()),
        TemplateResolutionAttributeValue::new(value.to_owned()),
    )])
}

fn format_selectors(selectors: Option<&[String]>) -> String {
    let Some(selectors) = selectors else {
        return "null".to_owned();
    };
    format!("[{}]", selectors.join(", "))
}

fn format_attributes(attributes: Option<&TemplateResolutionAttributes>) -> String {
    let Some(attributes) = attributes else {
        return "null".to_owned();
    };
    let mut entries = attributes
        .iter()
        .map(|(key, value)| format!("{}={value}", key.as_deref().unwrap_or("null"),))
        .collect::<Vec<_>>();
    entries.sort();
    format!("{{{}}}", entries.join(", "))
}

fn emit(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push('=');
    output.push_str(
        &value
            .replace('\\', "\\\\")
            .replace('\t', "\\t")
            .replace('\r', "\\r")
            .replace('\n', "\\n"),
    );
    output.push('\n');
}
