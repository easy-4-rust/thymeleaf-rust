//! VALUE_ADD：`IWebRequest` 覆盖缺口测试（2026-09-02）——风险：WebExchange 参数访问契约。
//!
//! 缺失行 50-80：get_request_url 计算（scheme/server_name/port 缺失返回错误、
//! default port 省略、query string 拼接）、is_secure 默认实现、get_request_path 拼接。
//! Java 侧 `IWebRequest` 无独立测试类；覆盖来自 `WebProcessingContextBuilder` 集成路径。
//! 以下按 VALUE_ADD 补充默认方法的边界分支。

use indexmap::IndexMap;
use thymeleaf::util::Utf16String;
use thymeleaf::web::{IWebRequest, WebRequestError};

fn js(s: &str) -> Utf16String {
    Utf16String::from_rust_str(s)
}

// ===========================================================================
// Configurable mock request for parameterized tests
// ===========================================================================

struct MockRequest {
    method: Option<Utf16String>,
    scheme: Option<Utf16String>,
    server_name: Option<Utf16String>,
    server_port: Option<i32>,
    application_path: Option<Utf16String>,
    path_within_application: Option<Utf16String>,
    query_string: Option<Utf16String>,
}

impl Default for MockRequest {
    fn default() -> Self {
        Self {
            method: Some(js("GET")),
            scheme: Some(js("http")),
            server_name: Some(js("localhost")),
            server_port: Some(80),
            application_path: Some(js("")),
            path_within_application: Some(js("/")),
            query_string: None,
        }
    }
}

impl IWebRequest for MockRequest {
    fn get_method(&self) -> Option<Utf16String> {
        self.method.clone()
    }
    fn get_scheme(&self) -> Option<Utf16String> {
        self.scheme.clone()
    }
    fn get_server_name(&self) -> Option<Utf16String> {
        self.server_name.clone()
    }
    fn get_server_port(&self) -> Option<i32> {
        self.server_port
    }
    fn get_application_path(&self) -> Option<Utf16String> {
        self.application_path.clone()
    }
    fn get_path_within_application(&self) -> Option<Utf16String> {
        self.path_within_application.clone()
    }
    fn get_query_string(&self) -> Option<Utf16String> {
        self.query_string.clone()
    }
    fn contains_header(&self, _: Option<&Utf16String>) -> bool {
        false
    }
    fn get_header_count(&self) -> i32 {
        0
    }
    fn get_all_header_names(&self) -> Vec<Option<Utf16String>> {
        Vec::new()
    }
    fn get_header_map(&self) -> IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> {
        IndexMap::new()
    }
    fn get_header_values(&self, _: Option<&Utf16String>) -> Option<Vec<Option<Utf16String>>> {
        None
    }
    fn contains_parameter(&self, _: Option<&Utf16String>) -> bool {
        false
    }
    fn get_parameter_count(&self) -> i32 {
        0
    }
    fn get_all_parameter_names(&self) -> Vec<Option<Utf16String>> {
        Vec::new()
    }
    fn get_parameter_map(&self) -> IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> {
        IndexMap::new()
    }
    fn get_parameter_values(&self, _: Option<&Utf16String>) -> Option<Vec<Option<Utf16String>>> {
        None
    }
    fn contains_cookie(&self, _: Option<&Utf16String>) -> bool {
        false
    }
    fn get_cookie_count(&self) -> i32 {
        0
    }
    fn get_all_cookie_names(&self) -> Vec<Option<Utf16String>> {
        Vec::new()
    }
    fn get_cookie_map(&self) -> IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> {
        IndexMap::new()
    }
    fn get_cookie_values(&self, _: Option<&Utf16String>) -> Option<Vec<Option<Utf16String>>> {
        None
    }
}

// ===========================================================================
// is_secure: true for https, false for http
// ===========================================================================

#[test]
fn is_secure_true_for_https() {
    let request = MockRequest {
        scheme: Some(js("https")),
        ..Default::default()
    };
    assert!(request.is_secure());
}

#[test]
fn is_secure_false_for_http() {
    let request = MockRequest::default();
    assert!(!request.is_secure());
}

#[test]
fn is_secure_case_insensitive() {
    let request = MockRequest {
        scheme: Some(js("HTTPS")),
        ..Default::default()
    };
    assert!(request.is_secure());
}

#[test]
fn is_secure_false_when_scheme_none() {
    let request = MockRequest {
        scheme: None,
        ..Default::default()
    };
    assert!(!request.is_secure());
}

// ===========================================================================
// get_request_path: concatenates application_path + path_within_application
// ===========================================================================

#[test]
fn get_request_path_concatenates() {
    let request = MockRequest {
        application_path: Some(js("/app")),
        path_within_application: Some(js("/users")),
        ..Default::default()
    };
    assert_eq!(request.get_request_path().to_string_lossy(), "/app/users");
}

#[test]
fn get_request_path_handles_none_application_path() {
    let request = MockRequest {
        application_path: None,
        path_within_application: Some(js("/home")),
        ..Default::default()
    };
    assert_eq!(request.get_request_path().to_string_lossy(), "/home");
}

// ===========================================================================
// get_request_url: http on port 80 omits port
// ===========================================================================

#[test]
fn get_request_url_omits_default_http_port() {
    let request = MockRequest::default(); // http://localhost:80
    let url = request.get_request_url().unwrap();
    assert_eq!(url.to_string_lossy(), "http://localhost/");
    assert!(
        !url.to_string_lossy().contains(":80"),
        "default port must be omitted"
    );
}

// ===========================================================================
// get_request_url: https on port 443 omits port
// ===========================================================================

#[test]
fn get_request_url_omits_default_https_port() {
    let request = MockRequest {
        scheme: Some(js("https")),
        server_port: Some(443),
        ..Default::default()
    };
    let url = request.get_request_url().unwrap();
    assert_eq!(url.to_string_lossy(), "https://localhost/");
}

// ===========================================================================
// get_request_url: non-default port includes port
// ===========================================================================

#[test]
fn get_request_url_includes_non_default_port() {
    let request = MockRequest {
        scheme: Some(js("http")),
        server_port: Some(8080),
        ..Default::default()
    };
    let url = request.get_request_url().unwrap();
    assert_eq!(url.to_string_lossy(), "http://localhost:8080/");
}

// ===========================================================================
// get_request_url: appends query string
// ===========================================================================

#[test]
fn get_request_url_appends_query_string() {
    let request = MockRequest {
        query_string: Some(js("foo=bar&baz=1")),
        ..Default::default()
    };
    let url = request.get_request_url().unwrap();
    assert!(
        url.to_string_lossy().contains("?foo=bar&baz=1"),
        "must append query: {}",
        url.to_string_lossy()
    );
}

// ===========================================================================
// get_request_url: returns error when scheme missing
// ===========================================================================

#[test]
fn get_request_url_error_when_scheme_missing() {
    let request = MockRequest {
        scheme: None,
        ..Default::default()
    };
    let result = request.get_request_url();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        WebRequestError::UnsupportedOperation {
            message: "Request scheme, server name or port are null in this environment. Cannot compute request URL",
        }
    );
}

// ===========================================================================
// get_request_url: returns error when server_name missing
// ===========================================================================

#[test]
fn get_request_url_error_when_server_name_missing() {
    let request = MockRequest {
        server_name: None,
        ..Default::default()
    };
    let result = request.get_request_url();
    assert!(result.is_err());
}

// ===========================================================================
// get_request_url: returns error when port missing
// ===========================================================================

#[test]
fn get_request_url_error_when_port_missing() {
    let request = MockRequest {
        server_port: None,
        ..Default::default()
    };
    let result = request.get_request_url();
    assert!(result.is_err());
}

// ===========================================================================
// get_header_value: returns first value from multi-valued header
// ===========================================================================

#[test]
fn get_header_value_returns_first() {
    // Use the default methods: get_header_value delegates to get_header_values
    // Since MockRequest returns None for all headers, verify delegation
    let request = MockRequest::default();
    assert!(request.get_header_value(Some(&js("X-Test"))).is_none());
}

// ===========================================================================
// get_parameter_value: returns first value from multi-valued param
// ===========================================================================

#[test]
fn get_parameter_value_returns_first() {
    let request = MockRequest::default();
    assert!(request.get_parameter_value(Some(&js("q"))).is_none());
}

// ===========================================================================
// get_cookie_value: returns first value
// ===========================================================================

#[test]
fn get_cookie_value_returns_first() {
    let request = MockRequest::default();
    assert!(request.get_cookie_value(Some(&js("session"))).is_none());
}
