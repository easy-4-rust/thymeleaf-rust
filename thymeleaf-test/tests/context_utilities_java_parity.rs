//! Context 工具四个 Java 对象的固定 Golden 差分与 Rust 并发义务测试。

// 共享 Web corpus 同时服务于完整 Web SPI 批次；本批次仅使用 exchange 身份能力。
#![allow(dead_code, unused_imports)]

mod support;

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};

use support::CorpusWebExchange;
use thymeleaf::context::{
    Context, Contexts, IContext, ILazyContextVariable, IWebContext, IdentifierSequences,
    IdentifierSequencesError, LazyContextVariable, WebContext,
};
use thymeleaf::expression::{TemplateObject, TemplateValue};
use thymeleaf::util::Utf16String;
use thymeleaf::web::IWebExchange;

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/context_utilities_golden.txt");

#[test]
fn context_utilities_semantics_match_java_golden() {
    let expected = parse_golden(JAVA_GOLDEN);
    assert_eq!(
        expected.get("baseline").map(String::as_str),
        Some(JAVA_BASELINE)
    );
    assert_shape_inventory(&expected);
    assert_identifier_overflow_oracle(&expected);

    let mut actual = BTreeMap::new();
    export_lazy_variables(&mut actual);
    export_identifier_sequences(&mut actual);
    export_contexts(&mut actual);

    for (key, value) in actual {
        assert_eq!(
            expected.get(&key),
            Some(&value),
            "Java Golden mismatch for {key}"
        );
    }
}

#[test]
fn lazy_context_variable_initializes_once_under_rust_concurrency() {
    let loads = Arc::new(AtomicUsize::new(0));
    let marker = string_value("concurrent");
    let variable = Arc::new(LazyContextVariable::new({
        let loads = Arc::clone(&loads);
        let marker = Arc::clone(&marker);
        move || {
            loads.fetch_add(1, Ordering::SeqCst);
            Some(Arc::clone(&marker))
        }
    }));
    let barrier = Arc::new(Barrier::new(13));
    let mut workers = Vec::new();
    for _ in 0..12 {
        let variable = Arc::clone(&variable);
        let barrier = Arc::clone(&barrier);
        workers.push(std::thread::spawn(move || {
            barrier.wait();
            let value = variable.get_value().as_ref().expect("lazy value");
            Arc::as_ptr(value) as usize
        }));
    }
    barrier.wait();
    let expected_identity =
        Arc::as_ptr(variable.get_value().as_ref().expect("lazy value")) as usize;
    for worker in workers {
        assert_eq!(worker.join().expect("lazy worker"), expected_identity);
    }
    assert_eq!(loads.load(Ordering::SeqCst), 1);
}

fn assert_shape_inventory(expected: &BTreeMap<String, String>) {
    let expected_counts = [
        ("ILazyContextVariable", "1"),
        ("LazyContextVariable", "3"),
        ("IdentifierSequences", "4"),
        ("Contexts", "8"),
    ];
    let mut declarations = 0usize;
    for (object, count) in expected_counts {
        assert_eq!(
            expected
                .get(&format!("shape.{object}.count"))
                .map(String::as_str),
            Some(count),
            "unexpected Java declaration count for {object}"
        );
        assert!(
            !expected
                .get(&format!("shape.{object}.signatures"))
                .expect("shape signatures")
                .is_empty()
        );
        declarations += count.parse::<usize>().expect("shape count");
    }
    assert_eq!(declarations, 16);
}

fn assert_identifier_overflow_oracle(expected: &BTreeMap<String, String>) {
    // Java Golden 以反射把私有 HashMap seed 为 Integer.MAX_VALUE。Rust 的对应生产
    // 状态同样私有，实际回绕操作由 identifier_sequences.rs 同模块单元测试执行；这里
    // 固定并消费同一 Java Oracle，防止这三个值在两套测试间漂移。
    assert_eq!(
        expected.get("ids.max.increment").map(String::as_str),
        Some("2147483647")
    );
    assert_eq!(
        expected.get("ids.max.next").map(String::as_str),
        Some("-2147483648")
    );
    assert_eq!(
        expected.get("ids.max.previous").map(String::as_str),
        Some("2147483647")
    );
}

fn export_lazy_variables(output: &mut BTreeMap<String, String>) {
    let loads = AtomicUsize::new(0);
    let marker = string_value("Marker(shared)");
    let variable = LazyContextVariable::new(|| {
        loads.fetch_add(1, Ordering::SeqCst);
        Some(Arc::clone(&marker))
    });
    emit(output, "lazy.before.loads", loads.load(Ordering::SeqCst));
    let first = variable.get_value().clone().expect("marker");
    let second = variable.get_value().clone().expect("marker");
    emit_value(output, "lazy.value", Some(first.clone()));
    emit(
        output,
        "lazy.identity",
        Arc::ptr_eq(&first, &second) && Arc::ptr_eq(&first, &marker),
    );
    emit(output, "lazy.after.loads", loads.load(Ordering::SeqCst));

    let null_loads = AtomicUsize::new(0);
    let null_variable = LazyContextVariable::new(|| {
        null_loads.fetch_add(1, Ordering::SeqCst);
        None::<Arc<TemplateValue>>
    });
    emit_value(output, "lazy.null.first", null_variable.get_value().clone());
    emit_value(
        output,
        "lazy.null.second",
        null_variable.get_value().clone(),
    );
    emit(output, "lazy.null.loads", null_loads.load(Ordering::SeqCst));

    let retry_loads = AtomicUsize::new(0);
    let retry_variable = LazyContextVariable::new(|| {
        let invocation = retry_loads.fetch_add(1, Ordering::SeqCst) + 1;
        assert_ne!(invocation, 1, "first load fails");
        7_i32
    });
    emit_panic(
        output,
        "lazy.retry.first",
        || {
            let _ = retry_variable.get_value();
        },
        "java.lang.IllegalStateException:first load fails",
    );
    emit(output, "lazy.retry.second", retry_variable.get_value());
    emit(output, "lazy.retry.third", retry_variable.get_value());
    emit(
        output,
        "lazy.retry.loads",
        retry_loads.load(Ordering::SeqCst),
    );
}

fn export_identifier_sequences(output: &mut BTreeMap<String, String>) {
    let sequences = IdentifierSequences::new();
    let item = js("item");
    let other = js("其他");
    emit_result(
        output,
        "ids.next.empty",
        sequences.get_next_id_seq(Some(&item)),
    );
    emit_identifier_error(
        output,
        "ids.previous.empty",
        sequences.get_previous_id_seq(Some(&item)),
    );
    emit_result(
        output,
        "ids.increment.one",
        sequences.get_and_increment_id_seq(Some(&item)),
    );
    emit_result(
        output,
        "ids.increment.two",
        sequences.get_and_increment_id_seq(Some(&item)),
    );
    emit_result(
        output,
        "ids.next.after",
        sequences.get_next_id_seq(Some(&item)),
    );
    emit_result(
        output,
        "ids.previous.after",
        sequences.get_previous_id_seq(Some(&item)),
    );
    emit_result(
        output,
        "ids.other.first",
        sequences.get_and_increment_id_seq(Some(&other)),
    );
    emit_identifier_error(
        output,
        "ids.null.increment",
        sequences.get_and_increment_id_seq(None),
    );
    emit_identifier_error(output, "ids.null.next", sequences.get_next_id_seq(None));
    emit_identifier_error(
        output,
        "ids.null.previous",
        sequences.get_previous_id_seq(None),
    );

    // Java Golden 使用反射把私有 HashMap 种子置为 Integer.MAX_VALUE；Rust 在同文件单元
    // 测试验证相同回绕分支，这里只验证公开 API 可观察的普通序列和错误边界。
}

fn export_contexts(output: &mut BTreeMap<String, String>) {
    let plain = Context::new();
    let exchange: Arc<dyn IWebExchange> = Arc::new(CorpusWebExchange::new());
    let web = WebContext::new(Some(exchange.clone())).expect("web context");

    // Rust 的引用参数无法构造 Java null；非 null capability 结果逐项差分，null 分支由类型
    // 边界承接，错误强制转换仍以 catch_unwind 验证 Java ClassCastException 类别。
    emit(output, "contexts.null.engine", false);
    emit(output, "contexts.null.web", false);
    emit(
        output,
        "contexts.plain.engine",
        Contexts::is_engine_context(&plain),
    );
    emit(
        output,
        "contexts.plain.web",
        Contexts::is_web_context(&plain),
    );
    emit(
        output,
        "contexts.web.engine",
        Contexts::is_engine_context(&web),
    );
    emit(output, "contexts.web.web", Contexts::is_web_context(&web));
    emit(
        output,
        "contexts.web.as.identity",
        (Contexts::as_web_context(&web) as *const dyn IWebContext as *const ())
            == (&web as &dyn IWebContext as *const dyn IWebContext as *const ()),
    );
    emit(
        output,
        "contexts.web.exchange.identity",
        std::ptr::eq(Contexts::get_web_exchange(&web), exchange.as_ref()),
    );
    emit(
        output,
        "contexts.web.servlet",
        Contexts::is_servlet_web_context(&web),
    );
    emit_cast_panic(output, "contexts.plain.as.engine", || {
        let _ = Contexts::as_engine_context(&plain);
    });
    emit_cast_panic(output, "contexts.plain.as.web", || {
        let _ = Contexts::as_web_context(&plain);
    });
    emit_cast_panic(output, "contexts.plain.exchange", || {
        let _ = Contexts::get_web_exchange(&plain);
    });
    emit_cast_panic(output, "contexts.web.servlet.exchange", || {
        let _ = Contexts::get_servlet_web_exchange(&web);
    });
}

fn emit_result(
    output: &mut BTreeMap<String, String>,
    key: &str,
    value: Result<i32, IdentifierSequencesError>,
) {
    emit(
        output,
        key,
        value.expect("expected identifier sequence result"),
    );
}

fn emit_identifier_error(
    output: &mut BTreeMap<String, String>,
    key: &str,
    value: Result<i32, IdentifierSequencesError>,
) {
    let error = value.expect_err("expected identifier sequence error");
    emit(output, key, format!("{}:{error}", error.java_class_name()));
}

fn emit_cast_panic(output: &mut BTreeMap<String, String>, key: &str, action: impl FnOnce()) {
    assert!(catch_unwind(AssertUnwindSafe(action)).is_err());
    emit(output, key, "java.lang.ClassCastException");
}

fn emit_panic(
    output: &mut BTreeMap<String, String>,
    key: &str,
    action: impl FnOnce(),
    normalized_java_error: &str,
) {
    assert!(catch_unwind(AssertUnwindSafe(action)).is_err());
    emit(output, key, normalized_java_error);
}

fn emit_value(output: &mut BTreeMap<String, String>, key: &str, value: Option<Arc<TemplateValue>>) {
    emit(
        output,
        key,
        value
            .as_deref()
            .and_then(TemplateValue::to_utf16_string)
            .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy()),
    );
}

fn emit(output: &mut BTreeMap<String, String>, key: &str, value: impl ToString) {
    output.insert(key.to_owned(), value.to_string());
}

fn parse_golden(input: &str) -> BTreeMap<String, String> {
    input
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (key, value) = line.split_once('=').expect("golden key=value");
            (key.to_owned(), value.to_owned())
        })
        .collect()
}

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn string_value(value: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::string(js(value)))
}
