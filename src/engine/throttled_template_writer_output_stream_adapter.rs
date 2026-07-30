use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use crate::exceptions::TemplateOutputException;

use super::template_flow_controller::TemplateFlowController;

/// 以输出字节为计量单位的节流 OutputStream 适配器。
///
/// 达到额度后的字节进入按构造增量扩展的溢出缓冲区；下一次 `allow` 会先排空
/// 缓冲区。所有额度、停止标志及诊断计数均与 Java 对象保持相同含义。
///
/// 对应 Java:
/// `org.thymeleaf.engine.ThrottledTemplateWriterOutputStreamAdapter`。
pub(crate) struct ThrottledTemplateWriterOutputStreamAdapter {
    template_name: String,
    flow_controller: Arc<Mutex<TemplateFlowController>>,
    overflow_increment_in_bytes: usize,
    output_stream: Option<Box<dyn Write + Send>>,
    overflow: Vec<u8>,
    overflow_size: usize,
    max_overflow_size: usize,
    overflow_grow_count: i32,
    unlimited: bool,
    limit: i32,
    written_count: i32,
}

impl ThrottledTemplateWriterOutputStreamAdapter {
    /// 创建尚未绑定输出且初始停止的字节适配器。
    pub(crate) fn new(
        template_name: String,
        flow_controller: Arc<Mutex<TemplateFlowController>>,
        overflow_increment_in_bytes: usize,
    ) -> Self {
        flow_controller
            .lock()
            .expect("template flow controller lock poisoned")
            .stop_processing = true;
        Self {
            template_name,
            flow_controller,
            overflow_increment_in_bytes,
            output_stream: None,
            overflow: Vec::new(),
            overflow_size: 0,
            max_overflow_size: 0,
            overflow_grow_count: 0,
            unlimited: false,
            limit: 0,
            written_count: 0,
        }
    }

    /// 绑定下一轮 OutputStream，并按 Java 语义仅重置本轮写出计数。
    pub(crate) fn set_output_stream(&mut self, output_stream: Box<dyn Write + Send>) {
        self.output_stream = Some(output_stream);
        self.written_count = 0;
    }

    /// 允许最多写出 `limit` 个字节，并优先排空已有溢出数据。
    pub(crate) fn allow(&mut self, limit: i32) -> Result<(), TemplateOutputException> {
        if limit == i32::MAX || limit < 0 {
            self.unlimited = true;
            self.limit = -1;
        } else {
            self.unlimited = false;
            self.limit = limit;
        }
        self.flow_controller
            .lock()
            .expect("template flow controller lock poisoned")
            .stop_processing = self.limit == 0;
        if self.overflow_size == 0 || self.limit == 0 {
            return Ok(());
        }

        let writable = if self.unlimited || self.limit as usize > self.overflow_size {
            self.overflow_size
        } else {
            self.limit as usize
        };
        let bytes = self.overflow[..writable].to_vec();
        if let Err(cause) = self
            .output_mut()
            .and_then(|output| output.write_all(&bytes))
        {
            return Err(TemplateOutputException::new(
                Some(
                    "Exception while trying to write overflowed buffer in throttled template"
                        .to_owned(),
                ),
                Some(self.template_name.clone()),
                -1,
                -1,
                cause,
            ));
        }
        if writable < self.overflow_size {
            self.overflow.copy_within(writable..self.overflow_size, 0);
        }
        self.overflow_size -= writable;
        self.written_count += writable as i32;
        if !self.unlimited {
            self.limit -= writable as i32;
        }
        if self.limit == 0 {
            self.flow_controller
                .lock()
                .expect("template flow controller lock poisoned")
                .stop_processing = true;
        }
        Ok(())
    }

    /// 写出字节；超过本轮额度的尾部进入溢出缓冲区。
    pub(crate) fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        if self.limit == 0 {
            self.overflow(bytes);
            return Ok(());
        }
        let writable = if self.unlimited || self.limit as usize > bytes.len() {
            bytes.len()
        } else {
            self.limit as usize
        };
        self.output_mut()?.write_all(&bytes[..writable])?;
        if writable < bytes.len() {
            self.overflow(&bytes[writable..]);
        }
        self.written_count += writable as i32;
        if !self.unlimited {
            self.limit -= writable as i32;
        }
        if self.limit == 0 {
            self.flow_controller
                .lock()
                .expect("template flow controller lock poisoned")
                .stop_processing = true;
        }
        Ok(())
    }

    /// 刷新当前 OutputStream。
    pub(crate) fn flush(&mut self) -> io::Result<()> {
        self.output_mut()?.flush()
    }

    /// Java OutputStream 的 close 会先刷新；Rust 所有权释放随后关闭资源。
    pub(crate) fn close(&mut self) -> io::Result<()> {
        self.output_mut()?.flush()
    }

    pub(crate) fn is_overflown(&self) -> bool {
        self.overflow_size > 0
    }

    pub(crate) fn is_stopped(&self) -> bool {
        self.limit == 0
    }

    pub(crate) fn get_written_count(&self) -> i32 {
        self.written_count
    }

    pub(crate) fn get_max_overflow_size(&self) -> i32 {
        self.max_overflow_size as i32
    }

    pub(crate) fn get_overflow_grow_count(&self) -> i32 {
        self.overflow_grow_count
    }

    fn overflow(&mut self, bytes: &[u8]) {
        self.ensure_overflow_capacity(bytes.len());
        let end = self.overflow_size + bytes.len();
        self.overflow[self.overflow_size..end].copy_from_slice(bytes);
        self.overflow_size = end;
        self.max_overflow_size = self.max_overflow_size.max(self.overflow_size);
    }

    fn ensure_overflow_capacity(&mut self, len: usize) {
        if self.overflow.is_empty() {
            let mut initial_size = self.overflow_increment_in_bytes * 3;
            while initial_size < len {
                initial_size += self.overflow_increment_in_bytes;
            }
            self.overflow.resize(initial_size, 0);
            return;
        }
        let target_len = self.overflow_size + len;
        if self.overflow.len() < target_len {
            let mut new_len = self.overflow.len();
            while new_len < target_len {
                new_len += self.overflow_increment_in_bytes;
            }
            self.overflow.resize(new_len, 0);
            self.overflow_grow_count += 1;
        }
    }

    fn output_mut(&mut self) -> io::Result<&mut (dyn Write + Send + 'static)> {
        self.output_stream
            .as_deref_mut()
            .ok_or_else(|| io::Error::other("Throttled output stream has not been initialized"))
    }
}
