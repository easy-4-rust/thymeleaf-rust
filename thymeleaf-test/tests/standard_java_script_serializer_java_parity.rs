//! `StandardJavaScriptSerializer`/`StandardJavaScriptInliner` 族 Java 1:1 差分。
//!
//! 转写上游 `thymeleaf-tests-core`：
//!
//! 1. `org.thymeleaf.standard.serializer.StandardJavaScriptSerializerTest`
//!    —— 全部 12 个用例：enum/字符串/record × `StandardJavaScriptSerializer(false|true)`
//!    （Rust 无 JVM 反射，Java enum 以 `TemplateValue::String` 呈现、record 以
//!    `TemplateValue::Map` 呈现，可观察输出与 Java 一致）；
//! 2. `org.thymeleaf.inline.ScriptInlineTest` —— 引擎级
//!    `th:inline="javascript"` 内联输出：字符串、JavaBean 可见属性
//!    （`SomeObjectA` 仅 getter 可见 ↔ `SomeObjectB` 全字段可见）、数组、集合。
//!
//! 排除项（如实记录，不伪称 MATCH）：`ScriptInlineTest#testDateInline` 的
//! calendar/date 两个断言依赖 JVM 默认时区（Rust `DateUtils` 无 TZ 环境变量时
//! 默认 UTC，`date_utils_java_parity.rs` 已对显式时区做差分），故不转写。

use std::any::Any;
use std::io;
use std::sync::{Arc, Mutex};

use thymeleaf::expression::{TemplateObject, TemplateValue};
use thymeleaf::serializer::{IStandardJavaScriptSerializer, StandardJavaScriptSerializer};
use thymeleaf::util::{DateUtils, NumberValue, TemplateWriter, Utf16String};
use thymeleaf::{ITemplateResolver, TemplateEngine};

// ===========================================================================
// 辅助
// ===========================================================================

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn string_value(value: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::string(js(value)))
}

fn integer_value(value: i32) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Number(NumberValue::Integer(value)))
}

/// Java record `PersonRecord(String name, int age)` 的 Rust 等价 Map。
fn person_record(name: &str, age: i32) -> TemplateValue {
    TemplateValue::Map(Arc::new(vec![
        (string_value("name"), string_value(name)),
        (string_value("age"), integer_value(age)),
    ]))
}

/// 捕获 UTF-16 输出的 Writer（对应 Java StringWriter）。
#[derive(Clone)]
struct CapturedWriter {
    buffer: Arc<Mutex<Vec<u16>>>,
}

impl TemplateWriter for CapturedWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        self.buffer
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .extend_from_slice(characters);
        Ok(())
    }
}

/// 对应 Java `IStandardJavaScriptSerializer#serializeValue(Object, Writer)`。
fn serialize_value(use_jackson: bool, value: Option<&TemplateValue>) -> String {
    let serializer = StandardJavaScriptSerializer::new(use_jackson);
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let mut writer = CapturedWriter {
        buffer: buffer.clone(),
    };
    serializer
        .serialize_value(value, &mut writer)
        .expect("serialize value");
    String::from_utf16_lossy(&buffer.lock().unwrap_or_else(|error| error.into_inner()))
}

// ===========================================================================
// ScriptInlineTest 的 SomeObjectA / SomeObjectB
// ===========================================================================

/// 对应 Java `SomeObjectA`：默认 `@JsonAutoDetect`（仅 getter 可见），
/// 只暴露 `one` 一个属性。
struct SomeObjectA;

impl TemplateObject for SomeObjectA {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.inline.ScriptInlineTest$SomeObjectA"
    }
    fn to_utf16_string(&self) -> Utf16String {
        js("SomeObjectA")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn java_serializable_properties(
        &self,
    ) -> Option<Vec<(Utf16String, Option<Arc<TemplateValue>>)>> {
        Some(vec![(js("one"), Some(string_value("value number one")))])
    }
}

/// 对应 Java `SomeObjectB`：`@JsonAutoDetect(fieldVisibility = ANY)`，
/// 四个字段全部可见。
struct SomeObjectB;

impl TemplateObject for SomeObjectB {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.inline.ScriptInlineTest$SomeObjectB"
    }
    fn to_utf16_string(&self) -> Utf16String {
        js("SomeObjectB")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn java_serializable_properties(
        &self,
    ) -> Option<Vec<(Utf16String, Option<Arc<TemplateValue>>)>> {
        Some(vec![
            (js("one"), Some(string_value("value number one"))),
            (js("two"), Some(integer_value(1231))),
            (
                js("three"),
                Some(Arc::new(TemplateValue::Number(NumberValue::Float(1231.12)))),
            ),
            (js("four"), Some(Arc::new(TemplateValue::Boolean(true)))),
        ])
    }
}

/// 对应 Java `ScriptInlineTest#testInlineResult`：完整模板引擎处理 +
/// 与 Java 完全相同的 UTF-16 提取逻辑。
fn script_inline(script: &str, variables: &[(&str, Arc<TemplateValue>)]) -> String {
    let complete_script = format!(
        "<script th:inline=\"javascript\">\n/*<![CDATA[ */\n{script}\n/* ]]> */\n</script>"
    );
    let engine = TemplateEngine::new();
    let resolver = thymeleaf::templateresolver::StringTemplateResolver::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("set resolver");
    let context = thymeleaf::context::Context::new();
    for (name, value) in variables {
        context.set_variable(Some(js(name)), Some(value.clone()));
    }
    let result = engine
        .process_template(&complete_script, &context)
        .expect("engine process");
    // Java: result.substring(0, result.indexOf("\n/* ]]> */\n</script>")).substring(24)
    let utf16: Vec<u16> = result.as_utf16().to_vec();
    let marker: Vec<u16> = "\n/* ]]> */\n</script>".encode_utf16().collect();
    let index = utf16
        .windows(marker.len())
        .position(|window| window == marker.as_slice())
        .expect("marker must be present");
    String::from_utf16_lossy(&utf16[24..index])
}

// ===========================================================================
// 1. StandardJavaScriptSerializerTest（10 个用例）
// ===========================================================================

const VALUE0: &str = "</script>&#22;";

/// testPrintTestEnumDefaultJS01
#[test]
fn print_test_enum_default_js01() {
    assert_eq!(
        "\"FIRST\"",
        serialize_value(false, Some(&TemplateValue::string(js("FIRST"))))
    );
}

/// testPrintTestEnumJacksonJS01
#[test]
fn print_test_enum_jackson_js01() {
    assert_eq!(
        "\"FIRST\"",
        serialize_value(true, Some(&TemplateValue::string(js("FIRST"))))
    );
}

/// testPrintAnonymousEnumDefaultJS01
#[test]
fn print_anonymous_enum_default_js01() {
    assert_eq!(
        "\"FIRST\"",
        serialize_value(false, Some(&TemplateValue::string(js("FIRST"))))
    );
}

/// testPrintAnonymousEnumJacksonJS01
#[test]
fn print_anonymous_enum_jackson_js01() {
    assert_eq!(
        "\"FIRST\"",
        serialize_value(true, Some(&TemplateValue::string(js("FIRST"))))
    );
}

/// testPrintTestEnumDefaultJS02
#[test]
fn print_test_enum_default_js02() {
    assert_eq!(
        "\"<\\/script>\\u0026#22;\"",
        serialize_value(false, Some(&TemplateValue::string(js(VALUE0))))
    );
}

/// testPrintTestEnumJacksonJS02
#[test]
fn print_test_enum_jackson_js02() {
    assert_eq!(
        "\"<\\/script>\\u0026#22;\"",
        serialize_value(true, Some(&TemplateValue::string(js(VALUE0))))
    );
}

/// testPrintAnonymousEnumDefaultJS02
#[test]
fn print_anonymous_enum_default_js02() {
    assert_eq!(
        "\"<\\/script>\\u0026#22;\"",
        serialize_value(false, Some(&TemplateValue::string(js(VALUE0))))
    );
}

/// testPrintAnonymousEnumJacksonJS02
#[test]
fn print_anonymous_enum_jackson_js02() {
    assert_eq!(
        "\"<\\/script>\\u0026#22;\"",
        serialize_value(true, Some(&TemplateValue::string(js(VALUE0))))
    );
}

/// testPrintRecordDefaultJS01
#[test]
fn print_record_default_js01() {
    assert_eq!(
        "{\"name\":\"Alice\",\"age\":30}",
        serialize_value(false, Some(&person_record("Alice", 30)))
    );
}

/// testPrintRecordJacksonJS01
#[test]
fn print_record_jackson_js01() {
    assert_eq!(
        "{\"name\":\"Alice\",\"age\":30}",
        serialize_value(true, Some(&person_record("Alice", 30)))
    );
}

/// testPrintRecordWithSpecialCharsDefaultJS01
#[test]
fn print_record_with_special_chars_default_js01() {
    assert_eq!(
        "{\"name\":\"<\\/script>\\u0026\",\"age\":0}",
        serialize_value(false, Some(&person_record("</script>&", 0)))
    );
}

/// testPrintRecordWithSpecialCharsJacksonJS01
#[test]
fn print_record_with_special_chars_jackson_js01() {
    assert_eq!(
        "{\"name\":\"<\\/script>\\u0026\",\"age\":0}",
        serialize_value(true, Some(&person_record("</script>&", 0)))
    );
}

// ===========================================================================
// 2. ScriptInlineTest（testDateInline 字符串部分 + 对象/数组/集合）
// ===========================================================================

/// testDateInline：字符串变量（calendar/date 两个断言依赖 JVM 默认时区，
/// 已在文件头如实记录排除）。
#[test]
fn script_inline_date_string_variables() {
    assert_eq!(
        "\"something\"",
        script_inline("[[${a}]]", &[("a", string_value("something"))])
    );
    assert_eq!(
        "   \"something\";",
        script_inline(
            "   /*[[${a}]]*/ 'prototype';",
            &[("a", string_value("something"))]
        )
    );
}

/// testObjectInline：SomeObjectA（仅 getter 可见）与 SomeObjectB（全字段可见）。
#[test]
fn script_inline_object_variables() {
    assert_eq!(
        "   {\"one\":\"value number one\"};",
        script_inline(
            "   /*[[${obj01}]]*/ 'whatever';",
            &[(
                "obj01",
                Arc::new(TemplateValue::Object(Arc::new(SomeObjectA)))
            )],
        )
    );
    assert_eq!(
        "   {\"one\":\"value number one\",\"two\":1231,\"three\":1231.12,\"four\":true};",
        script_inline(
            "   /*[[${obj02}]]*/ 'whatever';",
            &[(
                "obj02",
                Arc::new(TemplateValue::Object(Arc::new(SomeObjectB)))
            )],
        )
    );
}

/// testArrayInline。
#[test]
fn script_inline_array_variable() {
    assert_eq!(
        "   [\"hello\",\"goodbye\"];",
        script_inline(
            "   /*[[${array01}]]*/ 'whatever';",
            &[(
                "array01",
                Arc::new(TemplateValue::List(Arc::new(vec![
                    string_value("hello"),
                    string_value("goodbye"),
                ]))),
            )],
        )
    );
}

/// testCollectionInline。
#[test]
fn script_inline_collection_variable() {
    assert_eq!(
        "   [\"hello\",\"goodbye\"];",
        script_inline(
            "   /*[[${list01}]]*/ 'whatever';",
            &[(
                "list01",
                Arc::new(TemplateValue::List(Arc::new(vec![
                    string_value("hello"),
                    string_value("goodbye"),
                ]))),
            )],
        )
    );
}

// ===========================================================================
// DateValue 序列化偏移形态（Java 21 实测）：JacksonThymeleafISO8601DateFormat
// ===========================================================================

#[test]
fn java_date_serializes_with_colon_offset_never_z() {
    // Java JacksonThymeleafISO8601DateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSSZZZ"
    // + insert(26, ':')：UTC 日期 -> "...+00:00"（非 "Z"）。
    let utc = DateUtils::create(
        Some(2024),
        Some(5),
        Some(17),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some("UTC"),
        None,
    )
    .expect("utc date");
    let value = TemplateValue::Object(Arc::new(utc));
    assert_eq!(
        serialize_value(false, Some(&value)),
        "\"2024-05-17T00:00:00.000+00:00\"",
        "UTC 日期 JS 序列化必须是 +00:00（ZZZ+insert 行为）"
    );
    assert_eq!(
        serialize_value(true, Some(&value)),
        "\"2024-05-17T00:00:00.000+00:00\"",
        "jackson 分支同样 +00:00"
    );

    // 非零固定偏移 -> "+HH:MM"。
    let gmt5 = DateUtils::create(
        Some(2024),
        Some(5),
        Some(17),
        Some(0),
        Some(0),
        Some(0),
        Some(0),
        Some("Etc/GMT+5"),
        None,
    )
    .expect("gmt+5 date");
    let value = TemplateValue::Object(Arc::new(gmt5));
    assert_eq!(
        serialize_value(false, Some(&value)),
        "\"2024-05-17T00:00:00.000-05:00\"",
        "固定偏移 JS 日期带冒号"
    );
}
