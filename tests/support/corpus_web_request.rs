use indexmap::IndexMap;
use thymeleaf::util::JavaString;
use thymeleaf::web::IWebRequest;

/// 为上游语料提供空 HTTP 请求。
///
/// 对应 Java: `org.thymeleaf.testing.templateengine.context.web.WebProcessingContextBuilder`
/// 所创建的 `IWebRequest` 测试对象。
#[derive(Default)]
pub struct CorpusWebRequest;

impl IWebRequest for CorpusWebRequest {
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
        Some(JavaString::from_rust_str("/"))
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
    fn contains_parameter(&self, _name: Option<&JavaString>) -> bool {
        false
    }
    fn get_parameter_count(&self) -> i32 {
        0
    }
    fn get_all_parameter_names(&self) -> Vec<Option<JavaString>> {
        Vec::new()
    }
    fn get_parameter_map(&self) -> IndexMap<Option<JavaString>, Option<Vec<Option<JavaString>>>> {
        IndexMap::new()
    }
    fn get_parameter_values(&self, _name: Option<&JavaString>) -> Option<Vec<Option<JavaString>>> {
        None
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
