use encoding_rs::Encoding;
use thiserror::Error;

use crate::util::JavaString;

/// URI path、segment、fragment 与 query 参数转义工具。
///
/// 对应 Java: `org.thymeleaf.expression.Uris`。
pub struct Uris;

impl Uris {
    /// 创建无状态 URI 工具对象。
    pub const fn new() -> Self {
        Self
    }

    /// 使用 UTF-8 转义完整 URI path。
    pub fn escape_path(
        &self,
        text: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        escape(text, None, UriComponent::Path)
    }

    /// 使用指定字符集转义完整 URI path。
    pub fn escape_path_with_encoding(
        &self,
        text: Option<&JavaString>,
        encoding: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        escape(text, encoding, UriComponent::Path)
    }

    /// 使用 UTF-8 反转义完整 URI path。
    pub fn unescape_path(
        &self,
        text: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        unescape(text, None)
    }

    /// 使用指定字符集反转义完整 URI path。
    pub fn unescape_path_with_encoding(
        &self,
        text: Option<&JavaString>,
        encoding: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        unescape(text, encoding)
    }

    /// 使用 UTF-8 转义单个 URI path segment。
    pub fn escape_path_segment(
        &self,
        text: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        escape(text, None, UriComponent::PathSegment)
    }

    /// 使用指定字符集转义单个 URI path segment。
    pub fn escape_path_segment_with_encoding(
        &self,
        text: Option<&JavaString>,
        encoding: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        escape(text, encoding, UriComponent::PathSegment)
    }

    /// 使用 UTF-8 反转义 URI path segment。
    pub fn unescape_path_segment(
        &self,
        text: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        unescape(text, None)
    }

    /// 使用指定字符集反转义 URI path segment。
    pub fn unescape_path_segment_with_encoding(
        &self,
        text: Option<&JavaString>,
        encoding: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        unescape(text, encoding)
    }

    /// 使用 UTF-8 转义 URI fragment identifier。
    pub fn escape_fragment_id(
        &self,
        text: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        escape(text, None, UriComponent::Fragment)
    }

    /// 使用指定字符集转义 URI fragment identifier。
    pub fn escape_fragment_id_with_encoding(
        &self,
        text: Option<&JavaString>,
        encoding: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        escape(text, encoding, UriComponent::Fragment)
    }

    /// 使用 UTF-8 反转义 URI fragment identifier。
    pub fn unescape_fragment_id(
        &self,
        text: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        unescape(text, None)
    }

    /// 使用指定字符集反转义 URI fragment identifier。
    pub fn unescape_fragment_id_with_encoding(
        &self,
        text: Option<&JavaString>,
        encoding: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        unescape(text, encoding)
    }

    /// 使用 UTF-8 转义 URI query parameter 名称或值。
    pub fn escape_query_param(
        &self,
        text: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        escape(text, None, UriComponent::QueryParameter)
    }

    /// 使用指定字符集转义 URI query parameter。
    pub fn escape_query_param_with_encoding(
        &self,
        text: Option<&JavaString>,
        encoding: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        escape(text, encoding, UriComponent::QueryParameter)
    }

    /// 使用 UTF-8 反转义 URI query parameter。
    pub fn unescape_query_param(
        &self,
        text: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        unescape(text, None)
    }

    /// 使用指定字符集反转义 URI query parameter。
    pub fn unescape_query_param_with_encoding(
        &self,
        text: Option<&JavaString>,
        encoding: Option<&JavaString>,
    ) -> Result<Option<JavaString>, UriExpressionError> {
        unescape(text, encoding)
    }
}

impl Default for Uris {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
enum UriComponent {
    Path,
    PathSegment,
    Fragment,
    QueryParameter,
}

fn escape(
    text: Option<&JavaString>,
    encoding: Option<&JavaString>,
    component: UriComponent,
) -> Result<Option<JavaString>, UriExpressionError> {
    let Some(text) = text else {
        return Ok(None);
    };
    let encoding = resolve_encoding(encoding)?;
    let source = text.to_string_lossy();
    let (bytes, _, _) = encoding.encode(&source);
    let mut output = String::with_capacity(bytes.len());
    for byte in bytes.iter().copied() {
        if byte.is_ascii() && is_allowed(byte, component) {
            output.push(char::from(byte));
        } else {
            output.push('%');
            output.push(hex(byte >> 4));
            output.push(hex(byte & 0x0F));
        }
    }
    Ok(Some(JavaString::from_rust_str(&output)))
}

fn unescape(
    text: Option<&JavaString>,
    encoding: Option<&JavaString>,
) -> Result<Option<JavaString>, UriExpressionError> {
    let Some(text) = text else {
        return Ok(None);
    };
    let encoding = resolve_encoding(encoding)?;
    let source = text.to_string_lossy();
    if !source.as_bytes().contains(&b'%') {
        return Ok(Some(text.clone()));
    }
    let input = source.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut position = 0;
    while position < input.len() {
        if input[position] != b'%' {
            let character = source[position..]
                .chars()
                .next()
                .expect("position remains on UTF-8 boundary");
            output.push(character);
            position += character.len_utf8();
            continue;
        }
        let mut bytes = Vec::new();
        while position + 2 < input.len() && input[position] == b'%' {
            let Some(high) = from_hex(input[position + 1]) else {
                break;
            };
            let Some(low) = from_hex(input[position + 2]) else {
                break;
            };
            bytes.push((high << 4) | low);
            position += 3;
        }
        if bytes.is_empty() {
            output.push('%');
            position += 1;
        } else {
            let (decoded, _, _) = encoding.decode(&bytes);
            output.push_str(&decoded);
        }
    }
    Ok(Some(JavaString::from_rust_str(&output)))
}

fn resolve_encoding(
    encoding: Option<&JavaString>,
) -> Result<&'static Encoding, UriExpressionError> {
    let Some(encoding) = encoding else {
        return Ok(encoding_rs::UTF_8);
    };
    Encoding::for_label(encoding.to_string_lossy().as_bytes()).ok_or_else(|| {
        UriExpressionError::UnsupportedEncoding {
            encoding: encoding.to_string_lossy(),
        }
    })
}

fn is_allowed(byte: u8, component: UriComponent) -> bool {
    if byte.is_ascii_alphanumeric() || b"-._~!$'()*,:;@".contains(&byte) {
        return true;
    }
    match component {
        UriComponent::Path => matches!(byte, b'&' | b'+' | b'=' | b'/'),
        UriComponent::PathSegment => matches!(byte, b'&' | b'+' | b'='),
        UriComponent::Fragment => matches!(byte, b'&' | b'+' | b'=' | b'/' | b'?'),
        UriComponent::QueryParameter => matches!(byte, b'/' | b'?'),
    }
}

fn hex(value: u8) -> char {
    char::from(if value < 10 {
        b'0' + value
    } else {
        b'A' + value - 10
    })
}

fn from_hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

/// URI 转义/反转义错误。
#[derive(Debug, Error, Eq, PartialEq)]
pub enum UriExpressionError {
    /// Java Charset 名称在当前运行时不可用。
    #[error("Unsupported encoding: {encoding}")]
    UnsupportedEncoding {
        /// 原字符集名称。
        encoding: String,
    },
}
