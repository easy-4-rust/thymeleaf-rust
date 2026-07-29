#![expect(
    dead_code,
    reason = "由后续 ThrottledTemplateProcessor 对象直接构造并调用"
)]

use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

use crate::exceptions::TemplateOutputException;
use crate::util::{Charset, JavaWriter};

use super::i_sse_throttled_template_writer_control::ISSEThrottledTemplateWriterControl;
use super::i_throttled_template_writer_control::IThrottledTemplateWriterControl;
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
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "由后续 ThrottledTemplateProcessor 对象直接构造")
)]
pub(crate) struct SSEThrottledTemplateWriter {
    writer: ThrottledTemplateWriter,
    id: Option<Vec<u16>>,
    event: Option<Vec<u16>>,
    event_has_meta: bool,
    new_event: bool,
}

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "由后续 ThrottledTemplateProcessor 对象直接调用")
)]
impl SSEThrottledTemplateWriter {
    /// 创建尚未绑定输出的 SSE 节流 Writer。
    pub(crate) fn new(
        template_name: String,
        flow_controller: Rc<RefCell<TemplateFlowController>>,
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
    pub(crate) fn set_output_writer(
        &mut self,
        writer: Box<dyn JavaWriter>,
    ) -> Result<(), TemplateOutputException> {
        self.writer.set_output_writer(writer)
    }

    /// 绑定字节输出。
    pub(crate) fn set_output_stream(
        &mut self,
        output_stream: Box<dyn Write>,
        charset: &Charset,
        max_output_in_bytes: i32,
    ) -> Result<(), TemplateOutputException> {
        self.writer
            .set_output_stream(output_stream, charset, max_output_in_bytes)
    }

    /// 允许下一轮写出指定数量。
    pub(crate) fn allow(&mut self, limit: i32) -> Result<(), TemplateOutputException> {
        self.writer.allow(limit)
    }

    /// 写出 SSE 正文并在每个 LF 后追加 `data: `。
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

impl JavaWriter for SSEThrottledTemplateWriter {
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
