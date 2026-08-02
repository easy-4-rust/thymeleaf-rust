//! ClassLoader/WebApplication 模板资源的固定 Java Golden 差分测试。

use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use indexmap::IndexMap;
use thymeleaf::expression::TemplateValue;
use thymeleaf::util::JavaString;
use thymeleaf::web::IWebApplication;
use thymeleaf::{ClassLoaderTemplateResource, ITemplateResource, WebApplicationTemplateResource};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/host_template_resource_golden.txt");

#[test]
fn host_template_resources_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "java_baseline", JAVA_BASELINE);
    export_class_loader_validation(&mut output);
    export_class_loader_resource(&mut output);
    export_web_application_validation(&mut output);
    export_web_application_resource(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn export_class_loader_validation(output: &mut String) {
    for (name, path) in [
        ("null", None),
        ("empty", Some("")),
        ("whitespace", Some("\t \u{3000}")),
    ] {
        let error = ClassLoaderTemplateResource::with_search_roots(Vec::new(), path, None)
            .err()
            .expect("invalid class-loader path");
        emit_failure(
            output,
            &format!("class_loader.path.{name}"),
            "java.lang.IllegalArgumentException",
            &error.to_string(),
        );
    }
}

fn export_class_loader_resource(output: &mut String) {
    let root = temporary_directory("class-loader");
    let templates = root.join("templates");
    fs::create_dir_all(&templates).expect("create class-loader fixture directory");
    fs::write(templates.join("main-latin1.txt"), [b'c', b'a', b'f', 0xE9])
        .expect("write class-loader fixture");

    let resource = ClassLoaderTemplateResource::with_search_roots(
        vec![root.clone()],
        Some("/templates/../templates/main-latin1.txt"),
        Some("ISO-8859-1"),
    )
    .expect("class-loader resource");
    emit(
        output,
        "class_loader.description",
        &resource.get_description(),
    );
    emit_optional(
        output,
        "class_loader.base_name",
        resource.get_base_name().as_deref(),
    );
    emit_bool(output, "class_loader.exists", resource.exists());
    emit(
        output,
        "class_loader.reader",
        &read_all(resource.reader().expect("class-loader reader")),
    );
    let first = resource.reader().expect("first class-loader reader");
    let second = resource.reader().expect("second class-loader reader");
    emit_bool(
        output,
        "class_loader.fresh_readers",
        !std::ptr::eq::<dyn Read>(&*first, &*second),
    );

    let relative = resource
        .relative(Some("child.html"))
        .expect("relative class-loader resource");
    emit(
        output,
        "class_loader.relative.description",
        &relative.get_description(),
    );
    emit_optional(
        output,
        "class_loader.relative.base_name",
        relative.get_base_name().as_deref(),
    );
    emit_bool(output, "class_loader.relative.exists", relative.exists());

    for (name, path) in [
        ("null", None),
        ("empty", Some("")),
        ("whitespace", Some("\t \u{3000}")),
    ] {
        let error = resource
            .relative(path)
            .err()
            .expect("invalid relative class-loader path");
        emit_failure(
            output,
            &format!("class_loader.relative.{name}"),
            "java.lang.IllegalArgumentException",
            &error.to_string(),
        );
    }

    let missing = ClassLoaderTemplateResource::with_search_roots(
        vec![root.clone()],
        Some("templates/missing.txt"),
        Some("bad"),
    )
    .expect("missing class-loader resource object");
    emit_bool(output, "class_loader.missing.exists", missing.exists());
    let error = missing
        .reader()
        .err()
        .expect("missing class-loader reader must fail");
    emit_failure(
        output,
        "class_loader.missing.reader",
        "java.io.FileNotFoundException",
        &error.to_string(),
    );
    fs::remove_dir_all(root).expect("remove class-loader fixture");
}

fn export_web_application_validation(output: &mut String) {
    let error = WebApplicationTemplateResource::new(None, None, None)
        .err()
        .expect("null application is rejected before path");
    emit_failure(
        output,
        "web.validation.order",
        "java.lang.IllegalArgumentException",
        &error.to_string(),
    );

    let application: Arc<dyn IWebApplication> = Arc::new(TestWebApplication::default());
    for (name, path) in [
        ("null", None),
        ("empty", Some("")),
        ("whitespace", Some("\t \u{3000}")),
    ] {
        let error = WebApplicationTemplateResource::new(Some(Arc::clone(&application)), path, None)
            .err()
            .expect("invalid web resource path");
        emit_failure(
            output,
            &format!("web.path.{name}"),
            "java.lang.IllegalArgumentException",
            &error.to_string(),
        );
    }
}

fn export_web_application_resource(output: &mut String) {
    let application = Arc::new(TestWebApplication::default());
    application.insert_resource("/templates/main-latin1.txt", vec![b'c', b'a', b'f', 0xE9]);
    let dynamic_application: Arc<dyn IWebApplication> = application.clone();
    let resource = WebApplicationTemplateResource::new(
        Some(dynamic_application),
        Some("templates/./other/../main-latin1.txt"),
        Some("ISO-8859-1"),
    )
    .expect("web application resource");

    emit(output, "web.description", &resource.get_description());
    emit_optional(output, "web.base_name", resource.get_base_name().as_deref());
    emit_bool(output, "web.exists", resource.exists());
    emit_optional(
        output,
        "web.exists.path",
        application.last_exists_path().as_deref(),
    );
    emit(
        output,
        "web.reader",
        &read_all(resource.reader().expect("web resource reader")),
    );
    emit_optional(
        output,
        "web.reader.path",
        application.last_reader_path().as_deref(),
    );
    let first = resource.reader().expect("first web reader");
    let second = resource.reader().expect("second web reader");
    emit_bool(
        output,
        "web.fresh_readers",
        !std::ptr::eq::<dyn Read>(&*first, &*second),
    );

    let relative = resource
        .relative(Some("../messages.properties"))
        .expect("relative web resource");
    emit(
        output,
        "web.relative.description",
        &relative.get_description(),
    );
    emit_optional(
        output,
        "web.relative.base_name",
        relative.get_base_name().as_deref(),
    );
    emit_bool(output, "web.relative.exists", relative.exists());
    emit_optional(
        output,
        "web.relative.path",
        application.last_exists_path().as_deref(),
    );

    for (name, path) in [
        ("null", None),
        ("empty", Some("")),
        ("whitespace", Some("\t \u{3000}")),
    ] {
        let error = resource
            .relative(path)
            .err()
            .expect("invalid relative web path");
        emit_failure(
            output,
            &format!("web.relative.{name}"),
            "java.lang.IllegalArgumentException",
            &error.to_string(),
        );
    }

    let dynamic_application: Arc<dyn IWebApplication> = application.clone();
    let missing = WebApplicationTemplateResource::new(
        Some(dynamic_application),
        Some("/missing.html"),
        Some("bad"),
    )
    .expect("missing web resource object");
    emit_bool(output, "web.missing.exists", missing.exists());
    let error = missing
        .reader()
        .err()
        .expect("missing web reader must fail");
    emit_failure(
        output,
        "web.missing.reader",
        "java.io.FileNotFoundException",
        &error.to_string(),
    );
}

#[derive(Default)]
struct TestWebApplication {
    resources: Mutex<HashMap<String, Vec<u8>>>,
    last_exists_path: Mutex<Option<String>>,
    last_reader_path: Mutex<Option<String>>,
}

impl TestWebApplication {
    fn insert_resource(&self, path: &str, contents: Vec<u8>) {
        self.resources
            .lock()
            .expect("resource lock")
            .insert(path.to_owned(), contents);
    }

    fn last_exists_path(&self) -> Option<String> {
        self.last_exists_path
            .lock()
            .expect("exists path lock")
            .clone()
    }

    fn last_reader_path(&self) -> Option<String> {
        self.last_reader_path
            .lock()
            .expect("reader path lock")
            .clone()
    }
}

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

    fn resource_exists(&self, path: Option<&JavaString>) -> bool {
        let path = path.map(JavaString::to_string_lossy);
        *self.last_exists_path.lock().expect("exists path lock") = path.clone();
        path.is_some_and(|path| {
            self.resources
                .lock()
                .expect("resource lock")
                .contains_key(&path)
        })
    }

    fn get_resource_as_stream(&self, path: Option<&JavaString>) -> Option<Box<dyn Read + Send>> {
        let path = path.map(JavaString::to_string_lossy);
        *self.last_reader_path.lock().expect("reader path lock") = path.clone();
        path.and_then(|path| {
            self.resources
                .lock()
                .expect("resource lock")
                .get(&path)
                .cloned()
                .map(|contents| Box::new(io::Cursor::new(contents)) as Box<dyn Read + Send>)
        })
    }
}

fn temporary_directory(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "thymeleaf-host-resource-{label}-{}",
        std::process::id()
    ))
}

fn read_all(mut reader: Box<dyn Read>) -> String {
    let mut result = String::new();
    reader
        .read_to_string(&mut result)
        .expect("read UTF-8 output");
    result
}

fn emit(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push('=');
    output.push_str(value);
    output.push('\n');
}

fn emit_optional(output: &mut String, key: &str, value: Option<&str>) {
    emit(output, key, value.unwrap_or("null"));
}

fn emit_bool(output: &mut String, key: &str, value: bool) {
    emit(output, key, if value { "true" } else { "false" });
}

fn emit_failure(output: &mut String, key: &str, class: &str, message: &str) {
    emit(output, &format!("{key}.class"), class);
    emit(output, &format!("{key}.message"), message);
}
