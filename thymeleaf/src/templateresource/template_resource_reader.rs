//! 文件、URL、ClassLoader 与 Web 应用模板资源共享的 Java 字符流兼容入口。

use std::io::{self, Read};

use super::charset_decoder::CharsetDecoder;
use super::transcoding_reader::TranscodingReader;

/// 按 Java `InputStreamReader` 规则把输入字节流包装为 UTF-8 字节读取器。
///
/// 对应 Java: `FileTemplateResource#reader()`、`UrlTemplateResource#reader()`、
/// `ClassLoaderTemplateResource#reader()` 与 `WebApplicationTemplateResource#reader()`。
///
/// # 参数
/// - `input`：已经成功打开的原始资源输入流。
/// - `character_encoding`：Java `characterEncoding`；`None` 或 Java 空白使用 UTF-8。
///
/// # 返回值
/// 成功时返回增量解码读取器。
///
/// # 错误
/// 字符集名称非法或不受支持时返回 I/O 错误；输入流已由调用方先行打开，以保留 Java
/// 的“资源打开错误优先于 charset 错误”顺序。
pub(crate) fn transcoding_reader(
    input: Box<dyn Read>,
    character_encoding: Option<&str>,
) -> io::Result<Box<dyn Read>> {
    let decoder = CharsetDecoder::for_name(character_encoding)?;
    Ok(Box::new(TranscodingReader::new(input, decoder)))
}

/// 判断字符串是否为空或全部由 Java `Character.isWhitespace` 字符组成。
///
/// 对应 Java: `org.thymeleaf.util.StringUtils#isEmptyOrWhitespace(Object)`。
///
/// # 参数
/// - `value`：待检查文本。
///
/// # 返回值
/// 空字符串或全 Java 空白返回 `true`，否则返回 `false`。
pub(crate) fn is_java_empty_or_whitespace(value: &str) -> bool {
    value.is_empty() || value.chars().all(is_java_whitespace)
}

fn is_java_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000D}'
            | '\u{001C}'..='\u{0020}'
            | '\u{1680}'
            | '\u{2000}'..='\u{2006}'
            | '\u{2008}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{205F}'
            | '\u{3000}'
    )
}
