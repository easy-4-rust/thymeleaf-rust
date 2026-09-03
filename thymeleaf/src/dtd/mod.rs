//! DTD 验证集成（`dtd-validation` feature gate）。
//!
//! 本模块仅在 `dtd-validation` feature 启用时编译，零二进制/行为影响。
//! 提供 XHTML DTD 内嵌解析（`MemoryResolver`）+ 验证器封装（`DtdValidator`），
//! 供 XML 模式下的 `parse_xml` 可选验证。

#[cfg(feature = "dtd-validation")]
pub mod embedded_dtd;

#[cfg(feature = "dtd-validation")]
pub mod entity_budget;

#[cfg(feature = "dtd-validation")]
pub mod validator;

#[cfg(feature = "dtd-validation")]
pub use embedded_dtd::build_xhtml_resolver;

#[cfg(feature = "dtd-validation")]
pub use entity_budget::default_budget;

#[cfg(feature = "dtd-validation")]
pub use validator::{DtdValidator, ValidationPolicy, Validator, ValidityError};
