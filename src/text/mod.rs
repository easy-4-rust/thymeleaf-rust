//! Thymeleaf 文本模板解析器内部对象。

mod abstract_chained_text_handler;
mod abstract_text_handler;
mod abstract_text_template_parser;
mod comment_processor_text_handler;
mod css_template_parser;
mod event_processor_text_handler;
mod i_text_handler;
mod i_text_processor;
mod i_text_structure_handler;
mod inlined_output_expression_text_handler;
mod java_script_template_parser;
mod parsing_locator_util;
mod text_parse_exception;
mod text_parse_status;
mod text_parser;
mod text_parsing_attribute_sequence_util;
mod text_parsing_comment_util;
mod text_parsing_element_util;
mod text_parsing_literal_util;
mod text_parsing_util;
mod text_template_parser;

pub use text_parse_exception::{TextParseCause, TextParseException};
pub use text_parser::TextParser;
pub use {
    abstract_chained_text_handler::{AbstractChainedTextHandler, ChainedTextHandlerRuntimeError},
    abstract_text_handler::AbstractTextHandler,
    abstract_text_template_parser::AbstractTextTemplateParser,
    css_template_parser::CSSTemplateParser,
    i_text_handler::ITextHandler,
    i_text_processor::ITextProcessor,
    i_text_structure_handler::ITextStructureHandler,
    inlined_output_expression_text_handler::InlinedOutputExpressionTextHandler,
    java_script_template_parser::JavaScriptTemplateParser,
    text_parser::{TextParserReader, TextParserReaderError},
    text_template_parser::TextTemplateParser,
};
pub(crate) use {
    comment_processor_text_handler::{
        CommentProcessorTextHandler, CommentProcessorTextHandlerRuntimeError,
    },
    event_processor_text_handler::{
        EventProcessorTextHandler, EventProcessorTextHandlerRuntimeError,
    },
    parsing_locator_util::ParsingLocatorUtil,
    text_parse_status::TextParseStatus,
    text_parsing_attribute_sequence_util::{
        TextParsingAttributeSequenceError, TextParsingAttributeSequenceUtil,
    },
    text_parsing_comment_util::{TextParsingCommentError, TextParsingCommentUtil},
    text_parsing_element_util::{TextParsingElementError, TextParsingElementUtil},
    text_parsing_literal_util::TextParsingLiteralUtil,
    text_parsing_util::{TextParsingUtil, TextParsingUtilError},
};
mod abstract_text_processor;
pub use abstract_text_processor::AbstractTextProcessor;
