use crate::reader::block_aware_reader::{BlockAction, BlockAwareReader};
use crate::text::{TextParserReader, TextParserReaderError};

const PREFIX: &[u16] = &[0x003c, 0x0021, 0x002d, 0x002d, 0x002f, 0x002a, 0x002f];
const SUFFIX: &[u16] = &[0x002f, 0x002a, 0x002f, 0x002d, 0x002d, 0x003e];

/// 删除标记模板中的仅原型注释容器，同时保留容器内容。
///
/// 识别 `<!--/*/`…`/*/-->`，并且在任意底层 Reader/调用方缓冲边界下保持相同结果。
/// 对应 Java:
/// `org.thymeleaf.templateparser.reader.PrototypeOnlyCommentMarkupReader`。
pub struct PrototypeOnlyCommentMarkupReader {
    delegate: BlockAwareReader,
}

impl PrototypeOnlyCommentMarkupReader {
    /// 包装一个 Reader，并启用仅原型标记注释容器过滤。
    ///
    /// 参数 `reader` 为底层 UTF-16 Reader；返回可继续交给标记解析器或其他
    /// Reader 包装器的对象。对应 Java:
    /// `PrototypeOnlyCommentMarkupReader#PrototypeOnlyCommentMarkupReader`。
    #[must_use]
    pub fn new(reader: Box<dyn TextParserReader>) -> Self {
        Self {
            delegate: BlockAwareReader::new(reader, BlockAction::DiscardContainer, PREFIX, SUFFIX),
        }
    }
}

impl TextParserReader for PrototypeOnlyCommentMarkupReader {
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

#[cfg(test)]
mod tests {
    use super::PrototypeOnlyCommentMarkupReader;
    use crate::text::{TextParserReader, TextParserReaderError};

    #[derive(Debug)]
    struct StringReader {
        value: Vec<u16>,
        position: usize,
    }

    impl StringReader {
        fn new(value: &str) -> Self {
            Self {
                value: value.encode_utf16().collect(),
                position: 0,
            }
        }
    }

    impl TextParserReader for StringReader {
        fn read_range(
            &mut self,
            buffer: &mut [u16],
            offset: i32,
            len: i32,
        ) -> Result<i32, TextParserReaderError> {
            if self.position >= self.value.len() {
                return Ok(-1);
            }
            let copied = (len as usize).min(self.value.len() - self.position);
            let offset = offset as usize;
            buffer[offset..offset + copied]
                .copy_from_slice(&self.value[self.position..self.position + copied]);
            self.position += copied;
            Ok(copied as i32)
        }
    }

    fn reader(value: &str) -> Box<dyn TextParserReader> {
        Box::new(PrototypeOnlyCommentMarkupReader::new(Box::new(
            StringReader::new(value),
        )))
    }

    fn read_all(
        mut reader: Box<dyn TextParserReader>,
        buffer_size: usize,
        offset: usize,
        len: usize,
    ) -> String {
        let mut buffer = vec![0; buffer_size];
        let mut output = Vec::new();
        loop {
            let read = reader
                .read_range(&mut buffer, offset as i32, len as i32)
                .expect("完整上游 case 不能产生 Reader 异常");
            if read < 0 {
                return String::from_utf16_lossy(&output);
            }
            if read > 0 {
                output.extend_from_slice(&buffer[offset..offset + read as usize]);
            }
        }
    }

    fn equivalent(message: &str) -> String {
        let mut output = String::new();
        let mut was_open = false;
        let mut in_open_structure = false;
        let mut in_close_structure = false;
        let characters: Vec<char> = message.chars().collect();
        for (index, &character) in characters.iter().enumerate() {
            if !in_open_structure && !in_close_structure && character == '<' {
                in_open_structure = true;
                continue;
            }
            if !in_open_structure && !in_close_structure && was_open && character == '/' {
                in_close_structure = true;
                continue;
            }
            if in_close_structure && character == '>' {
                in_close_structure = false;
                was_open = false;
                continue;
            }
            if in_open_structure && character == '/' && index > 0 && characters[index - 1] == '*' {
                in_open_structure = false;
                was_open = true;
                continue;
            }
            if !in_open_structure && !in_close_structure {
                output.push(character);
            }
        }
        output
    }

    fn generated_messages() -> Vec<String> {
        let message = "0123456789";
        let prefix = "<!--/*/";
        let suffix = "/*/-->";
        let mut messages = Vec::new();
        for i in 0..=message.len() {
            let mut first = message.to_owned();
            first.insert_str(i, suffix);
            for j in 0..=i {
                let mut second = first.clone();
                second.insert_str(j, prefix);
                for k in 0..=j {
                    let mut third = second.clone();
                    third.insert_str(k, suffix);
                    messages.push(third.clone());
                    for l in 0..=k {
                        let mut fourth = third.clone();
                        fourth.insert_str(l, prefix);
                        messages.push(fourth);
                    }
                }
            }
        }
        messages
    }

    /// SOURCE_PARITY：迁移 `PrototypeOnlyCommentMarkupReaderTest#test01` 的完整
    /// 定界符位置生成器，并跨 1..=8 读取长度及所有合法 offset 验证。
    #[test]
    fn generated_structure_positions_match_upstream_equivalence_algorithm() {
        for message in generated_messages() {
            let expected = equivalent(&message);
            for len in 1..=8 {
                for offset in 0..len {
                    assert_eq!(
                        read_all(reader(&message), len + 2, offset, len + 2 - offset),
                        expected,
                        "message={message:?}, len={len}, offset={offset}"
                    );
                }
            }
        }
    }

    /// SOURCE_PARITY：逐项迁移 `PrototypeOnlyCommentMarkupReaderTest#test02`
    /// 的全部人工 case，并保留原始 buffer/len/offset 三重循环。
    #[test]
    fn handwritten_examples_match_every_original_buffer_shape() {
        let cases = [
            ("<!-- hello -->", "<!-- hello -->"),
            (
                "<!-- <!--/* hello /*/--> -->",
                "<!-- <!--/* hello /*/--> -->",
            ),
            (
                "<!-- <!--/* hello /*/--> */-->",
                "<!-- <!--/* hello /*/--> */-->",
            ),
            (
                "<!-- <!--/* hello /*/--> */--> -->",
                "<!-- <!--/* hello /*/--> */--> -->",
            ),
            (
                "<!-- <!--/*/ hello /*/--> */--> -->",
                "<!--  hello  */--> -->",
            ),
            (
                "<!-- <!--/*/ hello /*/--> */--> <!--/* -->",
                "<!--  hello  */--> <!--/* -->",
            ),
            (
                "<!-- <!--/*/ hello /*/--> */--> <!--/* */-->-->",
                "<!--  hello  */--> <!--/* */-->-->",
            ),
            ("hello", "hello"),
            ("<!--/* hello /*/-->", "<!--/* hello /*/-->"),
            ("<!--/* hello /*/--> a*/-->a", "<!--/* hello /*/--> a*/-->a"),
            ("<!--/* hello /*/--> */-->", "<!--/* hello /*/--> */-->"),
            ("<!--/*/ hello /*/-->", " hello "),
            ("<!--/*/ hello /*/--> */-->", " hello  */-->"),
            ("<!--/*/ hello /*/--> aa*/-->bb", " hello  aa*/-->bb"),
            ("<!--/*/hello/*/-->", "hello"),
            ("<!--/*/hello/*/--> */--> aa", "hello */--> aa"),
            ("<!--/*/hello/*/--> */-->", "hello */-->"),
            ("hey <!--/*/hello/*/-->", "hey hello"),
            ("hey <!--/*/hello/*/--> */-->", "hey hello */-->"),
            ("hey <!--/*/hello/*/--> /*/-->", "hey hello /*/-->"),
            ("<!--/*/ hello /*/--> */--> -->", " hello  */--> -->"),
            ("<!--/*/ hello /*/--> */--> <!--/*", " hello  */--> <!--/*"),
            (
                "<!--/*/ hello /*/--> */--> <!--/* -->",
                " hello  */--> <!--/* -->",
            ),
            (
                "<!--/*/ hello /*/--> */--> <!--/* */-->-->",
                " hello  */--> <!--/* */-->-->",
            ),
            (
                "<!--/*/ hello /*/--> */--> <!--/* */-->",
                " hello  */--> <!--/* */-->",
            ),
            ("<!--/*/hello /*/--> */-->", "hello  */-->"),
            ("<!--/*/hello /*/--> */--> -->", "hello  */--> -->"),
            ("<!--/*/hello /*/--> */--> <!--/*", "hello  */--> <!--/*"),
            (
                "<!--/*/hello /*/--> */--> <!--/* -->",
                "hello  */--> <!--/* -->",
            ),
            (
                "<!--/*/hello /*/--> */--> <!--/* */-->-->",
                "hello  */--> <!--/* */-->-->",
            ),
            (
                "<!--/*/hello /*/--> */--> <!--/* */-->",
                "hello  */--> <!--/* */-->",
            ),
            (
                "<!--/*<!--/*/hello /*/--> */--> <!--/* */-->",
                "<!--/*hello  */--> <!--/* */-->",
            ),
            ("<!--/*/hello/*/*/-->", "hello/*"),
        ];
        for (message, expected) in cases {
            for buffer_size in 1..=message.len() + 10 {
                for len in 1..=buffer_size {
                    for offset in 0..len {
                        assert_eq!(
                            read_all(reader(message), buffer_size, offset, len - offset),
                            expected,
                            "message={message:?}, buffer={buffer_size}, len={len}, offset={offset}"
                        );
                    }
                }
            }
        }
    }
}
