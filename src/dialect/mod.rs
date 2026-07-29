//! Thymeleaf 方言基础契约与共享实现。

mod abstract_dialect;
mod abstract_processor_dialect;
mod i_dialect;
mod i_execution_attribute_dialect;
mod i_processor_dialect;

pub use abstract_dialect::{AbstractDialect, AbstractDialectError};
pub use abstract_processor_dialect::AbstractProcessorDialect;
pub use i_dialect::IDialect;
pub use i_execution_attribute_dialect::{ExecutionAttributeMap, IExecutionAttributeDialect};
pub use i_processor_dialect::IProcessorDialect;
