//! 首页控制器 —— 对应 Java `HomeController.java`。

use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::util::DateValue;
use thymeleaf::web::IWebExchange;

use super::{ControllerResult, GtvgController, build_web_context, set_variable};

/// 首页控制器。
#[derive(Default)]
pub struct HomeController;

impl GtvgController for HomeController {
    fn process(
        &self,
        web_exchange: Arc<dyn IWebExchange>,
        template_engine: &TemplateEngine,
        now: DateValue,
    ) -> ControllerResult {
        // Java: ctx.setVariable("today", Calendar.getInstance())
        let context = build_web_context(&web_exchange);
        set_variable(
            &context,
            "today",
            Some(thymeleaf::util::DateUtils::into_template_value(now)),
        );
        Ok(template_engine.process_template("home", &context)?)
    }
}
