//! GTVG 示例端到端验收 —— 对应 Java `GTVGTest`（tests-core templateengine 包）
//! 的示例级断言 + 控制器映射 + 业务数据完整性。
//!
//! 页面渲染使用固定“今天”（Java `Calendar.getInstance()` 的注入等价物），
//! 其余全部走真实引擎链路：文件模板解析器、`StandardMessageResolver`
//! （模板并列 .properties 读取）、`StandardLinkBuilder`（`@{...}`）、
//! WebContext 会话/请求作用域。

use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::util::{JavaDate, JavaLocale, JavaString};
use thymeleaf::web::IWebExchange;
use thymeleaf_examples::business::calendar_util::calendar_for;
use thymeleaf_examples::controllers::GtvgController;
use thymeleaf_examples::controllers::controller_mappings::ControllerMapping;
use thymeleaf_examples::controllers::{
    home::HomeController, order_details::OrderDetailsController, order_list::OrderListController,
    product_comments::ProductCommentsController, product_list::ProductListController,
    subscribe::SubscribeController, user_profile::UserProfileController,
};
use thymeleaf_examples::web::gtvg_web_exchange::GtvgWebExchange;
use thymeleaf_examples::web::gtvg_web_request::GtvgWebRequest;

/// 固定“今天”：2011-11-11（与上游语料 gtvg/home.thtest 的
/// `today = #calendars.create(2011,11,11)` 一致）。
fn fixed_today() -> JavaDate {
    calendar_for(2011, 11, 11, 0, 0)
}

/// 引擎装配 —— 对应 `GTVGFilter#buildTemplateEngine`。
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
        .expect("template resolver");
    engine
}

/// 构造 exchange —— 对应 `GTVGFilter#doFilter` 的 session 注入 + `buildExchange`。
fn exchange_for(path: &str, parameters: &[(&str, &str)]) -> Arc<dyn IWebExchange> {
    let request = GtvgWebRequest::new(path, parameters);
    Arc::new(GtvgWebExchange::new(request, JavaLocale::get_default()))
}

fn render(controller: &dyn GtvgController, path: &str, parameters: &[(&str, &str)]) -> String {
    let engine = build_template_engine();
    controller
        .process(exchange_for(path, parameters), &engine, fixed_today())
        .expect("controller renders")
        .to_string_lossy()
}

// ===========================================================================
// 首页（HomeController）
// ===========================================================================

#[test]
fn home_renders_welcome_today_and_links() {
    let html = render(&HomeController, "/", &[]);
    // th:utext="#{home.welcome(${session.user.name})}"：消息 + 会话用户参数
    assert!(
        html.contains("Welcome to our grocery store, John Apricot (from default messages)!"),
        "welcome 消息与 session.user.name 参数"
    );
    // th:with="df=#{date.format}" + ${#calendars.format(today,df)}：
    // 消息 'MMMM dd'','' yyyy' → MessageFormat → 'MMMM dd',' yyyy' → 逗号日期
    assert!(
        html.contains("Today is: <span>November 11, 2011</span>"),
        "date.format 消息模式（含 '','' 引号转义）渲染为 November 11, 2011"
    );
    // th:alt-title="#{logo}"
    assert!(
        html.contains("alt=\"Good Thymes Virtual Grocery logo\""),
        "logo 消息"
    );
    // @{...} 链接表达式
    for link in ["/product/list", "/order/list", "/subscribe", "/userprofile"] {
        assert!(
            html.contains(&format!("href=\"{link}\"")),
            "链接表达式 @{{...}} -> {link}"
        );
    }
    // footer 片段
    assert!(
        html.contains("&copy; 2011 The Good Thymes Virtual Grocery"),
        "footer 片段"
    );
}

#[test]
fn home_message_resolution_is_locale_sensitive() {
    let engine = build_template_engine();
    // home_en.properties 覆盖 home.properties（en Locale）
    let locale = JavaLocale::new(
        JavaString::from_rust_str("en"),
        JavaString::from_rust_str(""),
    );
    let exchange: Arc<dyn IWebExchange> =
        Arc::new(GtvgWebExchange::new(GtvgWebRequest::new("/", &[]), locale));
    let html = HomeController
        .process(exchange, &engine, fixed_today())
        .expect("render en")
        .to_string_lossy();
    assert!(
        html.contains("Welcome to our <b>fantastic</b> grocery store, John Apricot!"),
        "en Locale 命中 home_en.properties"
    );
}

// ===========================================================================
// 产品页（ProductListController / ProductCommentsController）
// ===========================================================================

#[test]
fn product_list_renders_seed_data() {
    let html = render(&ProductListController, "/product/list", &[]);
    assert!(html.contains("Fresh Sweet Basil"), "产品 1");
    assert!(html.contains("Berry Chewy Granola Bars"), "产品 30（末项）");
    assert!(html.contains("<span>8</span> comment/s"), "产品 13 评论数");
    assert!(html.contains("<span>3</span> comment/s"), "产品 30 评论数");
    assert!(
        html.contains("href=\"/product/comments?prodId=13\""),
        "带参数链接表达式"
    );
    // #{{true}}/#{{false}} 消息解析自 product/list.properties（true=yes / false=no）
    assert!(html.contains("<td>yes</td>"), "inStock 真值消息");
    assert!(html.contains("<td>no</td>"), "inStock 假值消息");
}

#[test]
fn product_comments_renders_requested_product() {
    let html = render(
        &ProductCommentsController,
        "/product/comments",
        &[("prodId", "13")],
    );
    assert!(
        html.contains("Comments for product: <span>Vanilla Puff Cereal</span>"),
        "prodId=13 命中 Vanilla Puff Cereal"
    );
    assert!(
        html.contains("Very tasty! I&#39;d definitely buy it again!"),
        "评论 3（th:text 转义撇号）"
    );
    assert!(html.contains("Good. Pricey though."), "评论 10");
}

// ===========================================================================
// 订单页（OrderListController / OrderDetailsController）
// ===========================================================================

#[test]
fn order_list_renders_orders_with_aggregates() {
    let html = render(&OrderListController, "/order/list", &[]);
    assert!(html.contains("James Cucumber"), "订单 3 顾客");
    assert!(html.contains("Shannon Parsley"), "订单 1 顾客");
    assert!(html.contains("George Garlic"), "订单 2 顾客");
    // #aggregates.sum(o.orderLines.{purchasePrice * amount})：订单 1 = 0.99*2 + 2.50*4 + 15.50*1
    assert!(
        html.contains("27.48"),
        "订单 1 合计（0.99×2 + 2.50×4 + 15.50×1 = 27.48）"
    );
    // #calendars.format(o.date,'dd/MMM/yyyy')
    assert!(html.contains("12/Jan/2009"), "订单 1 日期模式 dd/MMM/yyyy");
    assert!(
        html.contains("href=\"/order/details?orderId=3\""),
        "带参数链接表达式"
    );
}

#[test]
fn order_details_renders_selection_object() {
    let html = render(
        &OrderDetailsController,
        "/order/details",
        &[("orderId", "3")],
    );
    assert!(
        html.contains("<p><b>Code:</b> <span>3</span></p>"),
        "th:object + *{{id}}"
    );
    assert!(
        html.contains("<b>Date:</b> <span>18 Jul 2010</span>"),
        "日期"
    );
    assert!(
        html.contains("<b>Name:</b> <span>James Cucumber</span>"),
        "*{{customer.name}}"
    );
    assert!(
        html.contains("<b>Since:</b> <span>02 Apr 2006</span>"),
        "customerSince"
    );
    // TOTAL = 8 × 5.99
    assert!(
        html.contains("<b>TOTAL:</b> <span>47.92</span>"),
        "聚合合计 47.92"
    );
}

// ===========================================================================
// 简单页（SubscribeController / UserProfileController）
// ===========================================================================

#[test]
fn subscribe_renders_submit_message() {
    let html = render(&SubscribeController, "/subscribe", &[]);
    assert!(
        html.contains("value=\"Subscribe me!\""),
        "subscribe.submit 消息"
    );
    assert!(
        html.contains("action=\"/subscribe\""),
        "th:action 链接表达式"
    );
}

#[test]
fn user_profile_renders_session_user() {
    let html = render(&UserProfileController, "/userprofile", &[]);
    // th:object="${session.user}" —— GTVGFilter#addUserToSession 的固定用户
    assert!(
        html.contains("<p>Name: <span>John</span>.</p>"),
        "firstName"
    );
    assert!(
        html.contains("<p>Surname: <span>Apricot</span>.</p>"),
        "lastName"
    );
    assert!(
        html.contains("<p>Nationality: <span>Antarctica</span>.</p>"),
        "nationality"
    );
    assert!(
        html.contains("<p>Age: <span>(no age specified)</span>.</p>"),
        "age 为 null 时的 ?: 回退"
    );
}

// ===========================================================================
// 控制器映射（ControllerMappings）
// ===========================================================================

#[test]
fn controller_mappings_resolve_all_urls() {
    for (path, expected) in [
        ("/", ControllerMapping::Home),
        ("/product/list", ControllerMapping::ProductList),
        ("/product/comments", ControllerMapping::ProductComments),
        ("/order/list", ControllerMapping::OrderList),
        ("/order/details", ControllerMapping::OrderDetails),
        ("/subscribe", ControllerMapping::Subscribe),
        ("/userprofile", ControllerMapping::UserProfile),
    ] {
        let exchange = exchange_for(path, &[]);
        assert_eq!(
            ControllerMapping::resolve_for_request(exchange.get_request()),
            Some(expected),
            "URL {path}"
        );
    }
}

#[test]
fn controller_mappings_strip_jsessionid_and_reject_unknown() {
    let exchange = exchange_for("/order/list;jsessionid=abc123", &[]);
    assert_eq!(
        ControllerMapping::resolve_for_request(exchange.get_request()),
        Some(ControllerMapping::OrderList),
        ";jsessionid 片段剥离（URL 重写）"
    );
    for unknown in ["/unknown", "/css/gtvg.css", "/favicon.ico"] {
        let exchange = exchange_for(unknown, &[]);
        assert_eq!(
            ControllerMapping::resolve_for_request(exchange.get_request()),
            None,
            "未映射 URL {unknown}"
        );
    }
}

// ===========================================================================
// 模板资产 1:1 字节校验（SHA-256 固定，与上游 webapp 逐字节一致）
// ===========================================================================

#[test]
fn templates_match_upstream_bytes() {
    let manifests: &[(&str, &str)] = &[
        (
            "home.html",
            "afd343a3a8482366cfd75a95cb225c267f69b92301915ecb820cc7d21fb1453a",
        ),
        (
            "footer.html",
            "d71c58cb95a06d25b6ed1d11b1d4f2ddee94f35f5ebcc9fafa4dda0b3a0e112d",
        ),
        (
            "subscribe.html",
            "7e677ce8b535c37b7f4b04e8843b06cd471ff4816d7fabdfdabb9588283470f0",
        ),
        (
            "userprofile.html",
            "fc4d2b3b90ebdee334725b8699037a1ca953965038e484e821ea0a9d87620572",
        ),
        (
            "order/list.html",
            "d7513c541118f0b656a4eb752a33e1263adb753c57e8e8dc985b4791b6564966",
        ),
        (
            "order/details.html",
            "af3c68605b02cf87c27361a72d047d3b651cfc4c384aa23f03fa607c29b89c2a",
        ),
        (
            "product/list.html",
            "ac96963d3f01e6e17400cd898ed3c7bf1a8a71eeaa3f6916a3eadeefb015be94",
        ),
        (
            "product/comments.html",
            "02416ec432dc0cebfe96b654a15341cfd2f083d4d89a02e01e6519807f07060a",
        ),
        (
            "home.properties",
            "b210fdf586e497d2064c1a5d19cebc62f09297a790898fe237399c8cb9734ee7",
        ),
        (
            "home_en.properties",
            "f5bcfa41420d04348ae478455dea4797d99a498e9a61380bf31935459e8c9481",
        ),
        (
            "product/list.properties",
            "7deb4e8c2c4a877c9d61b962839cdb27bc144c63237be5193bc62ca4ac7575de",
        ),
        (
            "subscribe.properties",
            "aa722e4c43e7856df6e66ae115dd31cb23d1e57c2ab872be5ea71214d2fb46da",
        ),
        (
            "subscribe_es.properties",
            "9a8a9876877bd1153e301788997b889f91cd2236619e1d2a9b217abcd2a40e97",
        ),
    ];
    for (relative, expected) in manifests {
        use sha2::Digest;
        let path = format!("{}/templates/{relative}", env!("CARGO_MANIFEST_DIR"));
        let bytes = std::fs::read(&path).expect("template asset exists");
        let digest = sha2::Sha256::digest(&bytes);
        let digest = digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(&digest, expected, "template 字节与上游不一致: {relative}");
    }
}

// ===========================================================================
// 业务数据（Repositories 种子）
// ===========================================================================

#[test]
fn repositories_match_java_seed_data() {
    use thymeleaf_examples::business::repositories::{
        CustomerRepository, OrderRepository, ProductRepository,
    };
    assert_eq!(CustomerRepository::find_all().len(), 6, "客户 6");
    assert_eq!(ProductRepository::find_all().len(), 30, "产品 30");
    assert_eq!(
        ProductRepository::find_by_id(13)
            .expect("prod13")
            .comments
            .len(),
        8,
        "产品 13 评论 8"
    );
    assert_eq!(
        ProductRepository::find_by_id(2)
            .expect("prod2")
            .comments
            .len(),
        2,
        "产品 2 评论 2"
    );
    let total_comments: usize = ProductRepository::find_all()
        .iter()
        .map(|product| product.comments.len())
        .sum();
    assert_eq!(total_comments, 21, "评论合计 21");
    let orders = OrderRepository::find_all();
    assert_eq!(orders.len(), 3, "订单 3");
    assert_eq!(
        orders
            .iter()
            .map(|order| order.order_lines.len())
            .sum::<usize>(),
        6,
        "订单行 6"
    );
    assert!(
        OrderRepository::find_by_id(99).is_none(),
        "缺失订单返回 None"
    );
}
