#![expect(
    dead_code,
    reason = "由后续 ThrottledTemplateProcessor 对象通过 SSE 或普通模式构造"
)]

use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

use encoding_rs::{CoderResult, Encoder, Encoding};

use crate::exceptions::TemplateOutputException;
use crate::util::{Charset, JavaWriter};

use super::i_throttled_template_writer_control::IThrottledTemplateWriterControl;
use super::template_flow_controller::TemplateFlowController;
use super::throttled_template_writer_output_stream_adapter::ThrottledTemplateWriterOutputStreamAdapter;
use super::throttled_template_writer_writer_adapter::ThrottledTemplateWriterWriterAdapter;

enum ThrottledTemplateWriterAdapter {
    Characters(ThrottledTemplateWriterWriterAdapter),
    Bytes {
        adapter: ThrottledTemplateWriterOutputStreamAdapter,
        encoder: Encoder,
    },
}

/// 模板引擎的节流输出 Writer。
///
/// 对象可在字符输出和指定字符集的字节输出之间选择一种模式；模式一经初始化不可
/// 切换。每轮额度由 `allow` 设置，超额内容交给对应适配器缓存，并通过共享
/// [`TemplateFlowController`] 停止上游处理。
///
/// 对应 Java: `org.thymeleaf.engine.ThrottledTemplateWriter`。
pub(crate) struct ThrottledTemplateWriter {
    template_name: String,
    flow_controller: Rc<RefCell<TemplateFlowController>>,
    adapter: Option<ThrottledTemplateWriterAdapter>,
    flushable: bool,
}

impl ThrottledTemplateWriter {
    /// 创建尚未绑定输出的节流 Writer。
    pub(crate) fn new(
        template_name: String,
        flow_controller: Rc<RefCell<TemplateFlowController>>,
    ) -> Self {
        Self {
            template_name,
            flow_controller,
            adapter: None,
            flushable: false,
        }
    }

    /// 绑定字符型 Writer；已选择字节模式时返回模板输出异常。
    pub(crate) fn set_output_writer(
        &mut self,
        writer: Box<dyn JavaWriter>,
    ) -> Result<(), TemplateOutputException> {
        if matches!(
            self.adapter,
            Some(ThrottledTemplateWriterAdapter::Bytes { .. })
        ) {
            return Err(self.mode_error(
                "The throttled processor has already been initialized to use byte-based output \
                 (OutputStream), but a Writer has been specified.",
            ));
        }
        if self.adapter.is_none() {
            self.adapter = Some(ThrottledTemplateWriterAdapter::Characters(
                ThrottledTemplateWriterWriterAdapter::new(
                    self.template_name.clone(),
                    Rc::clone(&self.flow_controller),
                ),
            ));
        }
        if let Some(ThrottledTemplateWriterAdapter::Characters(adapter)) = self.adapter.as_mut() {
            adapter.set_writer(writer);
        }
        Ok(())
    }

    /// 绑定字节型输出并配置字符集和首轮最大字节数。
    pub(crate) fn set_output_stream(
        &mut self,
        output_stream: Box<dyn Write>,
        charset: &Charset,
        max_output_in_bytes: i32,
    ) -> Result<(), TemplateOutputException> {
        if matches!(
            self.adapter,
            Some(ThrottledTemplateWriterAdapter::Characters(_))
        ) {
            return Err(self.mode_error(
                "The throttled processor has already been initialized to use char-based output \
                 (Writer), but an OutputStream has been specified.",
            ));
        }
        if self.adapter.is_none() {
            let increment = if max_output_in_bytes == i32::MAX {
                128
            } else {
                (max_output_in_bytes / 8).clamp(16, 128) as usize
            };
            let encoding = Encoding::for_label(charset.name().as_bytes())
                .expect("Charset guarantees an encoding_rs-supported canonical name");
            self.adapter = Some(ThrottledTemplateWriterAdapter::Bytes {
                adapter: ThrottledTemplateWriterOutputStreamAdapter::new(
                    self.template_name.clone(),
                    Rc::clone(&self.flow_controller),
                    increment,
                ),
                encoder: encoding.new_encoder(),
            });
        }
        if let Some(ThrottledTemplateWriterAdapter::Bytes { adapter, .. }) = self.adapter.as_mut() {
            adapter.set_output_stream(output_stream);
        }
        Ok(())
    }

    /// 允许下一轮最多写出指定数量的字符或字节。
    pub(crate) fn allow(&mut self, limit: i32) -> Result<(), TemplateOutputException> {
        match self.adapter_mut()? {
            ThrottledTemplateWriterAdapter::Characters(adapter) => adapter.allow(limit),
            ThrottledTemplateWriterAdapter::Bytes { adapter, .. } => adapter.allow(limit),
        }
    }

    /// 写出 UTF-16 内容，并保持 Java 字符计数或编码后的字节计数。
    pub(crate) fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        self.flushable = true;
        match self.adapter_io_mut()? {
            ThrottledTemplateWriterAdapter::Characters(adapter) => adapter.write_utf16(characters),
            ThrottledTemplateWriterAdapter::Bytes { adapter, encoder } => {
                let bytes = Self::encode_utf16(encoder, characters, false);
                adapter.write_bytes(&bytes)
            }
        }
    }

    /// 刷新当前底层输出。
    pub(crate) fn flush(&mut self) -> io::Result<()> {
        match self.adapter_io_mut()? {
            ThrottledTemplateWriterAdapter::Characters(adapter) => adapter.flush(),
            ThrottledTemplateWriterAdapter::Bytes { adapter, .. } => adapter.flush(),
        }
    }

    /// 关闭当前底层输出。
    pub(crate) fn close(&mut self) -> io::Result<()> {
        match self.adapter_io_mut()? {
            ThrottledTemplateWriterAdapter::Characters(adapter) => adapter.close(),
            ThrottledTemplateWriterAdapter::Bytes { adapter, encoder } => {
                let final_bytes = Self::encode_utf16(encoder, &[], true);
                adapter.write_bytes(&final_bytes)?;
                adapter.close()
            }
        }
    }

    fn flush_if_needed(&mut self) -> io::Result<()> {
        if self.flushable {
            self.flush()?;
            self.flushable = false;
        }
        Ok(())
    }

    fn adapter_mut(
        &mut self,
    ) -> Result<&mut ThrottledTemplateWriterAdapter, TemplateOutputException> {
        if self.adapter.is_none() {
            return Err(self.mode_error("The throttled processor output has not been initialized."));
        }
        Ok(self.adapter.as_mut().expect("checked above"))
    }

    fn adapter_io_mut(&mut self) -> io::Result<&mut ThrottledTemplateWriterAdapter> {
        self.adapter
            .as_mut()
            .ok_or_else(|| io::Error::other("Throttled processor output has not been initialized"))
    }

    fn mode_error(&self, message: &str) -> TemplateOutputException {
        TemplateOutputException::new(
            Some(message.to_owned()),
            Some(self.template_name.clone()),
            -1,
            -1,
            io::Error::other(message.to_owned()),
        )
    }

    fn encode_utf16(encoder: &mut Encoder, characters: &[u16], last: bool) -> Vec<u8> {
        let mut source_offset = 0;
        let mut output = Vec::with_capacity(characters.len().saturating_mul(4).saturating_add(32));
        loop {
            let mut buffer = [0_u8; 1024];
            let (result, read, written, _) =
                encoder.encode_from_utf16(&characters[source_offset..], &mut buffer, last);
            output.extend_from_slice(&buffer[..written]);
            source_offset += read;
            if result == CoderResult::InputEmpty {
                break;
            }
        }
        output
    }
}

impl JavaWriter for ThrottledTemplateWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        Self::write_utf16(self, characters)
    }

    fn flush(&mut self) -> io::Result<()> {
        Self::flush(self)
    }

    fn close(&mut self) -> io::Result<()> {
        Self::close(self)
    }
}

impl IThrottledTemplateWriterControl for ThrottledTemplateWriter {
    fn is_overflown(&mut self) -> io::Result<bool> {
        self.flush_if_needed()?;
        match self.adapter.as_ref() {
            Some(ThrottledTemplateWriterAdapter::Characters(adapter)) => Ok(adapter.is_overflown()),
            Some(ThrottledTemplateWriterAdapter::Bytes { adapter, .. }) => {
                Ok(adapter.is_overflown())
            }
            None => Err(io::Error::other(
                "Throttled processor output has not been initialized",
            )),
        }
    }

    fn is_stopped(&mut self) -> io::Result<bool> {
        self.flush_if_needed()?;
        match self.adapter.as_ref() {
            Some(ThrottledTemplateWriterAdapter::Characters(adapter)) => Ok(adapter.is_stopped()),
            Some(ThrottledTemplateWriterAdapter::Bytes { adapter, .. }) => Ok(adapter.is_stopped()),
            None => Err(io::Error::other(
                "Throttled processor output has not been initialized",
            )),
        }
    }

    fn get_written_count(&self) -> i32 {
        match self.adapter.as_ref() {
            Some(ThrottledTemplateWriterAdapter::Characters(adapter)) => {
                adapter.get_written_count()
            }
            Some(ThrottledTemplateWriterAdapter::Bytes { adapter, .. }) => {
                adapter.get_written_count()
            }
            None => 0,
        }
    }

    fn get_max_overflow_size(&self) -> i32 {
        match self.adapter.as_ref() {
            Some(ThrottledTemplateWriterAdapter::Characters(adapter)) => {
                adapter.get_max_overflow_size()
            }
            Some(ThrottledTemplateWriterAdapter::Bytes { adapter, .. }) => {
                adapter.get_max_overflow_size()
            }
            None => 0,
        }
    }

    fn get_overflow_grow_count(&self) -> i32 {
        match self.adapter.as_ref() {
            Some(ThrottledTemplateWriterAdapter::Characters(adapter)) => {
                adapter.get_overflow_grow_count()
            }
            Some(ThrottledTemplateWriterAdapter::Bytes { adapter, .. }) => {
                adapter.get_overflow_grow_count()
            }
            None => 0,
        }
    }
}
