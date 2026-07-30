//! 外部化消息解析器合同。

mod abstract_message_resolver;
mod i_message_resolver;
mod standard_message_resolution_utils;
mod standard_message_resolver;

pub use abstract_message_resolver::AbstractMessageResolver;
pub use i_message_resolver::{IMessageResolver, MessageResolutionError, MessageResolutionResult};
pub(crate) use standard_message_resolution_utils::StandardMessageResolutionUtils;
pub use standard_message_resolver::StandardMessageResolver;
