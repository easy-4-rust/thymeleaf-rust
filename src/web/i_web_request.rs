use indexmap::IndexMap;
use thiserror::Error;

use crate::util::JavaString;

/// Web 请求读取合同。
///
/// 对应 Java: `org.thymeleaf.web.IWebRequest`。
///
/// 路径字段保持宿主提供的已编码形式；Header、参数与 Cookie 的多值顺序不得改写。
pub trait IWebRequest {
    /// 返回 HTTP 方法。
    fn get_method(&self) -> Option<JavaString>;

    /// 根据 scheme 是否等于忽略大小写的 `https` 判断安全请求。
    fn is_secure(&self) -> bool {
        self.get_scheme()
            .is_some_and(|scheme| scheme.to_string_lossy().eq_ignore_ascii_case("https"))
    }

    /// 返回 URI scheme。
    fn get_scheme(&self) -> Option<JavaString>;
    /// 返回服务器名称。
    fn get_server_name(&self) -> Option<JavaString>;
    /// 返回服务器端口。
    fn get_server_port(&self) -> Option<i32>;
    /// 返回已编码的应用路径。
    fn get_application_path(&self) -> Option<JavaString>;
    /// 返回已编码的应用内路径。
    fn get_path_within_application(&self) -> Option<JavaString>;
    /// 返回不含问号的查询串。
    fn get_query_string(&self) -> Option<JavaString>;

    /// 拼接应用路径和应用内路径；null 分支按空串处理。
    fn get_request_path(&self) -> JavaString {
        let mut utf16 = self
            .get_application_path()
            .map_or_else(Vec::new, |value| value.as_utf16().to_vec());
        if let Some(value) = self.get_path_within_application() {
            utf16.extend_from_slice(value.as_utf16());
        }
        JavaString::from_utf16(utf16)
    }

    /// 按 Java 默认实现计算完整请求 URL。
    ///
    /// # 错误
    /// scheme、server name 或端口缺失时返回上游
    /// `UnsupportedOperationException` 对应错误。
    fn get_request_url(&self) -> Result<JavaString, WebRequestError> {
        let scheme = self.get_scheme();
        let server_name = self.get_server_name();
        let server_port = self.get_server_port();
        let request_path = self.get_request_path();
        let query_string = self.get_query_string();
        let (Some(scheme), Some(server_name), Some(server_port)) =
            (scheme, server_name, server_port)
        else {
            return Err(WebRequestError::UnsupportedOperation {
                message: "Request scheme, server name or port are null in this environment. Cannot compute request URL",
            });
        };

        let scheme_text = scheme.to_string_lossy();
        let default_port = (scheme_text == "http" && server_port == 80)
            || (scheme_text == "https" && server_port == 443);
        let mut result = scheme.as_utf16().to_vec();
        result.extend_from_slice(&[b':' as u16, b'/' as u16, b'/' as u16]);
        result.extend_from_slice(server_name.as_utf16());
        if !default_port {
            result.push(b':' as u16);
            result.extend(server_port.to_string().encode_utf16());
        }
        result.extend_from_slice(request_path.as_utf16());
        if let Some(query_string) = query_string {
            result.push(b'?' as u16);
            result.extend_from_slice(query_string.as_utf16());
        }
        Ok(JavaString::from_utf16(result))
    }

    /// 判断 Header 是否存在。
    fn contains_header(&self, name: Option<&JavaString>) -> bool;
    /// 返回 Header 数量。
    fn get_header_count(&self) -> i32;
    /// 返回 Header 名称快照。
    fn get_all_header_names(&self) -> Vec<Option<JavaString>>;
    /// 返回 Header 多值 Map 快照。
    fn get_header_map(&self) -> IndexMap<Option<JavaString>, Option<Vec<Option<JavaString>>>>;
    /// 返回第一个 Header 值。
    fn get_header_value(&self, name: Option<&JavaString>) -> Option<JavaString> {
        self.get_header_values(name)
            .and_then(|values| values.into_iter().next().flatten())
    }
    /// 返回 Header 的全部值。
    fn get_header_values(&self, name: Option<&JavaString>) -> Option<Vec<Option<JavaString>>>;

    /// 判断请求参数是否存在。
    fn contains_parameter(&self, name: Option<&JavaString>) -> bool;
    /// 返回请求参数数量。
    fn get_parameter_count(&self) -> i32;
    /// 返回请求参数名称快照。
    fn get_all_parameter_names(&self) -> Vec<Option<JavaString>>;
    /// 返回请求参数多值 Map 快照。
    fn get_parameter_map(&self) -> IndexMap<Option<JavaString>, Option<Vec<Option<JavaString>>>>;
    /// 返回第一个请求参数值。
    fn get_parameter_value(&self, name: Option<&JavaString>) -> Option<JavaString> {
        self.get_parameter_values(name)
            .and_then(|values| values.into_iter().next().flatten())
    }
    /// 返回请求参数的全部值。
    fn get_parameter_values(&self, name: Option<&JavaString>) -> Option<Vec<Option<JavaString>>>;

    /// 判断请求 Cookie 是否存在。
    fn contains_cookie(&self, name: Option<&JavaString>) -> bool;
    /// 返回请求 Cookie 数量。
    fn get_cookie_count(&self) -> i32;
    /// 返回请求 Cookie 名称快照。
    fn get_all_cookie_names(&self) -> Vec<Option<JavaString>>;
    /// 返回请求 Cookie 多值 Map 快照。
    fn get_cookie_map(&self) -> IndexMap<Option<JavaString>, Option<Vec<Option<JavaString>>>>;
    /// 返回第一个请求 Cookie 值。
    fn get_cookie_value(&self, name: Option<&JavaString>) -> Option<JavaString> {
        self.get_cookie_values(name)
            .and_then(|values| values.into_iter().next().flatten())
    }
    /// 返回请求 Cookie 的全部值。
    fn get_cookie_values(&self, name: Option<&JavaString>) -> Option<Vec<Option<JavaString>>>;
}

/// Web 请求默认计算方法的错误。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WebRequestError {
    /// Java `UnsupportedOperationException` 对应错误。
    #[error("{message}")]
    UnsupportedOperation {
        /// 固定上游错误消息。
        message: &'static str,
    },
}
