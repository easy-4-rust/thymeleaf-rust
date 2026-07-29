//! 元素 Processor 的匹配名称与基础合同。

mod element_processor_set;
mod i_element_model_processor;
mod i_element_model_structure_handler;
mod i_element_processor;
mod i_element_tag_processor;
mod i_element_tag_structure_handler;
mod matching_attribute_name;
mod matching_element_name;

pub(crate) use element_processor_set::read_set;
pub use element_processor_set::{ElementProcessorSet, UnmodifiableElementProcessorSet};
pub use i_element_model_processor::IElementModelProcessor;
pub use i_element_model_structure_handler::IElementModelStructureHandler;
pub use i_element_processor::IElementProcessor;
pub use i_element_tag_processor::IElementTagProcessor;
pub use i_element_tag_structure_handler::IElementTagStructureHandler;
pub use matching_attribute_name::{MatchingAttributeName, MatchingAttributeNameError};
pub use matching_element_name::{MatchingElementName, MatchingElementNameError};
