//! 与具体 Rust Web 框架无关的 Thymeleaf Web SPI。

mod i_web_application;
mod i_web_exchange;
mod i_web_request;
mod i_web_session;
mod render_error;
mod rendered_template;
mod thymeleaf_renderer;

pub use i_web_application::IWebApplication;
pub use i_web_exchange::IWebExchange;
pub use i_web_request::{IWebRequest, WebRequestError};
pub use i_web_session::IWebSession;
pub use render_error::RenderError;
pub use rendered_template::{RenderedTemplate, RenderedTemplateBody, RenderedTemplateStream};
pub use thymeleaf_renderer::ThymeleafRenderer;
