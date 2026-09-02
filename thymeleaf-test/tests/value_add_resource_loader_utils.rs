//! VALUE_ADD：`ResourceLoaderUtils` 覆盖缺口测试（2026-09-02）——风险：资源加载工具
//! 路径分支（classpath prefix strip、file 路径查找、thread roots 作用域、register/find class）。
//!
//! 缺失行 28-47/124-161：with_thread_resource_roots 作用域守卫、find_resource prefix strip、
//! load_resource_as_stream 错误路径、register/find class 往返。Java 侧 `ClassLoaderUtils`
//! 无独立测试；覆盖来自集成路径。

use std::fs;
use std::path::PathBuf;

use thymeleaf::util::ResourceLoaderUtils;

/// 用进程 ID + 时间戳构造唯一临时目录，避免测试间竞争。
fn unique_temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "thymeleaf-value-add-{}-{:?}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

// ===========================================================================
// get_resource_roots: always non-empty (includes at least CARGO_MANIFEST_DIR)
// ===========================================================================

#[test]
fn get_resource_roots_is_non_empty() {
    let roots = ResourceLoaderUtils::get_resource_roots();
    assert!(
        !roots.is_empty(),
        "resource roots must include at least CARGO_MANIFEST_DIR"
    );
}

// ===========================================================================
// with_thread_resource_roots: restores previous roots after closure
// ===========================================================================

#[test]
fn with_thread_resource_roots_restores_previous() {
    let roots_before = ResourceLoaderUtils::get_resource_roots();
    let custom = vec![PathBuf::from("/tmp/nonexistent")];
    ResourceLoaderUtils::with_thread_resource_roots(custom, || {
        let roots_inside = ResourceLoaderUtils::get_resource_roots();
        assert_eq!(roots_inside[0], PathBuf::from("/tmp/nonexistent"));
    });
    let roots_after = ResourceLoaderUtils::get_resource_roots();
    assert_eq!(
        roots_before, roots_after,
        "roots must be restored after closure"
    );
}

// ===========================================================================
// with_thread_resource_roots: restores even on panic
// ===========================================================================

#[test]
fn with_thread_resource_roots_restores_on_panic() {
    let roots_before = ResourceLoaderUtils::get_resource_roots();
    let custom = vec![PathBuf::from("/tmp/panic-test")];
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ResourceLoaderUtils::with_thread_resource_roots(custom, || {
            panic!("intentional panic");
        });
    }));
    let roots_after = ResourceLoaderUtils::get_resource_roots();
    assert_eq!(
        roots_before, roots_after,
        "roots must be restored after panic"
    );
}

// ===========================================================================
// register_class + find_class + is_class_present + load_class: round-trip
// ===========================================================================

#[test]
fn register_and_find_class_round_trip() {
    ResourceLoaderUtils::register_class("com.example.TestCapability");
    assert!(ResourceLoaderUtils::is_class_present(
        "com.example.TestCapability"
    ));
    let found = ResourceLoaderUtils::find_class("com.example.TestCapability");
    assert_eq!(found.as_deref(), Some("com.example.TestCapability"));
}

#[test]
fn load_class_returns_error_for_missing() {
    let result = ResourceLoaderUtils::load_class("com.example.DoesNotExist");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    assert!(error.to_string().contains("DoesNotExist"));
}

#[test]
fn is_class_present_returns_false_for_missing() {
    assert!(!ResourceLoaderUtils::is_class_present(
        "com.example.NoSuchClass"
    ));
}

// ===========================================================================
// find_resource: strips leading slash (classpath convention)
// ===========================================================================

#[test]
fn find_resource_strips_leading_slash() {
    let dir = unique_temp_dir();
    fs::write(dir.join("test-resource.txt"), "content").expect("write");

    ResourceLoaderUtils::with_thread_resource_roots(vec![dir], || {
        let with_slash = ResourceLoaderUtils::find_resource("/test-resource.txt");
        let without_slash = ResourceLoaderUtils::find_resource("test-resource.txt");
        assert!(with_slash.is_some(), "must find with leading slash");
        assert!(without_slash.is_some(), "must find without leading slash");
        assert_eq!(with_slash, without_slash, "results must be identical");
    });
}

// ===========================================================================
// find_resource: returns None for nonexistent
// ===========================================================================

#[test]
fn find_resource_returns_none_for_nonexistent() {
    let result = ResourceLoaderUtils::find_resource("definitely-does-not-exist-12345.txt");
    assert!(result.is_none());
}

// ===========================================================================
// is_resource_present: delegates to find_resource
// ===========================================================================

#[test]
fn is_resource_present_false_for_nonexistent() {
    assert!(!ResourceLoaderUtils::is_resource_present(
        "no-such-file-99999.txt"
    ));
}

// ===========================================================================
// load_resource_as_stream: returns NotFound for missing resource
// ===========================================================================

#[test]
fn load_resource_as_stream_not_found_for_missing() {
    match ResourceLoaderUtils::load_resource_as_stream("no-such-resource-99999.txt") {
        Ok(_) => panic!("must fail for missing resource"),
        Err(error) => assert_eq!(error.kind(), std::io::ErrorKind::NotFound),
    }
}

// ===========================================================================
// find_resource_as_stream: returns Ok(None) for missing
// ===========================================================================

#[test]
fn find_resource_as_stream_returns_none_for_missing() {
    let result = ResourceLoaderUtils::find_resource_as_stream("no-such-stream-99999.txt").unwrap();
    assert!(result.is_none());
}

// ===========================================================================
// load_resource_as_stream: reads content of found resource
// ===========================================================================

#[test]
fn load_resource_as_stream_reads_content() {
    let dir = unique_temp_dir();
    fs::write(dir.join("readable.txt"), "hello world").expect("write");

    ResourceLoaderUtils::with_thread_resource_roots(vec![dir], || {
        let mut reader = ResourceLoaderUtils::load_resource_as_stream("readable.txt")
            .expect("must find readable.txt");
        let mut content = String::new();
        std::io::Read::read_to_string(&mut reader, &mut content).expect("read");
        assert_eq!(content, "hello world");
    });
}
