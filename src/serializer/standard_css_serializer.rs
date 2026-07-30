use crate::exceptions::TemplateProcessingException;
use crate::expression::TemplateValue;
use crate::util::{JavaString, JavaWriter};

use super::IStandardCSSSerializer;

/// Standard Dialect 默认 CSS 值序列化器。
///
/// 对应 Java: `org.thymeleaf.standard.serializer.StandardCSSSerializer`。
#[derive(Clone, Copy, Debug, Default)]
pub struct StandardCSSSerializer;

impl StandardCSSSerializer {
    /// 创建无状态、线程安全的 CSS 序列化器。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl IStandardCSSSerializer for StandardCSSSerializer {
    fn serialize_value(
        &self,
        object: Option<&TemplateValue>,
        writer: &mut dyn JavaWriter,
    ) -> Result<(), TemplateProcessingException> {
        let Some(object) = object else {
            return Ok(());
        };
        if matches!(object, TemplateValue::Boolean(_) | TemplateValue::Number(_)) {
            let text = object
                .to_java_string()
                .unwrap_or_else(|| JavaString::from_rust_str(""));
            return writer.write_utf16(text.as_utf16()).map_err(|error| {
                TemplateProcessingException::with_cause(
                    Some(
                        "An exception was raised while trying to serialize object to CSS"
                            .to_owned(),
                    ),
                    error,
                )
            });
        }
        if matches!(object, TemplateValue::Null) {
            return Ok(());
        }
        let text = object
            .to_java_string()
            .unwrap_or_else(|| JavaString::from_rust_str(""));
        let escaped = escape_css_identifier(text.as_utf16());
        writer.write_utf16(&escaped).map_err(|error| {
            TemplateProcessingException::with_cause(
                Some("An exception was raised while trying to serialize object to CSS".to_owned()),
                error,
            )
        })
    }
}

fn escape_css_identifier(input: &[u16]) -> Vec<u16> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        let unit = input[index];
        let (codepoint, consumed) = if (0xD800..=0xDBFF).contains(&unit)
            && input
                .get(index + 1)
                .is_some_and(|low| (0xDC00..=0xDFFF).contains(low))
        {
            let high = u32::from(unit - 0xD800);
            let low = u32::from(input[index + 1] - 0xDC00);
            (0x1_0000 + ((high << 10) | low), 2)
        } else {
            (u32::from(unit), 1)
        };
        if !needs_css_escape(codepoint, index, input) {
            output.extend_from_slice(&input[index..index + consumed]);
            index += consumed;
            continue;
        }
        output.push(u16::from(b'\\'));
        if let Some(backslash) = css_backslash_escape(codepoint) {
            output.push(backslash);
        } else {
            let digits = format!("{codepoint:X}");
            output.extend(digits.encode_utf16());
            let next = input.get(index + consumed).copied().unwrap_or(0);
            if is_ascii_hex(next) {
                output.push(u16::from(b' '));
            }
        }
        index += consumed;
    }
    output
}

fn needs_css_escape(codepoint: u32, index: usize, input: &[u16]) -> bool {
    if index == 0 && codepoint <= 0x7F && (codepoint as u8).is_ascii_digit() {
        return true;
    }
    if codepoint == u32::from(b'-') {
        return index == 0
            && input.get(1).is_some_and(|next| {
                *next == u16::from(b'-') || (*next <= 0x7F && (*next as u8).is_ascii_digit())
            });
    }
    if codepoint == u32::from(b'_') {
        return index == 0;
    }
    if codepoint > 0x7F {
        return true;
    }
    matches!(codepoint, 0x00..=0x20 | 0x21..=0x2C | 0x2E..=0x2F | 0x3A..=0x40 | 0x5B..=0x5E | 0x60 | 0x7B..=0x9F)
}

fn css_backslash_escape(codepoint: u32) -> Option<u16> {
    let unit = u16::try_from(codepoint).ok()?;
    matches!(
        unit,
        0x20..=0x2F | 0x3B..=0x40 | 0x5B..=0x5F | 0x60 | 0x7B..=0x7E
    )
    .then_some(unit)
}

fn is_ascii_hex(unit: u16) -> bool {
    unit <= 0x7F && (unit as u8).is_ascii_hexdigit()
}
