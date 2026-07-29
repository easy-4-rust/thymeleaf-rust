use crate::reader::block_aware_reader::{BlockAction, BlockAwareReader};
use crate::text::{TextParserReader, TextParserReaderError};

const PREFIX: &[u16] = &[0x002f, 0x002a, 0x005b, 0x002d];
const SUFFIX: &[u16] = &[0x002d, 0x005d, 0x002a, 0x002f];

/// 删除文本模板中的解析器级注释块及其内容。
///
/// 识别 `/*[-`…`-]*/`，并且在任意底层 Reader/调用方缓冲边界下保持相同结果。
/// 对应 Java:
/// `org.thymeleaf.templateparser.reader.ParserLevelCommentTextReader`。
pub struct ParserLevelCommentTextReader {
    delegate: BlockAwareReader,
}

impl ParserLevelCommentTextReader {
    /// 包装一个 Reader，并启用解析器级文本注释过滤。
    ///
    /// 参数 `reader` 为底层 UTF-16 Reader；返回可继续交给 `TextParser` 或其他
    /// Reader 包装器的对象。对应 Java: `ParserLevelCommentTextReader#ParserLevelCommentTextReader`。
    #[must_use]
    pub fn new(reader: Box<dyn TextParserReader>) -> Self {
        Self {
            delegate: BlockAwareReader::new(reader, BlockAction::DiscardAll, PREFIX, SUFFIX),
        }
    }
}

impl TextParserReader for ParserLevelCommentTextReader {
    fn read_range(
        &mut self,
        buffer: &mut [u16],
        offset: i32,
        len: i32,
    ) -> Result<i32, TextParserReaderError> {
        self.delegate.read_range(buffer, offset, len)
    }

    fn close(&mut self) -> Result<(), TextParserReaderError> {
        self.delegate.close()
    }
}
