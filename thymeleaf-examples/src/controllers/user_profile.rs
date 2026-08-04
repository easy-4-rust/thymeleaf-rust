//! 用户资料控制器 —— 对应 Java `UserProfileController.java`。
//!
//! 页面数据来自会话中的固定 `user` 对象（`GTVGFilter#addUserToSession`），
//! 控制器本身不设置变量 —— 对应 Java 控制器体。

use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::util::DateValue;
use thymeleaf::web::IWebExchange;

use super::{ControllerResult, GtvgController, build_web_context};

/// 用户资料控制器。
#[derive(Default)]
pub struct UserProfileController;

impl GtvgController for UserProfileController {
    fn process(
        &self,
        web_exchange: Arc<dyn IWebExchange>,
        template_engine: &TemplateEngine,
        _now: DateValue,
    ) -> ControllerResult {
        let context = build_web_context(&web_exchange);
        Ok(template_engine.process_template("userprofile", &context)?)
    }
}
