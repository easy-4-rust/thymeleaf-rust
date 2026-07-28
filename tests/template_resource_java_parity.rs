//! 模板资源接口与字符串资源的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::io::Read;

use thymeleaf::{ITemplateResource, StringTemplateResource, TemplateResourceError};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const RUST_BASELINE: &str = "b6c97b2df175370c8b6a94feaed0955af67712f9";
const JAVA_GOLDEN: &str = include_str!("fixtures/template_resource_golden.txt");

#[test]
fn template_resource_objects_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "java_baseline", Some(JAVA_BASELINE));
    emit(&mut output, "rust_baseline", Some(RUST_BASELINE));
    export_null_constructor(&mut output);
    export_empty_resource(&mut output);
    export_unicode_resource(&mut output);
    export_relative_failures(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn export_null_constructor(output: &mut String) {
    let error = StringTemplateResource::new(None)
        .err()
        .expect("null resource must fail");
    emit(
        output,
        "string.null.type",
        Some("java.lang.IllegalArgumentException"),
    );
    emit(output, "string.null.message", Some(&error.to_string()));
}

fn export_empty_resource(output: &mut String) {
    let resource = StringTemplateResource::new(Some("")).expect("empty resource is legal");
    let resource: &dyn ITemplateResource = &resource;
    emit(
        output,
        "string.empty.description",
        Some(&resource.get_description()),
    );
    emit(
        output,
        "string.empty.base_name",
        resource.get_base_name().as_deref(),
    );
    emit_bool(output, "string.empty.exists", resource.exists());
    emit(
        output,
        "string.empty.reader",
        Some(&read_all(resource.reader().expect("empty reader"))),
    );
    let first = resource.reader().expect("first reader");
    let second = resource.reader().expect("second reader");
    emit_bool(
        output,
        "string.empty.fresh_readers",
        !std::ptr::eq::<dyn Read>(&*first, &*second),
    );
}

fn export_unicode_resource(output: &mut String) {
    let contents = "<p>你好 😀</p>\r\n\0tail";
    let resource = StringTemplateResource::new(Some(contents)).expect("valid resource");
    let resource: &dyn ITemplateResource = &resource;
    emit(
        output,
        "string.unicode.description",
        Some(&resource.get_description()),
    );
    emit(
        output,
        "string.unicode.base_name",
        resource.get_base_name().as_deref(),
    );
    emit_bool(output, "string.unicode.exists", resource.exists());

    let mut first = resource.reader().expect("first reader");
    let mut prefix = [0_u8; 3];
    let prefix_count = first.read(&mut prefix).expect("read prefix");
    emit(
        output,
        "string.unicode.prefix_count",
        Some(&prefix_count.to_string()),
    );
    emit(
        output,
        "string.unicode.prefix",
        Some(std::str::from_utf8(&prefix).expect("utf-8 prefix")),
    );
    emit(
        output,
        "string.unicode.second_full",
        Some(&read_all(resource.reader().expect("second reader"))),
    );
    emit(
        output,
        "string.unicode.first_remaining",
        Some(&read_all(first)),
    );
}

fn export_relative_failures(output: &mut String) {
    let resource = StringTemplateResource::new(Some("line1\n\"line2\"")).expect("valid resource");
    let resource: &dyn ITemplateResource = &resource;
    for (name, relative_location) in [
        ("null", None),
        ("empty", Some("")),
        ("child", Some("child.html")),
    ] {
        let error = resource
            .relative(relative_location)
            .err()
            .expect("relative string resource must fail");
        let error_type = match &error {
            TemplateResourceError::Input(_) => "org.thymeleaf.exceptions.TemplateInputException",
            TemplateResourceError::InvalidArgument(_) => "java.lang.IllegalArgumentException",
        };
        emit(
            output,
            &format!("string.relative.{name}.type"),
            Some(error_type),
        );
        emit(
            output,
            &format!("string.relative.{name}.message"),
            Some(&error.to_string()),
        );
    }
}

fn read_all(mut reader: Box<dyn Read>) -> String {
    let mut result = String::new();
    reader
        .read_to_string(&mut result)
        .expect("read template resource");
    result
}

fn emit_bool(output: &mut String, key: &str, value: bool) {
    emit(output, key, Some(&value.to_string()));
}

fn emit(output: &mut String, key: &str, value: Option<&str>) {
    output.push_str(key);
    output.push('=');
    output.push_str(&escape(value));
    output.push('\n');
}

fn escape(value: Option<&str>) -> String {
    let Some(value) = value else {
        return "null".to_owned();
    };
    value
        .replace('\\', "\\\\")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\0', "\\0")
}
