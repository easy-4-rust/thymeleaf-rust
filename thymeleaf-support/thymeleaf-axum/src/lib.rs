//! Thymeleaf 对 Axum 的独立响应适配。

mod thymeleaf_error;
mod thymeleaf_view;

pub use thymeleaf_error::ThymeleafError;
pub use thymeleaf_view::ThymeleafView;
