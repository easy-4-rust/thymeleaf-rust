//! CDATA 事件 Processor 与结构处理合同。

mod icdata_section_processor;
mod icdata_section_structure_handler;

pub use icdata_section_processor::ICDATASectionProcessor;
pub use icdata_section_structure_handler::ICDATASectionStructureHandler;
mod abstract_cdata_section_processor;
pub use abstract_cdata_section_processor::AbstractCDATASectionProcessor;
