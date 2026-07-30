//! 模板后处理器合同与实现。

mod i_post_processor;
mod post_processor;

pub use i_post_processor::{IPostProcessor, PostProcessorHandlerFactory};
pub use post_processor::PostProcessor;
