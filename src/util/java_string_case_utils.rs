use std::char::decode_utf16;

use super::{JavaLocale, JavaString};

/// Java `String` 的全字符串大小写转换适配。
///
/// 对应 Java: `java.lang.String#toLowerCase()`，由 HTML 名称规范化调用。
pub(crate) fn to_lower_case_default(value: &JavaString) -> JavaString {
    let locale = JavaLocale::get_default();
    let language = locale
        .to_language_tag()
        .to_string_lossy()
        .split(['-', '_'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let turkic = matches!(language.as_str(), "tr" | "az");
    let mut result = Vec::with_capacity(value.len());
    for decoded in decode_utf16(value.as_utf16().iter().copied()) {
        match decoded {
            Ok(character) if turkic && character == 'I' => {
                result.extend('\u{131}'.encode_utf16(&mut [0; 2]).iter().copied());
            }
            Ok(character) if turkic && character == '\u{130}' => {
                result.extend('i'.encode_utf16(&mut [0; 2]).iter().copied());
            }
            Ok(character) => {
                for lower in character.to_lowercase() {
                    let mut buffer = [0_u16; 2];
                    let encoded = lower.encode_utf16(&mut buffer);
                    result.extend_from_slice(encoded);
                }
            }
            Err(error) => result.push(error.unpaired_surrogate()),
        }
    }
    JavaString::from_utf16(result)
}
