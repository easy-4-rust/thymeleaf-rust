//! Thymeleaf 文本模板解析器内部对象。

mod i_text_handler;
mod parsing_locator_util;
mod text_parse_exception;
mod text_parse_status;
#[allow(
    dead_code,
    reason = "TextParser 消费者将在后续切片中迁移；当前先验证依赖对象"
)]
mod text_parsing_comment_util;
#[allow(
    dead_code,
    reason = "TextParser 消费者将在后续切片中迁移；当前先验证依赖对象"
)]
mod text_parsing_literal_util;

pub use i_text_handler::ITextHandler;
pub use text_parse_exception::{TextParseCause, TextParseException};
#[expect(unused_imports, reason = "text parser 消费者对象将在后续切片中迁移")]
pub(crate) use {
    parsing_locator_util::ParsingLocatorUtil,
    text_parse_status::TextParseStatus,
    text_parsing_comment_util::{TextParsingCommentError, TextParsingCommentUtil},
    text_parsing_literal_util::TextParsingLiteralUtil,
};
