//! Thymeleaf 标准方言内联模式。

mod i_inline_pre_processor_handler;
mod standard_inline_mode;

pub use i_inline_pre_processor_handler::IInlinePreProcessorHandler;
pub use standard_inline_mode::{StandardInlineMode, StandardInlineModeParseError};
