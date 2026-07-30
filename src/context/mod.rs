//! 模板执行上下文及惰性变量合同。

mod abstract_context;
mod abstract_engine_context;
mod abstract_expression_context;
#[expect(
    clippy::module_inception,
    reason = "文件名必须与 Java Context 对象一一对应"
)]
mod context;
mod context_variable_entries;
mod contexts;
mod engine_context;
mod expression_context;
mod i_context;
mod i_engine_context;
mod i_engine_context_factory;
mod i_expression_context;
mod i_lazy_context_variable;
mod i_template_context;
mod i_web_context;
mod identifier_sequences;
mod lazy_context_variable;
mod standard_engine_context_factory;
mod web_context;
mod web_engine_context;
mod web_expression_context;

pub use abstract_context::AbstractContext;
pub use abstract_engine_context::AbstractEngineContext;
pub use abstract_expression_context::AbstractExpressionContext;
pub use context::Context;
pub use context_variable_entries::{ContextVariableEntries, ContextVariableEntry};
pub use contexts::Contexts;
pub use engine_context::EngineContext;
pub use expression_context::ExpressionContext;
pub use i_context::{IContext, IContextVariableNames};
pub use i_engine_context::IEngineContext;
pub use i_engine_context_factory::IEngineContextFactory;
pub use i_expression_context::IExpressionContext;
pub use i_lazy_context_variable::ILazyContextVariable;
pub use i_template_context::ITemplateContext;
pub use i_web_context::IWebContext;
pub use identifier_sequences::{IdentifierSequences, IdentifierSequencesError};
pub use lazy_context_variable::LazyContextVariable;
pub use standard_engine_context_factory::StandardEngineContextFactory;
pub use web_context::WebContext;
pub use web_engine_context::{RequestParameterValues, WebEngineContext};
pub use web_expression_context::WebExpressionContext;
