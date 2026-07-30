//! Thymeleaf 对 Tonic 请求扩展的独立集成。

mod thymeleaf_interceptor;
mod tonic_request_ext;

pub use thymeleaf_interceptor::ThymeleafInterceptor;
pub use tonic_request_ext::TonicRequestExt;
