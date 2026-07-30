use std::sync::Arc;

use indexmap::IndexMap;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::expression::TemplateValue;
use crate::util::JavaString;

use super::ILinkBuilder;

#[derive(Clone, Copy, Eq, PartialEq)]
enum LinkType {
    Absolute,
    ContextRelative,
    ServerRelative,
    BaseRelative,
}

/// 构建绝对、上下文相对、服务器相对和基础相对 URL。
///
/// 对应 Java: `org.thymeleaf.linkbuilder.StandardLinkBuilder`。
pub struct StandardLinkBuilder {
    name: Option<JavaString>,
    order: Option<i32>,
}

impl StandardLinkBuilder {
    /// 创建使用 Java 具体类名且顺序为空的标准链接构建器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: Some(JavaString::from_rust_str(
                "org.thymeleaf.linkbuilder.StandardLinkBuilder",
            )),
            order: None,
        }
    }

    /// 设置可空构建器名称。
    pub fn set_name(&mut self, name: Option<JavaString>) {
        self.name = name;
    }

    /// 设置可空构建器顺序。
    pub fn set_order(&mut self, order: Option<i32>) {
        self.order = order;
    }

    fn build_standard_link(
        &self,
        context: &dyn IExpressionContext,
        base: Option<&JavaString>,
        parameters: Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
    ) -> Result<Option<JavaString>, TemplateProcessingException> {
        let Some(base) = base else {
            return Ok(None);
        };
        let base_text = base.to_string_lossy();
        filter_out_java_script_links(&base_text)?;

        let link_type = if is_link_base_absolute(&base_text) {
            LinkType::Absolute
        } else if is_link_base_context_relative(&base_text) {
            LinkType::ContextRelative
        } else if is_link_base_server_relative(&base_text) {
            LinkType::ServerRelative
        } else {
            LinkType::BaseRelative
        };
        let mut link_parameters = parameters
            .filter(|parameters| !parameters.is_empty())
            .cloned();
        let hash_position = base_text.rfind('#');
        let might_have_variable_templates = base_text.contains('{');
        let context_path = if link_type == LinkType::ContextRelative {
            self.compute_context_path(context, &base_text)?
        } else {
            None
        };
        let context_path_empty = context_path
            .as_ref()
            .is_none_or(|path| path.is_empty() || path == "/");

        if context_path_empty
            && link_type != LinkType::ServerRelative
            && link_parameters.as_ref().is_none_or(IndexMap::is_empty)
            && hash_position.is_none()
            && !might_have_variable_templates
        {
            return Ok(Some(self.process_link(context, base.clone())));
        }

        let mut link_base = base_text;
        let mut url_fragment = String::new();
        if let Some(hash_position) = hash_position.filter(|position| *position > 0) {
            url_fragment = link_base.split_off(hash_position);
        }
        if might_have_variable_templates {
            replace_template_params_in_base(&mut link_base, link_parameters.as_mut());
        }
        if let Some(parameters) = link_parameters.as_ref().filter(|value| !value.is_empty()) {
            link_base.push(if link_base.contains('?') { '&' } else { '?' });
            process_all_remaining_parameters_as_query_params(&mut link_base, parameters);
        }
        link_base.push_str(&url_fragment);
        if link_type == LinkType::ServerRelative {
            link_base.remove(0);
        }
        if link_type == LinkType::ContextRelative
            && !context_path_empty
            && let Some(context_path) = context_path
        {
            link_base.insert_str(0, &context_path);
        }
        Ok(Some(self.process_link(
            context,
            JavaString::from_rust_str(&link_base),
        )))
    }

    fn compute_context_path(
        &self,
        context: &dyn IExpressionContext,
        base: &str,
    ) -> Result<Option<String>, TemplateProcessingException> {
        let Some(exchange) = context.get_web_exchange() else {
            return Err(TemplateProcessingException::new(Some(format!(
                "Link base \"{base}\" cannot be context relative (/...) unless the context used \
                 for executing the engine implements the org.thymeleaf.context.IWebContext interface"
            ))));
        };
        Ok(exchange
            .get_request()
            .get_application_path()
            .map(|path| path.to_string_lossy()))
    }

    fn process_link(&self, context: &dyn IExpressionContext, link: JavaString) -> JavaString {
        context
            .get_web_exchange()
            .and_then(|exchange| exchange.transform_url(Some(&link)))
            .unwrap_or(link)
    }
}

impl Default for StandardLinkBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ILinkBuilder for StandardLinkBuilder {
    fn get_name(&self) -> Option<&JavaString> {
        self.name.as_ref()
    }

    fn get_order(&self) -> Option<i32> {
        self.order
    }

    fn build_link(
        &self,
        context: &dyn IExpressionContext,
        base: Option<&JavaString>,
        parameters: Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
    ) -> Result<Option<JavaString>, TemplateProcessingException> {
        self.build_standard_link(context, base, parameters)
    }
}

fn filter_out_java_script_links(base: &str) -> Result<(), TemplateProcessingException> {
    if base
        .get(..11)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("javascript:"))
    {
        return Err(TemplateProcessingException::new(Some(
            "'javascript:' is forbidden in this context. Link expressions cannot contain inlined \
             JavaScript code."
                .to_owned(),
        )));
    }
    Ok(())
}

fn is_link_base_absolute(base: &str) -> bool {
    (base.len() >= 7 && base[..7].eq_ignore_ascii_case("mailto:"))
        || base.starts_with("//")
        || base.contains("://")
}

fn is_link_base_context_relative(base: &str) -> bool {
    base.starts_with('/') && !base.starts_with("//")
}

fn is_link_base_server_relative(base: &str) -> bool {
    base.starts_with("~/")
}

fn replace_template_params_in_base(
    link_base: &mut String,
    parameters: Option<&mut IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
) {
    let Some(parameters) = parameters else {
        return;
    };
    let question_mark_position = link_base.rfind('?');
    let mut processed = Vec::new();
    for (parameter_name, parameter_value) in parameters.iter() {
        let parameter_name_text = parameter_name
            .as_ref()
            .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy);
        let direct_template = format!("{{{parameter_name_text}}}");
        let segment_template = format!("{{/{parameter_name_text}}}");
        let (template, escape_as_path_segment) = if link_base.contains(&direct_template) {
            (direct_template, false)
        } else if link_base.contains(&segment_template) {
            (segment_template, true)
        } else {
            continue;
        };
        processed.push(parameter_name.clone());
        let replacement =
            format_parameter_value_as_unescaped_variable_template(parameter_value.as_deref());
        while let Some(start) = link_base.find(&template) {
            let escaped = if question_mark_position.is_none_or(|question| start < question) {
                if escape_as_path_segment {
                    escape_uri_path_segment(&replacement)
                } else {
                    escape_uri_path(&replacement)
                }
            } else {
                escape_uri_query_param(&replacement)
            };
            link_base.replace_range(start..start + template.len(), &escaped);
        }
    }
    for parameter_name in processed {
        parameters.shift_remove(&parameter_name);
    }
}

fn format_parameter_value_as_unescaped_variable_template(
    parameter_value: Option<&TemplateValue>,
) -> String {
    match parameter_value {
        None | Some(TemplateValue::Null) => String::new(),
        Some(TemplateValue::List(values)) => values
            .iter()
            .map(|value| match value.as_ref() {
                TemplateValue::Null => String::new(),
                value => value
                    .to_java_string()
                    .map_or_else(String::new, |text| text.to_string_lossy()),
            })
            .collect::<Vec<_>>()
            .join(","),
        Some(value) => value
            .to_java_string()
            .map_or_else(String::new, |text| text.to_string_lossy()),
    }
}

fn process_all_remaining_parameters_as_query_params(
    result: &mut String,
    parameters: &IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>,
) {
    let mut first = true;
    for (parameter_name, value) in parameters {
        let parameter_name = parameter_name
            .as_ref()
            .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy);
        match value.as_deref() {
            None | Some(TemplateValue::Null) => {
                append_query_parameter(result, &mut first, &parameter_name, None);
            }
            Some(TemplateValue::List(values)) => {
                for value in values.iter() {
                    let value = match value.as_ref() {
                        TemplateValue::Null => None,
                        value => value.to_java_string(),
                    };
                    append_query_parameter(
                        result,
                        &mut first,
                        &parameter_name,
                        value.as_ref().map(JavaString::to_string_lossy).as_deref(),
                    );
                }
            }
            Some(value) => {
                let value = value.to_java_string();
                append_query_parameter(
                    result,
                    &mut first,
                    &parameter_name,
                    value.as_ref().map(JavaString::to_string_lossy).as_deref(),
                );
            }
        }
    }
}

fn append_query_parameter(result: &mut String, first: &mut bool, name: &str, value: Option<&str>) {
    if !*first {
        result.push('&');
    }
    *first = false;
    result.push_str(&escape_uri_query_param(name));
    if let Some(value) = value {
        result.push('=');
        result.push_str(&escape_uri_query_param(value));
    }
}

fn escape_uri_path(value: &str) -> String {
    percent_escape(value, |byte| is_pchar(byte) || byte == b'/')
}

fn escape_uri_path_segment(value: &str) -> String {
    percent_escape(value, is_pchar)
}

fn escape_uri_query_param(value: &str) -> String {
    percent_escape(value, |byte| {
        !matches!(byte, b'=' | b'&' | b'+' | b'#')
            && (is_pchar(byte) || matches!(byte, b'/' | b'?'))
    })
}

fn is_pchar(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'-' | b'.'
                | b'_'
                | b'~'
                | b'!'
                | b'$'
                | b'&'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b';'
                | b'='
                | b':'
                | b'@'
        )
}

fn percent_escape(value: &str, allowed: impl Fn(u8) -> bool) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut result = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii() && allowed(byte) {
            result.push(char::from(byte));
        } else {
            result.push('%');
            result.push(char::from(HEX[usize::from(byte >> 4)]));
            result.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    result
}
