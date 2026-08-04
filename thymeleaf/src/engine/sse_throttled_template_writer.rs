use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use crate::exceptions::TemplateOutputException;
use crate::util::{Charset, TemplateWriter};

use super::i_throttled_template_writer_control::IThrottledTemplateWriterControl;
use super::isse_throttled_template_writer_control::ISSEThrottledTemplateWriterControl;
use super::template_flow_controller::TemplateFlowController;
use super::throttled_template_writer::ThrottledTemplateWriter;

const SSE_ID_PREFIX: &[u16] = &[b'i' as u16, b'd' as u16, b':' as u16, b' ' as u16];
const SSE_EVENT_PREFIX: &[u16] = &[
    b'e' as u16,
    b'v' as u16,
    b'e' as u16,
    b'n' as u16,
    b't' as u16,
    b':' as u16,
    b' ' as u16,
];
const SSE_DATA_PREFIX: &[u16] = &[
    b'd' as u16,
    b'a' as u16,
    b't' as u16,
    b'a' as u16,
    b':' as u16,
    b' ' as u16,
];
const LINE_FEED: u16 = b'\n' as u16;

/// 为节流模板输出增加 Server-Sent Events 帧语义的 Writer。
///
/// 每个事件首次写正文时输出可选 `event`、`id` 元数据和 `data: ` 前缀；正文中
/// 每个换行后自动补充新的 `data: ` 前缀，事件结束时写出 SSE 空行边界。
///
/// 对应 Java: `org.thymeleaf.engine.SSEThrottledTemplateWriter`。
pub(crate) struct SSEThrottledTemplateWriter {
    writer: ThrottledTemplateWriter,
    id: Option<Vec<u16>>,
    event: Option<Vec<u16>>,
    event_has_meta: bool,
    new_event: bool,
}

impl SSEThrottledTemplateWriter {
    /// 创建尚未绑定输出的 SSE 节流 Writer。
    /// 对应 Java 语义：`SSEThrottledTemplateWriter` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new(
        template_name: String,
        flow_controller: Arc<Mutex<TemplateFlowController>>,
    ) -> Self {
        Self {
            writer: ThrottledTemplateWriter::new(template_name, flow_controller),
            id: None,
            event: None,
            event_has_meta: false,
            new_event: true,
        }
    }

    /// 绑定字符输出。
    /// 对应 Java 语义：`SSEThrottledTemplateWriter` 的 `set_output_writer` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn set_output_writer(
        &mut self,
        writer: Box<dyn TemplateWriter>,
    ) -> Result<(), TemplateOutputException> {
        self.writer.set_output_writer(writer)
    }

    /// 绑定字节输出。
    /// 对应 Java 语义：Java 接口/超类方法 `setOutputStream()` 的 Rust 移植（`SSEThrottledTemplateWriter` 继承路径）。
    pub(crate) fn set_output_stream(
        &mut self,
        output_stream: Box<dyn Write + Send>,
        charset: &Charset,
        max_output_in_bytes: i32,
    ) -> Result<(), TemplateOutputException> {
        self.writer
            .set_output_stream(output_stream, charset, max_output_in_bytes)
    }

    /// 允许下一轮写出指定数量。
    /// 对应 Java 语义：Java 接口/超类方法 `allow()` 的 Rust 移植（`SSEThrottledTemplateWriter` 继承路径）。
    pub(crate) fn allow(&mut self, limit: i32) -> Result<(), TemplateOutputException> {
        self.writer.allow(limit)
    }

    /// 写出 SSE 正文并在每个 LF 后追加 `data: `。
    /// 对应 Java 语义：`SSEThrottledTemplateWriter` 的 `write_utf16` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        if characters.is_empty() {
            return self.writer.write_utf16(characters);
        }
        if self.new_event {
            self.do_start_event()?;
            self.writer.write_utf16(SSE_DATA_PREFIX)?;
            self.new_event = false;
        }

        let mut segment_start = 0;
        for (index, character) in characters.iter().enumerate() {
            if *character == LINE_FEED {
                self.writer
                    .write_utf16(&characters[segment_start..=index])?;
                self.writer.write_utf16(SSE_DATA_PREFIX)?;
                segment_start = index + 1;
            }
        }
        if segment_start < characters.len() {
            self.writer.write_utf16(&characters[segment_start..])?;
        }
        Ok(())
    }

    fn do_start_event(&mut self) -> io::Result<()> {
        self.event_has_meta = false;
        if let Some(event) = self.event.as_deref() {
            if !Self::check_token_valid(event) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Event for SSE event cannot contain a newline (\\n) character",
                ));
            }
            self.writer.write_utf16(SSE_EVENT_PREFIX)?;
            self.writer.write_utf16(event)?;
            self.writer.write_utf16(&[LINE_FEED])?;
            self.event_has_meta = true;
        }
        if let Some(id) = self.id.as_deref() {
            if !Self::check_token_valid(id) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "ID for SSE event cannot contain a newline (\\n) character",
                ));
            }
            self.writer.write_utf16(SSE_ID_PREFIX)?;
            self.writer.write_utf16(id)?;
            self.writer.write_utf16(&[LINE_FEED])?;
            self.event_has_meta = true;
        }
        Ok(())
    }

    fn check_token_valid(token: &[u16]) -> bool {
        !token.contains(&LINE_FEED)
    }
}

impl TemplateWriter for SSEThrottledTemplateWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        Self::write_utf16(self, characters)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    fn close(&mut self) -> io::Result<()> {
        self.writer.close()
    }
}

impl IThrottledTemplateWriterControl for SSEThrottledTemplateWriter {
    fn as_sse_control(&mut self) -> Option<&mut dyn ISSEThrottledTemplateWriterControl> {
        Some(self)
    }

    fn is_overflown(&mut self) -> io::Result<bool> {
        self.writer.is_overflown()
    }

    fn is_stopped(&mut self) -> io::Result<bool> {
        self.writer.is_stopped()
    }

    fn get_written_count(&self) -> i32 {
        self.writer.get_written_count()
    }

    fn get_max_overflow_size(&self) -> i32 {
        self.writer.get_max_overflow_size()
    }

    fn get_overflow_grow_count(&self) -> i32 {
        self.writer.get_overflow_grow_count()
    }
}

impl ISSEThrottledTemplateWriterControl for SSEThrottledTemplateWriter {
    fn start_event(&mut self, id: Option<&[u16]>, event: Option<&[u16]>) {
        self.new_event = true;
        self.id = id.map(<[u16]>::to_vec);
        self.event = event.map(<[u16]>::to_vec);
    }

    fn end_event(&mut self) -> io::Result<()> {
        if !self.new_event {
            self.writer.write_utf16(&[LINE_FEED, LINE_FEED])
        } else if self.event_has_meta {
            self.writer.write_utf16(&[LINE_FEED])
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};

    use super::super::i_throttled_template_writer_control::IThrottledTemplateWriterControl;
    use super::super::isse_throttled_template_writer_control::ISSEThrottledTemplateWriterControl;
    use super::super::template_flow_controller::TemplateFlowController;
    use super::SSEThrottledTemplateWriter;
    use crate::util::Charset;
    use crate::util::TemplateWriter;

    #[test]
    fn framing_and_invalid_event_token_match_java_golden() {
        let controller = Arc::new(Mutex::new(TemplateFlowController::new()));
        let output = Arc::new(Mutex::new(Vec::new()));
        let mut writer = SSEThrottledTemplateWriter::new("template".to_owned(), controller);
        writer
            .set_output_writer(Box::new(RecordingWriter(Arc::clone(&output))))
            .expect("SSE output initialization");
        writer.allow(i32::MAX).expect("allow SSE output");
        {
            let control: &mut dyn ISSEThrottledTemplateWriterControl = &mut writer;
            control.start_event(Some(&utf16("id")), Some(&utf16("event")));
        }
        writer.write_utf16(&utf16("a\nb")).expect("write SSE body");
        {
            let control: &mut dyn ISSEThrottledTemplateWriterControl = &mut writer;
            control.end_event().expect("finish SSE event");
        }
        let actual = String::from_utf16(&output.lock().expect("output lock"))
            .expect("test output must be valid UTF-16")
            .replace('\n', "\\n");
        assert_golden("sse", &actual);

        let invalid_controller = Arc::new(Mutex::new(TemplateFlowController::new()));
        let mut invalid =
            SSEThrottledTemplateWriter::new("template".to_owned(), invalid_controller);
        invalid
            .set_output_writer(Box::new(RecordingWriter(Arc::new(Mutex::new(Vec::new())))))
            .expect("SSE output initialization");
        invalid.allow(i32::MAX).expect("allow SSE output");
        {
            let control: &mut dyn ISSEThrottledTemplateWriterControl = &mut invalid;
            control.start_event(None, Some(&utf16("bad\nname")));
        }
        let error = invalid
            .write_utf16(&utf16("x"))
            .expect_err("newlines are invalid in SSE event names");
        assert_golden("sseInvalid", &format!("IllegalArgumentException:{error}"));

        let invalid_id_controller = Arc::new(Mutex::new(TemplateFlowController::new()));
        let mut invalid_id =
            SSEThrottledTemplateWriter::new("template".to_owned(), invalid_id_controller);
        invalid_id
            .set_output_writer(Box::new(RecordingWriter(Arc::new(Mutex::new(Vec::new())))))
            .expect("SSE output initialization");
        invalid_id.allow(i32::MAX).expect("allow SSE output");
        {
            let control: &mut dyn ISSEThrottledTemplateWriterControl = &mut invalid_id;
            control.start_event(Some(&utf16("bad\nid")), None);
        }
        let error = invalid_id
            .write_utf16(&utf16("x"))
            .expect_err("newlines are invalid in SSE IDs");
        assert_golden("sseInvalidId", &format!("IllegalArgumentException:{error}"));

        let empty_controller = Arc::new(Mutex::new(TemplateFlowController::new()));
        let empty_output = Arc::new(Mutex::new(Vec::new()));
        let mut empty = SSEThrottledTemplateWriter::new("template".to_owned(), empty_controller);
        empty
            .set_output_writer(Box::new(RecordingWriter(Arc::clone(&empty_output))))
            .expect("SSE output initialization");
        empty.allow(i32::MAX).expect("allow SSE output");
        {
            let control: &mut dyn ISSEThrottledTemplateWriterControl = &mut empty;
            control.start_event(Some(&utf16("id")), Some(&utf16("event")));
        }
        empty
            .write_utf16(&[])
            .expect("empty SSE write must reach underlying writer");
        {
            let control: &mut dyn ISSEThrottledTemplateWriterControl = &mut empty;
            control.end_event().expect("empty SSE event must close");
        }
        let output = String::from_utf16(&empty_output.lock().expect("output lock"))
            .expect("test output must be valid UTF-16")
            .replace('\n', "\\n");
        assert_golden(
            "sseEmpty",
            &format!(
                "{output},{},{}",
                empty.is_overflown().expect("empty SSE overflow"),
                empty.is_stopped().expect("empty SSE stop")
            ),
        );
    }

    #[test]
    fn byte_output_and_parent_control_dispatch_match_java_golden() {
        let controller = Arc::new(Mutex::new(TemplateFlowController::new()));
        let output = Arc::new(Mutex::new(Vec::new()));
        let mut writer = SSEThrottledTemplateWriter::new("template".to_owned(), controller);
        let charset = Charset::for_name("UTF-8").expect("UTF-8 charset");
        writer
            .set_output_stream(
                Box::new(RecordingOutputStream(Arc::clone(&output))),
                &charset,
                i32::MAX,
            )
            .expect("SSE byte output initialization");
        writer.allow(i32::MAX).expect("allow SSE byte output");

        {
            let control: &mut dyn ISSEThrottledTemplateWriterControl = &mut writer;
            control.start_event(Some(&utf16("id")), Some(&utf16("event")));
        }
        writer
            .write_utf16(&utf16("x"))
            .expect("write SSE byte body");
        {
            let control: &mut dyn ISSEThrottledTemplateWriterControl = &mut writer;
            control.end_event().expect("finish SSE byte event");
        }
        writer.flush().expect("flush SSE byte event");

        assert_golden("sseBytes", &hex_output(&output));
    }

    struct RecordingWriter(Arc<Mutex<Vec<u16>>>);

    impl TemplateWriter for RecordingWriter {
        fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
            self.0
                .lock()
                .expect("recording writer lock")
                .extend_from_slice(characters);
            Ok(())
        }
    }

    /// 在 SSE 字节输出测试中记录 OutputStream 写入结果。
    struct RecordingOutputStream(Arc<Mutex<Vec<u8>>>);

    impl Write for RecordingOutputStream {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0
                .lock()
                .expect("recording output stream lock")
                .extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn assert_golden(key: &str, actual: &str) {
        let expected = include_str!("../../tests/fixtures/throttled_template_writer_golden.txt")
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}=")))
            .expect("Java Golden record");
        assert_eq!(actual, expected, "Java Golden key {key}");
    }

    fn utf16(value: &str) -> Vec<u16> {
        value.encode_utf16().collect()
    }

    fn hex_output(output: &Arc<Mutex<Vec<u8>>>) -> String {
        output
            .lock()
            .expect("recording output stream lock")
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}
