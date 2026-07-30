use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use thymeleaf::context::ITemplateContext;
use thymeleaf::expression::TemplateValue;
use thymeleaf::messageresolver::{IMessageResolver, MessageResolutionResult};
use thymeleaf::util::JavaString;

/// 上游测试框架的内存消息解析器。
///
/// 对应 Java:
/// `org.thymeleaf.testing.templateengine.engine.resolver.TestEngineMessageResolver`。
///
/// 先按处理 Locale、语言和无 Locale 消息查找，再使用 Java `MessageFormat`
/// 的索引占位符与单引号规则格式化结果。
pub struct TestEngineMessageResolver {
    messages_by_locale: HashMap<Option<String>, HashMap<JavaString, JavaString>>,
}

impl TestEngineMessageResolver {
    /// 使用按 Locale 限定的测试消息创建解析器。
    ///
    /// `None` 对应未限定的 `%MESSAGES`，字符串键对应 `%MESSAGES[locale]`。
    #[must_use]
    pub fn new(
        messages_by_locale: HashMap<Option<String>, HashMap<JavaString, JavaString>>,
    ) -> Self {
        Self { messages_by_locale }
    }

    fn resolve_for_locale(
        &self,
        context: &dyn ITemplateContext,
        key: &JavaString,
    ) -> Option<&JavaString> {
        let locale = context.get_locale();
        let exact = locale.to_string().to_ascii_lowercase();
        let language = locale.get_language().to_string_lossy().to_ascii_lowercase();
        let country = locale.get_country().to_string_lossy().to_ascii_lowercase();
        let language_country = (!country.is_empty()).then(|| format!("{language}_{country}"));

        [
            Some(exact.as_str()),
            language_country.as_deref(),
            Some(language.as_str()),
            None,
        ]
        .into_iter()
        .find_map(|candidate| {
            let locale_key = candidate.map(ToOwned::to_owned);
            self.messages_by_locale
                .get(&locale_key)
                .and_then(|messages| messages.get(key))
        })
    }
}

impl IMessageResolver for TestEngineMessageResolver {
    fn get_name(&self) -> Option<&JavaString> {
        None
    }

    fn get_order(&self) -> Option<i32> {
        None
    }

    fn resolve_message(
        &self,
        context: &dyn ITemplateContext,
        _origin: Option<TypeId>,
        key: &JavaString,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>> {
        Ok(self
            .resolve_for_locale(context, key)
            .map(|message| format_message_like_java(message, message_parameters.unwrap_or(&[]))))
    }

    fn create_absent_message_representation(
        &self,
        context: &dyn ITemplateContext,
        _origin: Option<TypeId>,
        key: &JavaString,
        _message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>> {
        Ok(Some(JavaString::from_rust_str(&format!(
            "??{}_{}??",
            key.to_string_lossy(),
            context.get_locale()
        ))))
    }
}

fn format_message_like_java(
    message: &JavaString,
    parameters: &[Option<Arc<TemplateValue>>],
) -> JavaString {
    let text = message.to_string_lossy();
    let characters = text.chars().collect::<Vec<_>>();
    let mut result = String::with_capacity(text.len());
    let mut position = 0_usize;
    let mut quoted = false;

    while position < characters.len() {
        let character = characters[position];
        if character == '\'' {
            if characters.get(position + 1) == Some(&'\'') {
                result.push('\'');
                position += 2;
                continue;
            }
            quoted = !quoted;
            position += 1;
            continue;
        }
        if character == '{'
            && !quoted
            && let Some(end) = characters[position + 1..]
                .iter()
                .position(|candidate| *candidate == '}')
                .map(|offset| position + 1 + offset)
        {
            let element = characters[position + 1..end].iter().collect::<String>();
            if let Some(parameter_index) = element
                .split(',')
                .next()
                .map(str::trim)
                .and_then(|value| value.parse::<usize>().ok())
            {
                let Some(parameter) = parameters.get(parameter_index) else {
                    // Java MessageFormat 在参数数组短于占位符索引时保留原占位符。
                    result.push('{');
                    result.push_str(&element);
                    result.push('}');
                    position = end + 1;
                    continue;
                };
                let value = parameter
                    .as_deref()
                    .and_then(TemplateValue::to_java_string)
                    .unwrap_or_else(|| JavaString::from_rust_str("null"));
                result.push_str(&value.to_string_lossy());
                position = end + 1;
                continue;
            }
        }
        result.push(character);
        position += 1;
    }
    JavaString::from_rust_str(&result)
}
