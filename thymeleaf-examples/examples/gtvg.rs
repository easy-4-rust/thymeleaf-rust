//! GTVG 示例入口 —— 对应 Java `GTVGFilter` + 7 个页面 URL。
//!
//! 模拟过滤器流程：
//! 1. 为请求注入固定用户会话（`addUserToSession`）
//! 2. 解析 URL → 控制器（`ControllerMappings`）
//! 3. 渲染模板并输出（响应头在引擎级示例中省略）
//!
//! 运行：`cargo run -p thymeleaf-examples --example gtvg`

use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::util::{JavaDate, JavaLocale, JavaString};
use thymeleaf::web::IWebExchange;
use thymeleaf_examples::controllers::controller_mappings::ControllerMapping;
use thymeleaf_examples::web::gtvg_web_exchange::GtvgWebExchange;
use thymeleaf_examples::web::gtvg_web_request::GtvgWebRequest;

/// 引擎装配 —— 对应 `GTVGFilter#buildTemplateEngine`。
///
/// Java 使用 `WebApplicationTemplateResolver`（prefix `/WEB-INF/templates/`、
/// suffix `.html`、HTML 模式、1 小时 TTL、可缓存）；Rust 用等价的文件解析器，
/// prefix 指向本 crate 的 `templates/` 目录（与 Java webapp 的绝对路径对应）。
fn build_template_engine() -> TemplateEngine {
    let mut template_resolver = thymeleaf::templateresolver::FileTemplateResolver::new();
    template_resolver.set_prefix(Some(JavaString::from_rust_str(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/"
    ))));
    template_resolver.set_suffix(Some(JavaString::from_rust_str(".html")));
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(template_resolver))
        .expect("template resolver 配置成功");
    engine
}

/// 当前时刻 —— 对应 Java `Calendar.getInstance()`（引擎默认 UTC 时区）。
fn now() -> JavaDate {
    JavaDate::calendar(chrono::Utc::now(), chrono_tz::UTC)
}

/// 处理单个 URL —— 对应 `GTVGFilter#process`。
fn process_url(engine: &TemplateEngine, path: &str, parameters: &[(&str, &str)]) -> String {
    let request = GtvgWebRequest::new(path, parameters);
    // Java: new WebContext(webExchange, webExchange.getLocale())，默认 Locale
    // 与 Java Locale.getDefault() 等价。
    let exchange = GtvgWebExchange::new(request, JavaLocale::get_default());
    let mapping = ControllerMapping::resolve_for_request(exchange.get_request())
        .expect("URL 必须命中控制器映射");
    let output = mapping
        .process(Arc::new(exchange), engine, now())
        .expect("controller 渲染成功");
    output.to_string_lossy()
}

fn main() {
    let engine = build_template_engine();

    let pages = [
        ("/", "home", vec![]),
        ("/product/list", "product/list", vec![]),
        (
            "/product/comments",
            "product/comments",
            vec![("prodId", "13")],
        ),
        ("/order/list", "order/list", vec![]),
        ("/order/details", "order/details", vec![("orderId", "3")]),
        ("/subscribe", "subscribe", vec![]),
        ("/userprofile", "userprofile", vec![]),
    ];

    for (path, label, parameters) in pages {
        println!("========== {label} ({path}) ==========");
        println!("{}", process_url(&engine, path, parameters.as_slice()));
        println!();
    }
}
