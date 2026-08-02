//! `StandardCache` 的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::sync::{Arc, Mutex};

use thymeleaf::cache::{ICache, ICacheEntryValidityChecker, StandardCache, StandardCacheError};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/standard_cache_golden.txt");

struct RecordingChecker {
    valid: bool,
    state: Mutex<Option<(String, String, i64)>>,
}

impl RecordingChecker {
    fn new(valid: bool) -> Self {
        Self {
            valid,
            state: Mutex::new(None),
        }
    }
}

impl ICacheEntryValidityChecker<String, String> for RecordingChecker {
    fn check_is_value_still_valid(
        &self,
        key: &String,
        value: &String,
        entry_creation_timestamp: i64,
    ) -> bool {
        *self.state.lock().expect("checker lock") =
            Some((key.clone(), value.clone(), entry_creation_timestamp));
        self.valid
    }
}

#[test]
fn standard_cache_matches_java_golden_except_documented_soft_gc_boundary() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    emit_failure(
        &mut output,
        "ctor.null_name",
        StandardCache::<String, String>::with_options(None, false, 0, 0, None, false, false),
    );
    emit_failure(
        &mut output,
        "ctor.empty_name",
        StandardCache::<String, String>::new(Some(""), false, 1),
    );
    emit_failure(
        &mut output,
        "ctor.whitespace_name",
        StandardCache::<String, String>::new(Some("\u{2003}"), false, 1),
    );
    emit_failure(
        &mut output,
        "ctor.capacity",
        StandardCache::<String, String>::new(Some("cache"), false, 0),
    );
    emit_failure(
        &mut output,
        "ctor.max_size",
        StandardCache::<String, String>::with_max_size(Some("cache"), false, 1, 0),
    );

    let unlimited = StandardCache::<String, String>::with_max_size(Some("\u{00A0}"), true, 2, -2)
        .expect("cache");
    emit(&mut output, "config.name", unlimited.get_name());
    emit_bool(
        &mut output,
        "config.soft",
        unlimited.get_use_soft_references(),
    );
    emit_bool(&mut output, "config.has_max", unlimited.has_max_size());
    emit(&mut output, "config.max", unlimited.get_max_size());
    emit(&mut output, "config.size", unlimited.size());
    emit_double(&mut output, "config.hit_ratio", unlimited.get_hit_ratio());
    emit_double(&mut output, "config.miss_ratio", unlimited.get_miss_ratio());

    let fifo =
        StandardCache::<String, String>::with_options(Some("fifo"), false, 2, 2, None, true, false)
            .expect("cache");
    let original = Arc::new("one".to_owned());
    fifo.put("a".to_owned(), Arc::clone(&original));
    fifo.put("a".to_owned(), Arc::new("replacement".to_owned()));
    fifo.put("b".to_owned(), Arc::new("two".to_owned()));
    emit_bool(
        &mut output,
        "fifo.put_if_absent.identity",
        Arc::ptr_eq(&fifo.get(&"a".to_owned()).expect("a"), &original),
    );
    fifo.put("c".to_owned(), Arc::new("three".to_owned()));
    emit_keys(&mut output, "fifo.keys", fifo.key_set());
    emit_bool(
        &mut output,
        "fifo.a_miss",
        fifo.get(&"a".to_owned()).is_none(),
    );
    emit_bool(
        &mut output,
        "fifo.b_hit",
        fifo.get(&"b".to_owned()).is_some(),
    );
    emit_bool(
        &mut output,
        "fifo.c_hit",
        fifo.get(&"c".to_owned()).is_some(),
    );
    emit_counters(&mut output, "fifo", &fifo);

    let invalid: Arc<dyn ICacheEntryValidityChecker<String, String>> =
        Arc::new(RecordingChecker::new(false));
    let checked = StandardCache::with_validity_checker(Some("checked"), false, 2, Some(invalid))
        .expect("cache");
    checked.put("key".to_owned(), Arc::new("value".to_owned()));
    let valid = RecordingChecker::new(true);
    emit_bool(
        &mut output,
        "checker.explicit_hit",
        checked
            .get_with_validity_checker(&"key".to_owned(), &valid)
            .is_some(),
    );
    let state = valid.state.lock().expect("checker lock");
    let (key, value, timestamp) = state.as_ref().expect("checker called");
    emit(&mut output, "checker.key", key);
    emit(&mut output, "checker.value", value);
    emit_bool(&mut output, "checker.timestamp_positive", *timestamp > 0);
    drop(state);
    emit_bool(
        &mut output,
        "checker.default_miss",
        checked.get(&"key".to_owned()).is_none(),
    );
    emit_bool(
        &mut output,
        "checker.removed",
        !checked.key_set().contains("key"),
    );

    checked.clear_key(&"missing".to_owned());
    checked.put("first".to_owned(), Arc::new("one".to_owned()));
    checked.clear();
    emit(&mut output, "clear.empty", checked.size());
    emit_counters(&mut output, "checked.disabled", &checked);

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_failure(
    output: &mut String,
    key: &str,
    result: Result<StandardCache<String, String>, StandardCacheError>,
) {
    match result {
        Ok(_) => emit(output, key, "NO_ERROR"),
        Err(error) => emit(
            output,
            key,
            format!("java.lang.IllegalArgumentException:{error}"),
        ),
    }
}

fn emit_counters(output: &mut String, prefix: &str, cache: &StandardCache<String, String>) {
    emit(
        output,
        &format!("{prefix}.put_count"),
        cache.get_put_count(),
    );
    emit(
        output,
        &format!("{prefix}.get_count"),
        cache.get_get_count(),
    );
    emit(
        output,
        &format!("{prefix}.hit_count"),
        cache.get_hit_count(),
    );
    emit(
        output,
        &format!("{prefix}.miss_count"),
        cache.get_miss_count(),
    );
    emit_double(
        output,
        &format!("{prefix}.hit_ratio"),
        cache.get_hit_ratio(),
    );
    emit_double(
        output,
        &format!("{prefix}.miss_ratio"),
        cache.get_miss_ratio(),
    );
}

fn emit_keys(output: &mut String, key: &str, values: std::collections::HashSet<String>) {
    let mut values: Vec<_> = values.into_iter().collect();
    values.sort();
    emit(output, key, format!("[{}]", values.join(", ")));
}

fn emit_bool(output: &mut String, key: &str, value: bool) {
    emit(output, key, value);
}

fn emit_double(output: &mut String, key: &str, value: f64) {
    emit(output, key, format!("{value:?}"));
}

fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
    use std::fmt::Write;
    writeln!(output, "{key}={value}").expect("string output");
}
