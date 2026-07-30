//! 模板预处理器合同与实现。

mod i_pre_processor;
mod pre_processor;

pub use i_pre_processor::{IPreProcessor, PreProcessorHandlerFactory};
pub use pre_processor::PreProcessor;
