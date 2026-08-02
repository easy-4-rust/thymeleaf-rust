use crate::exceptions::TemplateProcessingException;
use crate::expression::TemplateValue;
use crate::temporal::JavaTemporal;
use crate::util::{JavaDate, JavaString, JavaWriter, ResourceLoaderUtils};

use super::IStandardJavaScriptSerializer;

/// Standard Dialect 默认 JavaScript/JSON 值序列化器。
///
/// 对应 Java: `org.thymeleaf.standard.serializer.StandardJavaScriptSerializer`。
///
/// Rust 没有 JVM classpath/Jackson 反射分支，因此两个构造模式都落到 Thymeleaf
/// 自带 serializer 的可观察格式；字符串仍执行非 ASCII、`&` 与 `</` 注入防护。
#[derive(Clone, Copy, Debug)]
pub struct StandardJavaScriptSerializer {
    delegate: JavaScriptSerializerDelegate,
}

impl StandardJavaScriptSerializer {
    /// 创建 JavaScript 序列化器并保存 Java 构造参数。
    #[must_use]
    pub fn new(use_jackson_if_available: bool) -> Self {
        let delegate = if use_jackson_if_available {
            if ResourceLoaderUtils::is_class_present("tools.jackson.databind.ObjectMapper") {
                JavaScriptSerializerDelegate::Jackson3(Jackson3StandardJavaScriptSerializer)
            } else {
                JavaScriptSerializerDelegate::Jackson2(JacksonStandardJavaScriptSerializer)
            }
        } else {
            JavaScriptSerializerDelegate::Default(DefaultStandardJavaScriptSerializer)
        };
        Self { delegate }
    }

    /// 返回是否请求优先使用 Jackson 兼容格式。
    #[must_use]
    pub const fn is_use_jackson_if_available(&self) -> bool {
        matches!(
            self.delegate,
            JavaScriptSerializerDelegate::Jackson2(_) | JavaScriptSerializerDelegate::Jackson3(_)
        )
    }
}

impl IStandardJavaScriptSerializer for StandardJavaScriptSerializer {
    fn serialize_value(
        &self,
        object: Option<&TemplateValue>,
        writer: &mut dyn JavaWriter,
    ) -> Result<(), TemplateProcessingException> {
        self.delegate.serialize_value(object, writer).map_err(|error| {
            TemplateProcessingException::with_cause(
                Some(
                    "An exception was raised while trying to serialize object to JavaScript using the default serializer"
                        .to_owned(),
                ),
                error,
            )
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum JavaScriptSerializerDelegate {
    Jackson2(JacksonStandardJavaScriptSerializer),
    Jackson3(Jackson3StandardJavaScriptSerializer),
    Default(DefaultStandardJavaScriptSerializer),
}

impl JavaScriptSerializerDelegate {
    fn serialize_value(
        &self,
        object: Option<&TemplateValue>,
        writer: &mut dyn JavaWriter,
    ) -> std::io::Result<()> {
        match self {
            Self::Jackson2(serializer) => serializer.serialize_value(object, writer),
            Self::Jackson3(serializer) => serializer.serialize_value(object, writer),
            Self::Default(serializer) => serializer.serialize_value(object, writer),
        }
    }
}

/// Jackson 2 分支的 Rust 等价序列化器。
///
/// 对应 Java: `StandardJavaScriptSerializer.JacksonStandardJavaScriptSerializer`。
/// Rust 不需要反射装载 Jackson；该分支保留 Jackson 的“转义全部斜杠”安全输出。
#[derive(Clone, Copy, Debug)]
struct JacksonStandardJavaScriptSerializer;

impl JacksonStandardJavaScriptSerializer {
    fn serialize_value(
        &self,
        object: Option<&TemplateValue>,
        writer: &mut dyn JavaWriter,
    ) -> std::io::Result<()> {
        write_value(object, writer, true)
    }
}

/// Jackson 3 反射分支的 Rust 等价序列化器。
///
/// 对应 Java: `StandardJavaScriptSerializer.Jackson3StandardJavaScriptSerializer`。
/// Java 3.x 分支的缓冲后处理与 2.x 最终可观察输出一致，因此共享安全写入实现。
#[derive(Clone, Copy, Debug)]
struct Jackson3StandardJavaScriptSerializer;

impl Jackson3StandardJavaScriptSerializer {
    fn serialize_value(
        &self,
        object: Option<&TemplateValue>,
        writer: &mut dyn JavaWriter,
    ) -> std::io::Result<()> {
        write_value(object, writer, true)
    }
}

/// Thymeleaf ECMAScript 兼容 ISO-8601 日期格式器。
///
/// 对应 Java: `StandardJavaScriptSerializer.JacksonThymeleafISO8601DateFormat`。
struct JacksonThymeleafISO8601DateFormat;

impl JacksonThymeleafISO8601DateFormat {
    fn format(date: &JavaDate) -> JavaString {
        crate::util::DateUtils::format_iso(Some(date))
            .expect("non-null JavaDate always has an ISO representation")
    }

    /// 保留上游“日期格式器只允许写出”的合同。
    ///
    /// 对应 Java:
    /// `JacksonThymeleafISO8601DateFormat#parse(String, ParsePosition)`。
    #[expect(dead_code, reason = "保留 Java 只写日期格式器的拒绝解析合同")]
    fn parse(
        _source: &JavaString,
        _position: i32,
    ) -> Result<JavaDate, TemplateProcessingException> {
        Err(TemplateProcessingException::new(Some(
            "JacksonThymeleafISO8601DateFormat should never be asked for a 'parse' operation"
                .to_owned(),
        )))
    }
}

/// JavaScript 字符串的脚本/XHTML 注入防护转义表。
///
/// 对应 Java: `StandardJavaScriptSerializer.JacksonThymeleafCharacterEscapes`。
struct JacksonThymeleafCharacterEscapes;

impl JacksonThymeleafCharacterEscapes {
    #[expect(
        dead_code,
        reason = "保留 Jackson CharacterEscapes 对照方法和转义表语义"
    )]
    fn get_escape_codes_for_ascii() -> [i32; 128] {
        let mut escapes = [0_i32; 128];
        let mut control = 0;
        while control < 0x20 {
            escapes[control] = -1;
            control += 1;
        }
        escapes[usize::from(b'"')] = i32::from(b'"');
        escapes[usize::from(b'\\')] = i32::from(b'\\');
        // Jackson CharacterEscapes::ESCAPE_CUSTOM。
        escapes[usize::from(b'/')] = -2;
        escapes[usize::from(b'&')] = -2;
        escapes
    }

    fn get_escape_sequence(character: u16) -> Option<&'static str> {
        match character {
            0x002F => Some("\\/"),
            0x0026 => Some("\\u0026"),
            _ => None,
        }
    }

    fn custom_escape(unit: u16, previous: u16, escape_all_slashes: bool) -> Option<&'static str> {
        match unit {
            0x002F if escape_all_slashes || previous == u16::from(b'<') => {
                Self::get_escape_sequence(unit)
            }
            0x0026 => Self::get_escape_sequence(unit),
            _ => None,
        }
    }
}

/// 不依赖外部 JSON 库的标准 JavaScript 序列化器。
///
/// 对应 Java: `StandardJavaScriptSerializer.DefaultStandardJavaScriptSerializer`。
#[derive(Clone, Copy, Debug)]
struct DefaultStandardJavaScriptSerializer;

impl DefaultStandardJavaScriptSerializer {
    fn serialize_value(
        &self,
        object: Option<&TemplateValue>,
        writer: &mut dyn JavaWriter,
    ) -> std::io::Result<()> {
        write_value(object, writer, false)
    }
}

fn write_value(
    object: Option<&TemplateValue>,
    writer: &mut dyn JavaWriter,
    escape_all_slashes: bool,
) -> std::io::Result<()> {
    let Some(object) = object else {
        return write_ascii(writer, "null");
    };
    match object {
        TemplateValue::Null => write_ascii(writer, "null"),
        TemplateValue::Boolean(value) => write_ascii(writer, if *value { "true" } else { "false" }),
        TemplateValue::Number(_) => write_java_string(
            writer,
            &object
                .to_java_string()
                .unwrap_or_else(|| JavaString::from_rust_str("null")),
        ),
        TemplateValue::Character(unit) => write_json_string(writer, &[*unit], escape_all_slashes),
        TemplateValue::String(value) | TemplateValue::SafeHtml(value) => {
            write_json_string(writer, value.as_utf16(), escape_all_slashes)
        }
        TemplateValue::Bytes(bytes) => {
            writer.write_utf16(&[u16::from(b'[')])?;
            for (index, value) in bytes.iter().enumerate() {
                if index != 0 {
                    writer.write_utf16(&[u16::from(b',')])?;
                }
                write_ascii(writer, &value.to_string())?;
            }
            writer.write_utf16(&[u16::from(b']')])
        }
        TemplateValue::List(values) => {
            writer.write_utf16(&[u16::from(b'[')])?;
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    writer.write_utf16(&[u16::from(b',')])?;
                }
                write_value(Some(value.as_ref()), writer, escape_all_slashes)?;
            }
            writer.write_utf16(&[u16::from(b']')])
        }
        TemplateValue::Map(entries) => {
            writer.write_utf16(&[u16::from(b'{')])?;
            for (index, (key, value)) in entries.iter().enumerate() {
                if index != 0 {
                    writer.write_utf16(&[u16::from(b',')])?;
                }
                write_value(Some(key.as_ref()), writer, escape_all_slashes)?;
                writer.write_utf16(&[u16::from(b':')])?;
                write_value(Some(value.as_ref()), writer, escape_all_slashes)?;
            }
            writer.write_utf16(&[u16::from(b'}')])
        }
        TemplateValue::Literal(value) => {
            let text = value
                .get_value()
                .cloned()
                .unwrap_or_else(|| JavaString::from_rust_str("null"));
            write_json_string(writer, text.as_utf16(), escape_all_slashes)
        }
        TemplateValue::NoOp => write_json_string(writer, &[u16::from(b'_')], escape_all_slashes),
        TemplateValue::Object(value) => {
            if let Some(date) = value.as_any().downcast_ref::<JavaDate>() {
                let formatted = JacksonThymeleafISO8601DateFormat::format(date);
                write_json_string(writer, formatted.as_utf16(), escape_all_slashes)
            } else if let Some(temporal) = value.as_any().downcast_ref::<JavaTemporal>() {
                let formatted = temporal.to_javascript_iso_string();
                write_json_string(writer, formatted.as_utf16(), escape_all_slashes)
            } else if let Some(serializable) = value.java_serializable_value() {
                write_value(serializable.as_deref(), writer, escape_all_slashes)
            } else if let Some(properties) = value.java_serializable_properties() {
                writer.write_utf16(&[u16::from(b'{')])?;
                for (index, (name, property_value)) in properties.iter().enumerate() {
                    if index != 0 {
                        writer.write_utf16(&[u16::from(b',')])?;
                    }
                    write_json_string(writer, name.as_utf16(), escape_all_slashes)?;
                    writer.write_utf16(&[u16::from(b':')])?;
                    write_value(property_value.as_deref(), writer, escape_all_slashes)?;
                }
                writer.write_utf16(&[u16::from(b'}')])
            } else {
                write_json_string(
                    writer,
                    value.to_java_string().as_utf16(),
                    escape_all_slashes,
                )
            }
        }
    }
}

fn write_json_string(
    writer: &mut dyn JavaWriter,
    input: &[u16],
    escape_all_slashes: bool,
) -> std::io::Result<()> {
    writer.write_utf16(&[u16::from(b'"')])?;
    let mut previous = 0_u16;
    for &unit in input {
        match unit {
            0x0008 => write_ascii(writer, "\\b")?,
            0x0009 => write_ascii(writer, "\\t")?,
            0x000A => write_ascii(writer, "\\n")?,
            0x000C => write_ascii(writer, "\\f")?,
            0x000D => write_ascii(writer, "\\r")?,
            0x0022 => write_ascii(writer, "\\\"")?,
            0x005C => write_ascii(writer, "\\\\")?,
            0x002F | 0x0026 => {
                if let Some(escaped) = JacksonThymeleafCharacterEscapes::custom_escape(
                    unit,
                    previous,
                    escape_all_slashes,
                ) {
                    write_ascii(writer, escaped)?;
                } else {
                    writer.write_utf16(&[unit])?;
                }
            }
            0x0000..=0x001F | 0x007F..=0x009F | 0x0080..=0xFFFF => {
                write_ascii(writer, &format!("\\u{unit:04X}"))?;
            }
            _ => writer.write_utf16(&[unit])?,
        }
        previous = unit;
    }
    writer.write_utf16(&[u16::from(b'"')])
}

fn write_ascii(writer: &mut dyn JavaWriter, input: &str) -> std::io::Result<()> {
    writer.write_utf16(&input.encode_utf16().collect::<Vec<_>>())
}

fn write_java_string(writer: &mut dyn JavaWriter, input: &JavaString) -> std::io::Result<()> {
    writer.write_utf16(input.as_utf16())
}
