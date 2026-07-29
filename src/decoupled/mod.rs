//! Thymeleaf markup parser 的解耦模板逻辑对象。

mod decoupled_injected_attribute;
mod i_decoupled_template_logic_resolver;

pub use decoupled_injected_attribute::{
    DecoupledInjectedAttribute, DecoupledInjectedAttributeError,
};
pub use i_decoupled_template_logic_resolver::IDecoupledTemplateLogicResolver;
