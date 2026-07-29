//! Thymeleaf 文本模板解析器内部对象。

mod abstract_chained_text_handler;
mod abstract_text_handler;
#[allow(
    dead_code,
    reason = "TextParser 消费者将在后续切片迁移；当前先验证注释预处理器"
)]
mod comment_processor_text_handler;
#[allow(
    dead_code,
    reason = "TextParser 消费者将在后续切片迁移；当前先验证事件预处理器"
)]
mod event_processor_text_handler;
mod i_text_handler;
mod i_text_processor;
mod i_text_structure_handler;
mod parsing_locator_util;
mod text_parse_exception;
mod text_parse_status;
#[allow(
    dead_code,
    reason = "AbstractTextTemplateParser 消费者将在后续切片迁移"
)]
mod text_parser;
mod text_parsing_attribute_sequence_util;
#[allow(
    dead_code,
    reason = "TextParser 消费者将在后续切片中迁移；当前先验证依赖对象"
)]
mod text_parsing_comment_util;
#[allow(dead_code, reason = "TextParser 消费者将在后续切片迁移")]
mod text_parsing_element_util;
#[allow(
    dead_code,
    reason = "TextParser 消费者将在后续切片中迁移；当前先验证依赖对象"
)]
mod text_parsing_literal_util;
#[allow(
    dead_code,
    reason = "TextParser/element/attribute 消费者将在后续切片中迁移"
)]
mod text_parsing_util;

pub use text_parse_exception::{TextParseCause, TextParseException};
pub use {
    abstract_chained_text_handler::{AbstractChainedTextHandler, ChainedTextHandlerRuntimeError},
    abstract_text_handler::AbstractTextHandler,
    i_text_handler::ITextHandler,
    i_text_processor::ITextProcessor,
    i_text_structure_handler::ITextStructureHandler,
    text_parser::{TextParserReader, TextParserReaderError},
};
#[expect(unused_imports, reason = "text parser 消费者对象将在后续切片中迁移")]
pub(crate) use {
    comment_processor_text_handler::{
        CommentProcessorTextHandler, CommentProcessorTextHandlerRuntimeError,
    },
    event_processor_text_handler::{
        EventProcessorTextHandler, EventProcessorTextHandlerRuntimeError, StructureNamesRepository,
    },
    parsing_locator_util::ParsingLocatorUtil,
    text_parse_status::TextParseStatus,
    text_parser::{TextParser, TextParserRuntimeError},
    text_parsing_attribute_sequence_util::{
        TextParsingAttributeSequenceError, TextParsingAttributeSequenceUtil,
    },
    text_parsing_comment_util::{TextParsingCommentError, TextParsingCommentUtil},
    text_parsing_element_util::{TextParsingElementError, TextParsingElementUtil},
    text_parsing_literal_util::TextParsingLiteralUtil,
    text_parsing_util::{TextParsingUtil, TextParsingUtilError},
};
