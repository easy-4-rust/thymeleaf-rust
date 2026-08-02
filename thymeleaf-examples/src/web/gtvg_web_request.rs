//! 请求对象 —— 对应 Java `HttpServletRequest`（GTVG 仅使用路径与参数）。

use indexmap::IndexMap;
use thymeleaf::util::JavaString;
use thymeleaf::web::IWebRequest;

/// 携带路径内参数（`prodId`/`orderId`）的最小请求。
#[derive(Clone, Debug)]
pub struct GtvgWebRequest {
    /// 应用内路径（对应 `getPathWithinApplication()`）。
    path: String,
    /// 查询参数（对应 `getParameterValue(name)`）。
    parameters: IndexMap<String, String>,
}

impl GtvgWebRequest {
    /// 创建请求；`parameters` 支持 `"prodId=1"` 形式。
    #[must_use]
    pub fn new(path: &str, parameters: &[(&str, &str)]) -> Self {
        Self {
            path: path.to_owned(),
            parameters: parameters
                .iter()
                .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                .collect(),
        }
    }
}

impl IWebRequest for GtvgWebRequest {
    fn get_method(&self) -> Option<JavaString> {
        Some(JavaString::from_rust_str("GET"))
    }
    fn get_scheme(&self) -> Option<JavaString> {
        Some(JavaString::from_rust_str("http"))
    }
    fn get_server_name(&self) -> Option<JavaString> {
        Some(JavaString::from_rust_str("localhost"))
    }
    fn get_server_port(&self) -> Option<i32> {
        Some(80)
    }
    fn get_application_path(&self) -> Option<JavaString> {
        Some(JavaString::from_rust_str(""))
    }
    fn get_path_within_application(&self) -> Option<JavaString> {
        Some(JavaString::from_rust_str(&self.path))
    }
    fn get_query_string(&self) -> Option<JavaString> {
        None
    }
    fn contains_header(&self, _name: Option<&JavaString>) -> bool {
        false
    }
    fn get_header_count(&self) -> i32 {
        0
    }
    fn get_all_header_names(&self) -> Vec<Option<JavaString>> {
        Vec::new()
    }
    fn get_header_map(&self) -> IndexMap<Option<JavaString>, Option<Vec<Option<JavaString>>>> {
        IndexMap::new()
    }
    fn get_header_values(&self, _name: Option<&JavaString>) -> Option<Vec<Option<JavaString>>> {
        None
    }
    fn contains_parameter(&self, name: Option<&JavaString>) -> bool {
        name.is_some_and(|name| self.parameters.contains_key(&name.to_string_lossy()))
    }
    fn get_parameter_count(&self) -> i32 {
        self.parameters.len() as i32
    }
    fn get_all_parameter_names(&self) -> Vec<Option<JavaString>> {
        self.parameters
            .keys()
            .map(|name| Some(JavaString::from_rust_str(name)))
            .collect()
    }
    fn get_parameter_map(&self) -> IndexMap<Option<JavaString>, Option<Vec<Option<JavaString>>>> {
        self.parameters
            .iter()
            .map(|(name, value)| {
                (
                    Some(JavaString::from_rust_str(name)),
                    Some(vec![Some(JavaString::from_rust_str(value))]),
                )
            })
            .collect()
    }
    fn get_parameter_values(&self, name: Option<&JavaString>) -> Option<Vec<Option<JavaString>>> {
        name.and_then(|name| {
            self.parameters
                .get(&name.to_string_lossy())
                .map(|value| vec![Some(JavaString::from_rust_str(value))])
        })
    }
    fn contains_cookie(&self, _name: Option<&JavaString>) -> bool {
        false
    }
    fn get_cookie_count(&self) -> i32 {
        0
    }
    fn get_all_cookie_names(&self) -> Vec<Option<JavaString>> {
        Vec::new()
    }
    fn get_cookie_map(&self) -> IndexMap<Option<JavaString>, Option<Vec<Option<JavaString>>>> {
        IndexMap::new()
    }
    fn get_cookie_values(&self, _name: Option<&JavaString>) -> Option<Vec<Option<JavaString>>> {
        None
    }
}
