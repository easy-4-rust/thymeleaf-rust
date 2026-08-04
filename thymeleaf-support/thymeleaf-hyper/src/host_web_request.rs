use indexmap::IndexMap;
use thymeleaf::util::Utf16String;
use thymeleaf::web::IWebRequest;

/// Hyper `Request` 元数据的不可变 Thymeleaf Web 请求快照。
///
/// 对应 Java Servlet 的 `IServletWebRequest`、`JakartaServletWebRequest`
/// 与 `JavaxServletWebRequest`，保留 Header、查询参数和 Cookie 的多值顺序。
pub struct HostWebRequest {
    method: Utf16String,
    scheme: Option<Utf16String>,
    server_name: Option<Utf16String>,
    server_port: Option<i32>,
    application_path: Utf16String,
    path_within_application: Utf16String,
    query_string: Option<Utf16String>,
    headers: IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>>,
    parameters: IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>>,
    cookies: IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>>,
}

impl HostWebRequest {
    /// 从 Hyper/http 请求创建只读快照。对应 Java:
    /// `JakartaServletWebRequest#JakartaServletWebRequest`。
    ///
    /// # 参数
    /// - `request`：宿主 HTTP 请求。
    /// - `application_path`：Servlet context path 的中立等价文本。
    ///
    /// # 返回
    /// 保留 URI、Header、参数和 Cookie 多值顺序的请求快照。
    #[must_use]
    pub fn from_request<B>(request: &hyper::Request<B>, application_path: &str) -> Self {
        let uri = request.uri();
        let authority = uri.authority();
        let server_name = authority.map(|value| Utf16String::from_rust_str(value.host()));
        let server_port = authority
            .and_then(|value| value.port_u16())
            .map(i32::from)
            .or_else(|| match uri.scheme_str() {
                Some("http") => Some(80),
                Some("https") => Some(443),
                _ => None,
            });
        let application_path = normalize_application_path(application_path);
        let path = uri.path();
        let within = path
            .strip_prefix(&application_path)
            .filter(|_| !application_path.is_empty())
            .unwrap_or(path);
        let headers = collect_headers(request.headers());
        let parameters = collect_parameters(uri.query());
        let cookies = collect_cookies(request.headers());
        Self {
            method: Utf16String::from_rust_str(request.method().as_str()),
            scheme: uri.scheme_str().map(Utf16String::from_rust_str),
            server_name,
            server_port,
            application_path: Utf16String::from_rust_str(&application_path),
            path_within_application: Utf16String::from_rust_str(within),
            query_string: uri.query().map(Utf16String::from_rust_str),
            headers,
            parameters,
            cookies,
        }
    }
}

impl IWebRequest for HostWebRequest {
    fn get_method(&self) -> Option<Utf16String> {
        Some(self.method.clone())
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
        Some(self.application_path.clone())
    }

    fn get_path_within_application(&self) -> Option<Utf16String> {
        Some(self.path_within_application.clone())
    }

    fn get_query_string(&self) -> Option<Utf16String> {
        self.query_string.clone()
    }

    fn contains_header(&self, name: Option<&Utf16String>) -> bool {
        require_name(name);
        contains_case_insensitive(&self.headers, name)
    }

    fn get_header_count(&self) -> i32 {
        i32::try_from(self.headers.len()).unwrap_or(i32::MAX)
    }

    fn get_all_header_names(&self) -> Vec<Option<Utf16String>> {
        self.headers.keys().cloned().collect()
    }

    fn get_header_map(&self) -> IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> {
        self.headers.clone()
    }

    fn get_header_values(&self, name: Option<&Utf16String>) -> Option<Vec<Option<Utf16String>>> {
        require_name(name);
        get_case_insensitive(&self.headers, name)
    }

    fn contains_parameter(&self, name: Option<&Utf16String>) -> bool {
        require_name(name);
        self.parameters.contains_key(&name.cloned())
    }

    fn get_parameter_count(&self) -> i32 {
        i32::try_from(self.parameters.len()).unwrap_or(i32::MAX)
    }

    fn get_all_parameter_names(&self) -> Vec<Option<Utf16String>> {
        self.parameters.keys().cloned().collect()
    }

    fn get_parameter_map(&self) -> IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> {
        self.parameters.clone()
    }

    fn get_parameter_values(&self, name: Option<&Utf16String>) -> Option<Vec<Option<Utf16String>>> {
        require_name(name);
        self.parameters.get(&name.cloned()).cloned().flatten()
    }

    fn contains_cookie(&self, name: Option<&Utf16String>) -> bool {
        require_name(name);
        self.cookies.contains_key(&name.cloned())
    }

    fn get_cookie_count(&self) -> i32 {
        let count = self
            .cookies
            .values()
            .filter_map(Option::as_ref)
            .map(Vec::len)
            .sum::<usize>();
        i32::try_from(count).unwrap_or(i32::MAX)
    }

    fn get_all_cookie_names(&self) -> Vec<Option<Utf16String>> {
        self.cookies.keys().cloned().collect()
    }

    fn get_cookie_map(&self) -> IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> {
        self.cookies.clone()
    }

    fn get_cookie_values(&self, name: Option<&Utf16String>) -> Option<Vec<Option<Utf16String>>> {
        require_name(name);
        self.cookies.get(&name.cloned()).cloned().flatten()
    }
}

fn normalize_application_path(application_path: &str) -> String {
    if application_path == "/" {
        String::new()
    } else {
        application_path.trim_end_matches('/').to_owned()
    }
}

fn collect_headers(
    headers: &hyper::HeaderMap,
) -> IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> {
    let mut output = IndexMap::new();
    for name in headers.keys() {
        let values = headers
            .get_all(name)
            .iter()
            .map(|value| {
                Some(Utf16String::from_rust_str(
                    value.to_str().unwrap_or_default(),
                ))
            })
            .collect();
        output.insert(
            Some(Utf16String::from_rust_str(name.as_str())),
            Some(values),
        );
    }
    output
}

fn collect_parameters(
    query: Option<&str>,
) -> IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> {
    let mut output: IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> =
        IndexMap::new();
    for (name, value) in url::form_urlencoded::parse(query.unwrap_or_default().as_bytes()) {
        output
            .entry(Some(Utf16String::from_rust_str(&name)))
            .or_insert_with(|| Some(Vec::new()))
            .as_mut()
            .expect("parameter entry is initialized")
            .push(Some(Utf16String::from_rust_str(&value)));
    }
    output
}

fn collect_cookies(
    headers: &hyper::HeaderMap,
) -> IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> {
    let mut output: IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>> =
        IndexMap::new();
    for header in headers.get_all(hyper::header::COOKIE) {
        for cookie in header.to_str().unwrap_or_default().split(';') {
            let Some((name, value)) = cookie.trim().split_once('=') else {
                continue;
            };
            output
                .entry(Some(Utf16String::from_rust_str(name)))
                .or_insert_with(|| Some(Vec::new()))
                .as_mut()
                .expect("cookie entry is initialized")
                .push(Some(Utf16String::from_rust_str(value)));
        }
    }
    output
}

fn contains_case_insensitive(
    values: &IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>>,
    name: Option<&Utf16String>,
) -> bool {
    get_case_insensitive(values, name).is_some()
}

fn get_case_insensitive(
    values: &IndexMap<Option<Utf16String>, Option<Vec<Option<Utf16String>>>>,
    name: Option<&Utf16String>,
) -> Option<Vec<Option<Utf16String>>> {
    let name = name?.to_string_lossy();
    values.iter().find_map(|(candidate, value)| {
        candidate
            .as_ref()
            .is_some_and(|candidate| candidate.to_string_lossy().eq_ignore_ascii_case(&name))
            .then(|| value.clone())
            .flatten()
    })
}

fn require_name(name: Option<&Utf16String>) {
    if name.is_none() {
        panic!("Name cannot be null");
    }
}
