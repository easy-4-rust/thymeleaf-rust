//! 产品评论控制器 —— 对应 Java `ProductCommentsController.java`。

use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::util::JavaDate;
use thymeleaf::web::IWebExchange;

use crate::business::services::ProductService;

use super::{ControllerResult, GtvgController, build_web_context, set_variable, template_object};

/// 产品评论控制器（读取 `prodId` 参数）。
#[derive(Default)]
pub struct ProductCommentsController;

impl GtvgController for ProductCommentsController {
    fn process(
        &self,
        web_exchange: Arc<dyn IWebExchange>,
        template_engine: &TemplateEngine,
        _now: JavaDate,
    ) -> ControllerResult {
        // Java: Integer.valueOf(webExchange.getRequest().getParameterValue("prodId"))
        let prod_id = web_exchange
            .get_request()
            .get_parameter_value(Some(&thymeleaf::util::Utf16String::from_rust_str("prodId")))
            .ok_or_else(|| super::ControllerError("missing prodId parameter".to_owned()))?;
        let prod_id: i32 = prod_id
            .to_string_lossy()
            .parse()
            .map_err(|_| super::ControllerError("prodId is not an integer".to_owned()))?;

        let product = ProductService::find_by_id(prod_id)
            .ok_or_else(|| super::ControllerError(format!("product {prod_id} not found")))?;

        let context = build_web_context(&web_exchange);
        set_variable(&context, "prod", Some(template_object(product)));
        Ok(template_engine.process_template("product/comments", &context)?)
    }
}
