//! 标记注释 Reader 的固定 Java Oracle 差分测试。

use std::cell::RefCell;
use std::fmt::Write;
use std::rc::Rc;

use thymeleaf::reader::{ParserLevelCommentMarkupReader, PrototypeOnlyCommentMarkupReader};
use thymeleaf::text::{TextParserReader, TextParserReaderError};

const JAVA_GOLDEN: &str = include_str!("fixtures/markup_comment_reader_golden.txt");

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

#[derive(Debug, Default)]
struct CloseState {
    close_count: usize,
}

#[derive(Debug)]
struct FailingCloseReader {
    state: Rc<RefCell<CloseState>>,
}

impl TextParserReader for FailingCloseReader {
    fn read_range(
        &mut self,
        _buffer: &mut [u16],
        _offset: i32,
        _len: i32,
    ) -> Result<i32, TextParserReaderError> {
        Ok(-1)
    }

    fn close(&mut self) -> Result<(), TextParserReaderError> {
        self.state.borrow_mut().close_count += 1;
        Err(TextParserReaderError::io("close-boom"))
    }
}

fn parser_reader(value: &str) -> Box<dyn TextParserReader> {
    Box::new(ParserLevelCommentMarkupReader::new(Box::new(
        StringReader::new(value),
    )))
}

fn prototype_reader(value: &str) -> Box<dyn TextParserReader> {
    Box::new(PrototypeOnlyCommentMarkupReader::new(Box::new(
        StringReader::new(value),
    )))
}

fn combined_reader(value: &str) -> Box<dyn TextParserReader> {
    Box::new(ParserLevelCommentMarkupReader::new(Box::new(
        PrototypeOnlyCommentMarkupReader::new(Box::new(StringReader::new(value))),
    )))
}

fn describe_error(error: &TextParserReaderError) -> String {
    format!(
        "{}:{}",
        error.java_class_name(),
        error
            .java_message()
            .map_or_else(|| "null".to_owned(), |message| message.to_string_lossy())
    )
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('|', "\\|")
}

fn emit_read(
    output: &mut String,
    key: &str,
    mut reader: Box<dyn TextParserReader>,
    buffer_size: usize,
    offset: i32,
    len: i32,
) {
    let mut buffer = vec![0; buffer_size];
    let mut result = Vec::new();
    let mut returns = Vec::new();
    let mut throwable = "none".to_owned();
    loop {
        match reader.read_range(&mut buffer, offset, len) {
            Ok(read) => {
                returns.push(read.to_string());
                if read < 0 {
                    break;
                }
                if read > 0 {
                    result.extend_from_slice(
                        &buffer[offset as usize..offset as usize + read as usize],
                    );
                }
            }
            Err(error) => {
                throwable = describe_error(&error);
                break;
            }
        }
    }
    writeln!(
        output,
        "{key}={}|returns={}|throwable={}",
        escape(&String::from_utf16_lossy(&result)),
        returns.join(","),
        escape(&throwable)
    )
    .expect("write markup reader golden");
}

fn rust_golden() -> String {
    let mut output = String::new();
    writeln!(output, "baseline=10f9dd2eb8cbd98515ce14b149d115e0287d0add").expect("write baseline");

    let parser_cases = [
        "",
        "plain",
        "<!-- hello -->",
        "a<!--/*hidden*/-->b",
        "<!-- <!--/* hello /*/--> -->",
        "中<!--/*😀*/-->文",
    ];
    let prototype_cases = [
        "",
        "plain",
        "<!-- hello -->",
        "a<!--/*/shown/*/-->b",
        "<!-- <!--/*/ hello /*/--> */--> -->",
        "中<!--/*/😀/*/-->文",
    ];
    let requests = [(1, 0, 1), (4, 0, 4), (9, 2, 5), (15, 5, 8)];

    for (case_index, case) in parser_cases.iter().enumerate() {
        for (request_index, &(size, offset, len)) in requests.iter().enumerate() {
            emit_read(
                &mut output,
                &format!("parser.{case_index}.{request_index}"),
                parser_reader(case),
                size,
                offset,
                len,
            );
        }
    }
    for (case_index, case) in prototype_cases.iter().enumerate() {
        for (request_index, &(size, offset, len)) in requests.iter().enumerate() {
            emit_read(
                &mut output,
                &format!("prototype.{case_index}.{request_index}"),
                prototype_reader(case),
                size,
                offset,
                len,
            );
        }
    }

    let combined_cases = [
        "a<!--/*/shown/*/-->b<!--/*hidden*/-->c",
        "<!--/*x<!--/*/not-shown/*/-->y*/-->tail",
    ];
    for (case_index, case) in combined_cases.iter().enumerate() {
        for (request_index, &(size, offset, len)) in requests.iter().enumerate() {
            emit_read(
                &mut output,
                &format!("combined.{case_index}.{request_index}"),
                combined_reader(case),
                size,
                offset,
                len,
            );
        }
    }

    emit_read(
        &mut output,
        "unfinished.parser",
        parser_reader("a<!--/*open"),
        3,
        0,
        3,
    );
    emit_read(
        &mut output,
        "unfinished.prototype",
        prototype_reader("a<!--/*/open"),
        5,
        1,
        3,
    );

    let close_state = Rc::new(RefCell::new(CloseState::default()));
    let mut close_reader = ParserLevelCommentMarkupReader::new(Box::new(FailingCloseReader {
        state: Rc::clone(&close_state),
    }));
    let close_error = close_reader
        .close()
        .expect_err("configured close failure must propagate");
    writeln!(output, "close.throwable={}", describe_error(&close_error))
        .expect("write close failure");
    writeln!(output, "close.count={}", close_state.borrow().close_count)
        .expect("write close count");
    output
}

/// SOURCE_PARITY：61 条固定 Java Oracle 同时锁定四组标记定界符、逐次 read 返回、
/// UTF-16、双层包装、未闭合结构和 close 异常。
#[test]
fn markup_comment_readers_match_java_golden() {
    assert_eq!(rust_golden(), JAVA_GOLDEN);

    let mut prototype =
        PrototypeOnlyCommentMarkupReader::new(Box::new(StringReader::new("")));
    assert_eq!(prototype.close(), Ok(()));
}
