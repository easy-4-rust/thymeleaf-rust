//! Vernal HTTP 请求到 Thymeleaf `IWebRequest` 的适配。
//!
//! 对应 Java `org.thymeleaf.web.IWebRequest` 的 Vernal 宿主实现：从
//! `vernal_http::HttpRequestSnapshot`（method/uri/headers）读取方法、URL、Header
//! 与 Cookie；查询串按 URI query 解析为请求参数。参数与 Cookie 的多值顺序保持
//! 快照原始顺序。

use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use thymeleaf::util::JavaString;
use thymeleaf::web::IWebRequest;
use vernal_http::HttpRequestSnapshot;

/// 惰性解析后的字符串多值参数/ Cookie 缓存类型。
type LazyStringMap = RwLock<Option<Arc<IndexMap<String, Vec<String>>>>>;

/// 把 Vernal HTTP 请求快照适配为 Thymeleaf Web 请求。
pub struct VernalWebRequest {
    snapshot: Arc<HttpRequestSnapshot>,
    parameters: LazyStringMap,
    cookies: LazyStringMap,
}

impl VernalWebRequest {
    /// 包装请求快照。
    #[must_use]
    pub fn new(snapshot: Arc<HttpRequestSnapshot>) -> Self {
        Self {
            snapshot,
            parameters: RwLock::new(None),
            cookies: RwLock::new(None),
        }
    }

    /// 返回被包装的请求快照。
    #[must_use]
    pub const fn snapshot(&self) -> &Arc<HttpRequestSnapshot> {
        &self.snapshot
    }

    /// 惰性解析查询串参数（`?a=1&b=2`，重复键合并为多值）。
    fn parameters(&self) -> Arc<IndexMap<String, Vec<String>>> {
        if let Some(parameters) = self
            .parameters
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
        {
            return Arc::clone(parameters);
        }
        let mut map = IndexMap::new();
        if let Some(query) = self.snapshot.uri().query() {
            for (key, value) in form_urlencoded::parse(query.as_bytes()) {
                map.entry(key.into_owned())
                    .or_insert_with(Vec::new)
                    .push(value.into_owned());
            }
        }
        let parameters = Arc::new(map);
        *self
            .parameters
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Arc::clone(&parameters));
        parameters
    }

    /// 惰性解析 Cookie 头（`name=value; name2=value2`）。
    fn cookies(&self) -> Arc<IndexMap<String, Vec<String>>> {
        if let Some(cookies) = self
            .cookies
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
        {
            return Arc::clone(cookies);
        }
        let mut map = IndexMap::new();
        if let Some(cookie_header) = self.snapshot.headers().get(http::header::COOKIE)
            && let Ok(cookie_text) = cookie_header.to_str()
        {
            for pair in cookie_text.split(';') {
                let pair = pair.trim();
                if let Some((name, value)) = pair.split_once('=') {
                    map.entry(name.trim().to_owned())
                        .or_insert_with(Vec::new)
                        .push(value.trim().to_owned());
                }
            }
        }
        let cookies = Arc::new(map);
        *self
            .cookies
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Arc::clone(&cookies));
        cookies
    }
}

fn to_java_strings(values: &[String]) -> Vec<Option<JavaString>> {
    values
        .iter()
        .map(|value| Some(JavaString::from_rust_str(value)))
        .collect()
}

impl IWebRequest for VernalWebRequest {
    fn get_method(&self) -> Option<JavaString> {
        Some(JavaString::from_rust_str(self.snapshot.method().as_str()))
    }

    fn get_scheme(&self) -> Option<JavaString> {
        self.snapshot
            .uri()
            .scheme()
            .map(|scheme| JavaString::from_rust_str(scheme.as_str()))
    }

    fn get_server_name(&self) -> Option<JavaString> {
        self.snapshot.uri().host().map(JavaString::from_rust_str)
    }

    fn get_server_port(&self) -> Option<i32> {
        self.snapshot.uri().port_u16().map(i32::from)
    }

    fn get_application_path(&self) -> Option<JavaString> {
        None
    }

    fn get_path_within_application(&self) -> Option<JavaString> {
        Some(JavaString::from_rust_str(self.snapshot.uri().path()))
    }

    fn get_query_string(&self) -> Option<JavaString> {
        self.snapshot.uri().query().map(JavaString::from_rust_str)
    }

    fn contains_header(&self, name: Option<&JavaString>) -> bool {
        self.get_header_value(name).is_some()
    }

    fn get_header_count(&self) -> i32 {
        self.snapshot.headers().len() as i32
    }

    fn get_all_header_names(&self) -> Vec<Option<JavaString>> {
        self.snapshot
            .headers()
            .keys()
            .map(|name| Some(JavaString::from_rust_str(name.as_str())))
            .collect()
    }

    fn get_header_map(&self) -> IndexMap<Option<JavaString>, Option<Vec<Option<JavaString>>>> {
        let mut map: IndexMap<Option<JavaString>, Vec<Option<JavaString>>> = IndexMap::new();
        for (name, value) in self.snapshot.headers() {
            let key = Some(JavaString::from_rust_str(name.as_str()));
            let value = value.to_str().ok().map(JavaString::from_rust_str);
            map.entry(key).or_default().push(value);
        }
        map.into_iter().map(|(k, v)| (k, Some(v))).collect()
    }

    fn get_header_values(&self, name: Option<&JavaString>) -> Option<Vec<Option<JavaString>>> {
        let name = name?;
        let mut values = Vec::new();
        for (header_name, value) in self.snapshot.headers() {
            if header_name
                .as_str()
                .eq_ignore_ascii_case(&name.to_string_lossy())
            {
                values.push(value.to_str().ok().map(JavaString::from_rust_str));
            }
        }
        (!values.is_empty()).then_some(values)
    }

    fn contains_parameter(&self, name: Option<&JavaString>) -> bool {
        self.get_parameter_value(name).is_some()
    }

    fn get_parameter_count(&self) -> i32 {
        self.parameters().values().map(Vec::len).sum::<usize>() as i32
    }

    fn get_all_parameter_names(&self) -> Vec<Option<JavaString>> {
        self.parameters()
            .keys()
            .map(|name| Some(JavaString::from_rust_str(name)))
            .collect()
    }

    fn get_parameter_map(&self) -> IndexMap<Option<JavaString>, Option<Vec<Option<JavaString>>>> {
        self.parameters()
            .iter()
            .map(|(name, values)| {
                (
                    Some(JavaString::from_rust_str(name)),
                    Some(to_java_strings(values)),
                )
            })
            .collect()
    }

    fn get_parameter_values(&self, name: Option<&JavaString>) -> Option<Vec<Option<JavaString>>> {
        let name = name?;
        self.parameters()
            .get(&name.to_string_lossy())
            .map(|values| to_java_strings(values))
    }

    fn contains_cookie(&self, name: Option<&JavaString>) -> bool {
        self.get_cookie_value(name).is_some()
    }

    fn get_cookie_count(&self) -> i32 {
        self.cookies().values().map(Vec::len).sum::<usize>() as i32
    }

    fn get_all_cookie_names(&self) -> Vec<Option<JavaString>> {
        self.cookies()
            .keys()
            .map(|name| Some(JavaString::from_rust_str(name)))
            .collect()
    }

    fn get_cookie_map(&self) -> IndexMap<Option<JavaString>, Option<Vec<Option<JavaString>>>> {
        self.cookies()
            .iter()
            .map(|(name, values)| {
                (
                    Some(JavaString::from_rust_str(name)),
                    Some(to_java_strings(values)),
                )
            })
            .collect()
    }

    fn get_cookie_values(&self, name: Option<&JavaString>) -> Option<Vec<Option<JavaString>>> {
        let name = name?;
        self.cookies()
            .get(&name.to_string_lossy())
            .map(|values| to_java_strings(values))
    }
}
