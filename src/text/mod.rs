//! Thymeleaf 文本模板解析器内部对象。

mod parsing_locator_util;
mod text_parse_status;

#[expect(unused_imports, reason = "text parser 消费者对象将在后续切片中迁移")]
pub(crate) use {parsing_locator_util::ParsingLocatorUtil, text_parse_status::TextParseStatus};
