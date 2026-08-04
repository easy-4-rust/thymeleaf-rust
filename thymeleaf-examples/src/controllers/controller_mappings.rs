//! 控制器映射 —— 对应 Java `ControllerMappings.java` + `IGTVGController` 分派。
//!
//! `resolve_controller_for_request` 按应用内路径分派；路径中的 `;jsessionid`
//! 片段（URL 重写产物）在比较前剥离 —— 对应 Java `getRequestPath`。

use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::web::{IWebExchange, IWebRequest};

use super::home::HomeController;
use super::order_details::OrderDetailsController;
use super::order_list::OrderListController;
use super::product_comments::ProductCommentsController;
use super::product_list::ProductListController;
use super::subscribe::SubscribeController;
use super::user_profile::UserProfileController;
use super::{ControllerResult, GtvgController};

/// 按 URL 分派的控制器表 —— 对应 Java `controllersByURL`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControllerMapping {
    /// `/`
    Home,
    /// `/product/list`
    ProductList,
    /// `/product/comments`
    ProductComments,
    /// `/order/list`
    OrderList,
    /// `/order/details`
    OrderDetails,
    /// `/subscribe`
    Subscribe,
    /// `/userprofile`
    UserProfile,
}

impl ControllerMapping {
    /// `ControllerMappings#resolveControllerForRequest(IWebRequest)`。
    ///
    /// 资源 URL（`/css`、`/images`、`/favicon`）由 `GTVGFilter#process` 前置拦截，
    /// 不在映射表内。
    #[must_use]
    pub fn resolve_for_request(request: &dyn IWebRequest) -> Option<Self> {
        let path = get_request_path(request);
        match path.as_str() {
            "/" => Some(Self::Home),
            "/product/list" => Some(Self::ProductList),
            "/product/comments" => Some(Self::ProductComments),
            "/order/list" => Some(Self::OrderList),
            "/order/details" => Some(Self::OrderDetails),
            "/subscribe" => Some(Self::Subscribe),
            "/userprofile" => Some(Self::UserProfile),
            _ => None,
        }
    }

    /// 执行控制器并渲染 —— 对应 `GTVGFilter#process` 的
    /// `controller.process(webExchange, templateEngine, writer)`。
    pub fn process(
        &self,
        web_exchange: Arc<dyn IWebExchange>,
        template_engine: &TemplateEngine,
        now: thymeleaf::util::DateValue,
    ) -> ControllerResult {
        match self {
            Self::Home => HomeController.process(web_exchange, template_engine, now),
            Self::ProductList => ProductListController.process(web_exchange, template_engine, now),
            Self::ProductComments => {
                ProductCommentsController.process(web_exchange, template_engine, now)
            }
            Self::OrderList => OrderListController.process(web_exchange, template_engine, now),
            Self::OrderDetails => {
                OrderDetailsController.process(web_exchange, template_engine, now)
            }
            Self::Subscribe => SubscribeController.process(web_exchange, template_engine, now),
            Self::UserProfile => UserProfileController.process(web_exchange, template_engine, now),
        }
    }
}

/// `ControllerMappings#getRequestPath`：剥离 `;jsessionid` 片段。
fn get_request_path(request: &dyn IWebRequest) -> String {
    let request_path = request
        .get_path_within_application()
        .map(|value| value.to_string_lossy())
        .unwrap_or_default();
    match request_path.split_once(';') {
        Some((head, _)) => head.to_owned(),
        None => request_path,
    }
}
