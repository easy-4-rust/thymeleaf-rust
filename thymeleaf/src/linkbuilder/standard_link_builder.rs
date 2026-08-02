use std::sync::Arc;

use indexmap::IndexMap;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::expression::TemplateValue;
use crate::util::{JavaString, java_lower};

use super::ILinkBuilder;

type LinkParameters = IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>;
type ContextPathHook = dyn Fn(
        &dyn IExpressionContext,
        &JavaString,
        Option<&LinkParameters>,
    ) -> Result<Option<JavaString>, TemplateProcessingException>
    + Send
    + Sync;
type ProcessLinkHook = dyn Fn(
        &dyn IExpressionContext,
        &JavaString,
    ) -> Result<Option<JavaString>, TemplateProcessingException>
    + Send
    + Sync;

/// 标准链接构建器识别的链接类型。
///
/// 对应 Java: `StandardLinkBuilder.LinkType`。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LinkType {
    Absolute,
    ContextRelative,
    ServerRelative,
    BaseRelative,
}

/// 构建绝对、上下文相对、服务器相对和基础相对 URL。
///
/// 对应 Java: `org.thymeleaf.linkbuilder.StandardLinkBuilder`。
///
/// 本实现只在 `base` 为空时返回 `None`。它先复制参数 Map，替换 path/query 中的
/// `{name}` 与 `{/name}` 模板变量，再按插入顺序附加剩余查询参数、恢复最后一个
/// fragment、删除服务器相对链接的 `~`，并按需插入应用路径。所有路径和搜索偏移均按
/// Java UTF-16 code unit 计算，URI 转义与上游 `UriEscape` 的 UTF-8 输出一致。
///
/// Java 将应用路径计算和最终 URL 重写暴露为 protected 扩展点。Rust 通过两个组合
/// 钩子承接同一动态语义，使核心保持中立且不依赖 Servlet 或特定 Web 框架。
///
/// 自 Thymeleaf 3.0.0 起提供。
pub struct StandardLinkBuilder {
    name: Option<JavaString>,
    order: Option<i32>,
    context_path_hook: Option<Arc<ContextPathHook>>,
    process_link_hook: Option<Arc<ProcessLinkHook>>,
}

impl StandardLinkBuilder {
    /// 创建使用 Java 具体类名、顺序为空且未安装扩展钩子的标准链接构建器。
    ///
    /// 对应 Java: `StandardLinkBuilder#StandardLinkBuilder()`。
    ///
    /// # 返回值
    ///
    /// 新的线程安全标准链接构建器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: Some(JavaString::from_rust_str(
                "org.thymeleaf.linkbuilder.StandardLinkBuilder",
            )),
            order: None,
            context_path_hook: None,
            process_link_hook: None,
        }
    }

    /// 组合替换 Java protected `computeContextPath` 扩展点。
    ///
    /// 对应 Java:
    /// `StandardLinkBuilder#computeContextPath(IExpressionContext,String,Map)`。
    ///
    /// # 参数
    ///
    /// - `hook`：接收上下文、原始 base 和原始参数 Map，返回可空应用路径。
    ///
    /// # 返回值
    ///
    /// 安装钩子后的构建器。
    ///
    /// # 错误
    ///
    /// 钩子可返回模板处理异常，并由链接构建调用原样传播。
    #[must_use]
    pub fn with_context_path_hook<F>(mut self, hook: F) -> Self
    where
        F: Fn(
                &dyn IExpressionContext,
                &JavaString,
                Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
            ) -> Result<Option<JavaString>, TemplateProcessingException>
            + Send
            + Sync
            + 'static,
    {
        self.context_path_hook = Some(Arc::new(hook));
        self
    }

    /// 组合替换 Java protected `processLink` 扩展点。
    ///
    /// 对应 Java: `StandardLinkBuilder#processLink(IExpressionContext,String)`。
    ///
    /// # 参数
    ///
    /// - `hook`：接收上下文和已经完整构建的 URL，可执行宿主重写或返回 `None`。
    ///
    /// # 返回值
    ///
    /// 安装钩子后的构建器。
    ///
    /// # 错误
    ///
    /// 钩子可返回模板处理异常，并由链接构建调用原样传播。
    #[must_use]
    pub fn with_process_link_hook<F>(mut self, hook: F) -> Self
    where
        F: Fn(
                &dyn IExpressionContext,
                &JavaString,
            ) -> Result<Option<JavaString>, TemplateProcessingException>
            + Send
            + Sync
            + 'static,
    {
        self.process_link_hook = Some(Arc::new(hook));
        self
    }

    /// 返回可空构建器名称。
    ///
    /// 对应 Java: `AbstractLinkBuilder#getName()`。
    ///
    /// # 返回值
    ///
    /// 当前名称；`None` 对应 Java `null`。
    #[must_use]
    pub const fn get_name(&self) -> Option<&JavaString> {
        self.name.as_ref()
    }

    /// 设置可空构建器名称。
    ///
    /// 对应 Java: `AbstractLinkBuilder#setName(String)`。
    ///
    /// # 参数
    ///
    /// - `name`：新的可空名称。
    pub fn set_name(&mut self, name: Option<JavaString>) {
        self.name = name;
    }

    /// 返回可空构建器执行顺序。
    ///
    /// 对应 Java: `AbstractLinkBuilder#getOrder()`。
    ///
    /// # 返回值
    ///
    /// 当前顺序；`None` 对应 Java `null`。
    #[must_use]
    pub const fn get_order(&self) -> Option<i32> {
        self.order
    }

    /// 设置可空构建器顺序。
    ///
    /// 对应 Java: `AbstractLinkBuilder#setOrder(Integer)`。
    ///
    /// # 参数
    ///
    /// - `order`：新的可空执行顺序。
    pub fn set_order(&mut self, order: Option<i32>) {
        self.order = order;
    }

    fn build_standard_link(
        &self,
        context: &dyn IExpressionContext,
        base: Option<&JavaString>,
        parameters: Option<&LinkParameters>,
    ) -> Result<Option<JavaString>, TemplateProcessingException> {
        let Some(base) = base else {
            return Ok(None);
        };

        filter_out_java_script_links(base)?;
        let link_type = classify_link(base);

        // Java 创建防御性、可变的 LinkedHashMap；模板变量消费不能修改调用方 Map。
        let mut link_parameters = parameters.filter(|value| !value.is_empty()).cloned();
        let hash_position = find_last_unit(base.as_utf16(), u16::from(b'#'));
        let might_have_variable_templates =
            find_last_unit(base.as_utf16(), u16::from(b'{')).is_some();
        let context_path = if link_type == LinkType::ContextRelative {
            self.compute_context_path(context, base, parameters)?
        } else {
            None
        };
        let context_path_empty = context_path
            .as_ref()
            .is_none_or(|path| path.is_empty() || path.as_utf16() == [u16::from(b'/')]);

        if context_path_empty
            && link_type != LinkType::ServerRelative
            && link_parameters.as_ref().is_none_or(IndexMap::is_empty)
            && hash_position.is_none()
            && !might_have_variable_templates
        {
            return self.process_link(context, base);
        }

        let mut link_base = base.as_utf16().to_vec();
        let mut url_fragment = Vec::new();
        if let Some(position) = hash_position.filter(|position| *position > 0) {
            url_fragment.extend_from_slice(&link_base[position..]);
            link_base.truncate(position);
        }

        if might_have_variable_templates {
            replace_template_params_in_base(&mut link_base, link_parameters.as_mut());
        }

        if let Some(parameters) = link_parameters.as_ref().filter(|value| !value.is_empty()) {
            link_base.push(if find_last_unit(&link_base, u16::from(b'?')).is_some() {
                u16::from(b'&')
            } else {
                u16::from(b'?')
            });
            process_all_remaining_parameters_as_query_params(&mut link_base, parameters);
        }

        link_base.extend_from_slice(&url_fragment);
        if link_type == LinkType::ServerRelative {
            link_base.remove(0);
        }
        if link_type == LinkType::ContextRelative
            && !context_path_empty
            && let Some(context_path) = context_path
        {
            link_base.splice(0..0, context_path.as_utf16().iter().copied());
        }

        self.process_link(context, &JavaString::from_utf16(link_base))
    }

    fn compute_context_path(
        &self,
        context: &dyn IExpressionContext,
        base: &JavaString,
        parameters: Option<&LinkParameters>,
    ) -> Result<Option<JavaString>, TemplateProcessingException> {
        if let Some(hook) = &self.context_path_hook {
            return hook(context, base, parameters);
        }
        let Some(exchange) = context.get_web_exchange() else {
            return Err(TemplateProcessingException::new(Some(format!(
                "Link base \"{}\" cannot be context relative (/...) unless the context used for \
                 executing the engine implements the org.thymeleaf.context.IWebContext interface",
                base.to_string_lossy()
            ))));
        };
        Ok(exchange.get_request().get_application_path())
    }

    fn process_link(
        &self,
        context: &dyn IExpressionContext,
        link: &JavaString,
    ) -> Result<Option<JavaString>, TemplateProcessingException> {
        if let Some(hook) = &self.process_link_hook {
            return hook(context, link);
        }
        Ok(context.get_web_exchange().map_or_else(
            || Some(link.clone()),
            |exchange| exchange.transform_url(Some(link)),
        ))
    }
}

impl Default for StandardLinkBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ILinkBuilder for StandardLinkBuilder {
    fn get_name(&self) -> Option<&JavaString> {
        self.get_name()
    }

    fn get_order(&self) -> Option<i32> {
        self.get_order()
    }

    fn build_link(
        &self,
        context: &dyn IExpressionContext,
        base: Option<&JavaString>,
        parameters: Option<&LinkParameters>,
    ) -> Result<Option<JavaString>, TemplateProcessingException> {
        self.build_standard_link(context, base, parameters)
    }
}

fn classify_link(base: &JavaString) -> LinkType {
    if is_link_base_absolute(base) {
        LinkType::Absolute
    } else if is_link_base_context_relative(base) {
        LinkType::ContextRelative
    } else if is_link_base_server_relative(base) {
        LinkType::ServerRelative
    } else {
        LinkType::BaseRelative
    }
}

fn filter_out_java_script_links(base: &JavaString) -> Result<(), TemplateProcessingException> {
    if starts_with_java_ignore_case(base.as_utf16(), b"javascript:") {
        return Err(TemplateProcessingException::new(Some(
            "'javascript:' is forbidden in this context. Link expressions cannot contain inlined \
             JavaScript code."
                .to_owned(),
        )));
    }
    Ok(())
}

fn is_link_base_absolute(base: &JavaString) -> bool {
    let units = base.as_utf16();
    if units.len() < 2 {
        return false;
    }
    if starts_with_java_ignore_case(units, b"mailto:") {
        return true;
    }
    if units.starts_with(&[u16::from(b'/'), u16::from(b'/')]) {
        return true;
    }
    units
        .windows(3)
        .any(|window| window == [u16::from(b':'), u16::from(b'/'), u16::from(b'/')])
}

fn is_link_base_context_relative(base: &JavaString) -> bool {
    let units = base.as_utf16();
    units.first() == Some(&u16::from(b'/'))
        && units.get(1).is_none_or(|unit| *unit != u16::from(b'/'))
}

fn is_link_base_server_relative(base: &JavaString) -> bool {
    base.as_utf16()
        .starts_with(&[u16::from(b'~'), u16::from(b'/')])
}

fn starts_with_java_ignore_case(units: &[u16], expected: &[u8]) -> bool {
    units.len() >= expected.len()
        && units
            .iter()
            .zip(expected)
            .all(|(unit, expected)| java_lower(*unit) == u16::from(*expected))
}

fn find_last_unit(units: &[u16], needle: u16) -> Option<usize> {
    units.iter().rposition(|unit| *unit == needle)
}

fn replace_template_params_in_base(
    link_base: &mut Vec<u16>,
    parameters: Option<&mut LinkParameters>,
) {
    let Some(parameters) = parameters else {
        return;
    };
    let question_mark_position = find_last_unit(link_base, u16::from(b'?'));
    let mut processed = Vec::new();

    for (parameter_name, parameter_value) in parameters.iter() {
        let parameter_name_text = parameter_name
            .clone()
            .unwrap_or_else(|| JavaString::from_rust_str("null"));
        let direct_template = surrounded_template(parameter_name_text.as_utf16(), false);
        let segment_template = surrounded_template(parameter_name_text.as_utf16(), true);
        let (template, escape_as_path_segment, mut start) =
            if let Some(start) = find_subsequence(link_base, &direct_template, 0) {
                (direct_template, false, start)
            } else if let Some(start) = find_subsequence(link_base, &segment_template, 0) {
                (segment_template, true, start)
            } else {
                continue;
            };

        processed.push(parameter_name.clone());
        let replacement =
            format_parameter_value_as_unescaped_variable_template(parameter_value.as_deref());
        let replacement_len = replacement.len();
        while start < link_base.len() {
            let escaped = if question_mark_position.is_none_or(|question| start < question) {
                if escape_as_path_segment {
                    escape_uri_path_segment(&replacement)
                } else {
                    escape_uri_path(&replacement)
                }
            } else {
                escape_uri_query_param(&replacement)
            };
            link_base.splice(
                start..start + template.len(),
                escaped.as_utf16().iter().copied(),
            );
            let next_start = start.saturating_add(replacement_len);
            let Some(next) = find_subsequence(link_base, &template, next_start) else {
                break;
            };
            start = next;
        }
    }

    for parameter_name in processed {
        parameters.shift_remove(&parameter_name);
    }
}

fn surrounded_template(name: &[u16], segment: bool) -> Vec<u16> {
    let mut template = Vec::with_capacity(name.len() + usize::from(segment) + 2);
    template.push(u16::from(b'{'));
    if segment {
        template.push(u16::from(b'/'));
    }
    template.extend_from_slice(name);
    template.push(u16::from(b'}'));
    template
}

fn find_subsequence(haystack: &[u16], needle: &[u16], start: usize) -> Option<usize> {
    if start > haystack.len() || needle.len() > haystack.len().saturating_sub(start) {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|position| start + position)
}

fn format_parameter_value_as_unescaped_variable_template(
    parameter_value: Option<&TemplateValue>,
) -> JavaString {
    match parameter_value {
        None | Some(TemplateValue::Null) => JavaString::from_utf16(Vec::new()),
        Some(TemplateValue::List(values)) => {
            let mut result = Vec::new();
            for value in values.iter() {
                // Java 只在当前 StringBuilder 已有内容时加入逗号；前导 null/空值不产生逗号。
                if !result.is_empty() {
                    result.push(u16::from(b','));
                }
                if !matches!(value.as_ref(), TemplateValue::Null) {
                    result.extend_from_slice(java_value_string(value).as_utf16());
                }
            }
            JavaString::from_utf16(result)
        }
        Some(value) => java_value_string(value),
    }
}

fn process_all_remaining_parameters_as_query_params(
    result: &mut Vec<u16>,
    parameters: &LinkParameters,
) {
    let mut parameter_index = 0usize;
    for (parameter_name, value) in parameters {
        let parameter_name = parameter_name
            .clone()
            .unwrap_or_else(|| JavaString::from_rust_str("null"));
        match value.as_deref() {
            None | Some(TemplateValue::Null) => {
                if parameter_index > 0 {
                    result.push(u16::from(b'&'));
                }
                append_java_string(result, &escape_uri_query_param(&parameter_name));
                parameter_index += 1;
                continue;
            }
            Some(TemplateValue::List(values)) => {
                for (value_index, value) in values.iter().enumerate() {
                    if parameter_index > 0 || value_index > 0 {
                        result.push(u16::from(b'&'));
                    }
                    append_java_string(result, &escape_uri_query_param(&parameter_name));
                    if !matches!(value.as_ref(), TemplateValue::Null) {
                        result.push(u16::from(b'='));
                        append_java_string(
                            result,
                            &escape_uri_query_param(&java_value_string(value)),
                        );
                    }
                }
            }
            Some(value) => {
                if parameter_index > 0 {
                    result.push(u16::from(b'&'));
                }
                append_java_string(result, &escape_uri_query_param(&parameter_name));
                result.push(u16::from(b'='));
                append_java_string(result, &escape_uri_query_param(&java_value_string(value)));
            }
        }
        parameter_index += 1;
    }
}

fn java_value_string(value: &TemplateValue) -> JavaString {
    value
        .to_java_string()
        .unwrap_or_else(|| JavaString::from_rust_str("null"))
}

fn append_java_string(result: &mut Vec<u16>, value: &JavaString) {
    result.extend_from_slice(value.as_utf16());
}

fn escape_uri_path(value: &JavaString) -> JavaString {
    percent_escape(value, |byte| is_pchar(byte) || byte == b'/')
}

fn escape_uri_path_segment(value: &JavaString) -> JavaString {
    percent_escape(value, is_pchar)
}

fn escape_uri_query_param(value: &JavaString) -> JavaString {
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

fn percent_escape(value: &JavaString, allowed: impl Fn(u8) -> bool) -> JavaString {
    let units = value.as_utf16();
    let mut result = Vec::with_capacity(units.len());
    let mut index = 0usize;
    while index < units.len() {
        let unit = units[index];
        if unit <= 0x7f {
            let byte = unit as u8;
            if allowed(byte) {
                result.push(unit);
            } else {
                append_percent_byte(&mut result, byte);
            }
            index += 1;
            continue;
        }

        if (0xd800..=0xdbff).contains(&unit)
            && let Some(low) = units.get(index + 1).copied()
            && (0xdc00..=0xdfff).contains(&low)
        {
            let scalar = 0x1_0000 + ((u32::from(unit) - 0xd800) << 10) + (u32::from(low) - 0xdc00);
            let character = char::from_u32(scalar).expect("valid surrogate pair");
            let mut buffer = [0u8; 4];
            for byte in character.encode_utf8(&mut buffer).as_bytes() {
                append_percent_byte(&mut result, *byte);
            }
            index += 2;
            continue;
        }

        if (0xd800..=0xdfff).contains(&unit) {
            // JDK UTF-8 encoder replaces each isolated surrogate with '?'，而 UriEscape 会继续
            // 百分号编码该替代字节，因为原输入 code unit 不是 ASCII。
            append_percent_byte(&mut result, b'?');
            index += 1;
            continue;
        }

        let character = char::from_u32(u32::from(unit)).expect("non-surrogate BMP unit");
        let mut buffer = [0u8; 3];
        for byte in character.encode_utf8(&mut buffer).as_bytes() {
            append_percent_byte(&mut result, *byte);
        }
        index += 1;
    }
    JavaString::from_utf16(result)
}

fn append_percent_byte(result: &mut Vec<u16>, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    result.push(u16::from(b'%'));
    result.push(u16::from(HEX[usize::from(byte >> 4)]));
    result.push(u16::from(HEX[usize::from(byte & 0x0f)]));
}
