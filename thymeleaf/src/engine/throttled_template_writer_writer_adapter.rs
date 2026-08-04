use std::io;
use std::sync::{Arc, Mutex};

use crate::exceptions::TemplateOutputException;
use crate::util::TemplateWriter;

use super::template_flow_controller::TemplateFlowController;

const OVERFLOW_BUFFER_INCREMENT: usize = 256;

/// 以 Java UTF-16 字符为计量单位的节流 Writer 适配器。
///
/// 该对象在达到本轮额度后把后续代码单元保存到溢出缓冲区，并同步停止模板处理。
/// 再次调用 `allow` 时先排空已有溢出数据，完全保留 Java 的严格大于分支、计数和
/// 256 个代码单元的扩容策略。
///
/// 对应 Java:
/// `org.thymeleaf.engine.ThrottledTemplateWriterWriterAdapter`。
pub(crate) struct ThrottledTemplateWriterWriterAdapter {
    template_name: String,
    flow_controller: Arc<Mutex<TemplateFlowController>>,
    writer: Option<Box<dyn TemplateWriter>>,
    overflow: Vec<u16>,
    overflow_size: usize,
    max_overflow_size: usize,
    overflow_grow_count: i32,
    unlimited: bool,
    limit: i32,
    written_count: i32,
}

impl ThrottledTemplateWriterWriterAdapter {
    /// 创建尚未绑定输出且初始停止的字符适配器。
    /// 对应 Java 语义：`ThrottledTemplateWriterWriterAdapter` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new(
        template_name: String,
        flow_controller: Arc<Mutex<TemplateFlowController>>,
    ) -> Self {
        flow_controller
            .lock()
            .expect("template flow controller lock poisoned")
            .stop_processing = true;
        Self {
            template_name,
            flow_controller,
            writer: None,
            overflow: Vec::new(),
            overflow_size: 0,
            max_overflow_size: 0,
            overflow_grow_count: 0,
            unlimited: false,
            limit: 0,
            written_count: 0,
        }
    }

    /// 绑定下一轮输出 Writer，并按 Java 语义仅重置本轮写出计数。
    /// 对应 Java: `ThrottledTemplateWriterWriterAdapter#setWriter()`。
    pub(crate) fn set_writer(&mut self, writer: Box<dyn TemplateWriter>) {
        self.writer = Some(writer);
        self.written_count = 0;
    }

    /// 允许最多写出 `limit` 个 UTF-16 代码单元，并优先排空溢出缓冲。
    /// 对应 Java: `ThrottledTemplateWriterWriterAdapter#allow()`。
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
        let characters = self.overflow[..writable].to_vec();
        if let Err(cause) = self
            .writer_mut()
            .and_then(|writer| writer.write_utf16(&characters))
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

    /// 写出 UTF-16 代码单元；超过额度的尾部进入溢出缓冲区。
    /// 对应 Java 语义：`ThrottledTemplateWriterWriterAdapter` 的 `write_utf16` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        if self.limit == 0 {
            self.overflow(characters);
            return Ok(());
        }

        let writable = if self.unlimited || self.limit as usize > characters.len() {
            characters.len()
        } else {
            self.limit as usize
        };
        self.writer_mut()?.write_utf16(&characters[..writable])?;
        if writable < characters.len() {
            self.overflow(&characters[writable..]);
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

    /// 刷新底层 Writer；溢出状态由查询方法单独报告。
    /// 对应 Java: `ThrottledTemplateWriterWriterAdapter#flush()`。
    pub(crate) fn flush(&mut self) -> io::Result<()> {
        self.writer_mut()?.flush()
    }

    /// 关闭当前 Writer。
    /// 对应 Java: `ThrottledTemplateWriterWriterAdapter#close()`。
    pub(crate) fn close(&mut self) -> io::Result<()> {
        self.writer_mut()?.close()
    }

    /// 对应 Java: `ThrottledTemplateWriterWriterAdapter#isOverflown()`。
    pub(crate) fn is_overflown(&self) -> bool {
        self.overflow_size > 0
    }

    /// 对应 Java: `ThrottledTemplateWriterWriterAdapter#isStopped()`。
    pub(crate) fn is_stopped(&self) -> bool {
        self.limit == 0
    }

    /// 对应 Java: `ThrottledTemplateWriterWriterAdapter#getWrittenCount()`。
    pub(crate) fn get_written_count(&self) -> i32 {
        self.written_count
    }

    /// 对应 Java: `ThrottledTemplateWriterWriterAdapter#getMaxOverflowSize()`。
    pub(crate) fn get_max_overflow_size(&self) -> i32 {
        self.max_overflow_size as i32
    }

    /// 对应 Java: `ThrottledTemplateWriterWriterAdapter#getOverflowGrowCount()`。
    pub(crate) fn get_overflow_grow_count(&self) -> i32 {
        self.overflow_grow_count
    }

    fn overflow(&mut self, characters: &[u16]) {
        self.ensure_overflow_capacity(characters.len());
        let end = self.overflow_size + characters.len();
        self.overflow[self.overflow_size..end].copy_from_slice(characters);
        self.overflow_size = end;
        self.max_overflow_size = self.max_overflow_size.max(self.overflow_size);
    }

    fn ensure_overflow_capacity(&mut self, len: usize) {
        if self.overflow.is_empty() {
            let capacity = (len / OVERFLOW_BUFFER_INCREMENT + 1) * OVERFLOW_BUFFER_INCREMENT;
            self.overflow.resize(capacity, 0);
            return;
        }
        let target_len = self.overflow_size + len;
        if self.overflow.len() < target_len {
            let capacity = (target_len / OVERFLOW_BUFFER_INCREMENT + 1) * OVERFLOW_BUFFER_INCREMENT;
            self.overflow.resize(capacity, 0);
            self.overflow_grow_count += 1;
        }
    }

    fn writer_mut(&mut self) -> io::Result<&mut (dyn TemplateWriter + 'static)> {
        self.writer
            .as_deref_mut()
            .ok_or_else(|| io::Error::other("Throttled writer output has not been initialized"))
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::{Arc, Mutex};

    use crate::util::TemplateWriter;

    use super::super::template_flow_controller::TemplateFlowController;
    use super::ThrottledTemplateWriterWriterAdapter;

    /// 在适配器测试中收集 UTF-16 输出的最小 Java Writer 实现。
    struct RecordingWriter {
        output: Arc<Mutex<Vec<u16>>>,
    }

    impl TemplateWriter for RecordingWriter {
        fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
            self.output
                .lock()
                .expect("recording writer lock poisoned")
                .extend_from_slice(characters);
            Ok(())
        }
    }

    fn utf16(value: &str) -> Vec<u16> {
        value.encode_utf16().collect()
    }

    fn output_string(output: &Arc<Mutex<Vec<u16>>>) -> String {
        String::from_utf16_lossy(
            output
                .lock()
                .expect("recording writer lock poisoned")
                .as_slice(),
        )
    }

    #[test]
    fn direct_writer_adapter_state_machine_matches_java_golden() {
        let controller = Arc::new(Mutex::new(TemplateFlowController::new()));
        let output = Arc::new(Mutex::new(Vec::new()));
        let mut adapter = ThrottledTemplateWriterWriterAdapter::new(
            "template".to_owned(),
            Arc::clone(&controller),
        );
        adapter.set_writer(Box::new(RecordingWriter {
            output: Arc::clone(&output),
        }));

        adapter.allow(2).expect("Java Golden allow(2) must succeed");
        adapter
            .write_utf16(&utf16("abcd"))
            .expect("Java Golden adapter write must succeed");
        assert_eq!(output_string(&output), "ab");
        assert_eq!(adapter.get_written_count(), 2);
        assert!(adapter.is_overflown());
        assert!(adapter.is_stopped());
        assert!(
            controller
                .lock()
                .expect("template flow controller lock poisoned")
                .stop_processing
        );
        assert_eq!(adapter.get_max_overflow_size(), 2);
        assert_eq!(adapter.get_overflow_grow_count(), 0);

        adapter
            .allow(i32::MAX)
            .expect("Java Golden unlimited allow must drain overflow");
        assert_eq!(output_string(&output), "abcd");
        assert_eq!(adapter.get_written_count(), 4);
        assert!(!adapter.is_overflown());
        assert!(!adapter.is_stopped());
        assert!(
            !controller
                .lock()
                .expect("template flow controller lock poisoned")
                .stop_processing
        );
        assert_eq!(adapter.get_max_overflow_size(), 2);
        assert_eq!(adapter.get_overflow_grow_count(), 0);
    }

    #[test]
    fn overflow_drain_io_failure_matches_java_golden() {
        let controller = Arc::new(Mutex::new(TemplateFlowController::new()));
        let mut adapter =
            ThrottledTemplateWriterWriterAdapter::new("template".to_owned(), controller);
        adapter.set_writer(Box::new(FailOnSecondWriteWriter { writes: 0 }));
        adapter
            .allow(1)
            .expect("initial Java Golden allow must succeed");
        adapter
            .write_utf16(&utf16("ab"))
            .expect("first direct write must buffer its overflow");

        let error = adapter
            .allow(i32::MAX)
            .expect_err("Java Golden overflow drain must wrap I/O failure");
        assert_golden(
            "adapterOverflowIo",
            &format!("TemplateOutputException:{error}"),
        );
    }

    /// 第一次写入成功、第二次写入失败的 Java Writer。
    struct FailOnSecondWriteWriter {
        writes: usize,
    }

    impl TemplateWriter for FailOnSecondWriteWriter {
        fn write_utf16(&mut self, _characters: &[u16]) -> io::Result<()> {
            self.writes += 1;
            if self.writes > 1 {
                return Err(io::Error::other("overflow sink failure"));
            }
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
}
