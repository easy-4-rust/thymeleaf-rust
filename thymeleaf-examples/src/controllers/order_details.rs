//! 订单详情控制器 —— 对应 Java `OrderDetailsController.java`。

use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::util::{JavaDate, JavaString};
use thymeleaf::web::IWebExchange;

use crate::business::services::OrderService;

use super::{
    ControllerError, ControllerResult, GtvgController, build_web_context, set_variable,
    template_object,
};

/// 订单详情控制器（读取 `orderId` 参数）。
#[derive(Default)]
pub struct OrderDetailsController;

impl GtvgController for OrderDetailsController {
    fn process(
        &self,
        web_exchange: Arc<dyn IWebExchange>,
        template_engine: &TemplateEngine,
        _now: JavaDate,
    ) -> ControllerResult {
        // Java: Integer.valueOf(webExchange.getRequest().getParameterValue("orderId"))
        let order_id = web_exchange
            .get_request()
            .get_parameter_value(Some(&JavaString::from_rust_str("orderId")))
            .ok_or_else(|| ControllerError("missing orderId parameter".to_owned()))?;
        let order_id: i32 = order_id
            .to_string_lossy()
            .parse()
            .map_err(|_| ControllerError("orderId is not an integer".to_owned()))?;

        let order = OrderService::find_by_id(order_id)
            .ok_or_else(|| ControllerError(format!("order {order_id} not found")))?;

        let context = build_web_context(&web_exchange);
        set_variable(&context, "order", Some(template_object(order)));
        Ok(template_engine.process_template("order/details", &context)?)
    }
}
