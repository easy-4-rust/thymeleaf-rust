//! 订阅控制器 —— 对应 Java `SubscribeController.java`。

use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::util::JavaDate;
use thymeleaf::web::IWebExchange;

use super::{ControllerResult, GtvgController, build_web_context};

/// 订阅控制器（无额外变量）。
#[derive(Default)]
pub struct SubscribeController;

impl GtvgController for SubscribeController {
    fn process(
        &self,
        web_exchange: Arc<dyn IWebExchange>,
        template_engine: &TemplateEngine,
        _now: JavaDate,
    ) -> ControllerResult {
        let context = build_web_context(&web_exchange);
        Ok(template_engine.process_template("subscribe", &context)?)
    }
}
