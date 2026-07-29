//! Thymeleaf 模板输入 Reader 适配器。

mod block_aware_reader;
mod parser_level_comment_text_reader;
mod prototype_only_comment_text_reader;

pub use parser_level_comment_text_reader::ParserLevelCommentTextReader;
pub use prototype_only_comment_text_reader::PrototypeOnlyCommentTextReader;
