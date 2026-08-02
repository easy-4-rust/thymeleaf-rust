use std::fmt::{Display, Write};

use crate::dialect::IDialect;
use crate::postprocessor::IPostProcessor;
use crate::preprocessor::IPreProcessor;
use crate::processor::IProcessor;
use crate::util::{JavaString, ProcessorComparators};
use crate::{IEngineConfiguration, TemplateMode, Thymeleaf};

/// 模板引擎配置的诊断日志格式化器。
///
/// 该内部对象按照当前日志级别输出 Resolver、LinkBuilder、方言及方言贡献的全部
/// Processor、表达式对象和执行属性。TRACE 优先于 DEBUG；未启用这两个级别时不会
/// 发出配置日志。
///
/// 对应 Java: `org.thymeleaf.ConfigurationPrinterHelper`。
///
/// # 起始版本
///
/// 上游自 Thymeleaf 1.0 起提供该对象。
pub(crate) struct ConfigurationPrinterHelper;

impl ConfigurationPrinterHelper {
    /// 构建并按当前 tracing 级别输出完整引擎配置。
    ///
    /// DEBUG 和 TRACE 都会包含方言的详细贡献；TRACE 同时启用时只发出 TRACE 事件。
    /// 返回值是 Rust 内部的可测试适配，不改变 Java `void` 方法的日志副作用。
    ///
    /// 对应 Java:
    /// `ConfigurationPrinterHelper#printConfiguration(IEngineConfiguration)`。
    ///
    /// # 参数
    ///
    /// - `configuration`：已经初始化并冻结的线程安全引擎配置。
    ///
    /// # 返回值
    ///
    /// 返回实际写入日志事件的完整消息；末尾不附加换行。
    pub(crate) fn print_configuration(configuration: &dyn IEngineConfiguration) -> String {
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        let debug_enabled = tracing::enabled!(tracing::Level::DEBUG);
        let output = Self::render_configuration(configuration, debug_enabled || trace_enabled);
        // Java 在 TRACE 与 DEBUG 同时开启时只走 TRACE 分支。
        if trace_enabled {
            tracing::trace!("{output}");
        } else if debug_enabled {
            tracing::debug!("{output}");
        }
        output
    }

    /// 构建完整配置诊断文本。
    ///
    /// 该函数把 Java 日志级别分支变成显式参数，以便固定上游 Golden 可以逐字验证
    /// DEBUG 与非 DEBUG 两种内容；生产入口仍从 tracing 元数据获取真实级别。
    fn render_configuration(
        configuration: &dyn IEngineConfiguration,
        debug_enabled: bool,
    ) -> String {
        let mut log = ConfigLogBuilder::new();
        log.line("Initializing Thymeleaf Template engine configuration...");
        log.line("[THYMELEAF] TEMPLATE ENGINE CONFIGURATION:");

        let version = Thymeleaf::get_version();
        if !version.trim().is_empty() {
            match Thymeleaf::get_build_timestamp().filter(|value| !value.trim().is_empty()) {
                Some(timestamp) => log.parameters(
                    "[THYMELEAF] * Thymeleaf version: {} (built {})",
                    &[&version, &timestamp],
                ),
                None => log.parameter("[THYMELEAF] * Thymeleaf version: {}", version),
            }
        }

        log.parameter(
            "[THYMELEAF] * Cache Manager implementation: {}",
            configuration
                .get_cache_manager()
                .map_or("[no caches]", |cache_manager| {
                    cache_manager.java_class_name()
                }),
        );

        log.line("[THYMELEAF] * Template resolvers:");
        for resolver in configuration.get_template_resolvers() {
            resolver_line(&mut log, resolver.get_order(), resolver.get_name());
        }
        log.line("[THYMELEAF] * Message resolvers:");
        for resolver in configuration.get_message_resolvers() {
            resolver_line(&mut log, resolver.get_order(), resolver.get_name());
        }
        log.line("[THYMELEAF] * Link builders:");
        for builder in configuration.get_link_builders() {
            resolver_line(&mut log, builder.get_order(), builder.get_name());
        }

        let dialects = configuration.get_dialect_configurations();
        for (index, dialect_configuration) in dialects.iter().enumerate() {
            let dialect = dialect_configuration.get_dialect();
            let dialect_name = dialect.get_name().map(str::to_owned);
            let dialect_class_name = dialect.java_class_name().to_owned();
            if dialects.len() > 1 {
                log.optional_parameters(
                    "[THYMELEAF] * Dialect [{} of {}]: {} ({})",
                    &[
                        Some(&(index + 1)),
                        Some(&dialects.len()),
                        dialect_name.as_ref().map(|value| value as &dyn Display),
                        Some(&dialect_class_name),
                    ],
                );
            } else {
                log.optional_parameters(
                    "[THYMELEAF] * Dialect: {} ({})",
                    &[
                        dialect_name.as_ref().map(|value| value as &dyn Display),
                        Some(&dialect_class_name),
                    ],
                );
            }

            let mut dialect_prefix = None;
            if let Some(processor_dialect) = dialect.as_processor_dialect() {
                dialect_prefix = if dialect_configuration.is_prefix_specified() {
                    dialect_configuration.get_prefix()
                } else {
                    processor_dialect.get_prefix()
                };
                log.parameter(
                    "[THYMELEAF]     * Prefix: \"{}\"",
                    dialect_prefix.unwrap_or("(none)"),
                );
            }

            if debug_enabled {
                print_debug_configuration(&mut log, dialect, dialect_prefix);
            }
        }

        log.end("[THYMELEAF] TEMPLATE ENGINE CONFIGURED OK");
        log.to_string()
    }
}

/// 输出单个方言在 DEBUG 级别可见的全部配置贡献。
///
/// 对应 Java:
/// `ConfigurationPrinterHelper#printDebugConfiguration(ConfigLogBuilder, IDialect, String)`。
fn print_debug_configuration(
    log: &mut ConfigLogBuilder,
    dialect: &dyn IDialect,
    dialect_prefix: Option<&str>,
) {
    if let Some(processor_dialect) = dialect.as_processor_dialect()
        && let Some(processors) = processor_dialect.get_processors(dialect_prefix)
    {
        for template_mode in template_modes() {
            print_processors_for_template_mode(log, &processors, template_mode);
        }
    }

    if let Some(pre_processor_dialect) = dialect.as_pre_processor_dialect()
        && let Some(pre_processors) = pre_processor_dialect.get_pre_processors()
    {
        for template_mode in template_modes() {
            print_pre_processors_for_template_mode(log, &pre_processors, template_mode);
        }
    }

    if let Some(post_processor_dialect) = dialect.as_post_processor_dialect()
        && let Some(post_processors) = post_processor_dialect.get_post_processors()
    {
        for template_mode in template_modes() {
            print_post_processors_for_template_mode(log, &post_processors, template_mode);
        }
    }

    if let Some(expression_dialect) = dialect.as_expression_object_dialect()
        && let Some(factory) = expression_dialect.get_expression_object_factory()
        && let Some(names) = factory.get_all_expression_object_names()
        && !names.is_empty()
    {
        log.line("[THYMELEAF]     * Expression Objects:");
        for name in names.iter() {
            let name = name.as_ref().map(JavaString::to_string_lossy);
            log.optional_parameter(
                "[THYMELEAF]         * #{}",
                name.as_ref().map(|value| value as &dyn Display),
            );
        }
    }

    if let Some(execution_dialect) = dialect.as_execution_attribute_dialect()
        && let Some(attributes) = execution_dialect.get_execution_attributes()
        && !attributes.is_empty()
    {
        log.line("[THYMELEAF]     * Execution Attributes:");
        for (name, value) in attributes {
            let rendered_value = value.as_ref().map(|value| value.diagnostic_string());
            log.optional_parameters(
                "[THYMELEAF]         * \"{}\": {}",
                &[
                    name.as_ref().map(|value| value as &dyn Display),
                    rendered_value.as_ref().map(|value| value as &dyn Display),
                ],
            );
        }
    }
}

/// 按模板模式分类、排序并输出普通 Processor。
///
/// Processor 同时实现多个子接口时只进入 Java `instanceof` 链命中的第一个分类。
/// 对应 Java:
/// `ConfigurationPrinterHelper#printProcessorsForTemplateMode(...)`。
fn print_processors_for_template_mode(
    log: &mut ConfigLogBuilder,
    processors: &crate::processor::ProcessorSet,
    template_mode: TemplateMode,
) {
    if processors.is_empty() {
        return;
    }

    let mut cdata_section_processors = Vec::<&dyn IProcessor>::new();
    let mut comment_processors = Vec::<&dyn IProcessor>::new();
    let mut doc_type_processors = Vec::<&dyn IProcessor>::new();
    let mut element_tag_processors = Vec::<&dyn IProcessor>::new();
    let mut element_model_processors = Vec::<&dyn IProcessor>::new();
    let mut processing_instruction_processors = Vec::<&dyn IProcessor>::new();
    let mut text_processors = Vec::<&dyn IProcessor>::new();
    let mut xml_declaration_processors = Vec::<&dyn IProcessor>::new();
    let mut processors_for_template_mode_exist = false;

    for processor in processors.iter().flatten() {
        let processor = processor.as_ref();
        if processor.get_template_mode() != Some(template_mode) {
            continue;
        }
        processors_for_template_mode_exist = true;

        // 保持 Java 的互斥 instanceof/else-if 分类顺序。
        if processor.as_cdata_section_processor().is_some() {
            cdata_section_processors.push(processor);
        } else if processor.as_comment_processor().is_some() {
            comment_processors.push(processor);
        } else if processor.as_doc_type_processor().is_some() {
            doc_type_processors.push(processor);
        } else if processor
            .as_element_processor()
            .and_then(|element| element.as_element_tag_processor())
            .is_some()
        {
            element_tag_processors.push(processor);
        } else if processor
            .as_element_processor()
            .and_then(|element| element.as_element_model_processor())
            .is_some()
        {
            element_model_processors.push(processor);
        } else if processor.as_processing_instruction_processor().is_some() {
            processing_instruction_processors.push(processor);
        } else if processor.as_text_processor().is_some() {
            text_processors.push(processor);
        } else if processor.as_xml_declaration_processor().is_some() {
            xml_declaration_processors.push(processor);
        }
    }

    if !processors_for_template_mode_exist {
        // 当前模式没有任何 Processor 时，Java 不输出标题。
        return;
    }

    log.parameter(
        "[THYMELEAF]     * Processors for Template Mode: {}",
        template_mode,
    );

    for values in [
        &mut cdata_section_processors,
        &mut comment_processors,
        &mut doc_type_processors,
        &mut element_tag_processors,
        &mut element_model_processors,
        &mut processing_instruction_processors,
        &mut text_processors,
        &mut xml_declaration_processors,
    ] {
        values.sort_by(|left, right| ProcessorComparators::compare_processors(*left, *right));
    }

    if !element_tag_processors.is_empty() {
        log.line("[THYMELEAF]         * Element Tag Processors by [matching element and attribute name] [precedence]:");
        for processor in element_tag_processors {
            print_element_processor(log, processor);
        }
    }
    if !element_model_processors.is_empty() {
        log.line("[THYMELEAF]         * Element Model Processors by [matching element and attribute name] [precedence]:");
        for processor in element_model_processors {
            print_element_processor(log, processor);
        }
    }
    print_processor_group(log, "Text Processors", &text_processors);
    print_processor_group(log, "DOCTYPE Processors", &doc_type_processors);
    print_processor_group(log, "CDATA Section Processors", &cdata_section_processors);
    print_processor_group(log, "Comment Processors", &comment_processors);
    print_processor_group(
        log,
        "XML Declaration Processors",
        &xml_declaration_processors,
    );
    print_processor_group(
        log,
        "Processing Instruction Processors",
        &processing_instruction_processors,
    );
}

fn print_element_processor(log: &mut ConfigLogBuilder, processor: &dyn IProcessor) {
    let element_processor = processor
        .as_element_processor()
        .expect("element processor category was checked");
    let element_name = element_processor.get_matching_element_name().map_or_else(
        || "*".to_owned(),
        |value| {
            value
                .to_java_string()
                .expect("configured matching element name remains valid")
                .to_string_lossy()
        },
    );
    let attribute_name = element_processor.get_matching_attribute_name().map_or_else(
        || "*".to_owned(),
        |value| {
            value
                .to_java_string()
                .expect("configured matching attribute name remains valid")
                .to_string_lossy()
        },
    );
    log.parameters(
        "[THYMELEAF]             * [{} {}] [{}]: {}",
        &[
            &element_name,
            &attribute_name,
            &processor.get_precedence(),
            &processor.java_class_name(),
        ],
    );
}

fn print_processor_group(log: &mut ConfigLogBuilder, label: &str, processors: &[&dyn IProcessor]) {
    if processors.is_empty() {
        return;
    }
    log.parameter("[THYMELEAF]         * {} by [precedence]:", label);
    for processor in processors {
        log.parameters(
            "[THYMELEAF]             * [{}]: {}",
            &[&processor.get_precedence(), &processor.java_class_name()],
        );
    }
}

/// 输出指定模式的 PreProcessor，保持 Java 比较器顺序和 Handler 类名。
///
/// 对应 Java:
/// `ConfigurationPrinterHelper#printPreProcessorsForTemplateMode(...)`。
fn print_pre_processors_for_template_mode(
    log: &mut ConfigLogBuilder,
    pre_processors: &[Option<std::sync::Arc<dyn IPreProcessor>>],
    template_mode: TemplateMode,
) {
    let mut values = pre_processors
        .iter()
        .flatten()
        .map(std::sync::Arc::as_ref)
        .filter(|processor| processor.get_template_mode() == Some(template_mode))
        .collect::<Vec<_>>();
    if values.is_empty() {
        return;
    }
    values.sort_by(|left, right| ProcessorComparators::compare_pre_processors(*left, *right));
    log.parameter(
        "[THYMELEAF]     * Pre-Processors for Template Mode: {} by [precedence]",
        template_mode,
    );
    for processor in values {
        log.parameters(
            "[THYMELEAF]             * [{}]: {}",
            &[&processor.get_precedence(), &processor.java_class_name()],
        );
    }
}

/// 输出指定模式的 PostProcessor，保持 Java 比较器顺序和 Handler 类名。
///
/// 对应 Java:
/// `ConfigurationPrinterHelper#printPostProcessorsForTemplateMode(...)`。
fn print_post_processors_for_template_mode(
    log: &mut ConfigLogBuilder,
    post_processors: &[Option<std::sync::Arc<dyn IPostProcessor>>],
    template_mode: TemplateMode,
) {
    let mut values = post_processors
        .iter()
        .flatten()
        .map(std::sync::Arc::as_ref)
        .filter(|processor| processor.get_template_mode() == Some(template_mode))
        .collect::<Vec<_>>();
    if values.is_empty() {
        return;
    }
    values.sort_by(|left, right| ProcessorComparators::compare_post_processors(*left, *right));
    log.parameter(
        "[THYMELEAF]     * Post-Processors for Template Mode: {} by [precedence]",
        template_mode,
    );
    for processor in values {
        log.parameters(
            "[THYMELEAF]             * [{}]: {}",
            &[&processor.get_precedence(), &processor.java_class_name()],
        );
    }
}

fn resolver_line(
    log: &mut ConfigLogBuilder,
    order: Option<i32>,
    name: Option<&crate::util::JavaString>,
) {
    let name = name.map(crate::util::JavaString::to_string_lossy);
    if let Some(order) = order {
        log.optional_parameters(
            "[THYMELEAF]     * [{}] {}",
            &[
                Some(&order),
                name.as_ref().map(|value| value as &dyn Display),
            ],
        );
    } else {
        log.optional_parameter(
            "[THYMELEAF]     * {}",
            name.as_ref().map(|value| value as &dyn Display),
        );
    }
}

fn template_modes() -> [TemplateMode; 6] {
    [
        TemplateMode::HTML,
        TemplateMode::XML,
        TemplateMode::TEXT,
        TemplateMode::JAVASCRIPT,
        TemplateMode::CSS,
        TemplateMode::RAW,
    ]
}

/// Java `ConfigLogBuilder` 的逐占位符日志文本构建器。
///
/// 参数中的 `$` 按上游实现替换为 `.`，空参数替换为空字符串；`line` 追加换行，
/// `end` 不追加换行。
struct ConfigLogBuilder {
    value: String,
}

impl Display for ConfigLogBuilder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.value)
    }
}

impl ConfigLogBuilder {
    fn new() -> Self {
        Self {
            value: String::with_capacity(4096),
        }
    }

    fn line(&mut self, line: &str) {
        let _ = writeln!(self.value, "{line}");
    }

    fn end(&mut self, line: &str) {
        let _ = write!(self.value, "{line}");
    }

    fn parameter(&mut self, line: &str, value: impl Display) {
        self.optional_parameter(line, Some(&value));
    }

    fn optional_parameter(&mut self, line: &str, value: Option<&dyn Display>) {
        self.optional_parameters(line, &[value]);
    }

    fn parameters(&mut self, line: &str, parameters: &[&dyn Display]) {
        let parameters = parameters
            .iter()
            .map(|value| Some(*value))
            .collect::<Vec<_>>();
        self.optional_parameters(line, &parameters);
    }

    fn optional_parameters(&mut self, line: &str, parameters: &[Option<&dyn Display>]) {
        let mut rendered = line.to_owned();
        for parameter in parameters {
            let replacement = parameter
                .map(ToString::to_string)
                .unwrap_or_default()
                .replace('$', ".");
            rendered = rendered.replacen("{}", &replacement, 1);
        }
        self.line(&rendered);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use super::{
        ConfigLogBuilder, ConfigurationPrinterHelper, print_post_processors_for_template_mode,
        print_pre_processors_for_template_mode,
    };
    use crate::engine::{
        AbstractTemplateHandler, ITemplateHandler, TemplateHandlerClass,
        TemplateHandlerConstructorError,
    };
    use crate::postprocessor::{IPostProcessor, PostProcessor};
    use crate::preprocessor::{IPreProcessor, PreProcessor};
    use crate::{ITemplateEngine, TemplateEngine, TemplateMode};

    fn golden() -> BTreeMap<String, String> {
        include_str!("../tests/fixtures/engine_configuration_golden.txt")
            .lines()
            .map(|line| {
                let (key, value) = line.split_once('=').expect("golden key/value");
                (key.to_owned(), unescape(value))
            })
            .collect()
    }

    fn unescape(value: &str) -> String {
        let mut result = String::with_capacity(value.len());
        let mut characters = value.chars();
        while let Some(character) = characters.next() {
            if character != '\\' {
                result.push(character);
                continue;
            }
            match characters.next().expect("escaped character") {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                '\\' => result.push('\\'),
                other => panic!("unexpected escape: {other}"),
            }
        }
        result
    }

    fn normalize_configuration_log(value: &str) -> String {
        let normalized = value
            .lines()
            .map(|line| {
                if line.starts_with("[THYMELEAF] * Thymeleaf version:")
                    && let Some((prefix, _)) = line.split_once(" (built ")
                {
                    return format!("{prefix} (built <BUILD_TIMESTAMP>)");
                }
                if line.starts_with("[THYMELEAF]         * \"")
                    && let Some((prefix, _)) = line.split_once(": ")
                {
                    return format!("{prefix}: <VALUE>");
                }
                line.to_owned()
            })
            .collect::<Vec<_>>()
            .join("\n");
        canonicalize_identity_ties(&normalized)
    }

    fn canonicalize_identity_ties(value: &str) -> String {
        let mut output = Vec::new();
        let mut processor_lines = Vec::new();
        for line in value.lines() {
            if line.starts_with("[THYMELEAF]             * [") {
                processor_lines.push(line.to_owned());
                continue;
            }
            if !processor_lines.is_empty() {
                processor_lines.sort();
                output.append(&mut processor_lines);
            }
            output.push(line.to_owned());
        }
        if !processor_lines.is_empty() {
            processor_lines.sort();
            output.append(&mut processor_lines);
        }
        output.join("\n")
    }

    fn assert_lines_equal(actual: &str, expected: &str) {
        if actual == expected {
            return;
        }
        let actual_lines = actual.lines().collect::<Vec<_>>();
        let expected_lines = expected.lines().collect::<Vec<_>>();
        let index = actual_lines
            .iter()
            .zip(&expected_lines)
            .position(|(left, right)| left != right)
            .unwrap_or(actual_lines.len().min(expected_lines.len()));
        panic!(
            "configuration log differs at line {}:\nactual={:?}\nexpected={:?}\nactual_lines={}\nexpected_lines={}",
            index + 1,
            actual_lines.get(index),
            expected_lines.get(index),
            actual_lines.len(),
            expected_lines.len(),
        );
    }

    fn handler() -> Result<Box<dyn ITemplateHandler>, TemplateHandlerConstructorError> {
        Ok(Box::new(AbstractTemplateHandler::new()))
    }

    fn handler_class() -> TemplateHandlerClass {
        TemplateHandlerClass::new("org.thymeleaf.engine.AbstractTemplateHandler", handler)
    }

    #[test]
    fn complete_debug_configuration_matches_java_golden() {
        let fixture = golden();
        let configuration = TemplateEngine::new()
            .get_configuration()
            .expect("default configuration");
        let actual = ConfigurationPrinterHelper::render_configuration(configuration.as_ref(), true);
        let expected = fixture
            .get("printer.debug.output")
            .expect("Java debug output");
        assert_lines_equal(&normalize_configuration_log(&actual), expected);
        assert_eq!(
            fixture.get("printer.trace.output"),
            Some(expected),
            "Java TRACE and DEBUG branches must emit identical messages"
        );

        let basic = ConfigurationPrinterHelper::render_configuration(configuration.as_ref(), false);
        assert!(basic.contains("[THYMELEAF] * Dialect: Standard"));
        assert!(!basic.contains("Processors for Template Mode"));
        assert!(!basic.contains("Expression Objects:"));
        assert!(!basic.contains("Execution Attributes:"));
    }

    #[test]
    fn config_log_builder_and_pre_post_sections_match_java_golden() {
        let fixture = golden();
        let mut builder = ConfigLogBuilder::new();
        builder.line("plain");
        builder.parameter("single={}", "a$b");
        builder.optional_parameters("double={}|{}", &[None, Some(&"tail")]);
        builder.optional_parameters("array={}|{}|{}", &[Some(&"x"), None, Some(&3)]);
        builder.end("end");
        assert_eq!(
            builder.to_string(),
            fixture["builder.output"],
            "placeholder replacement, null and newline behavior"
        );

        let pre_processors: Vec<Option<Arc<dyn IPreProcessor>>> = vec![
            Some(Arc::new(
                PreProcessor::new(Some(TemplateMode::HTML), Some(handler_class()), 20)
                    .expect("pre processor"),
            )),
            Some(Arc::new(
                PreProcessor::new(Some(TemplateMode::HTML), Some(handler_class()), -1)
                    .expect("pre processor"),
            )),
            Some(Arc::new(
                PreProcessor::new(Some(TemplateMode::XML), Some(handler_class()), 0)
                    .expect("pre processor"),
            )),
        ];
        let mut pre_log = ConfigLogBuilder::new();
        print_pre_processors_for_template_mode(&mut pre_log, &pre_processors, TemplateMode::HTML);
        assert_eq!(pre_log.to_string(), fixture["printer.pre.output"]);

        let post_processors: Vec<Option<Arc<dyn IPostProcessor>>> = vec![
            Some(Arc::new(
                PostProcessor::new(Some(TemplateMode::HTML), Some(handler_class()), 30)
                    .expect("post processor"),
            )),
            Some(Arc::new(
                PostProcessor::new(Some(TemplateMode::HTML), Some(handler_class()), 5)
                    .expect("post processor"),
            )),
            Some(Arc::new(
                PostProcessor::new(Some(TemplateMode::XML), Some(handler_class()), 0)
                    .expect("post processor"),
            )),
        ];
        let mut post_log = ConfigLogBuilder::new();
        print_post_processors_for_template_mode(
            &mut post_log,
            &post_processors,
            TemplateMode::HTML,
        );
        assert_eq!(post_log.to_string(), fixture["printer.post.output"]);
    }
}
