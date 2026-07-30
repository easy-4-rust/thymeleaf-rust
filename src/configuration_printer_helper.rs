use std::fmt::{Display, Write};

use crate::{IEngineConfiguration, TemplateMode, Thymeleaf};

/// 模板引擎配置的调试日志格式化器。
///
/// 对应 Java: `org.thymeleaf.ConfigurationPrinterHelper`。
pub(crate) struct ConfigurationPrinterHelper;

impl ConfigurationPrinterHelper {
    /// 构建并按当前 tracing 级别输出完整引擎配置。
    pub(crate) fn print_configuration(configuration: &dyn IEngineConfiguration) -> String {
        let mut log = ConfigLogBuilder::new();
        log.line("Initializing Thymeleaf Template engine configuration...");
        log.line("[THYMELEAF] TEMPLATE ENGINE CONFIGURATION:");
        match Thymeleaf::get_build_timestamp() {
            Some(timestamp) => log.parameters(
                "[THYMELEAF] * Thymeleaf version: {} (built {})",
                &[&Thymeleaf::get_version(), &timestamp],
            ),
            None => log.parameter(
                "[THYMELEAF] * Thymeleaf version: {}",
                Thymeleaf::get_version(),
            ),
        }
        log.parameter(
            "[THYMELEAF] * Cache Manager implementation: {}",
            if configuration.get_cache_manager().is_some() {
                "dyn thymeleaf::cache::ICacheManager"
            } else {
                "[no caches]"
            },
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
            let name = dialect.get_name().unwrap_or("null");
            if dialects.len() > 1 {
                log.parameters(
                    "[THYMELEAF] * Dialect [{} of {}]: {}",
                    &[&(index + 1), &dialects.len(), &name],
                );
            } else {
                log.parameter("[THYMELEAF] * Dialect: {}", name);
            }
            if let Some(processor_dialect) = dialect.as_processor_dialect() {
                let prefix = if dialect_configuration.is_prefix_specified() {
                    dialect_configuration.get_prefix()
                } else {
                    processor_dialect.get_prefix()
                };
                log.parameter(
                    "[THYMELEAF]     * Prefix: \"{}\"",
                    prefix.unwrap_or("(none)"),
                );
                if tracing::enabled!(tracing::Level::DEBUG)
                    && let Some(processors) = processor_dialect.get_processors(prefix)
                {
                    for mode in template_modes() {
                        let mut values = processors
                            .iter()
                            .flatten()
                            .filter(|processor| processor.get_template_mode() == Some(mode))
                            .map(|processor| processor.get_precedence())
                            .collect::<Vec<_>>();
                        values.sort_unstable();
                        if !values.is_empty() {
                            log.parameters(
                                "[THYMELEAF]     * Processors for Template Mode {}: {}",
                                &[&mode, &values.len()],
                            );
                            for precedence in values {
                                log.parameter(
                                    "[THYMELEAF]         * [{}] dyn IProcessor",
                                    precedence,
                                );
                            }
                        }
                    }
                }
            }
        }
        log.end("[THYMELEAF] TEMPLATE ENGINE CONFIGURED OK");
        let output = log.to_string();
        if tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace!("{output}");
        } else if tracing::enabled!(tracing::Level::DEBUG) {
            tracing::debug!("{output}");
        }
        output
    }
}

fn resolver_line(
    log: &mut ConfigLogBuilder,
    order: Option<i32>,
    name: Option<&crate::util::JavaString>,
) {
    let name = name.map_or_else(|| "null".to_owned(), |name| name.to_string_lossy());
    if let Some(order) = order {
        log.parameters("[THYMELEAF]     * [{}] {}", &[&order, &name]);
    } else {
        log.parameter("[THYMELEAF]     * {}", name);
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
        self.parameters(line, &[&value]);
    }

    fn parameters(&mut self, line: &str, parameters: &[&dyn Display]) {
        let mut rendered = line.to_owned();
        for parameter in parameters {
            rendered = rendered.replacen("{}", &parameter.to_string(), 1);
        }
        self.line(&rendered);
    }
}
