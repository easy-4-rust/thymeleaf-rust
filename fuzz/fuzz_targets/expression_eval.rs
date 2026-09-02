#![no_main]

//! 随机字节 → `${...}` 表达式注入模板 → 求值链（OGNL 兼容求值器/
//! 字面量替换/解析深度守卫）。只关心不 panic/不挂起。

use libfuzzer_sys::fuzz_target;
use std::sync::{Arc, OnceLock};
use thymeleaf::context::Context;
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

fn engine() -> &'static TemplateEngine {
    static ENGINE: OnceLock<TemplateEngine> = OnceLock::new();
    ENGINE.get_or_init(|| {
        let mut resolver = StringTemplateResolver::new();
        resolver.set_template_mode(TemplateMode::HTML);
        let engine = TemplateEngine::new();
        engine
            .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
            .expect("resolver");
        engine
    })
}

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let template = format!("<p th:text=\"{text}\">x</p>");
    let context = Context::new();
    let _ = engine().process_template(&template, &context);
});
