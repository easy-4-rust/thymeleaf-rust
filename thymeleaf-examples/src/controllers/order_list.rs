//! 订单列表控制器 —— 对应 Java `OrderListController.java`。

use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::util::JavaDate;
use thymeleaf::web::IWebExchange;

use crate::business::services::OrderService;

use super::{
    ControllerResult, GtvgController, build_web_context, set_variable, template_list,
    template_object,
};

/// 订单列表控制器。
#[derive(Default)]
pub struct OrderListController;

impl GtvgController for OrderListController {
    fn process(
        &self,
        web_exchange: Arc<dyn IWebExchange>,
        template_engine: &TemplateEngine,
        _now: JavaDate,
    ) -> ControllerResult {
        let all_orders = OrderService::find_all();
        let context = build_web_context(&web_exchange);
        set_variable(
            &context,
            "orders",
            Some(template_list(
                all_orders.into_iter().map(template_object).collect(),
            )),
        );
        Ok(template_engine.process_template("order/list", &context)?)
    }
}
