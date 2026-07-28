//! Thymeleaf 方言基础契约与共享实现。

mod abstract_dialect;
mod i_dialect;

pub use abstract_dialect::{AbstractDialect, AbstractDialectError};
pub use i_dialect::IDialect;
