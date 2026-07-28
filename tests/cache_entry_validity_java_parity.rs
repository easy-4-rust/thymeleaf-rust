//! 缓存条目有效性策略的 Thymeleaf 3.1.5 Java/Rust Golden 差分测试。

use std::thread;
use std::time::Duration;

use thymeleaf::cache::{
    AlwaysValidCacheEntryValidity, ICacheEntryValidity, NonCacheableCacheEntryValidity,
    TTLCacheEntryValidity,
};

const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
const JAVA_GOLDEN: &str = include_str!("fixtures/cache_entry_validity_golden.txt");

#[test]
fn cache_entry_validity_objects_match_java_golden() {
    let mut output = String::new();
    emit(&mut output, "baseline", JAVA_BASELINE);
    export_always_valid(&mut output);
    export_non_cacheable(&mut output);
    export_ttl(&mut output);
    assert_eq!(output, JAVA_GOLDEN);
}

fn export_always_valid(output: &mut String) {
    let singleton: &dyn ICacheEntryValidity = AlwaysValidCacheEntryValidity::INSTANCE;
    let first = AlwaysValidCacheEntryValidity::new();
    let second = AlwaysValidCacheEntryValidity::new();

    emit_bool(
        output,
        "always.instance.cacheable",
        singleton.is_cacheable(),
    );
    emit_bool(
        output,
        "always.instance.valid",
        singleton.is_cache_still_valid(),
    );
    emit_bool(
        output,
        "always.instance.identity",
        std::ptr::eq(
            AlwaysValidCacheEntryValidity::INSTANCE,
            AlwaysValidCacheEntryValidity::INSTANCE,
        ),
    );
    emit_bool(output, "always.new.cacheable", first.is_cacheable());
    emit_bool(output, "always.new.valid", first.is_cache_still_valid());
    emit_bool(
        output,
        "always.new_not_instance",
        !std::ptr::eq(&first, AlwaysValidCacheEntryValidity::INSTANCE),
    );
    emit_bool(
        output,
        "always.new_identity",
        !std::ptr::eq(&first, &second),
    );
}

fn export_non_cacheable(output: &mut String) {
    let singleton: &dyn ICacheEntryValidity = NonCacheableCacheEntryValidity::INSTANCE;
    let first = NonCacheableCacheEntryValidity::new();
    let second = NonCacheableCacheEntryValidity::new();

    emit_bool(
        output,
        "non_cacheable.instance.cacheable",
        singleton.is_cacheable(),
    );
    emit_bool(
        output,
        "non_cacheable.instance.valid",
        singleton.is_cache_still_valid(),
    );
    emit_bool(
        output,
        "non_cacheable.instance.identity",
        std::ptr::eq(
            NonCacheableCacheEntryValidity::INSTANCE,
            NonCacheableCacheEntryValidity::INSTANCE,
        ),
    );
    emit_bool(output, "non_cacheable.new.cacheable", first.is_cacheable());
    emit_bool(
        output,
        "non_cacheable.new.valid",
        first.is_cache_still_valid(),
    );
    emit_bool(
        output,
        "non_cacheable.new_not_instance",
        !std::ptr::eq(&first, NonCacheableCacheEntryValidity::INSTANCE),
    );
    emit_bool(
        output,
        "non_cacheable.new_identity",
        !std::ptr::eq(&first, &second),
    );
}

fn export_ttl(output: &mut String) {
    export_ttl_case(output, "positive", 60_000);
    export_ttl_case(output, "zero", 0);
    export_ttl_case(output, "negative", -1);
    export_ttl_case(output, "max", i64::MAX);
    export_ttl_case(output, "min", i64::MIN);

    let expiring = TTLCacheEntryValidity::new(1);
    thread::sleep(Duration::from_millis(10));
    emit_bool(output, "ttl.expired.valid", expiring.is_cache_still_valid());
}

fn export_ttl_case(output: &mut String, name: &str, ttl: i64) {
    let validity = TTLCacheEntryValidity::new(ttl);
    emit(
        output,
        &format!("ttl.{name}.value"),
        &validity.get_cache_ttl_ms().to_string(),
    );
    emit_bool(
        output,
        &format!("ttl.{name}.cacheable"),
        validity.is_cacheable(),
    );
    emit_bool(
        output,
        &format!("ttl.{name}.valid"),
        validity.is_cache_still_valid(),
    );
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
