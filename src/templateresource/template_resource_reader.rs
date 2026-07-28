//! 文件与 URL 模板资源共享的 Java 字符流兼容层。
//!
//! 这是 Rust 内部实现协作者，不对应独立的 Thymeleaf Java 主对象；其语义来自
//! `FileTemplateResource#reader()`、`UrlTemplateResource#reader()` 与 JDK
//! `InputStreamReader`/`Charset`。

use std::io::{self, Read};

use encoding_rs::{Decoder, Encoding, UTF_8, UTF_16BE, UTF_16LE};

const INPUT_BUFFER_SIZE: usize = 8 * 1024;

/// 按 Java `InputStreamReader` 规则把输入字节流包装为 UTF-8 字节读取器。
///
/// # 参数
/// - `input`：已经成功打开的原始资源输入流。
/// - `character_encoding`：Java `characterEncoding`；`None` 或 Java 空白使用 UTF-8。
///
/// # 返回
/// 成功时返回增量解码读取器；字符集名称非法或不受支持时返回 I/O 错误。
pub(crate) fn transcoding_reader(
    input: Box<dyn Read>,
    character_encoding: Option<&str>,
) -> io::Result<Box<dyn Read>> {
    let decoder = JavaCharsetDecoder::for_name(character_encoding)?;
    Ok(Box::new(TranscodingReader::new(input, decoder)))
}

/// JDK `CharsetDecoder` 在当前迁移范围内的内部等价实现。
///
/// 对应 Java: `java.nio.charset.CharsetDecoder`，由 Thymeleaf 文件与 URL 资源间接使用。
pub(crate) enum JavaCharsetDecoder {
    EncodingRs(Decoder),
    Iso88591,
    UsAscii,
    Windows1252,
}

impl JavaCharsetDecoder {
    /// 按 Java `Charset.forName` 的名称、别名和空白规则创建解码器。
    ///
    /// # 参数
    /// - `character_encoding`：Java 字符集名称；`None` 或全 Java 空白采用 JDK 21 默认 UTF-8。
    ///
    /// # 返回
    /// 返回可增量解码的内部策略；非法或未知名称返回 `Unsupported` I/O 错误。
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
            // Java 的 UTF-16 会消费并遵循 BOM，无 BOM 时默认 big-endian。
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

    fn decode_chunk(&mut self, input: &[u8], last: bool) -> Vec<u8> {
        match self {
            Self::EncodingRs(decoder) => decode_encoding_rs(decoder, input, last),
            Self::Iso88591 => decode_iso_8859_1(input),
            Self::UsAscii => decode_us_ascii(input),
            Self::Windows1252 => decode_windows_1252(input),
        }
    }
}

/// 把 Java 字符流语义适配为 Rust UTF-8 `Read` 的增量读取器。
///
/// 对应 Java: `java.io.InputStreamReader`，由 Thymeleaf 资源对象持有而不对外发布。
pub(crate) struct TranscodingReader {
    input: Box<dyn Read>,
    decoder: JavaCharsetDecoder,
    decoded: Vec<u8>,
    decoded_position: usize,
    finished: bool,
}

impl TranscodingReader {
    /// 使用已打开输入流和已选择字符集创建增量读取器。
    ///
    /// # 参数
    /// - `input`：原始字节输入流。
    /// - `decoder`：按 Java 字符集规则创建的解码器。
    ///
    /// # 返回
    /// 从输入流当前位置开始读取的新适配器。
    pub(crate) fn new(input: Box<dyn Read>, decoder: JavaCharsetDecoder) -> Self {
        Self {
            input,
            decoder,
            decoded: Vec::new(),
            decoded_position: 0,
            finished: false,
        }
    }
}

impl Read for TranscodingReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }

        loop {
            if self.decoded_position < self.decoded.len() {
                let remaining = &self.decoded[self.decoded_position..];
                let length = remaining.len().min(output.len());
                output[..length].copy_from_slice(&remaining[..length]);
                self.decoded_position += length;
                return Ok(length);
            }
            if self.finished {
                return Ok(0);
            }

            let mut input_buffer = [0_u8; INPUT_BUFFER_SIZE];
            let read = self.input.read(&mut input_buffer)?;
            let last = read == 0;
            self.decoded = self.decoder.decode_chunk(&input_buffer[..read], last);
            self.decoded_position = 0;
            self.finished = last;
        }
    }
}

/// 判断字符串是否为空或全部由 Java `Character.isWhitespace` 字符组成。
///
/// # 参数
/// - `value`：待检查文本。
///
/// # 返回
/// 空字符串或全 Java 空白返回 `true`，否则返回 `false`。
pub(crate) fn is_java_empty_or_whitespace(value: &str) -> bool {
    value.is_empty() || value.chars().all(is_java_whitespace)
}

fn decode_encoding_rs(decoder: &mut Decoder, input: &[u8], last: bool) -> Vec<u8> {
    let capacity = decoder
        .max_utf8_buffer_length(input.len())
        .expect("8 KiB input cannot overflow the decoder output-size calculation")
        .max(16);
    let mut decoded = vec![0_u8; capacity];
    let (_, _, written, _) = decoder.decode_to_utf8(input, &mut decoded, last);

    // max_utf8_buffer_length 保证本次调用不会耗尽输出区。
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
        if byte.is_ascii() {
            decoded.push(char::from(*byte));
        } else {
            decoded.push('\u{FFFD}');
        }
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
