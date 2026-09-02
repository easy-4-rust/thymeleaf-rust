#![no_main]

//! 随机字节 → 模板文本 → 完整渲染链（parser/doctype/inline/求值/序列化）。
//! 只关心不 panic/不挂起；`Err` 是合法的解析失败路径。

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
    let context = Context::new();
    let _ = engine().process_template(&text, &context);
});
