//! 产品列表控制器 —— 对应 Java `ProductListController.java`。

use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::web::IWebExchange;

use crate::business::services::ProductService;

use super::{
    ControllerResult, GtvgController, build_web_context, set_variable, template_list,
    template_object,
};

/// 产品列表控制器。
#[derive(Default)]
pub struct ProductListController;

impl GtvgController for ProductListController {
    fn process(
        &self,
        web_exchange: Arc<dyn IWebExchange>,
        template_engine: &TemplateEngine,
        _now: thymeleaf::util::DateValue,
    ) -> ControllerResult {
        let all_products = ProductService::find_all();
        let context = build_web_context(&web_exchange);
        set_variable(
            &context,
            "prods",
            Some(template_list(
                all_products.into_iter().map(template_object).collect(),
            )),
        );
        Ok(template_engine.process_template("product/list", &context)?)
    }
}
