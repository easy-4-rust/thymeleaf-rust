//! 通用缓存接口的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use thymeleaf::cache::{ICache, ICacheEntryValidityChecker};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("../../thymeleaf/tests/fixtures/cache_contract_golden.txt");

struct Entry {
    value: Arc<String>,
    creation_timestamp: i64,
}

#[derive(Default)]
struct ContractCache {
    entries: Mutex<HashMap<String, Entry>>,
}

impl ICache<String, String> for ContractCache {
    fn put(&self, key: String, value: Arc<String>) {
        self.entries.lock().expect("cache lock").insert(
            key,
            Entry {
                value,
                creation_timestamp: 7,
            },
        );
    }

    fn get(&self, key: &String) -> Option<Arc<String>> {
        self.entries
            .lock()
            .expect("cache lock")
            .get(key)
            .map(|entry| Arc::clone(&entry.value))
    }

    fn get_with_validity_checker(
        &self,
        key: &String,
        validity_checker: &dyn ICacheEntryValidityChecker<String, String>,
    ) -> Option<Arc<String>> {
        let mut entries = self.entries.lock().expect("cache lock");
        let entry = entries.get(key)?;
        if !validity_checker.check_is_value_still_valid(key, &entry.value, entry.creation_timestamp)
        {
            entries.remove(key);
            return None;
        }
        entries.get(key).map(|entry| Arc::clone(&entry.value))
    }

    fn clear(&self) {
        self.entries.lock().expect("cache lock").clear();
    }

    fn clear_key(&self, key: &String) {
        self.entries.lock().expect("cache lock").remove(key);
    }

    fn key_set(&self) -> HashSet<String> {
        self.entries
            .lock()
            .expect("cache lock")
            .keys()
            .cloned()
            .collect()
    }
}

struct CheckerState {
    key: String,
    value: String,
    timestamp: i64,
}

struct RecordingChecker {
    valid: bool,
    state: Mutex<Option<CheckerState>>,
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
        *self.state.lock().expect("checker lock") = Some(CheckerState {
            key: key.clone(),
            value: value.clone(),
            timestamp: entry_creation_timestamp,
        });
        self.valid
    }
}

#[test]
fn cache_contract_objects_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);

    let cache = ContractCache::default();
    let key = "key".to_owned();
    let value = Arc::new("value".to_owned());

    emit_bool(&mut output, "cache.miss", cache.get(&key).is_none());
    cache.put(key.clone(), Arc::clone(&value));
    emit_bool(
        &mut output,
        "cache.hit.identity",
        Arc::ptr_eq(&cache.get(&key).expect("cache hit"), &value),
    );
    emit_keys(&mut output, "cache.keys", cache.key_set());

    let valid = RecordingChecker::new(true);
    emit_bool(
        &mut output,
        "cache.checked.identity",
        Arc::ptr_eq(
            &cache
                .get_with_validity_checker(&key, &valid)
                .expect("valid entry"),
            &value,
        ),
    );
    {
        let state = valid.state.lock().expect("checker lock");
        let state = state.as_ref().expect("checker called");
        emit(&mut output, "checker.key", &state.key);
        emit(&mut output, "checker.value", &state.value);
        emit(
            &mut output,
            "checker.timestamp",
            &state.timestamp.to_string(),
        );
    }

    let invalid = RecordingChecker::new(false);
    emit_bool(
        &mut output,
        "cache.invalid.miss",
        cache.get_with_validity_checker(&key, &invalid).is_none(),
    );
    emit_bool(
        &mut output,
        "cache.invalid.removed",
        cache.get(&key).is_none(),
    );
    emit_keys(&mut output, "cache.invalid.keys", cache.key_set());
    emit_bool(
        &mut output,
        "cache.missing.checked",
        cache
            .get_with_validity_checker(&"missing".to_owned(), &valid)
            .is_none(),
    );

    cache.clear_key(&"missing".to_owned());
    cache.put("first".to_owned(), Arc::new("one".to_owned()));
    cache.put("second".to_owned(), Arc::new("two".to_owned()));
    cache.clear_key(&"first".to_owned());
    emit_keys(&mut output, "cache.clear_key.remaining", cache.key_set());
    cache.clear();
    emit_bool(&mut output, "cache.clear.empty", cache.key_set().is_empty());

    assert_eq!(output, JAVA_GOLDEN);
}

fn emit_keys(output: &mut String, key: &str, values: HashSet<String>) {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort();
    emit(output, key, &format!("[{}]", values.join(", ")));
}

fn emit_bool(output: &mut String, key: &str, value: bool) {
    emit(output, key, &value.to_string());
}

fn emit(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push('=');
    output.push_str(value);
    output.push('\n');
}
