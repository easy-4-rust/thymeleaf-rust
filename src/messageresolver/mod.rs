//! 外部化消息解析器合同。

mod abstract_message_resolver;
mod i_message_resolver;
mod message_resolution_error;
mod message_resolution_result;
mod standard_message_resolution_utils;
mod standard_message_resolver;

pub use abstract_message_resolver::AbstractMessageResolver;
pub use i_message_resolver::IMessageResolver;
pub use message_resolution_error::MessageResolutionError;
pub use message_resolution_result::MessageResolutionResult;
pub(crate) use standard_message_resolution_utils::StandardMessageResolutionUtils;
pub use standard_message_resolver::StandardMessageResolver;
