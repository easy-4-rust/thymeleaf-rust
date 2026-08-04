//! `.thtest` 语料共享驱动机制（CONTEXT/命名 INPUT/命名模板解析器）。
//!
//! 从 `thtest_upstream_plain_batch.rs` 提取，供批量语料运行器与各 fixture
//! 差分测试共用，避免复制漂移：
//! - `%CONTEXT` 按 Java `java.util.Properties` + OGNL 语义解析（行续行、
//!   括号配平、赋值切分、`\uXXXX` 解码、Map/List 字面量、`param.*` 等
//!   变异目标）；
//! - `%INPUT[name]` 命名片段与 `%TEMPLATE_MODE[name]` 命名模式注册；
//! - 根模板名解析：任意缺失模板名不当作模板正文（Java 测试解析器语义）。

use std::sync::Arc;

use indexmap::IndexMap;
use thymeleaf::context::{Context, ExpressionContext, IContext};
use thymeleaf::expression::{IStandardExpression, TemplateValue, VariableExpression};
use thymeleaf::templateresolver::{
    StringTemplateResolver, TemplateResolution, TemplateResolverError,
};
use thymeleaf::util::{Locale, Utf16String};
use thymeleaf::{
    IEngineConfiguration, ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode,
    TemplateResolutionAttributes,
};

use super::CorpusRequestParameterValues;

// ===========================================================================
// `%CONTEXT` 解析（Properties + OGNL 语义）
// ===========================================================================

/// Java `java.util.Properties.load` 的转义解码。
pub fn decode_java_properties_value(input: &str) -> Result<String, String> {
    let characters = input.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(input.len());
    let mut position = 0_usize;
    while position < characters.len() {
        if characters[position] != '\\' {
            output.push(characters[position]);
            position += 1;
            continue;
        }
        position += 1;
        let escaped = *characters
            .get(position)
            .ok_or_else(|| "CONTEXT property value ends with an escape prefix".to_owned())?;
        match escaped {
            't' => output.push('\t'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            'f' => output.push('\u{000c}'),
            'u' => {
                let end = position + 5;
                let digits = characters.get(position + 1..end).ok_or_else(|| {
                    "CONTEXT property contains an incomplete Unicode escape".to_owned()
                })?;
                let hexadecimal = digits.iter().collect::<String>();
                let code_unit = u16::from_str_radix(&hexadecimal, 16).map_err(|_| {
                    format!("CONTEXT property contains invalid Unicode escape: \\u{hexadecimal}")
                })?;
                let decoded = char::decode_utf16([code_unit])
                    .next()
                    .expect("one UTF-16 code unit always produces one decode result")
                    .map_err(|_| {
                        format!("CONTEXT property contains an unpaired surrogate: \\u{hexadecimal}")
                    })?;
                output.push(decoded);
                position = end - 1;
            }
            // java.util.Properties 对其余转义只删除反斜杠，包括空格、
            // 分隔符、注释前缀及普通引号。
            value => output.push(value),
        }
        position += 1;
    }
    Ok(output)
}

/// 把 `%CONTEXT` 拆成 `name = expression` 赋值单元（含反斜杠行续行与
/// 括号/引号配平）。
pub fn split_context_assignments(context: &str) -> Result<Vec<String>, String> {
    let source = context
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let units = source.chars().collect::<Vec<_>>();
    let mut current = String::with_capacity(source.len());
    let mut assignments = Vec::new();
    let mut parentheses = 0_i32;
    let mut brackets = 0_i32;
    let mut braces = 0_i32;
    let mut quote = None;
    let mut position = 0_usize;
    while position < units.len() {
        let character = units[position];
        if character == '\\' && units.get(position + 1) == Some(&'\n') {
            // Properties.load 在 OGNL 看到值之前先删除物理行续行；该规则在
            // 字符串字面量内部同样生效。
            position += 2;
            continue;
        }
        if let Some(active_quote) = quote {
            current.push(character);
            if character == active_quote && !is_escaped_character(&units, position) {
                quote = None;
            }
            position += 1;
            continue;
        }
        match character {
            '\'' | '"' => {
                quote = Some(character);
                current.push(character);
            }
            '(' => {
                parentheses += 1;
                current.push(character);
            }
            ')' => {
                parentheses -= 1;
                current.push(character);
            }
            '[' => {
                brackets += 1;
                current.push(character);
            }
            ']' => {
                brackets -= 1;
                current.push(character);
            }
            '{' => {
                braces += 1;
                current.push(character);
            }
            '}' => {
                braces -= 1;
                current.push(character);
            }
            ',' | '\n' if parentheses == 0 && brackets == 0 && braces == 0 => {
                let assignment = current.trim();
                if !assignment.is_empty() {
                    assignments.push(assignment.to_owned());
                }
                current.clear();
            }
            _ => current.push(character),
        }
        if parentheses < 0 || brackets < 0 || braces < 0 {
            return Err("unbalanced CONTEXT delimiters".to_owned());
        }
        position += 1;
    }
    if quote.is_some() || parentheses != 0 || brackets != 0 || braces != 0 {
        return Err("unterminated CONTEXT literal or delimiter".to_owned());
    }
    let assignment = current.trim();
    if !assignment.is_empty() {
        assignments.push(assignment.to_owned());
    }
    Ok(assignments)
}

/// 把单个 `name = expression` 赋值单元拆成名称与表达式。
pub fn split_context_assignment(assignment: &str) -> Result<(&str, &str), String> {
    let (name, expression) = assignment
        .split_once('=')
        .ok_or_else(|| format!("CONTEXT assignment has no `=`: {assignment}"))?;
    let name = name.trim();
    let expression = expression.trim();
    if name.is_empty() || expression.is_empty() {
        return Err(format!("Invalid CONTEXT assignment: {assignment}"));
    }
    Ok((name, expression))
}

/// 判断 CONTEXT 目标是否为简单变量名（Java 测试框架对简单名直接绑定）。
pub fn is_simple_context_name(name: &str) -> bool {
    name.chars().enumerate().all(|(index, character)| {
        character == '_' || character.is_alphanumeric() && (index > 0 || !character.is_numeric())
    })
}

/// Properties 原文中反斜杠是否转义了引号（每两个反斜杠先折叠为一个）。
fn is_escaped_character(input: &[char], position: usize) -> bool {
    let mut slashes = 0_usize;
    let mut cursor = position;
    while cursor > 0 && input[cursor - 1] == '\\' {
        slashes += 1;
        cursor -= 1;
    }
    (slashes / 2) % 2 == 1
}

/// 解析 Java 语言环境说明（`en`、`en_US`、`en_US_variant`）。
pub fn parse_locale(value: &str) -> Result<Locale, String> {
    let parts = value.split('_').collect::<Vec<_>>();
    if parts.is_empty() || parts.len() > 3 || parts[0].is_empty() {
        return Err(format!("Invalid locale specification: {value}"));
    }
    let country = parts.get(1).copied().unwrap_or("").to_ascii_uppercase();
    let mut tag = parts[0].to_ascii_lowercase();
    if !country.is_empty() {
        tag.push('-');
        tag.push_str(&country);
    }
    if let Some(variant) = parts.get(2).filter(|variant| !variant.is_empty()) {
        tag.push('-');
        tag.push_str(variant);
    }
    Ok(Locale::new(
        Utf16String::from_rust_str(&tag),
        Utf16String::from_rust_str(&country),
    ))
}

/// 把 CONTEXT 赋值写入普通 Context 与 ExpressionContext（对应 Java 测试框架
/// `DefaultContextStandardTestFieldEvaluator` 的两种写入口）。
pub fn build_context(engine: &TemplateEngine, source: Option<&str>) -> Result<Context, String> {
    let default_locale = Locale::new(
        Utf16String::from_rust_str("en"),
        Utf16String::from_rust_str(""),
    );
    let context = Context::with_locale(Some(default_locale.clone()));
    let configuration = engine
        .get_configuration()
        .map_err(|error| error.to_string())?;
    let expression_context =
        ExpressionContext::new(Some(configuration)).map_err(|error| error.to_string())?;
    expression_context
        .set_locale(Some(default_locale))
        .map_err(|error| error.to_string())?;
    // Java 测试框架的 WebProcessingContextBuilder 总是暴露四个 Web 作用域。
    // 即使测试尚未向其中写入值，表达式也应看到空 Map，而不是 null。
    for scope_name in ["param", "request", "session", "application"] {
        let name = Utf16String::from_rust_str(scope_name);
        let value = Some(Arc::new(TemplateValue::Map(Arc::new(Vec::new()))));
        context.set_variable(Some(name.clone()), value.clone());
        expression_context.set_variable(Some(name), value);
    }
    let Some(source) = source else {
        return Ok(context);
    };
    for assignment in split_context_assignments(source)? {
        let (name, expression) = split_context_assignment(&assignment)?;
        // Java 基准的 DefaultContextStandardTestFieldEvaluator 先通过
        // java.util.Properties.load 读取 `%CONTEXT`，之后才把值交给 OGNL。
        // 这里必须保留同一层夹具语义，否则 `\\'`、`\uXXXX` 等会被错误地
        // 当成 OGNL 自身的转义。
        let expression = decode_java_properties_value(expression)?;
        let expression = VariableExpression::new(Some(Utf16String::from_rust_str(&expression)))
            .map_err(|error| format!("CONTEXT `{assignment}`: {error}"))?;
        let value = expression
            .execute(expression_context.as_ref())
            .map_err(|error| format!("CONTEXT `{assignment}`: {error}"))?;
        if name.eq_ignore_ascii_case("locale") {
            if let Some(locale) = value
                .as_deref()
                .and_then(TemplateValue::to_utf16_string)
                .map(|locale| parse_locale(&locale.to_string_lossy()))
                .transpose()?
            {
                context
                    .set_locale(Some(locale.clone()))
                    .map_err(|error| error.to_string())?;
                expression_context
                    .set_locale(Some(locale))
                    .map_err(|error| error.to_string())?;
            }
            continue;
        }
        if !is_simple_context_name(name) {
            apply_context_mutation(&context, &expression_context, name, value, &assignment)?;
            continue;
        }
        let name = Utf16String::from_rust_str(name);
        if std::env::var_os("THYMELEAF_DEBUG_CONTEXT").is_some() {
            eprintln!("CONTEXT {} = {value:?}", name.to_string_lossy());
        }
        context.remove_variable(Some(&name));
        expression_context.remove_variable(Some(&name));
        expression_context.set_variable(Some(name.clone()), value.clone());
        context.set_variable(Some(name), value);
    }
    Ok(context)
}

/// 非简单 CONTEXT 目标（`param.x`、`request[x]`、`a.b.c` 等）的变异写入。
pub fn apply_context_mutation(
    context: &Context,
    expression_context: &ExpressionContext,
    target: &str,
    value: Option<Arc<TemplateValue>>,
    assignment: &str,
) -> Result<(), String> {
    let bracket_position = target.find('[');
    let dot_position = target.find('.');
    let (root, key_expressions, request_parameter) = if bracket_position
        .is_some_and(|bracket| dot_position.is_none_or(|dot| bracket < dot))
    {
        let bracket = bracket_position.expect("checked above");
        let root = target[..bracket].trim();
        let key = target
            .get(bracket + 1..target.len().saturating_sub(1))
            .filter(|_| target.ends_with(']'))
            .ok_or_else(|| format!("Unsupported CONTEXT mutation target: {target}"))?;
        (root, vec![key.to_owned()], false)
    } else if let Some((root, properties)) = target.split_once('.') {
        (
            root.trim(),
            properties
                .split('.')
                // OGNL 的单引号单字符字面量是 Character；Web 作用域属性名必须
                // 保持为 String，否则 `session.a` 写入的键无法由属性导航读回。
                .map(|property| format!("\"{property}\""))
                .collect::<Vec<_>>(),
            root.trim() == "param",
        )
    } else {
        return Err(format!(
            "CONTEXT assignment is not a supported variable binding or map mutation: {assignment}"
        ));
    };
    if !is_simple_context_name(root) {
        return Err(format!("Unsupported CONTEXT mutation root: {root}"));
    }
    let keys = key_expressions
        .iter()
        .map(|key_expression| {
            let key_expression = decode_java_properties_value(key_expression)?;
            VariableExpression::new(Some(Utf16String::from_rust_str(&key_expression)))
                .map_err(|error| format!("CONTEXT `{assignment}` key: {error}"))?
                .execute(expression_context)
                .map_err(|error| format!("CONTEXT `{assignment}` key: {error}"))
                .map(|value| value.unwrap_or_else(|| Arc::new(TemplateValue::Null)))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let root_name = Utf16String::from_rust_str(root);
    let current = match expression_context.get_variable(Some(&root_name)) {
        Some(value) if matches!(value.as_ref(), TemplateValue::Map(_)) => value,
        None if request_parameter => Arc::new(TemplateValue::Map(Arc::new(Vec::new()))),
        _ => {
            return Err(format!(
                "CONTEXT mutation root `{root}` is not a map: {assignment}"
            ));
        }
    };
    let value = value.unwrap_or_else(|| Arc::new(TemplateValue::Null));
    let updated_value = if request_parameter {
        update_request_parameter_map(current.as_ref(), &keys, Arc::clone(&value))?
    } else {
        update_context_map_path(current.as_ref(), &keys, Arc::clone(&value))?
    };
    let updated = Some(updated_value);
    expression_context.set_variable(Some(root_name.clone()), updated.clone());
    context.set_variable(Some(root_name.clone()), updated);
    if root == "request"
        && let [key] = keys.as_slice()
        && let Some(attribute_name) = key.to_utf16_string()
    {
        // WebProcessingContextBuilder 把 request.* 写入 exchange 属性；
        // WebEngineContext 对普通变量名也从该作用域读取。
        expression_context.set_variable(Some(attribute_name.clone()), Some(Arc::clone(&value)));
        context.set_variable(Some(attribute_name), Some(value));
    }
    if std::env::var_os("THYMELEAF_DEBUG_CONTEXT").is_some() {
        eprintln!(
            "CONTEXT mutation {assignment} => {:?}",
            context.get_variable(Some(&root_name))
        );
    }
    Ok(())
}

/// 请求参数映射追加一个同名值（对应 Java `String[]` 追加语义）。
fn update_request_parameter_map(
    current: &TemplateValue,
    keys: &[Arc<TemplateValue>],
    value: Arc<TemplateValue>,
) -> Result<Arc<TemplateValue>, String> {
    let [key] = keys else {
        return Err("request parameter mutation requires exactly one key".to_owned());
    };
    let TemplateValue::Map(current_entries) = current else {
        return Err("request parameter root is not a map".to_owned());
    };
    let mut entries = current_entries.as_ref().clone();
    if let Some((_, current_value)) = entries
        .iter_mut()
        .find(|(candidate, _)| candidate.template_equals(key.as_ref()))
    {
        let TemplateValue::Object(values) = current_value.as_ref() else {
            return Err("request parameter value is not an object".to_owned());
        };
        let values = values
            .as_any()
            .downcast_ref::<CorpusRequestParameterValues>()
            .ok_or_else(|| "request parameter value has an unexpected type".to_owned())?;
        *current_value = Arc::new(TemplateValue::Object(Arc::new(values.with_appended(value))));
    } else {
        entries.push((
            Arc::clone(key),
            Arc::new(TemplateValue::Object(Arc::new(
                CorpusRequestParameterValues::new(value),
            ))),
        ));
    }
    Ok(Arc::new(TemplateValue::Map(Arc::new(entries))))
}

/// 按键路径在 Map 值上写入/更新（含 `MILLISECONDS`/`SECONDS` 排序特例）。
fn update_context_map_path(
    current: &TemplateValue,
    keys: &[Arc<TemplateValue>],
    value: Arc<TemplateValue>,
) -> Result<Arc<TemplateValue>, String> {
    let Some((key, remaining)) = keys.split_first() else {
        return Ok(value);
    };
    let TemplateValue::Map(current_entries) = current else {
        return Err("CONTEXT nested mutation crossed a non-map value".to_owned());
    };
    let mut entries = current_entries.as_ref().clone();
    if let Some((_, existing)) = entries
        .iter_mut()
        .find(|(candidate, _)| candidate.template_equals(key.as_ref()))
    {
        *existing = if remaining.is_empty() {
            value
        } else {
            update_context_map_path(existing.as_ref(), remaining, value)?
        };
    } else {
        let inserted = if remaining.is_empty() {
            value
        } else {
            update_context_map_path(&TemplateValue::Map(Arc::new(Vec::new())), remaining, value)?
        };
        entries.push((Arc::clone(key), inserted));
    }
    if entries.iter().all(|(key, _)| {
        key.to_utf16_string()
            .is_some_and(|key| matches!(key.to_string_lossy().as_str(), "MILLISECONDS" | "SECONDS"))
    }) {
        entries.sort_by_key(|(key, _)| {
            key.to_utf16_string()
                .map_or(usize::MAX, |key| match key.to_string_lossy().as_str() {
                    "MILLISECONDS" => 0,
                    "SECONDS" => 1,
                    _ => usize::MAX,
                })
        });
    }
    Ok(Arc::new(TemplateValue::Map(Arc::new(entries))))
}

// ===========================================================================
// `%INPUT[name]` 命名片段与命名模板模式
// ===========================================================================

/// 提取标记所在行之后直到下一个 `%` 指令的正文（剔除列首 `#` 描述行）。
pub fn directive_section_for_marker(source: &str, marker: &str) -> Option<String> {
    let mut lines = source.split_inclusive('\n');
    lines.find(|line| line.trim_end() == marker)?;
    let mut section = String::new();
    for line in lines {
        if line.starts_with('%') {
            break;
        }
        // thymeleaf-testing 把列首 `#` 识别为测试描述或分隔线，不属于模板内容。
        if line.starts_with('#') {
            continue;
        }
        section.push_str(line);
    }
    while section.ends_with("\r\n") {
        section.truncate(section.len() - 2);
    }
    while section.ends_with('\n') {
        section.pop();
    }
    Some(section)
}

/// 提取 `%INPUT[qualifier]` 命名片段（按声明顺序）。
pub fn named_input_sections(source: &str) -> Result<IndexMap<Utf16String, Utf16String>, String> {
    let mut templates = IndexMap::new();
    for line in source.lines() {
        let Some(qualifier) = line
            .strip_prefix("%INPUT[")
            .and_then(|line| line.strip_suffix(']'))
        else {
            continue;
        };
        if qualifier.is_empty() {
            return Err("INPUT qualifier cannot be empty".to_owned());
        }
        let marker = format!("%INPUT[{qualifier}]");
        let content = directive_section_for_marker(source, &marker)
            .ok_or_else(|| format!("missing section for {marker}"))?;
        templates.insert(
            Utf16String::from_rust_str(qualifier),
            Utf16String::from_rust_str(&content),
        );
    }
    Ok(templates)
}

/// 提取 `%TEMPLATE_MODE[qualifier]` 命名模板模式。
pub fn named_template_modes(source: &str) -> Result<IndexMap<Utf16String, TemplateMode>, String> {
    let mut modes = IndexMap::new();
    for line in source.lines() {
        let Some((marker, value)) = line
            .strip_prefix("%TEMPLATE_MODE[")
            .and_then(|line| line.split_once("] "))
        else {
            continue;
        };
        if marker.is_empty() {
            return Err("TEMPLATE_MODE qualifier cannot be empty".to_owned());
        }
        let mode = value
            .trim()
            .parse::<TemplateMode>()
            .map_err(|error| error.to_string())?;
        modes.insert(Utf16String::from_rust_str(marker), mode);
    }
    Ok(modes)
}

// ===========================================================================
// 命名模板解析器
// ===========================================================================

/// Java `.thtest` 语料的测试解析器不会把任意缺失模板名本身当作模板正文；直接使用
/// `StringTemplateResolver` 会把 `~{fragg}` 错误解析成文本 `fragg`。
pub struct CorpusStringTemplateResolver {
    delegate: StringTemplateResolver,
    root_template_name: Utf16String,
    root_template: Utf16String,
    named_templates: IndexMap<Utf16String, Utf16String>,
    named_template_modes: IndexMap<Utf16String, TemplateMode>,
}

impl CorpusStringTemplateResolver {
    /// 创建绑定根模板与命名片段的解析器。
    #[must_use]
    pub fn new(
        mode: TemplateMode,
        root_template_name: &str,
        root_template: &str,
        named_templates: IndexMap<Utf16String, Utf16String>,
        named_template_modes: IndexMap<Utf16String, TemplateMode>,
    ) -> Self {
        let mut delegate = StringTemplateResolver::new();
        delegate.set_template_mode(mode);
        Self {
            delegate,
            root_template_name: Utf16String::from_rust_str(root_template_name),
            root_template: Utf16String::from_rust_str(root_template),
            named_templates,
            named_template_modes,
        }
    }
}

impl ITemplateResolver for CorpusStringTemplateResolver {
    fn get_name(&self) -> Option<&Utf16String> {
        self.delegate.get_name()
    }

    fn get_order(&self) -> Option<i32> {
        self.delegate.get_order()
    }

    fn resolve_template(
        &self,
        configuration: &dyn IEngineConfiguration,
        owner_template: Option<&Utf16String>,
        template: &Utf16String,
        attributes: Option<&TemplateResolutionAttributes>,
    ) -> Result<Option<TemplateResolution>, TemplateResolverError> {
        if template == &self.root_template_name {
            return self.delegate.resolve_template(
                configuration,
                owner_template,
                &self.root_template,
                attributes,
            );
        }
        if let Some(content) = self.named_templates.get(template) {
            let Some(mode) = self.named_template_modes.get(template) else {
                return self.delegate.resolve_template(
                    configuration,
                    owner_template,
                    content,
                    attributes,
                );
            };
            let mut resolver = StringTemplateResolver::new();
            resolver.set_template_mode(*mode);
            return resolver.resolve_template(configuration, owner_template, content, attributes);
        }
        if owner_template.is_some_and(|owner| owner != template) {
            return Ok(None);
        }
        self.delegate
            .resolve_template(configuration, owner_template, template, attributes)
    }
}
