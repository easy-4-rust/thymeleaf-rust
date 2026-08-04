use std::io;
use std::panic::panic_any;
use std::sync::{Arc, Mutex};

use crate::expression::TemplateValue;
use crate::serializer::StandardSerializers;
use crate::{IEngineConfiguration, TemplateMode};

use super::{
    AbstractLazyCharSequence, IWritableCharSequence, JavaCharSequence, JavaWriter,
    LazyCharSequenceResolver, TextUtilsError, Utf16String, ValidateError,
};

/// 在真正写出时执行模板模式转义的延迟字符序列。
///
/// HTML/XML/TEXT 按 UTF-16 内容转义，JAVASCRIPT/CSS 委托 Standard Serializer，
/// RAW 原样写出。对应 Java: `org.thymeleaf.util.LazyEscapingCharSequence`。
pub struct LazyEscapingCharSequence {
    sequence: AbstractLazyCharSequence<LazyEscapingResolver>,
}

impl LazyEscapingCharSequence {
    /// 创建延迟转义序列。
    ///
    /// 对应 Java 构造器；配置或模板模式为 null 时保留精确参数错误消息。
    pub fn new(
        configuration: Option<Arc<dyn IEngineConfiguration>>,
        template_mode: Option<TemplateMode>,
        input: Option<Arc<TemplateValue>>,
    ) -> Result<Self, ValidateError> {
        let configuration = configuration.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Engine Configuraion is null, which is forbidden".to_owned()),
        })?;
        let template_mode = template_mode.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Template Mode is null, which is forbidden".to_owned()),
        })?;
        Ok(Self {
            sequence: AbstractLazyCharSequence::new(LazyEscapingResolver {
                configuration,
                template_mode,
                input,
            }),
        })
    }
}

/// 立即生成与 `LazyEscapingCharSequence` 相同的 HTML/XML/TEXT 转义结果。
///
/// 该入口供 `StandardTextTagProcessor` 在短文本分支复用同一套字符级语义。
/// 对应 Java 语义：`LazyEscapingCharSequence` 的 `escape_text_immediately` 行为（Rust 侧辅助/私有路径）。
pub(crate) fn escape_text_immediately(
    template_mode: TemplateMode,
    input: &Utf16String,
) -> Result<Utf16String, crate::exceptions::TemplateProcessingException> {
    let output = Arc::new(Mutex::new(Vec::new()));
    let mut writer = SharedWriter {
        output: Arc::clone(&output),
    };
    match template_mode {
        TemplateMode::TEXT | TemplateMode::HTML => write_html4_xml(input, &mut writer)?,
        TemplateMode::XML => write_xml10(input, &mut writer)?,
        _ => {
            return Err(crate::exceptions::TemplateProcessingException::new(Some(
                format!(
                    "Unrecognized template mode {template_mode:?}. Cannot produce escaped output for this template mode."
                ),
            )));
        }
    }
    let units = output
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    Ok(Utf16String::from_utf16(units))
}

impl JavaCharSequence for LazyEscapingCharSequence {
    fn java_sequence_class_name(&self) -> &str {
        "org.thymeleaf.util.LazyEscapingCharSequence"
    }

    fn java_length(&self) -> Result<i32, TextUtilsError> {
        self.sequence.java_length()
    }

    fn java_char_at(&self, index: i32) -> Result<u16, TextUtilsError> {
        self.sequence.java_char_at(index)
    }

    fn as_utf16_string(&self) -> Option<&Utf16String> {
        None
    }

    fn java_to_string(&self) -> Result<Utf16String, TextUtilsError> {
        self.sequence.java_to_string()
    }

    fn java_sub_sequence(&self, start: i32, end: i32) -> Result<Utf16String, TextUtilsError> {
        self.sequence.java_sub_sequence(start, end)
    }

    fn write_direct(&self, writer: &mut dyn JavaWriter) -> Option<io::Result<()>> {
        Some(self.write(writer))
    }
}

impl IWritableCharSequence for LazyEscapingCharSequence {
    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
        self.sequence.write(writer)
    }
}

struct LazyEscapingResolver {
    configuration: Arc<dyn IEngineConfiguration>,
    template_mode: TemplateMode,
    input: Option<Arc<TemplateValue>>,
}

impl LazyEscapingResolver {
    fn produce_escaped_output(
        &self,
        writer: &mut dyn JavaWriter,
    ) -> Result<(), crate::exceptions::TemplateProcessingException> {
        match self.template_mode {
            TemplateMode::TEXT | TemplateMode::HTML => {
                if let Some(input) = self.input.as_deref()
                    && let Some(text) = input.to_utf16_string()
                {
                    write_html4_xml(&text, writer)?;
                }
            }
            TemplateMode::XML => {
                if let Some(input) = self.input.as_deref()
                    && let Some(text) = input.to_utf16_string()
                {
                    write_xml10(&text, writer)?;
                }
            }
            TemplateMode::JAVASCRIPT => {
                StandardSerializers::get_java_script_serializer(self.configuration.as_ref())?
                    .serialize_value(self.input.as_deref(), writer)?;
            }
            TemplateMode::CSS => {
                StandardSerializers::get_css_serializer(self.configuration.as_ref())?
                    .serialize_value(self.input.as_deref(), writer)?;
            }
            TemplateMode::RAW => {
                if let Some(input) = self.input.as_deref()
                    && let Some(text) = input.to_utf16_string()
                {
                    writer.write_utf16(text.as_utf16()).map_err(|error| {
                        crate::exceptions::TemplateProcessingException::with_cause(
                            Some(
                                "An error happened while trying to produce escaped output"
                                    .to_owned(),
                            ),
                            error,
                        )
                    })?;
                }
            }
        }
        Ok(())
    }
}

impl LazyCharSequenceResolver for LazyEscapingResolver {
    fn java_class_name(&self) -> &str {
        "org.thymeleaf.util.LazyEscapingCharSequence"
    }

    fn resolve_text(&self) -> Option<Utf16String> {
        let output = Arc::new(Mutex::new(Vec::new()));
        let mut writer = SharedWriter {
            output: Arc::clone(&output),
        };
        if let Err(error) = self.produce_escaped_output(&mut writer) {
            panic_any(error);
        }
        Some(Utf16String::from_utf16(
            output
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        ))
    }

    fn write_unresolved(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
        self.produce_escaped_output(writer)
            .map_err(|error| io::Error::other(error.to_string()))
    }
}

struct SharedWriter {
    output: Arc<Mutex<Vec<u16>>>,
}

impl JavaWriter for SharedWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        self.output
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(characters);
        Ok(())
    }
}

fn write_html4_xml(
    input: &Utf16String,
    writer: &mut dyn JavaWriter,
) -> Result<(), crate::exceptions::TemplateProcessingException> {
    for unit in input.as_utf16() {
        match *unit {
            0x26 => writer.write_utf16(&[38, 97, 109, 112, 59]),
            0x3C => writer.write_utf16(&[38, 108, 116, 59]),
            0x3E => writer.write_utf16(&[38, 103, 116, 59]),
            0x22 => writer.write_utf16(&[38, 113, 117, 111, 116, 59]),
            0x27 => writer.write_utf16(&[38, 35, 51, 57, 59]),
            _ => writer.write_utf16(std::slice::from_ref(unit)),
        }
        .map_err(output_error)?;
    }
    Ok(())
}

fn write_xml10(
    input: &Utf16String,
    writer: &mut dyn JavaWriter,
) -> Result<(), crate::exceptions::TemplateProcessingException> {
    let units = input.as_utf16();
    let mut index = 0;
    while index < units.len() {
        let first = units[index];
        let (codepoint, consumed) = decode_utf16(units, index);
        index += consumed;
        if !is_valid_xml10(codepoint, first, consumed) {
            continue;
        }
        match codepoint {
            0x26 => writer.write_utf16(&[38, 97, 109, 112, 59]),
            0x3C => writer.write_utf16(&[38, 108, 116, 59]),
            0x3E => writer.write_utf16(&[38, 103, 116, 59]),
            _ => writer.write_utf16(&units[index - consumed..index]),
        }
        .map_err(output_error)?;
    }
    Ok(())
}

fn decode_utf16(units: &[u16], index: usize) -> (u32, usize) {
    let first = units[index];
    if (0xD800..=0xDBFF).contains(&first)
        && let Some(second) = units.get(index + 1)
        && (0xDC00..=0xDFFF).contains(second)
    {
        return (
            0x10000 + ((u32::from(first) - 0xD800) << 10) + (u32::from(*second) - 0xDC00),
            2,
        );
    }
    (u32::from(first), 1)
}

fn is_valid_xml10(codepoint: u32, first: u16, consumed: usize) -> bool {
    if consumed == 1 && (0xD800..=0xDFFF).contains(&first) {
        return false;
    }
    matches!(codepoint, 0x09 | 0x0A | 0x0D | 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF)
}

fn output_error(error: io::Error) -> crate::exceptions::TemplateProcessingException {
    crate::exceptions::TemplateProcessingException::with_cause(
        Some("An error happened while trying to produce escaped output".to_owned()),
        error,
    )
}
