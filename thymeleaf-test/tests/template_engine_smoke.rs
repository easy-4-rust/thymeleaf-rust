//! `TemplateEngine` 主调用链的最小端到端门禁。

use std::sync::Arc;

use thymeleaf::context::Context;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

#[test]
fn parsing001_runs_through_the_complete_html_engine_chain() {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);

    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("template resolver must be configurable before first process");

    let input = "<!DOCTYPE html>\n<html>\n</html>";
    let output = engine
        .process_template(input, &Context::new())
        .expect("upstream parsing001 input must render successfully");

    assert_eq!(output.to_string_lossy(), input);
}
