use std::any::TypeId;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{OnceLock, RwLock};

use crate::TemplateInputException;
use crate::expression::TemplateValue;
use crate::templateresource::ITemplateResource;
use crate::util::{JavaLocale, JavaString};

type Messages = HashMap<JavaString, JavaString>;
type OriginMessages = HashMap<(TypeId, JavaLocale), Messages>;

static ORIGIN_MESSAGES: OnceLock<RwLock<OriginMessages>> = OnceLock::new();

/// 标准消息资源定位、合并与格式化工具。
///
/// 对应 Java: `org.thymeleaf.messageresolver.StandardMessageResolutionUtils`。
pub(crate) struct StandardMessageResolutionUtils;

impl StandardMessageResolutionUtils {
    /// 按基础资源、语言、国家和变体由低到高合并模板消息。
    pub(crate) fn resolve_messages_for_template(
        template_resource: &dyn ITemplateResource,
        locale: &JavaLocale,
    ) -> Result<Messages, TemplateInputException> {
        let Some(resource_base_name) = template_resource
            .get_base_name()
            .filter(|base_name| !base_name.is_empty())
        else {
            return Ok(HashMap::new());
        };

        let mut combined_messages = HashMap::new();
        for message_resource_name in
            Self::compute_message_resource_names_from_base(&resource_base_name, locale)?
        {
            let Ok(message_resource) = template_resource.relative(Some(&message_resource_name))
            else {
                // Java 版本忽略派生消息文件不存在时产生的 IOException。
                continue;
            };
            let Ok(reader) = message_resource.reader() else {
                continue;
            };
            combined_messages.extend(Self::read_messages_resource(reader)?);
        }
        Ok(combined_messages)
    }

    /// 返回宿主为 Rust 类型注册的 classpath 等价消息。
    pub(crate) fn resolve_messages_for_origin(origin: TypeId, locale: &JavaLocale) -> Messages {
        read_lock(origin_messages())
            .get(&(origin, locale.clone()))
            .cloned()
            .unwrap_or_default()
    }

    /// 注册 Rust 类型对应的 origin 消息资源。
    ///
    /// Rust 没有 JVM `ClassLoader#getResourceAsStream`；宿主集成层在加载等价
    /// classpath 资源后通过此入口登记，解析器仍按 origin 与 Locale 缓存。
    pub(crate) fn register_origin_messages(origin: TypeId, locale: JavaLocale, messages: Messages) {
        write_lock(origin_messages()).insert((origin, locale), messages);
    }

    /// 使用 Java `MessageFormat` 的索引占位符和引号规则格式化消息。
    pub(crate) fn format_message(
        _locale: &JavaLocale,
        message: &JavaString,
        message_parameters: Option<&[Option<std::sync::Arc<TemplateValue>>]>,
    ) -> JavaString {
        let text = message.to_string_lossy();
        if !text.contains('}') && !text.contains('\'') {
            return message.clone();
        }

        let parameters = message_parameters.unwrap_or(&[]);
        let characters = text.chars().collect::<Vec<_>>();
        let mut result = String::with_capacity(text.len());
        let mut index = 0;
        let mut quoted = false;
        while index < characters.len() {
            let character = characters[index];
            if character == '\'' {
                if characters.get(index + 1) == Some(&'\'') {
                    result.push('\'');
                    index += 2;
                    continue;
                }
                quoted = !quoted;
                index += 1;
                continue;
            }
            if character == '{'
                && !quoted
                && let Some(end) = find_format_element_end(&characters, index + 1)
            {
                let element = characters[index + 1..end].iter().collect::<String>();
                let parameter_index = element
                    .split(',')
                    .next()
                    .map(str::trim)
                    .and_then(|value| value.parse::<usize>().ok());
                if let Some(parameter_index) = parameter_index {
                    let value = parameters
                        .get(parameter_index)
                        .and_then(Option::as_deref)
                        .and_then(TemplateValue::to_java_string)
                        .unwrap_or_else(|| JavaString::from_rust_str("null"));
                    result.push_str(&value.to_string_lossy());
                    index = end + 1;
                    continue;
                }
            }
            result.push(character);
            index += 1;
        }
        JavaString::from_rust_str(&result)
    }

    fn compute_message_resource_names_from_base(
        resource_base_name: &str,
        locale: &JavaLocale,
    ) -> Result<Vec<String>, TemplateInputException> {
        let language = locale.get_language().to_string_lossy();
        if language.trim().is_empty() {
            return Err(TemplateInputException::new(Some(format!(
                "Locale \"{locale}\" cannot be used as it does not specify a language."
            ))));
        }

        let country = locale.get_country().to_string_lossy();
        let variant = locale.get_variant().to_string_lossy();
        let mut resource_names = Vec::with_capacity(4);
        resource_names.push(format!("{resource_base_name}.properties"));
        resource_names.push(format!("{resource_base_name}_{language}.properties"));
        if !country.trim().is_empty() {
            resource_names.push(format!(
                "{resource_base_name}_{language}_{country}.properties"
            ));
        }
        if !variant.trim().is_empty() {
            resource_names.push(format!(
                "{resource_base_name}_{language}_{country}-{variant}.properties"
            ));
        }
        Ok(resource_names)
    }

    fn read_messages_resource(
        mut properties_reader: Box<dyn Read>,
    ) -> Result<Messages, TemplateInputException> {
        let mut bytes = Vec::new();
        properties_reader.read_to_end(&mut bytes).map_err(|error| {
            TemplateInputException::with_cause(
                Some("Exception loading messages file".to_owned()),
                error,
            )
        })?;
        java_properties::read(bytes.as_slice())
            .map(|properties| {
                properties
                    .into_iter()
                    .map(|(key, value)| {
                        (
                            JavaString::from_rust_str(&key),
                            JavaString::from_rust_str(&value),
                        )
                    })
                    .collect()
            })
            .map_err(|error| {
                TemplateInputException::with_cause(
                    Some("Exception loading messages file".to_owned()),
                    error,
                )
            })
    }
}

fn find_format_element_end(characters: &[char], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    for (index, character) in characters.iter().enumerate().skip(start) {
        if *character == '\'' {
            quoted = !quoted;
        } else if !quoted && *character == '{' {
            depth += 1;
        } else if !quoted && *character == '}' {
            if depth == 0 {
                return Some(index);
            }
            depth -= 1;
        }
    }
    None
}

fn origin_messages() -> &'static RwLock<OriginMessages> {
    ORIGIN_MESSAGES.get_or_init(|| RwLock::new(HashMap::new()))
}

fn read_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
