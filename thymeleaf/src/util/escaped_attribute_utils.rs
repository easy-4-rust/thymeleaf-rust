use html_escape::NAMED_ENTITIES;

use crate::TemplateMode;
use crate::exceptions::TemplateProcessingException;

use super::Utf16String;

/// 按模板模式转义或反转义元素属性值。
///
/// 对应 Java: `org.thymeleaf.util.EscapedAttributeUtils`。
///
/// HTML 分支保持 unbescape 的 HTML4/XML 转义及 HTML5 反转义语义；XML 分支保持
/// XML 1.0 属性规范化保护、非法码点过滤及实体规则；文本模式按 Java 原实现选择
/// HTML、JavaScript、CSS 或 RAW 规则。
pub struct EscapedAttributeUtils;

impl EscapedAttributeUtils {
    /// 按模板模式转义属性值。对应 Java: `EscapedAttributeUtils#escapeAttribute`。
    ///
    /// # 参数
    /// - `template_mode`：当前模板模式；`None` 对应 Java `null`。
    /// - `input`：原始属性值；`None` 对应 Java `null`。
    ///
    /// # 返回
    /// 空输入保持为空；HTML 与 XML 返回相应规则的已转义值。
    ///
    /// # 错误
    /// 非空输入配合空模式，或不支持输出属性的文本模式时返回处理异常。
    pub fn escape_attribute(
        template_mode: Option<TemplateMode>,
        input: Option<&Utf16String>,
    ) -> Result<Option<Utf16String>, TemplateProcessingException> {
        let Some(input) = input else {
            return Ok(None);
        };
        let template_mode = require_template_mode(template_mode)?;
        let output = match template_mode {
            TemplateMode::HTML => escape_html4_xml(input),
            TemplateMode::XML => escape_xml10_attribute(input),
            _ => {
                return Err(TemplateProcessingException::new(Some(format!(
                    "Unrecognized template mode {template_mode}. Cannot produce escaped attributes for this template mode."
                ))));
            }
        };
        Ok(Some(output))
    }

    /// 按模板模式反转义属性值。对应 Java: `EscapedAttributeUtils#unescapeAttribute`。
    ///
    /// # 参数
    /// - `template_mode`：当前模板模式；`None` 对应 Java `null`。
    /// - `input`：模板中的属性值；`None` 对应 Java `null`。
    ///
    /// # 返回
    /// 空输入保持为空；RAW 返回相同代码单元，其余模式按各自语言规则反转义。
    ///
    /// # 错误
    /// 非空输入配合空模板模式时返回处理异常。
    pub fn unescape_attribute(
        template_mode: Option<TemplateMode>,
        input: Option<&Utf16String>,
    ) -> Result<Option<Utf16String>, TemplateProcessingException> {
        let Some(input) = input else {
            return Ok(None);
        };
        let template_mode = require_template_mode(template_mode)?;
        let output = match template_mode {
            TemplateMode::TEXT | TemplateMode::HTML => unescape_html(input),
            TemplateMode::XML => unescape_xml(input),
            TemplateMode::JAVASCRIPT => unescape_javascript(input),
            TemplateMode::CSS => unescape_css(input),
            TemplateMode::RAW => input.clone(),
        };
        Ok(Some(output))
    }
}

fn require_template_mode(
    template_mode: Option<TemplateMode>,
) -> Result<TemplateMode, TemplateProcessingException> {
    template_mode.ok_or_else(|| {
        TemplateProcessingException::new(Some("Template mode cannot be null".to_owned()))
    })
}

fn escape_html4_xml(input: &Utf16String) -> Utf16String {
    let mut output = Vec::with_capacity(input.len());
    for unit in input.as_utf16() {
        match *unit {
            0x26 => output.extend("&amp;".encode_utf16()),
            0x3C => output.extend("&lt;".encode_utf16()),
            0x3E => output.extend("&gt;".encode_utf16()),
            0x22 => output.extend("&quot;".encode_utf16()),
            0x27 => output.extend("&#39;".encode_utf16()),
            value => output.push(value),
        }
    }
    Utf16String::from_utf16(output)
}

fn escape_xml10_attribute(input: &Utf16String) -> Utf16String {
    let units = input.as_utf16();
    let mut output = Vec::with_capacity(units.len());
    let mut index = 0;
    while index < units.len() {
        let first = units[index];
        let (codepoint, consumed) = decode_utf16_codepoint(units, index);
        index += consumed;

        // XML 1.0 非法码点会像 unbescape 一样被直接丢弃。
        if !is_valid_xml10_codepoint(codepoint, first, consumed) {
            continue;
        }
        match codepoint {
            0x09 | 0x0A | 0x0D => append_hex_reference(&mut output, codepoint),
            0x22 => output.extend("&quot;".encode_utf16()),
            0x26 => output.extend("&amp;".encode_utf16()),
            0x27 => output.extend("&apos;".encode_utf16()),
            0x3C => output.extend("&lt;".encode_utf16()),
            0x3E => output.extend("&gt;".encode_utf16()),
            0x80.. => append_hex_reference(&mut output, codepoint),
            _ => output.push(first),
        }
    }
    Utf16String::from_utf16(output)
}

fn unescape_html(input: &Utf16String) -> Utf16String {
    let units = input.as_utf16();
    let mut output = Vec::with_capacity(units.len());
    let mut index = 0;
    while index < units.len() {
        if units[index] != u16::from(b'&') || index + 1 >= units.len() {
            output.push(units[index]);
            index += 1;
            continue;
        }
        if let Some((decoded, consumed)) = decode_html_reference(&units[index..]) {
            output.extend(decoded);
            index += consumed;
        } else {
            output.push(units[index]);
            index += 1;
        }
    }
    Utf16String::from_utf16(output)
}

fn decode_html_reference(units: &[u16]) -> Option<(Vec<u16>, usize)> {
    if units.get(1) == Some(&u16::from(b'#')) {
        let (radix, start) = match units.get(2) {
            Some(value) if *value == u16::from(b'x') || *value == u16::from(b'X') => (16, 3),
            _ => (10, 2),
        };
        let end = take_digits(units, start, radix);
        if end == start {
            return None;
        }
        let consumed = end + usize::from(units.get(end) == Some(&u16::from(b';')));
        let codepoint = parse_codepoint(&units[start..end], radix);
        return Some((
            encode_codepoint(translate_html_codepoint(codepoint)),
            consumed,
        ));
    }

    let mut end = 1;
    while end < units.len() && is_ascii_alphanumeric(units[end]) {
        end += 1;
    }
    let has_semicolon = units.get(end) == Some(&u16::from(b';'));
    let candidate_end = end + usize::from(has_semicolon);
    for name_end in (2..=end).rev() {
        let name = ascii_bytes(&units[1..name_end])?;
        if let Ok(position) =
            NAMED_ENTITIES.binary_search_by(|(entry, _)| entry.cmp(&name.as_slice()))
        {
            let decoded = Utf16String::from_rust_str(NAMED_ENTITIES[position].1)
                .as_utf16()
                .to_vec();
            let consumed = if name_end == end && has_semicolon {
                candidate_end
            } else {
                name_end
            };
            return Some((decoded, consumed));
        }
    }
    None
}

fn unescape_xml(input: &Utf16String) -> Utf16String {
    unescape_ampersand_references(input, false)
}

fn unescape_ampersand_references(input: &Utf16String, html: bool) -> Utf16String {
    let units = input.as_utf16();
    let mut output = Vec::with_capacity(units.len());
    let mut index = 0;
    while index < units.len() {
        if units[index] != u16::from(b'&') {
            output.push(units[index]);
            index += 1;
            continue;
        }
        let remaining = &units[index..];
        let decoded = if html {
            decode_html_reference(remaining)
        } else {
            decode_xml_reference(remaining)
        };
        if let Some((value, consumed)) = decoded {
            output.extend(value);
            index += consumed;
        } else {
            output.push(units[index]);
            index += 1;
        }
    }
    Utf16String::from_utf16(output)
}

fn decode_xml_reference(units: &[u16]) -> Option<(Vec<u16>, usize)> {
    if units.get(1) == Some(&u16::from(b'#')) {
        let (radix, start) = match units.get(2) {
            Some(value) if *value == u16::from(b'x') => (16, 3),
            _ => (10, 2),
        };
        let end = take_digits(units, start, radix);
        if end == start || units.get(end) != Some(&u16::from(b';')) {
            return None;
        }
        return Some((
            encode_codepoint(parse_codepoint(&units[start..end], radix)),
            end + 1,
        ));
    }
    for (name, value) in [
        ("&amp;", 0x26),
        ("&apos;", 0x27),
        ("&gt;", 0x3E),
        ("&lt;", 0x3C),
        ("&quot;", 0x22),
    ] {
        let expected = name.encode_utf16().collect::<Vec<_>>();
        if units.starts_with(&expected) {
            return Some((vec![value], expected.len()));
        }
    }
    None
}

fn unescape_javascript(input: &Utf16String) -> Utf16String {
    let units = input.as_utf16();
    let mut output = Vec::with_capacity(units.len());
    let mut index = 0;
    while index < units.len() {
        if units[index] != u16::from(b'\\') || index + 1 >= units.len() {
            output.push(units[index]);
            index += 1;
            continue;
        }
        let next = units[index + 1];
        let simple = match next {
            0x30 if !is_javascript_octal(&units[index + 1..]) => Some(0x00),
            0x62 => Some(0x08),
            0x74 => Some(0x09),
            0x6E => Some(0x0A),
            0x76 => Some(0x0B),
            0x66 => Some(0x0C),
            0x72 => Some(0x0D),
            0x22 | 0x27 | 0x5C | 0x2F => Some(next),
            0x0A => {
                index += 2;
                continue;
            }
            _ => None,
        };
        if let Some(value) = simple {
            output.push(value);
            index += 2;
            continue;
        }
        if next == u16::from(b'x') || next == u16::from(b'u') {
            let digits = if next == u16::from(b'x') { 2 } else { 4 };
            let start = index + 2;
            let end = start + digits;
            if end <= units.len() && units[start..end].iter().all(|unit| is_digit(*unit, 16)) {
                output.push(parse_codepoint(&units[start..end], 16) as u16);
                index = end;
                continue;
            }
            output.extend_from_slice(&units[index..(index + 2).min(units.len())]);
            index += 2;
            continue;
        }
        if (u16::from(b'0')..=u16::from(b'7')).contains(&next) {
            let mut end = index + 2;
            while end < units.len() && end < index + 4 && is_digit(units[end], 8) {
                end += 1;
            }
            let mut value = parse_codepoint(&units[index + 1..end], 8);
            if value > 0xFF {
                end -= 1;
                value = parse_codepoint(&units[index + 1..end], 8);
            }
            output.push(value as u16);
            index = end;
            continue;
        }
        if matches!(next, 0x38 | 0x39 | 0x0D | 0x2028 | 0x2029) {
            output.extend_from_slice(&units[index..index + 2]);
        } else {
            output.push(next);
        }
        index += 2;
    }
    Utf16String::from_utf16(output)
}

fn unescape_css(input: &Utf16String) -> Utf16String {
    let units = input.as_utf16();
    let mut output = Vec::with_capacity(units.len());
    let mut index = 0;
    while index < units.len() {
        if units[index] != u16::from(b'\\') || index + 1 >= units.len() {
            output.push(units[index]);
            index += 1;
            continue;
        }
        let next = units[index + 1];
        if next == 0x0A {
            index += 2;
            continue;
        }
        if is_digit(next, 16) {
            let mut end = index + 2;
            while end < units.len() && end < index + 7 && is_digit(units[end], 16) {
                end += 1;
            }
            output.extend(encode_codepoint(parse_codepoint(
                &units[index + 1..end],
                16,
            )));
            if units.get(end) == Some(&u16::from(b' ')) {
                end += 1;
            }
            index = end;
            continue;
        }
        if matches!(next, 0x0D | 0x0C) {
            output.extend_from_slice(&units[index..index + 2]);
        } else {
            output.push(next);
        }
        index += 2;
    }
    Utf16String::from_utf16(output)
}

fn decode_utf16_codepoint(units: &[u16], index: usize) -> (u32, usize) {
    let first = units[index];
    if (0xD800..=0xDBFF).contains(&first)
        && units
            .get(index + 1)
            .is_some_and(|second| (0xDC00..=0xDFFF).contains(second))
    {
        let second = units[index + 1];
        return (
            0x10000 + ((u32::from(first) - 0xD800) << 10) + (u32::from(second) - 0xDC00),
            2,
        );
    }
    (u32::from(first), 1)
}

fn is_valid_xml10_codepoint(codepoint: u32, first: u16, consumed: usize) -> bool {
    if consumed == 1 && (0xD800..=0xDFFF).contains(&first) {
        return false;
    }
    matches!(codepoint, 0x09 | 0x0A | 0x0D | 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF)
}

fn append_hex_reference(output: &mut Vec<u16>, codepoint: u32) {
    output.extend(format!("&#x{codepoint:x};").encode_utf16());
}

fn take_digits(units: &[u16], start: usize, radix: u32) -> usize {
    let mut end = start;
    while end < units.len() && is_digit(units[end], radix) {
        end += 1;
    }
    end
}

fn is_digit(unit: u16, radix: u32) -> bool {
    match radix {
        8 => (u16::from(b'0')..=u16::from(b'7')).contains(&unit),
        10 => (u16::from(b'0')..=u16::from(b'9')).contains(&unit),
        16 => {
            (u16::from(b'0')..=u16::from(b'9')).contains(&unit)
                || (u16::from(b'a')..=u16::from(b'f')).contains(&unit)
                || (u16::from(b'A')..=u16::from(b'F')).contains(&unit)
        }
        _ => false,
    }
}

fn parse_codepoint(units: &[u16], radix: u32) -> u32 {
    units.iter().fold(0_u32, |value, unit| {
        value
            .checked_mul(radix)
            .and_then(|value| value.checked_add(digit_value(*unit)))
            .unwrap_or(0xFFFD)
    })
}

fn digit_value(unit: u16) -> u32 {
    match unit {
        value if (u16::from(b'0')..=u16::from(b'9')).contains(&value) => {
            u32::from(value - u16::from(b'0'))
        }
        value if (u16::from(b'a')..=u16::from(b'f')).contains(&value) => {
            10 + u32::from(value - u16::from(b'a'))
        }
        value => 10 + u32::from(value - u16::from(b'A')),
    }
}

fn encode_codepoint(codepoint: u32) -> Vec<u16> {
    if codepoint <= 0xFFFF {
        return vec![codepoint as u16];
    }
    if codepoint <= 0x10FFFF {
        let adjusted = codepoint - 0x10000;
        return vec![
            (0xD800 + (adjusted >> 10)) as u16,
            (0xDC00 + (adjusted & 0x3FF)) as u16,
        ];
    }
    vec![0xFFFD]
}

fn translate_html_codepoint(codepoint: u32) -> u32 {
    match codepoint {
        0x00 => 0xFFFD,
        0x80 => 0x20AC,
        0x82 => 0x201A,
        0x83 => 0x0192,
        0x84 => 0x201E,
        0x85 => 0x2026,
        0x86 => 0x2020,
        0x87 => 0x2021,
        0x88 => 0x02C6,
        0x89 => 0x2030,
        0x8A => 0x0160,
        0x8B => 0x2039,
        0x8C => 0x0152,
        0x8E => 0x017D,
        0x91 => 0x2018,
        0x92 => 0x2019,
        0x93 => 0x201C,
        0x94 => 0x201D,
        0x95 => 0x2022,
        0x96 => 0x2013,
        0x97 => 0x2014,
        0x98 => 0x02DC,
        0x99 => 0x2122,
        0x9A => 0x0161,
        0x9B => 0x203A,
        0x9C => 0x0153,
        0x9E => 0x017E,
        0x9F => 0x0178,
        0xD800..=0xDFFF | 0x110000.. => 0xFFFD,
        value => value,
    }
}

fn is_ascii_alphanumeric(unit: u16) -> bool {
    (u16::from(b'0')..=u16::from(b'9')).contains(&unit)
        || (u16::from(b'a')..=u16::from(b'z')).contains(&unit)
        || (u16::from(b'A')..=u16::from(b'Z')).contains(&unit)
}

fn ascii_bytes(units: &[u16]) -> Option<Vec<u8>> {
    units.iter().map(|unit| u8::try_from(*unit).ok()).collect()
}

fn is_javascript_octal(units: &[u16]) -> bool {
    if units.len() < 2 || !is_digit(units[0], 8) || !is_digit(units[1], 8) {
        return false;
    }
    if units.len() < 3 || !is_digit(units[2], 8) {
        return units[0] != u16::from(b'0') || units[1] != u16::from(b'0');
    }
    units[0] != u16::from(b'0') || units[1] != u16::from(b'0') || units[2] != u16::from(b'0')
}
