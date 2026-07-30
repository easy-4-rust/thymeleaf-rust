use std::fmt::{Display, Formatter};

use encoding_rs::Encoding;
use indexmap::IndexMap;
use thiserror::Error;

use crate::TemplateMode;

const MIME_TYPES_HTML: [&str; 2] = ["text/html", "application/xhtml+xml"];
const MIME_TYPES_XML: [&str; 2] = ["application/xml", "text/xml"];
const MIME_TYPES_RSS: [&str; 1] = ["application/rss+xml"];
const MIME_TYPES_ATOM: [&str; 1] = ["application/atom+xml"];
const MIME_TYPES_JAVASCRIPT: [&str; 5] = [
    "application/javascript",
    "application/x-javascript",
    "application/ecmascript",
    "text/javascript",
    "text/ecmascript",
];
const MIME_TYPES_JSON: [&str; 1] = ["application/json"];
const MIME_TYPES_CSS: [&str; 1] = ["text/css"];
const MIME_TYPES_TEXT: [&str; 1] = ["text/plain"];
const MIME_TYPES_SSE: [&str; 1] = ["text/event-stream"];

/// Java `Charset` 在内容类型工具中的 Rust 值对象映射。
///
/// 对应 Java: `java.nio.charset.Charset`，用于
/// `org.thymeleaf.util.ContentTypeUtils` 的字符集参数和返回值。
///
/// 对象只暴露 Thymeleaf 在本工具中可观察到的规范名称语义；实际模板字节编解码仍由
/// 模板资源层负责。名称按 Java 的大小写不敏感规则解析，并保留 Java 必备字符集的
/// 规范拼写。
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Charset {
    canonical_name: String,
}

impl Charset {
    /// 按名称取得字符集。
    ///
    /// 对应 Java: `Charset#forName(String)`。
    ///
    /// # 参数
    /// - `charset_name`：Java 参数 `charsetName`。
    ///
    /// # 返回
    /// 名称受支持时返回使用规范名称的字符集值。
    ///
    /// # 错误
    /// 名称违反 Java 字符集名称语法或当前 Rust 字符集注册表不支持时返回错误。
    pub fn for_name(charset_name: &str) -> Result<Self, CharsetError> {
        validate_charset_name(charset_name)?;
        let canonical_name =
            canonical_charset_name(charset_name).ok_or_else(|| CharsetError::Unsupported {
                charset_name: charset_name.to_owned(),
            })?;
        Ok(Self {
            canonical_name: canonical_name.to_owned(),
        })
    }

    /// 返回字符集的规范名称。
    ///
    /// 对应 Java: `Charset#name()`。
    ///
    /// # 返回
    /// 可写入 MIME `charset` 参数的规范名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.canonical_name
    }
}

impl Display for Charset {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.name())
    }
}

/// 字符集名称解析错误。
///
/// 对应 Java: `IllegalCharsetNameException` 与 `UnsupportedCharsetException`。
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum CharsetError {
    /// 名称为空或含 Java 不允许的字符。
    #[error("{charset_name}")]
    Illegal {
        /// 产生错误的原始名称。
        charset_name: String,
    },
    /// 名称合法但当前字符集注册表不支持。
    #[error("{charset_name}")]
    Unsupported {
        /// 产生错误的原始名称。
        charset_name: String,
    },
}

/// 内容类型识别、模板模式推导和字符集合并工具。
///
/// 对应 Java: `org.thymeleaf.util.ContentTypeUtils`。
///
/// XHTML 按上游约定归入 HTML；RSS 与 Atom 按 XML 处理；JSON 按 JavaScript
/// 文本模式处理。模板名称扩展名会转为小写并执行 Java `trim()`，请求路径扩展名则
/// 故意保持原始大小写和空白。
pub struct ContentTypeUtils;

impl ContentTypeUtils {
    /// 判断内容类型是否属于 HTML 族。
    ///
    /// 对应 Java: `ContentTypeUtils#isContentTypeHTML(String)`。
    ///
    /// # 参数
    /// - `content_type`：Java 参数 `contentType`；`None` 对应 `null`。
    ///
    /// # 错误
    /// 非空输入没有任何分号分隔令牌时保留 Java 数组越界错误。
    pub fn is_content_type_html(content_type: Option<&str>) -> Result<bool, ContentTypeError> {
        is_content_type(content_type, MIME_TYPES_HTML[0])
    }

    /// 判断内容类型是否属于 XML 族。
    ///
    /// 对应 Java: `ContentTypeUtils#isContentTypeXML(String)`。
    ///
    /// `content_type` 对应 Java 参数 `contentType`；归一化后属于 XML 族时返回
    /// `true`，解析失败返回 `ContentTypeError`。
    pub fn is_content_type_xml(content_type: Option<&str>) -> Result<bool, ContentTypeError> {
        is_content_type(content_type, MIME_TYPES_XML[0])
    }

    /// 判断内容类型是否为 RSS。
    ///
    /// 对应 Java: `ContentTypeUtils#isContentTypeRSS(String)`。
    ///
    /// `content_type` 对应 Java 参数 `contentType`；匹配 RSS MIME 时返回 `true`，
    /// 解析失败返回 `ContentTypeError`。
    pub fn is_content_type_rss(content_type: Option<&str>) -> Result<bool, ContentTypeError> {
        is_content_type(content_type, MIME_TYPES_RSS[0])
    }

    /// 判断内容类型是否为 Atom。
    ///
    /// 对应 Java: `ContentTypeUtils#isContentTypeAtom(String)`。
    ///
    /// `content_type` 对应 Java 参数 `contentType`；匹配 Atom MIME 时返回 `true`，
    /// 解析失败返回 `ContentTypeError`。
    pub fn is_content_type_atom(content_type: Option<&str>) -> Result<bool, ContentTypeError> {
        is_content_type(content_type, MIME_TYPES_ATOM[0])
    }

    /// 判断内容类型是否属于 JavaScript 族。
    ///
    /// 对应 Java: `ContentTypeUtils#isContentTypeJavaScript(String)`。
    ///
    /// `content_type` 对应 Java 参数 `contentType`；匹配任一 JavaScript 别名时返回
    /// `true`，解析失败返回 `ContentTypeError`。
    pub fn is_content_type_java_script(
        content_type: Option<&str>,
    ) -> Result<bool, ContentTypeError> {
        is_content_type(content_type, MIME_TYPES_JAVASCRIPT[0])
    }

    /// 判断内容类型是否为 JSON。
    ///
    /// 对应 Java: `ContentTypeUtils#isContentTypeJSON(String)`。
    ///
    /// `content_type` 对应 Java 参数 `contentType`；匹配 JSON MIME 时返回 `true`，
    /// 解析失败返回 `ContentTypeError`。
    pub fn is_content_type_json(content_type: Option<&str>) -> Result<bool, ContentTypeError> {
        is_content_type(content_type, MIME_TYPES_JSON[0])
    }

    /// 判断内容类型是否为 CSS。
    ///
    /// 对应 Java: `ContentTypeUtils#isContentTypeCSS(String)`。
    ///
    /// `content_type` 对应 Java 参数 `contentType`；匹配 CSS MIME 时返回 `true`，
    /// 解析失败返回 `ContentTypeError`。
    pub fn is_content_type_css(content_type: Option<&str>) -> Result<bool, ContentTypeError> {
        is_content_type(content_type, MIME_TYPES_CSS[0])
    }

    /// 判断内容类型是否为纯文本。
    ///
    /// 对应 Java: `ContentTypeUtils#isContentTypeText(String)`。
    ///
    /// `content_type` 对应 Java 参数 `contentType`；匹配纯文本 MIME 时返回 `true`，
    /// 解析失败返回 `ContentTypeError`。
    pub fn is_content_type_text(content_type: Option<&str>) -> Result<bool, ContentTypeError> {
        is_content_type(content_type, MIME_TYPES_TEXT[0])
    }

    /// 判断内容类型是否为 Server-Sent Events。
    ///
    /// 对应 Java: `ContentTypeUtils#isContentTypeSSE(String)`。
    ///
    /// `content_type` 对应 Java 参数 `contentType`；匹配 SSE MIME 时返回 `true`，
    /// 解析失败返回 `ContentTypeError`。
    pub fn is_content_type_sse(content_type: Option<&str>) -> Result<bool, ContentTypeError> {
        is_content_type(content_type, MIME_TYPES_SSE[0])
    }

    /// 根据 MIME 内容类型推导模板模式。
    ///
    /// 对应 Java: `ContentTypeUtils#computeTemplateModeForContentType(String)`。
    ///
    /// # 返回
    /// 空白或未知 MIME 返回 `None`；已知 MIME 返回对应模式。
    pub fn compute_template_mode_for_content_type(
        content_type: Option<&str>,
    ) -> Result<Option<TemplateMode>, ContentTypeError> {
        let Some(content_type) = parse_nonblank_content_type(content_type)? else {
            return Ok(None);
        };
        Ok(normalized_mime_type(content_type.mime_type()).and_then(template_mode_for_mime_type))
    }

    /// 根据模板名称的最后一个扩展名推导模板模式。
    ///
    /// 对应 Java: `ContentTypeUtils#computeTemplateModeForTemplateName(String)`。
    ///
    /// `template_name` 对应 Java 参数 `templateName`；已识别时返回模板模式，否则
    /// 返回 `None`。
    #[must_use]
    pub fn compute_template_mode_for_template_name(
        template_name: Option<&str>,
    ) -> Option<TemplateMode> {
        normalized_template_extension(template_name)
            .as_deref()
            .and_then(mime_type_for_extension)
            .and_then(template_mode_for_mime_type)
    }

    /// 根据请求路径最后一段的扩展名推导模板模式。
    ///
    /// 对应 Java: `ContentTypeUtils#computeTemplateModeForRequestPath(String)`。
    ///
    /// 查询串、片段和矩阵参数会依次剥离；请求扩展名保持大小写。
    /// `request_path` 对应 Java 参数 `requestPath`；返回已推导模式或 `None`，
    /// null 路径返回 `ContentTypeError`。
    pub fn compute_template_mode_for_request_path(
        request_path: Option<&str>,
    ) -> Result<Option<TemplateMode>, ContentTypeError> {
        Ok(compute_file_extension_from_request_path(request_path)?
            .and_then(mime_type_for_extension)
            .and_then(template_mode_for_mime_type))
    }

    /// 判断模板名称是否带有受支持的扩展名。
    ///
    /// 对应 Java: `ContentTypeUtils#hasRecognizedFileExtension(String)`。
    ///
    /// `template_name` 对应 Java 参数 `templateName`；扩展名受支持时返回 `true`。
    #[must_use]
    pub fn has_recognized_file_extension(template_name: Option<&str>) -> bool {
        normalized_template_extension(template_name)
            .is_some_and(|extension| mime_type_for_extension(&extension).is_some())
    }

    /// 根据模板名称计算内容类型，并可附加字符集。
    ///
    /// 对应 Java:
    /// `ContentTypeUtils#computeContentTypeForTemplateName(String, Charset)`。
    ///
    /// `template_name` 与 `charset` 对应同名 Java 参数；返回 MIME 与可选字符集，
    /// 无已知扩展名时返回 `None`。
    #[must_use]
    pub fn compute_content_type_for_template_name(
        template_name: Option<&str>,
        charset: Option<&Charset>,
    ) -> Option<String> {
        let extension = normalized_template_extension(template_name)?;
        let mime_type = mime_type_for_extension(&extension)?;
        Some(format_content_type(mime_type, charset))
    }

    /// 根据请求路径计算内容类型，并可附加字符集。
    ///
    /// 对应 Java:
    /// `ContentTypeUtils#computeContentTypeForRequestPath(String, Charset)`。
    ///
    /// `request_path` 与 `charset` 对应同名 Java 参数；返回 MIME 与可选字符集，
    /// 无已知扩展名时返回 `None`，null 路径返回 `ContentTypeError`。
    pub fn compute_content_type_for_request_path(
        request_path: Option<&str>,
        charset: Option<&Charset>,
    ) -> Result<Option<String>, ContentTypeError> {
        let Some(mime_type) = compute_file_extension_from_request_path(request_path)?
            .and_then(mime_type_for_extension)
        else {
            return Ok(None);
        };
        Ok(Some(format_content_type(mime_type, charset)))
    }

    /// 从内容类型的 `charset` 参数计算字符集。
    ///
    /// 对应 Java: `ContentTypeUtils#computeCharsetFromContentType(String)`。
    ///
    /// 不支持的合法名称与缺少参数都返回 `None`；非法名称保留为类型化错误。
    /// `content_type` 对应 Java 参数 `contentType`。
    pub fn compute_charset_from_content_type(
        content_type: Option<&str>,
    ) -> Result<Option<Charset>, ContentTypeError> {
        let Some(content_type) = parse_nonblank_content_type(content_type)? else {
            return Ok(None);
        };
        content_type
            .charset()
            .map_err(ContentTypeError::InvalidCharset)
    }

    /// 将内容类型与字符集合并。
    ///
    /// 对应 Java:
    /// `ContentTypeUtils#combineContentTypeAndCharset(String, Charset)`。
    ///
    /// 字符集为空时原样返回内容类型；否则解析并保留参数插入顺序，覆盖已有
    /// `charset` 的值但不改变其位置。
    /// `content_type` 与 `charset` 对应同名 Java 参数；返回合并后的可选内容类型，
    /// 解析失败返回 `ContentTypeError`。
    pub fn combine_content_type_and_charset(
        content_type: Option<&str>,
        charset: Option<&Charset>,
    ) -> Result<Option<String>, ContentTypeError> {
        let Some(charset) = charset else {
            return Ok(content_type.map(str::to_owned));
        };
        let Some(mut content_type) = parse_nonblank_content_type(content_type)? else {
            return Ok(None);
        };
        content_type.set_charset(Some(charset));
        Ok(Some(content_type.to_string()))
    }
}

/// 内容类型解析期间可观察到的 Java 兼容错误。
///
/// 对应 Java: `ArrayIndexOutOfBoundsException`、`NullPointerException` 与
/// `IllegalCharsetNameException`。
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ContentTypeError {
    /// Java `StringUtils.split` 没有产生任何令牌。
    #[error("Index 0 out of bounds for length 0")]
    MissingMimeType,
    /// 请求路径参数为 Java `null`。
    #[error("Cannot invoke \"String.indexOf(int)\" because \"<local1>\" is null")]
    NullRequestPath,
    /// `charset` 参数不是合法的 Java 字符集名称。
    #[error(transparent)]
    InvalidCharset(#[from] CharsetError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ContentType {
    mime_type: String,
    parameters: IndexMap<String, String>,
}

impl ContentType {
    fn parse_content_type(content_type: Option<&str>) -> Result<Option<Self>, ContentTypeError> {
        let Some(content_type) = content_type else {
            return Ok(None);
        };
        if java_trim(content_type).is_empty() {
            return Ok(None);
        }

        // 上游 StringUtils.split 基于 StringTokenizer，忽略连续、前导和尾随分隔符。
        let mut tokens = content_type.split(';').filter(|token| !token.is_empty());
        let mime_type = tokens
            .next()
            .ok_or(ContentTypeError::MissingMimeType)?
            .to_lowercase();
        let mime_type = java_trim(&mime_type).to_owned();
        let mut parameters = IndexMap::with_capacity(2);
        for token in tokens {
            let token = java_trim(&token.to_lowercase()).to_owned();
            if let Some(equal_position) = token.find('=') {
                parameters.insert(
                    java_trim(&token[..equal_position]).to_owned(),
                    java_trim(&token[equal_position + 1..]).to_owned(),
                );
            } else {
                parameters.insert(java_trim(&token).to_owned(), String::new());
            }
        }
        Ok(Some(Self {
            mime_type,
            parameters,
        }))
    }

    fn mime_type(&self) -> &str {
        &self.mime_type
    }

    #[allow(
        dead_code,
        reason = "保留解析后的 MIME 参数视图，供后续 Java 对照诊断使用"
    )]
    fn parameters(&self) -> &IndexMap<String, String> {
        &self.parameters
    }

    fn charset(&self) -> Result<Option<Charset>, CharsetError> {
        let Some(charset_name) = self.parameters.get("charset") else {
            return Ok(None);
        };
        match Charset::for_name(charset_name) {
            Ok(charset) => Ok(Some(charset)),
            Err(CharsetError::Unsupported { .. }) => Ok(None),
            Err(error @ CharsetError::Illegal { .. }) => Err(error),
        }
    }

    fn set_charset(&mut self, charset: Option<&Charset>) {
        if let Some(charset) = charset {
            self.parameters
                .insert("charset".to_owned(), charset.name().to_owned());
        }
    }
}

impl Display for ContentType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.mime_type)?;
        for (name, value) in &self.parameters {
            write!(formatter, ";{name}={value}")?;
        }
        Ok(())
    }
}

fn parse_nonblank_content_type(
    content_type: Option<&str>,
) -> Result<Option<ContentType>, ContentTypeError> {
    ContentType::parse_content_type(content_type)
}

fn is_content_type(content_type: Option<&str>, matcher: &str) -> Result<bool, ContentTypeError> {
    let Some(content_type) = parse_nonblank_content_type(content_type)? else {
        return Ok(false);
    };
    Ok(normalized_mime_type(content_type.mime_type()) == Some(matcher))
}

fn normalized_mime_type(mime_type: &str) -> Option<&'static str> {
    if MIME_TYPES_HTML.contains(&mime_type) {
        Some(MIME_TYPES_HTML[0])
    } else if MIME_TYPES_XML.contains(&mime_type) {
        Some(MIME_TYPES_XML[0])
    } else if MIME_TYPES_RSS.contains(&mime_type) {
        Some(MIME_TYPES_RSS[0])
    } else if MIME_TYPES_ATOM.contains(&mime_type) {
        Some(MIME_TYPES_ATOM[0])
    } else if MIME_TYPES_JAVASCRIPT.contains(&mime_type) {
        Some(MIME_TYPES_JAVASCRIPT[0])
    } else if MIME_TYPES_JSON.contains(&mime_type) {
        Some(MIME_TYPES_JSON[0])
    } else if MIME_TYPES_CSS.contains(&mime_type) {
        Some(MIME_TYPES_CSS[0])
    } else if MIME_TYPES_TEXT.contains(&mime_type) {
        Some(MIME_TYPES_TEXT[0])
    } else if MIME_TYPES_SSE.contains(&mime_type) {
        Some(MIME_TYPES_SSE[0])
    } else {
        None
    }
}

fn mime_type_for_extension(extension: &str) -> Option<&'static str> {
    match extension {
        ".html" | ".htm" | ".xhtml" => Some(MIME_TYPES_HTML[0]),
        ".xml" => Some(MIME_TYPES_XML[0]),
        ".rss" => Some(MIME_TYPES_RSS[0]),
        ".atom" => Some(MIME_TYPES_ATOM[0]),
        ".js" => Some(MIME_TYPES_JAVASCRIPT[0]),
        ".json" => Some(MIME_TYPES_JSON[0]),
        ".css" => Some(MIME_TYPES_CSS[0]),
        ".txt" => Some(MIME_TYPES_TEXT[0]),
        _ => None,
    }
}

fn template_mode_for_mime_type(mime_type: &str) -> Option<TemplateMode> {
    match mime_type {
        "text/html" => Some(TemplateMode::HTML),
        "application/xml" | "application/rss+xml" | "application/atom+xml" => {
            Some(TemplateMode::XML)
        }
        "application/javascript" | "application/json" => Some(TemplateMode::JAVASCRIPT),
        "text/css" => Some(TemplateMode::CSS),
        "text/plain" => Some(TemplateMode::TEXT),
        _ => None,
    }
}

fn normalized_template_extension(template_name: Option<&str>) -> Option<String> {
    let template_name = template_name?;
    if java_trim(template_name).is_empty() {
        return None;
    }
    let point_position = template_name.rfind('.')?;
    Some(java_trim(&template_name[point_position..].to_lowercase()).to_owned())
}

fn compute_file_extension_from_request_path(
    request_path: Option<&str>,
) -> Result<Option<&str>, ContentTypeError> {
    let mut path = request_path.ok_or(ContentTypeError::NullRequestPath)?;
    for delimiter in ['?', '#', ';'] {
        if let Some(position) = path.find(delimiter) {
            path = &path[..position];
        }
    }
    if let Some(position) = path.rfind('/') {
        path = &path[position + 1..];
    }
    Ok(path.rfind('.').map(|position| &path[position..]))
}

fn format_content_type(mime_type: &str, charset: Option<&Charset>) -> String {
    match charset {
        Some(charset) => format!("{mime_type};charset={}", charset.name()),
        None => mime_type.to_owned(),
    }
}

fn validate_charset_name(charset_name: &str) -> Result<(), CharsetError> {
    let mut characters = charset_name.chars();
    let Some(first) = characters.next() else {
        return Err(CharsetError::Illegal {
            charset_name: charset_name.to_owned(),
        });
    };
    if !first.is_ascii_alphanumeric()
        || characters.any(|character| {
            !character.is_ascii_alphanumeric() && !matches!(character, '-' | '+' | ':' | '_' | '.')
        })
    {
        return Err(CharsetError::Illegal {
            charset_name: charset_name.to_owned(),
        });
    }
    Ok(())
}

fn canonical_charset_name(charset_name: &str) -> Option<&'static str> {
    match charset_name.to_ascii_lowercase().as_str() {
        "us-ascii" | "ascii" | "iso646-us" | "646" => Some("US-ASCII"),
        "iso-8859-1" | "iso_8859-1" | "latin1" | "l1" | "ibm819" | "cp819" => Some("ISO-8859-1"),
        "utf-8" | "utf8" | "unicode-1-1-utf-8" => Some("UTF-8"),
        "utf-16" | "utf16" | "unicode" => Some("UTF-16"),
        "utf-16be" | "utf_16be" | "unicodebigunmarked" => Some("UTF-16BE"),
        "utf-16le" | "utf_16le" | "unicodelittleunmarked" => Some("UTF-16LE"),
        "utf-32" | "utf32" => Some("UTF-32"),
        "utf-32be" | "utf_32be" => Some("UTF-32BE"),
        "utf-32le" | "utf_32le" => Some("UTF-32LE"),
        "csiso2022kr" | "iso-2022-kr" => Some("ISO-2022-KR"),
        "iso-2022-cn" => Some("ISO-2022-CN"),
        "hz-gb-2312" | "iso-2022-cn-ext" | "replacement" => None,
        _ => Some(Encoding::for_label(charset_name.as_bytes())?.name()),
    }
}

fn java_trim(value: &str) -> &str {
    value.trim_matches(|character| character <= '\u{0020}')
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use super::{
        Charset, CharsetError, ContentType, ContentTypeError, ContentTypeUtils,
        normalized_template_extension,
    };
    use crate::TemplateMode;

    #[test]
    fn recognizes_every_mime_family_and_alias() {
        assert_eq!(
            ContentTypeUtils::is_content_type_html(Some(" APPLICATION/XHTML+XML ; q=1")),
            Ok(true)
        );
        assert_eq!(
            ContentTypeUtils::is_content_type_xml(Some("text/xml")),
            Ok(true)
        );
        assert_eq!(
            ContentTypeUtils::is_content_type_rss(Some("application/rss+xml")),
            Ok(true)
        );
        assert_eq!(
            ContentTypeUtils::is_content_type_atom(Some("application/atom+xml")),
            Ok(true)
        );
        assert_eq!(
            ContentTypeUtils::is_content_type_java_script(Some("text/ecmascript")),
            Ok(true)
        );
        assert_eq!(
            ContentTypeUtils::is_content_type_json(Some("application/json")),
            Ok(true)
        );
        assert_eq!(
            ContentTypeUtils::is_content_type_css(Some("text/css")),
            Ok(true)
        );
        assert_eq!(
            ContentTypeUtils::is_content_type_text(Some("text/plain")),
            Ok(true)
        );
        assert_eq!(
            ContentTypeUtils::is_content_type_sse(Some("text/event-stream")),
            Ok(true)
        );
        assert_eq!(
            ContentTypeUtils::is_content_type_html(Some("application/octet-stream")),
            Ok(false)
        );
        assert_eq!(ContentTypeUtils::is_content_type_html(None), Ok(false));
        assert_eq!(
            ContentTypeUtils::is_content_type_html(Some(";;;")),
            Err(ContentTypeError::MissingMimeType)
        );
    }

    #[test]
    fn computes_modes_from_mime_names_and_paths() {
        assert_eq!(
            ContentTypeUtils::compute_template_mode_for_content_type(Some("application/json")),
            Ok(Some(TemplateMode::JAVASCRIPT))
        );
        assert_eq!(
            ContentTypeUtils::compute_template_mode_for_content_type(Some("text/event-stream")),
            Ok(None)
        );
        assert_eq!(
            ContentTypeUtils::compute_template_mode_for_template_name(Some("views/INDEX.HTML ")),
            Some(TemplateMode::HTML)
        );
        assert_eq!(
            ContentTypeUtils::compute_template_mode_for_template_name(Some("archive.tar.xml")),
            Some(TemplateMode::XML)
        );
        assert_eq!(
            ContentTypeUtils::compute_template_mode_for_request_path(Some(
                "/asset/app.js?x=.css#part;v=1"
            )),
            Ok(Some(TemplateMode::JAVASCRIPT))
        );
        assert_eq!(
            ContentTypeUtils::compute_template_mode_for_request_path(Some("/INDEX.HTML")),
            Ok(None)
        );
        assert_eq!(
            ContentTypeUtils::compute_template_mode_for_request_path(None),
            Err(ContentTypeError::NullRequestPath)
        );
        assert_eq!(
            normalized_template_extension(Some("INDEX.HTML ")).as_deref(),
            Some(".html")
        );
    }

    #[test]
    fn computes_content_types_and_recognized_extensions() {
        let utf8 = Charset::for_name("utf8").unwrap();
        assert!(ContentTypeUtils::has_recognized_file_extension(Some(
            "view.HTML "
        )));
        assert!(!ContentTypeUtils::has_recognized_file_extension(Some(
            "view.unknown"
        )));
        assert_eq!(
            ContentTypeUtils::compute_content_type_for_template_name(
                Some("feed.atom"),
                Some(&utf8)
            ),
            Some("application/atom+xml;charset=UTF-8".to_owned())
        );
        assert_eq!(
            ContentTypeUtils::compute_content_type_for_request_path(Some("/style.css;v=2"), None),
            Ok(Some("text/css".to_owned()))
        );
        assert_eq!(
            ContentTypeUtils::compute_content_type_for_request_path(Some("/style.CSS"), None),
            Ok(None)
        );
    }

    #[test]
    fn parses_and_combines_parameters_in_java_order() {
        let mut parsed =
            ContentType::parse_content_type(Some(" Text/HTML ; Foo = A ; flag ; foo=B ")).unwrap();
        let mut parsed = parsed.take().unwrap();
        assert_eq!(parsed.mime_type(), "text/html");
        assert_eq!(
            parsed.parameters().get("flag").map(String::as_str),
            Some("")
        );
        assert_eq!(parsed.to_string(), "text/html;foo=b;flag=");
        parsed.set_charset(None);
        assert_eq!(parsed.to_string(), "text/html;foo=b;flag=");

        struct FailingWriter {
            remaining_successful_writes: usize,
        }

        impl Write for FailingWriter {
            fn write_str(&mut self, _: &str) -> std::fmt::Result {
                if self.remaining_successful_writes == 0 {
                    return Err(std::fmt::Error);
                }
                self.remaining_successful_writes -= 1;
                Ok(())
            }
        }

        for remaining_successful_writes in [0, 1] {
            let mut writer = FailingWriter {
                remaining_successful_writes,
            };
            assert_eq!(write!(&mut writer, "{parsed}"), Err(std::fmt::Error));
        }

        let utf16 = Charset::for_name("Unicode").unwrap();
        assert_eq!(
            ContentTypeUtils::combine_content_type_and_charset(
                Some("TEXT/HTML;CHARSET=us-ascii;q=1"),
                Some(&utf16)
            ),
            Ok(Some("text/html;charset=UTF-16;q=1".to_owned()))
        );
        assert_eq!(
            ContentTypeUtils::combine_content_type_and_charset(Some("RAW"), None),
            Ok(Some("RAW".to_owned()))
        );
    }

    #[test]
    fn preserves_charset_lookup_categories() {
        for (alias, canonical) in [
            ("US-ASCII", "US-ASCII"),
            ("ascii", "US-ASCII"),
            ("iso646-us", "US-ASCII"),
            ("646", "US-ASCII"),
            ("iso-8859-1", "ISO-8859-1"),
            ("iso_8859-1", "ISO-8859-1"),
            ("latin1", "ISO-8859-1"),
            ("l1", "ISO-8859-1"),
            ("ibm819", "ISO-8859-1"),
            ("cp819", "ISO-8859-1"),
            ("utf-8", "UTF-8"),
            ("utf8", "UTF-8"),
            ("unicode-1-1-utf-8", "UTF-8"),
            ("utf-16", "UTF-16"),
            ("utf16", "UTF-16"),
            ("unicode", "UTF-16"),
            ("utf-16be", "UTF-16BE"),
            ("utf_16be", "UTF-16BE"),
            ("unicodebigunmarked", "UTF-16BE"),
            ("utf-16le", "UTF-16LE"),
            ("utf_16le", "UTF-16LE"),
            ("unicodelittleunmarked", "UTF-16LE"),
            ("utf-32", "UTF-32"),
            ("utf32", "UTF-32"),
            ("utf-32be", "UTF-32BE"),
            ("utf_32be", "UTF-32BE"),
            ("utf-32le", "UTF-32LE"),
            ("utf_32le", "UTF-32LE"),
        ] {
            assert_eq!(Charset::for_name(alias).unwrap().name(), canonical);
        }
        assert_eq!(Charset::for_name("latin1").unwrap().name(), "ISO-8859-1");
        assert_eq!(
            Charset::for_name("latin1").unwrap().to_string(),
            "ISO-8859-1"
        );
        assert_eq!(Charset::for_name("utf-32le").unwrap().name(), "UTF-32LE");
        assert_eq!(
            Charset::for_name("\"utf-8\""),
            Err(CharsetError::Illegal {
                charset_name: "\"utf-8\"".to_owned()
            })
        );
        assert_eq!(
            Charset::for_name("x-no-such-charset"),
            Err(CharsetError::Unsupported {
                charset_name: "x-no-such-charset".to_owned()
            })
        );
        assert_eq!(
            Charset::for_name("replacement"),
            Err(CharsetError::Unsupported {
                charset_name: "replacement".to_owned()
            })
        );
        for invalid in ["", "-utf-8", "a/b", "a b", "a,b"] {
            assert_eq!(
                Charset::for_name(invalid),
                Err(CharsetError::Illegal {
                    charset_name: invalid.to_owned()
                })
            );
        }
        assert_eq!(
            Charset::for_name("a+b:c_d.e"),
            Err(CharsetError::Unsupported {
                charset_name: "a+b:c_d.e".to_owned()
            })
        );
        assert_eq!(
            ContentTypeUtils::compute_charset_from_content_type(Some("text/html;charset=UTF-8")),
            Ok(Some(Charset::for_name("UTF-8").unwrap()))
        );
        assert_eq!(
            ContentTypeUtils::compute_charset_from_content_type(Some(
                "text/html;charset=x-no-such-charset"
            )),
            Ok(None)
        );
        assert_eq!(
            ContentTypeUtils::compute_charset_from_content_type(Some(
                "text/html;charset=\"utf-8\""
            )),
            Err(ContentTypeError::InvalidCharset(CharsetError::Illegal {
                charset_name: "\"utf-8\"".to_owned()
            }))
        );
    }
}
