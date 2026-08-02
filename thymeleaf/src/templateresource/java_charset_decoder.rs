use std::io;

use encoding_rs::{Decoder, Encoding, UTF_8, UTF_16BE, UTF_16LE};

use super::template_resource_reader::is_java_empty_or_whitespace;

/// JDK `CharsetDecoder` 在模板资源迁移范围内的内部等价实现。
///
/// 对应 Java: `java.nio.charset.CharsetDecoder`，由 Thymeleaf 文件、URL、ClassLoader
/// 和 Web 应用资源通过 `InputStreamReader` 间接使用。
pub(crate) enum JavaCharsetDecoder {
    EncodingRs(Decoder),
    Iso88591,
    UsAscii,
    Windows1252,
}

impl JavaCharsetDecoder {
    /// 按 Java `Charset.forName` 的名称、别名和空白规则创建解码器。
    ///
    /// 对应 Java: `java.nio.charset.Charset#forName(String)`。
    ///
    /// # 参数
    /// - `character_encoding`：Java 字符集名称；`None` 或全 Java 空白采用 JDK 21
    ///   默认 UTF-8。
    ///
    /// # 返回值
    /// 返回可增量解码的内部策略。
    ///
    /// # 错误
    /// 字符集名称非法、带首尾 ASCII 空白或不受支持时返回 `Unsupported` I/O 错误。
    pub(crate) fn for_name(character_encoding: Option<&str>) -> io::Result<Self> {
        let Some(character_encoding) =
            character_encoding.filter(|value| !is_java_empty_or_whitespace(value))
        else {
            // JEP 400 自 Java 18 起规定默认 charset 为 UTF-8；这是当前 Thymeleaf
            // 基线运行环境的无显式编码语义。
            return Ok(Self::EncodingRs(UTF_8.new_decoder_without_bom_handling()));
        };

        // Charset.forName 不会修剪名称；encoding_rs 会修剪 ASCII 空白，因此先拒绝
        // 这种 Java IllegalCharsetNameException 场景。
        if character_encoding.trim_matches(is_ascii_charset_whitespace) != character_encoding {
            return Err(unsupported_encoding(character_encoding));
        }

        let normalized = character_encoding.to_ascii_lowercase();
        if is_iso_8859_1_alias(&normalized) {
            return Ok(Self::Iso88591);
        }
        if is_us_ascii_alias(&normalized) {
            return Ok(Self::UsAscii);
        }
        if is_windows_1252_alias(&normalized) {
            return Ok(Self::Windows1252);
        }
        if is_utf_16_alias(&normalized) {
            return Ok(Self::EncodingRs(UTF_16BE.new_decoder()));
        }
        if is_utf_16be_alias(&normalized) {
            return Ok(Self::EncodingRs(
                UTF_16BE.new_decoder_without_bom_handling(),
            ));
        }
        if is_utf_16le_alias(&normalized) {
            return Ok(Self::EncodingRs(
                UTF_16LE.new_decoder_without_bom_handling(),
            ));
        }
        if is_utf_8_alias(&normalized) {
            return Ok(Self::EncodingRs(UTF_8.new_decoder_without_bom_handling()));
        }

        let Some(encoding) = Encoding::for_label(character_encoding.as_bytes()) else {
            return Err(unsupported_encoding(character_encoding));
        };
        Ok(Self::EncodingRs(
            encoding.new_decoder_without_bom_handling(),
        ))
    }
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。

    pub(super) fn decode_chunk(&mut self, input: &[u8], last: bool) -> Vec<u8> {
        match self {
            Self::EncodingRs(decoder) => decode_encoding_rs(decoder, input, last),
            Self::Iso88591 => decode_iso_8859_1(input),
            Self::UsAscii => decode_us_ascii(input),
            Self::Windows1252 => decode_windows_1252(input),
        }
    }
}

fn decode_encoding_rs(decoder: &mut Decoder, input: &[u8], last: bool) -> Vec<u8> {
    let capacity = decoder
        .max_utf8_buffer_length(input.len())
        .expect("8 KiB input cannot overflow the decoder output-size calculation")
        .max(16);
    let mut decoded = vec![0_u8; capacity];
    let (_, _, written, _) = decoder.decode_to_utf8(input, &mut decoded, last);
    decoded.truncate(written);
    decoded
}

fn decode_iso_8859_1(input: &[u8]) -> Vec<u8> {
    let mut decoded = String::with_capacity(input.len().saturating_mul(2));
    for byte in input {
        decoded.push(char::from(*byte));
    }
    decoded.into_bytes()
}

fn decode_us_ascii(input: &[u8]) -> Vec<u8> {
    let mut decoded = String::with_capacity(input.len().saturating_mul(3));
    for byte in input {
        decoded.push(if byte.is_ascii() {
            char::from(*byte)
        } else {
            '\u{FFFD}'
        });
    }
    decoded.into_bytes()
}

fn decode_windows_1252(input: &[u8]) -> Vec<u8> {
    let mut decoded = String::with_capacity(input.len().saturating_mul(3));
    for byte in input {
        let character = match byte {
            0x80 => '\u{20AC}',
            0x81 | 0x8D | 0x8F | 0x90 | 0x9D => '\u{FFFD}',
            0x82 => '\u{201A}',
            0x83 => '\u{0192}',
            0x84 => '\u{201E}',
            0x85 => '\u{2026}',
            0x86 => '\u{2020}',
            0x87 => '\u{2021}',
            0x88 => '\u{02C6}',
            0x89 => '\u{2030}',
            0x8A => '\u{0160}',
            0x8B => '\u{2039}',
            0x8C => '\u{0152}',
            0x8E => '\u{017D}',
            0x91 => '\u{2018}',
            0x92 => '\u{2019}',
            0x93 => '\u{201C}',
            0x94 => '\u{201D}',
            0x95 => '\u{2022}',
            0x96 => '\u{2013}',
            0x97 => '\u{2014}',
            0x98 => '\u{02DC}',
            0x99 => '\u{2122}',
            0x9A => '\u{0161}',
            0x9B => '\u{203A}',
            0x9C => '\u{0153}',
            0x9E => '\u{017E}',
            0x9F => '\u{0178}',
            value => char::from(*value),
        };
        decoded.push(character);
    }
    decoded.into_bytes()
}

fn unsupported_encoding(character_encoding: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, character_encoding.to_owned())
}

fn is_ascii_charset_whitespace(character: char) -> bool {
    matches!(character, '\u{0009}'..='\u{000D}' | '\u{0020}')
}

fn is_utf_8_alias(name: &str) -> bool {
    matches!(name, "utf-8" | "utf8" | "unicode-1-1-utf-8")
}

fn is_utf_16_alias(name: &str) -> bool {
    matches!(name, "utf-16" | "utf_16" | "utf16" | "unicode")
}

fn is_utf_16be_alias(name: &str) -> bool {
    matches!(
        name,
        "utf-16be"
            | "utf_16be"
            | "utf16be"
            | "iso-10646-ucs-2"
            | "x-utf-16be"
            | "unicodebigunmarked"
    )
}

fn is_utf_16le_alias(name: &str) -> bool {
    matches!(
        name,
        "utf-16le" | "utf_16le" | "utf16le" | "x-utf-16le" | "unicodelittleunmarked"
    )
}

fn is_iso_8859_1_alias(name: &str) -> bool {
    matches!(
        name,
        "iso-8859-1"
            | "iso8859_1"
            | "iso_8859-1:1987"
            | "iso_8859-1"
            | "8859_1"
            | "iso8859-1"
            | "iso88591"
            | "latin1"
            | "l1"
            | "ibm819"
            | "cp819"
            | "csisolatin1"
            | "819"
    )
}

fn is_us_ascii_alias(name: &str) -> bool {
    matches!(
        name,
        "us-ascii" | "ascii" | "iso646-us" | "646" | "cp367" | "csascii"
    )
}

fn is_windows_1252_alias(name: &str) -> bool {
    matches!(name, "windows-1252" | "cp1252" | "cp-1252" | "ibm-1252")
}
