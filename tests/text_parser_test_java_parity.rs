//! `TextParserTest` 1:1 差分测试。
//!
//! 对应上游 `thymeleaf-tests-core` 的
//! `org.thymeleaf.templateparser.text.TextParserTest`：以追踪型
//! `TraceBuilderTextHandler`（对应上游同名测试辅助类）装配
//! `EventProcessorTextHandler` + `CommentProcessorTextHandler` 链，
//! 逐 buffer size（1..=16384，与上游 `testDoc` 完全一致）解析同一输入，
//! 同时验证普通 Reader 与带 5 字符前/后填充的 Reader 两条路径，
//! 最终把事件流组装为上游 `[T(...){l,c}...]` 格式并逐字节比较。
//!
//! 用例输入与期望输出由 `tools/extract_text_parser_test_cases.py` 从上游
//! Java 源码逐字提取（含转义还原），fixture 见
//! `tests/fixtures/text_parser_test_cases.json`。

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

use serde_json::Value;
use thymeleaf::text::{
    ITextHandler, TextParseException, TextParser, TextParserReader, TextParserReaderError,
};

const MAX_BUFFER_SIZE: i32 = 16384;

// ===========================================================================
// fixture：从上游 Java 源码提取的用例
// ===========================================================================

#[derive(Debug, Clone)]
struct TestCase {
    kind: String,
    input: String,
    out_proc: Option<String>,
    out_unproc: Option<String>,
    offset: i32,
    len: i32,
    process_comments: Option<bool>,
    error_line: Option<i32>,
    error_col: Option<i32>,
}

fn load_cases() -> &'static Vec<TestCase> {
    static CASES: OnceLock<Vec<TestCase>> = OnceLock::new();
    CASES.get_or_init(|| {
        let fixture: Value = serde_json::from_str(include_str!("fixtures/text_parser_test_cases.json"))
            .expect("text parser test cases fixture");
        let baseline = fixture["baseline"].as_str().expect("baseline");
        assert_eq!(
            baseline, "10f9dd2eb8cbd98515ce14b149d115e0287d0add",
            "fixture baseline 必须与上游锁定提交一致"
        );
        fixture["cases"]
            .as_array()
            .expect("cases array")
            .iter()
            .map(|case| TestCase {
                kind: case["kind"].as_str().expect("kind").to_owned(),
                input: case["input"].as_str().expect("input").to_owned(),
                out_proc: case["outProc"].as_str().map(str::to_owned),
                out_unproc: case["outUnproc"].as_str().map(str::to_owned),
                offset: case.get("offset").and_then(Value::as_i64).map_or(0, |v| v as i32),
                len: case.get("len").and_then(Value::as_i64).map_or(0, |v| v as i32),
                process_comments: case
                    .get("processComments")
                    .map(Value::as_bool)
                    .unwrap_or(None),
                error_line: case.get("errorLine").and_then(Value::as_i64).map(|v| v as i32),
                error_col: case.get("errorCol").and_then(Value::as_i64).map(|v| v as i32),
            })
            .collect()
    })
}

fn case(index: usize) -> &'static TestCase {
    &load_cases()[index]
}

// ===========================================================================
// 追踪事件（对应 Java TextTraceEvent#toString 语义）
// ===========================================================================

/// 事件类型字符串，与上游 `TextTraceEvent.EventType` 的
/// `stringRepresentation` 完全一致。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraceType {
    DocumentStart,
    DocumentEnd,
    StandaloneElementStart,
    StandaloneElementEnd,
    OpenElementStart,
    OpenElementEnd,
    CloseElementStart,
    CloseElementEnd,
    Attribute,
    Comment,
    Text,
}

impl TraceType {
    fn as_str(self) -> &'static str {
        match self {
            TraceType::DocumentStart => "DS",
            TraceType::DocumentEnd => "DE",
            TraceType::StandaloneElementStart => "SES",
            TraceType::StandaloneElementEnd => "SEE",
            TraceType::OpenElementStart => "OES",
            TraceType::OpenElementEnd => "OEE",
            TraceType::CloseElementStart => "CES",
            TraceType::CloseElementEnd => "CEE",
            TraceType::Attribute => "A",
            TraceType::Comment => "C",
            TraceType::Text => "T",
        }
    }
}

/// 一个追踪事件：类型 + 并行内容/行/列数组。
///
/// 对应 Java `TextTraceEvent`。`toString` 逐内容项输出
/// `(content){line,col}`；文档开始/结束由调用方特判为 `[`/`]`。
#[derive(Debug)]
struct TraceEvent {
    trace_type: TraceType,
    contents: Vec<Option<Vec<u16>>>,
    lines: Vec<i32>,
    cols: Vec<i32>,
}

impl TraceEvent {
    fn single(trace_type: TraceType, content: Option<Vec<u16>>, line: i32, col: i32) -> Self {
        Self {
            trace_type,
            contents: vec![content],
            lines: vec![line],
            cols: vec![col],
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn attribute(
        name: Vec<u16>,
        name_line: i32,
        name_col: i32,
        operator: Vec<u16>,
        operator_line: i32,
        operator_col: i32,
        value: Vec<u16>,
        value_line: i32,
        value_col: i32,
    ) -> Self {
        Self {
            trace_type: TraceType::Attribute,
            contents: vec![Some(name), Some(operator), Some(value)],
            lines: vec![name_line, operator_line, value_line],
            cols: vec![name_col, operator_col, value_col],
        }
    }

    /// 按 Java `TextTraceEvent#toString` 输出，UTF-16 精确。
    fn render(&self) -> Vec<u16> {
        let mut out: Vec<u16> = self.trace_type.as_str().encode_utf16().collect();
        for index in 0..self.contents.len() {
            out.push(b'(' as u16);
            if let Some(content) = &self.contents[index] {
                out.extend_from_slice(content);
            }
            out.push(b')' as u16);
            out.push(b'{' as u16);
            out.extend(self.lines[index].to_string().as_bytes().iter().map(|b| *b as u16));
            out.push(b',' as u16);
            out.extend(self.cols[index].to_string().as_bytes().iter().map(|b| *b as u16));
            out.push(b'}' as u16);
        }
        out
    }
}

/// 组装完整文档追踪：`[` + 事件流 + `]`，UTF-16 精确。
///
/// 文档开始/结束事件本身渲染为 `[`/`]`，与上游
/// `TextParserTest.testDoc` 的组装逻辑一致。
fn assemble(trace: &[TraceEvent]) -> Vec<u16> {
    let mut out = Vec::new();
    for event in trace {
        match event.trace_type {
            TraceType::DocumentStart => out.push(b'[' as u16),
            TraceType::DocumentEnd => out.push(b']' as u16),
            _ => out.extend_from_slice(&event.render()),
        }
    }
    out
}

// ===========================================================================
// 追踪构建 Handler（对应 Java TraceBuilderTextHandler）
// ===========================================================================

#[derive(Clone, Default)]
struct TraceBuilderTextHandler {
    trace: Rc<RefCell<Vec<TraceEvent>>>,
}

impl TraceBuilderTextHandler {
    fn new() -> Self {
        Self {
            trace: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

fn content(buffer: Option<&mut [u16]>, offset: i32, len: i32) -> Vec<u16> {
    let buffer = buffer.expect("buffer cannot be null during trace");
    buffer[offset as usize..(offset + len) as usize].to_vec()
}

impl ITextHandler for TraceBuilderTextHandler {
    fn handle_document_start(
        &mut self,
        start_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.trace.borrow_mut().push(TraceEvent::single(
            TraceType::DocumentStart,
            Some(
                start_time_nanos
                    .to_string()
                    .as_bytes()
                    .iter()
                    .map(|b| *b as u16)
                    .collect(),
            ),
            line,
            col,
        ));
        Ok(())
    }

    fn handle_document_end(
        &mut self,
        end_time_nanos: i64,
        total_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        let contents = [end_time_nanos.to_string(), total_time_nanos.to_string()]
            .map(|s| Some(s.as_bytes().iter().map(|b| *b as u16).collect()));
        self.trace.borrow_mut().push(TraceEvent {
            trace_type: TraceType::DocumentEnd,
            contents: contents.to_vec(),
            lines: vec![line],
            cols: vec![col],
        });
        Ok(())
    }

    fn handle_standalone_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        _minimized: bool,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.trace.borrow_mut().push(TraceEvent::single(
            TraceType::StandaloneElementStart,
            Some(content(buffer, name_offset, name_len)),
            line,
            col,
        ));
        Ok(())
    }

    fn handle_standalone_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        _minimized: bool,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.trace.borrow_mut().push(TraceEvent::single(
            TraceType::StandaloneElementEnd,
            Some(content(buffer, name_offset, name_len)),
            line,
            col,
        ));
        Ok(())
    }

    fn handle_open_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.trace.borrow_mut().push(TraceEvent::single(
            TraceType::OpenElementStart,
            Some(content(buffer, name_offset, name_len)),
            line,
            col,
        ));
        Ok(())
    }

    fn handle_open_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.trace.borrow_mut().push(TraceEvent::single(
            TraceType::OpenElementEnd,
            Some(content(buffer, name_offset, name_len)),
            line,
            col,
        ));
        Ok(())
    }

    fn handle_close_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.trace.borrow_mut().push(TraceEvent::single(
            TraceType::CloseElementStart,
            Some(content(buffer, name_offset, name_len)),
            line,
            col,
        ));
        Ok(())
    }

    fn handle_close_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.trace.borrow_mut().push(TraceEvent::single(
            TraceType::CloseElementEnd,
            Some(content(buffer, name_offset, name_len)),
            line,
            col,
        ));
        Ok(())
    }

    fn handle_attribute(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        name_line: i32,
        name_col: i32,
        operator_offset: i32,
        operator_len: i32,
        operator_line: i32,
        operator_col: i32,
        _value_content_offset: i32,
        _value_content_len: i32,
        value_outer_offset: i32,
        value_outer_len: i32,
        value_line: i32,
        value_col: i32,
    ) -> Result<(), Box<TextParseException>> {
        let buffer = buffer.expect("buffer cannot be null during trace");
        self.trace.borrow_mut().push(TraceEvent::attribute(
            content(Some(buffer), name_offset, name_len),
            name_line,
            name_col,
            content(Some(buffer), operator_offset, operator_len),
            operator_line,
            operator_col,
            content(Some(buffer), value_outer_offset, value_outer_len),
            value_line,
            value_col,
        ));
        Ok(())
    }

    fn handle_comment(
        &mut self,
        buffer: Option<&mut [u16]>,
        content_offset: i32,
        content_len: i32,
        _outer_offset: i32,
        _outer_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.trace.borrow_mut().push(TraceEvent::single(
            TraceType::Comment,
            Some(content(buffer, content_offset, content_len)),
            line,
            col,
        ));
        Ok(())
    }

    fn handle_text(
        &mut self,
        buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.trace.borrow_mut().push(TraceEvent::single(
            TraceType::Text,
            Some(content(buffer, offset, len)),
            line,
            col,
        ));
        Ok(())
    }
}

// ===========================================================================
// Reader（对应 Java CharArrayReader，支持 offset/len 与前后填充）
// ===========================================================================

struct CharArrayTextParserReader {
    input: Vec<u16>,
    position: usize,
    end: usize,
}

impl CharArrayTextParserReader {
    fn new(input: Vec<u16>, offset: usize, len: usize) -> Self {
        Self {
            input,
            position: offset,
            end: offset + len,
        }
    }
}

impl TextParserReader for CharArrayTextParserReader {
    fn read_range(
        &mut self,
        buffer: &mut [u16],
        offset: i32,
        len: i32,
    ) -> Result<i32, TextParserReaderError> {
        if len == 0 {
            return Ok(0);
        }
        if self.position >= self.end {
            return Ok(-1);
        }
        let copied = (len as usize).min(self.end - self.position);
        let destination = offset as usize;
        buffer[destination..destination + copied]
            .copy_from_slice(&self.input[self.position..self.position + copied]);
        self.position += copied;
        Ok(copied as i32)
    }
}

// ===========================================================================
// 用例驱动（对应 Java testDoc 的 bufferSize 1..=16384 双路径扫描）
// ===========================================================================

/// 校验单个用例的全部 buffer size × processComments × Reader 路径组合。
fn check_case(case: &TestCase) {
    let input_u16: Vec<u16> = case.input.encode_utf16().collect();
    let offset = case.offset;
    let len = case.len;

    let variants: &[bool] = match case.process_comments {
        Some(flag) => &[flag],
        None => &[true, false],
    };
    let expected_for = |flag: bool| -> Option<Vec<u16>> {
        let expected = if flag {
            case.out_proc.as_deref()
        } else {
            case.out_unproc.as_deref()
        };
        expected.map(|text| text.encode_utf16().collect())
    };

    for process_comments in variants {
        let Some(expected) = expected_for(*process_comments) else {
            continue;
        };
        for buffer_size in 1..=MAX_BUFFER_SIZE {
            // 路径 1：普通 Reader（对应 CharArrayReader(input, offset, len)）
            let reader =
                CharArrayTextParserReader::new(input_u16.to_vec(), offset as usize, len as usize);
            let parser = TextParser::new(2, buffer_size, *process_comments, true);
            let handler = TraceBuilderTextHandler::new();
            let parse_result = parser.parse_reader(
                Some(Box::new(reader)),
                Some(Box::new(handler.clone())),
            );
            assert_trace(
                &case.input,
                buffer_size,
                *process_comments,
                parse_result,
                &handler,
                &expected,
            );

            // 路径 2：带 5 字符前/后填充的 Reader（对应
            // CharArrayReader(newInput, 5, len)）
            let mut padded: Vec<u16> = Vec::with_capacity(len as usize + 10);
            padded.extend(std::iter::repeat_n(b'X' as u16, 5));
            padded.extend_from_slice(&input_u16[offset as usize..(offset + len) as usize]);
            padded.extend(std::iter::repeat_n(b'X' as u16, 5));
            let padded_reader = CharArrayTextParserReader::new(padded, 5, len as usize);
            let parser = TextParser::new(2, buffer_size, *process_comments, true);
            let handler = TraceBuilderTextHandler::new();
            let parse_result = parser.parse_reader(
                Some(Box::new(padded_reader)),
                Some(Box::new(handler.clone())),
            );
            assert_trace(
                &case.input,
                buffer_size,
                *process_comments,
                parse_result,
                &handler,
                &expected,
            );
        }
    }
}

fn assert_trace(
    input: &str,
    buffer_size: i32,
    process_comments: bool,
    parse_result: Result<(), Box<TextParseException>>,
    handler: &TraceBuilderTextHandler,
    expected: &[u16],
) {
    let assembled = assemble(&handler.trace.borrow());
    let message = |detail: String| {
        format!(
            "解析失败 case 输入 {:?}（buffer size {buffer_size}，processComments {process_comments}）：{detail}",
            input.chars().take(40).collect::<String>()
        )
    };
    match parse_result {
        Ok(()) => assert_eq!(
            assembled,
            expected,
            "{}",
            message(format!(
                "追踪输出不匹配\n  期望: {}\n  实际: {}",
                String::from_utf16_lossy(expected),
                String::from_utf16_lossy(&assembled)
            ))
        ),
        Err(error) => panic!(
            "{}",
            message(format!(
                "意外抛出 TextParseException: {}",
                error.get_message().map_or_else(|| "null".to_owned(), |m| m.to_string_lossy())
            ))
        ),
    }
}

fn check_error_case(case: &TestCase) {
    let input_u16: Vec<u16> = case.input.encode_utf16().collect();
    let variants: &[bool] = match case.process_comments {
        Some(flag) => &[flag],
        None => &[true, false],
    };
    let message = |detail: String| {
        format!(
            "case 输入 {:?}：{detail}",
            case.input.chars().take(40).collect::<String>()
        )
    };
    for process_comments in variants {
        for buffer_size in 1..=MAX_BUFFER_SIZE {
            for padded in [false, true] {
                let reader = if padded {
                    let mut padded: Vec<u16> = Vec::with_capacity(case.input.len() + 10);
                    padded.extend(std::iter::repeat_n(b'X' as u16, 5));
                    padded.extend_from_slice(&input_u16);
                    padded.extend(std::iter::repeat_n(b'X' as u16, 5));
                    CharArrayTextParserReader::new(padded, 5, case.input.len())
                } else {
                    CharArrayTextParserReader::new(input_u16.to_vec(), 0, input_u16.len())
                };
                let parser = TextParser::new(2, buffer_size, *process_comments, true);
                let handler = TraceBuilderTextHandler::new();
                let parse_result = parser
                    .parse_reader(Some(Box::new(reader)), Some(Box::new(handler.clone())));
                let error = match parse_result {
                    Ok(()) => panic!(
                        "{}",
                        message(format!(
                            "应抛出 TextParseException 但成功（buffer size {buffer_size}）"
                        ))
                    ),
                    Err(error) => error,
                };
                let input_preview = case.input.chars().take(40).collect::<String>();
                if case.error_line == Some(-1) {
                    assert_eq!(error.get_line(), None, "{input_preview} 的 line 应为 null");
                } else {
                    assert_eq!(error.get_line(), case.error_line, "{input_preview} 的 line 不匹配");
                }
                if case.error_col == Some(-1) {
                    assert_eq!(error.get_col(), None, "{input_preview} 的 col 应为 null");
                } else {
                    assert_eq!(error.get_col(), case.error_col, "{input_preview} 的 col 不匹配");
                }
            }
        }
    }
}

fn run_case(index: usize) {
    let case = case(index);
    match case.kind.as_str() {
        "doc" => check_case(case),
        "error" => check_error_case(case),
        other => panic!("未知用例类型 {other}"),
    }
}

// ===========================================================================
// 每个用例一个测试（并行执行，失败信息含用例序号）
// ===========================================================================

macro_rules! parity_tests {
    ($($name:ident => $index:expr),* $(,)?) => {
        $(#[test]
        fn $name() {
            run_case($index);
        })*
    };
}

parity_tests! {
    case_000 => 0, case_001 => 1, case_002 => 2, case_003 => 3,
    case_004 => 4, case_005 => 5, case_006 => 6, case_007 => 7,
    case_008 => 8, case_009 => 9, case_010 => 10, case_011 => 11,
    case_012 => 12, case_013 => 13, case_014 => 14, case_015 => 15,
    case_016 => 16, case_017 => 17, case_018 => 18, case_019 => 19,
    case_020 => 20, case_021 => 21, case_022 => 22, case_023 => 23,
    case_024 => 24, case_025 => 25, case_026 => 26, case_027 => 27,
    case_028 => 28, case_029 => 29, case_030 => 30, case_031 => 31,
    case_032 => 32, case_033 => 33, case_034 => 34, case_035 => 35,
    case_036 => 36, case_037 => 37, case_038 => 38, case_039 => 39,
    case_040 => 40, case_041 => 41, case_042 => 42, case_043 => 43,
    case_044 => 44, case_045 => 45, case_046 => 46, case_047 => 47,
    case_048 => 48, case_049 => 49, case_050 => 50, case_051 => 51,
    case_052 => 52, case_053 => 53, case_054 => 54, case_055 => 55,
    case_056 => 56, case_057 => 57, case_058 => 58, case_059 => 59,
    case_060 => 60, case_061 => 61, case_062 => 62, case_063 => 63,
    case_064 => 64, case_065 => 65, case_066 => 66, case_067 => 67,
}
