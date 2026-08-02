//! DOCTYPE 事件 Processor 与结构处理合同。

mod i_doc_type_processor;
mod i_doc_type_structure_handler;

pub use i_doc_type_processor::IDocTypeProcessor;
pub use i_doc_type_structure_handler::IDocTypeStructureHandler;
mod abstract_doc_type_processor;
pub use abstract_doc_type_processor::AbstractDocTypeProcessor;
