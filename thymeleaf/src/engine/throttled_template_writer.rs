use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use encoding_rs::{CoderResult, Encoder, Encoding};

use crate::exceptions::TemplateOutputException;
use crate::util::{Charset, JavaWriter};

use super::i_throttled_template_writer_control::IThrottledTemplateWriterControl;
use super::template_flow_controller::TemplateFlowController;
use super::throttled_template_writer_output_stream_adapter::ThrottledTemplateWriterOutputStreamAdapter;
use super::throttled_template_writer_writer_adapter::ThrottledTemplateWriterWriterAdapter;

/// 字符与字节节流适配器共享的状态控制合同。
///
/// 对应 Java: `ThrottledTemplateWriter.IThrottledTemplateWriterAdapter`。
#[expect(dead_code, reason = "保留 Java 私有适配器接口的完整对象级合同")]
trait IThrottledTemplateWriterAdapter {
    fn is_overflown(&self) -> bool;
    fn is_stopped(&self) -> bool;
    fn get_written_count(&self) -> i32;
    fn get_max_overflow_size(&self) -> i32;
    fn get_overflow_grow_count(&self) -> i32;
    fn allow(&mut self, limit: i32) -> Result<(), TemplateOutputException>;
}

impl IThrottledTemplateWriterAdapter for ThrottledTemplateWriterWriterAdapter {
    fn is_overflown(&self) -> bool {
        ThrottledTemplateWriterWriterAdapter::is_overflown(self)
    }

    fn is_stopped(&self) -> bool {
        ThrottledTemplateWriterWriterAdapter::is_stopped(self)
    }

    fn get_written_count(&self) -> i32 {
        ThrottledTemplateWriterWriterAdapter::get_written_count(self)
    }

    fn get_max_overflow_size(&self) -> i32 {
        ThrottledTemplateWriterWriterAdapter::get_max_overflow_size(self)
    }

    fn get_overflow_grow_count(&self) -> i32 {
        ThrottledTemplateWriterWriterAdapter::get_overflow_grow_count(self)
    }

    fn allow(&mut self, limit: i32) -> Result<(), TemplateOutputException> {
        ThrottledTemplateWriterWriterAdapter::allow(self, limit)
    }
}

impl IThrottledTemplateWriterAdapter for ThrottledTemplateWriterOutputStreamAdapter {
    fn is_overflown(&self) -> bool {
        ThrottledTemplateWriterOutputStreamAdapter::is_overflown(self)
    }

    fn is_stopped(&self) -> bool {
        ThrottledTemplateWriterOutputStreamAdapter::is_stopped(self)
    }

    fn get_written_count(&self) -> i32 {
        ThrottledTemplateWriterOutputStreamAdapter::get_written_count(self)
    }

    fn get_max_overflow_size(&self) -> i32 {
        ThrottledTemplateWriterOutputStreamAdapter::get_max_overflow_size(self)
    }

    fn get_overflow_grow_count(&self) -> i32 {
        ThrottledTemplateWriterOutputStreamAdapter::get_overflow_grow_count(self)
    }

    fn allow(&mut self, limit: i32) -> Result<(), TemplateOutputException> {
        ThrottledTemplateWriterOutputStreamAdapter::allow(self, limit)
    }
}

enum ThrottledTemplateWriterAdapterMode {
    Characters(ThrottledTemplateWriterWriterAdapter),
    Bytes {
        adapter: ThrottledTemplateWriterOutputStreamAdapter,
        encoder: Encoder,
        pending_bytes: Vec<u8>,
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
    flow_controller: Arc<Mutex<TemplateFlowController>>,
    adapter: Option<ThrottledTemplateWriterAdapterMode>,
    flushable: bool,
}

impl ThrottledTemplateWriter {
    /// 创建尚未绑定输出的节流 Writer。
    /// 对应 Java 语义：`ThrottledTemplateWriter` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new(
        template_name: String,
        flow_controller: Arc<Mutex<TemplateFlowController>>,
    ) -> Self {
        Self {
            template_name,
            flow_controller,
            adapter: None,
            flushable: false,
        }
    }

    /// 绑定字符型 Writer；已选择字节模式时返回模板输出异常。
    /// 对应 Java 语义：`ThrottledTemplateWriter` 的 `set_output_writer` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn set_output_writer(
        &mut self,
        writer: Box<dyn JavaWriter>,
    ) -> Result<(), TemplateOutputException> {
        if matches!(
            self.adapter,
            Some(ThrottledTemplateWriterAdapterMode::Bytes { .. })
        ) {
            return Err(self.mode_error(
                "The throttled processor has already been initialized to use byte-based output \
                 (OutputStream), but a Writer has been specified.",
            ));
        }
        if self.adapter.is_none() {
            self.adapter = Some(ThrottledTemplateWriterAdapterMode::Characters(
                ThrottledTemplateWriterWriterAdapter::new(
                    self.template_name.clone(),
                    Arc::clone(&self.flow_controller),
                ),
            ));
        }
        if let Some(ThrottledTemplateWriterAdapterMode::Characters(adapter)) = self.adapter.as_mut()
        {
            adapter.set_writer(writer);
        }
        Ok(())
    }

    /// 绑定字节型输出并配置字符集和首轮最大字节数。
    /// 对应 Java 语义：Java 接口/超类方法 `setOutputStream()` 的 Rust 移植（`ThrottledTemplateWriter` 继承路径）。
    pub(crate) fn set_output_stream(
        &mut self,
        output_stream: Box<dyn Write + Send>,
        charset: &Charset,
        max_output_in_bytes: i32,
    ) -> Result<(), TemplateOutputException> {
        if matches!(
            self.adapter,
            Some(ThrottledTemplateWriterAdapterMode::Characters(_))
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
            self.adapter = Some(ThrottledTemplateWriterAdapterMode::Bytes {
                adapter: ThrottledTemplateWriterOutputStreamAdapter::new(
                    self.template_name.clone(),
                    Arc::clone(&self.flow_controller),
                    increment,
                ),
                encoder: encoding.new_encoder(),
                pending_bytes: Vec::new(),
            });
        }
        if let Some(ThrottledTemplateWriterAdapterMode::Bytes { adapter, .. }) =
            self.adapter.as_mut()
        {
            adapter.set_output_stream(output_stream);
        }
        Ok(())
    }

    /// 允许下一轮最多写出指定数量的字符或字节。
    /// 对应 Java: `ThrottledTemplateWriter#allow()`。
    pub(crate) fn allow(&mut self, limit: i32) -> Result<(), TemplateOutputException> {
        match self.adapter_mut()? {
            ThrottledTemplateWriterAdapterMode::Characters(adapter) => adapter.allow(limit),
            ThrottledTemplateWriterAdapterMode::Bytes { adapter, .. } => adapter.allow(limit),
        }
    }

    /// 写出 UTF-16 内容，并保持 Java 字符计数或编码后的字节计数。
    /// 对应 Java 语义：`ThrottledTemplateWriter` 的 `write_utf16` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        self.flushable = true;
        match self.adapter_io_mut()? {
            ThrottledTemplateWriterAdapterMode::Characters(adapter) => {
                adapter.write_utf16(characters)
            }
            ThrottledTemplateWriterAdapterMode::Bytes {
                encoder,
                pending_bytes,
                ..
            } => {
                // Java Channels.newWriter 会先保留字符编码缓冲；只有 flush（包括
                // isOverflown/isStopped 内部触发的 flush）才把字节交给限流适配器。
                pending_bytes.extend(Self::encode_utf16(encoder, characters, false));
                Ok(())
            }
        }
    }

    /// 刷新当前底层输出。
    /// 对应 Java: `ThrottledTemplateWriter#flush()`。
    pub(crate) fn flush(&mut self) -> io::Result<()> {
        match self.adapter_io_mut()? {
            ThrottledTemplateWriterAdapterMode::Characters(adapter) => adapter.flush(),
            ThrottledTemplateWriterAdapterMode::Bytes {
                adapter,
                pending_bytes,
                ..
            } => {
                if !pending_bytes.is_empty() {
                    adapter.write_bytes(pending_bytes)?;
                    pending_bytes.clear();
                }
                adapter.flush()
            }
        }
    }

    /// 关闭当前底层输出。
    /// 对应 Java: `ThrottledTemplateWriter#close()`。
    pub(crate) fn close(&mut self) -> io::Result<()> {
        match self.adapter_io_mut()? {
            ThrottledTemplateWriterAdapterMode::Characters(adapter) => adapter.close(),
            ThrottledTemplateWriterAdapterMode::Bytes {
                adapter,
                encoder,
                pending_bytes,
            } => {
                pending_bytes.extend(Self::encode_utf16(encoder, &[], true));
                if !pending_bytes.is_empty() {
                    adapter.write_bytes(pending_bytes)?;
                    pending_bytes.clear();
                }
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
    ) -> Result<&mut ThrottledTemplateWriterAdapterMode, TemplateOutputException> {
        if self.adapter.is_none() {
            return Err(self.mode_error("The throttled processor output has not been initialized."));
        }
        Ok(self.adapter.as_mut().expect("checked above"))
    }

    fn adapter_io_mut(&mut self) -> io::Result<&mut ThrottledTemplateWriterAdapterMode> {
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
            Some(ThrottledTemplateWriterAdapterMode::Characters(adapter)) => {
                Ok(adapter.is_overflown())
            }
            Some(ThrottledTemplateWriterAdapterMode::Bytes { adapter, .. }) => {
                Ok(adapter.is_overflown())
            }
            // Rust 用 Result 承接 Java 未检查异常；错误文本保留 JDK 对 null adapter
            // 调用 isOverflown 的精确诊断。
            None => Err(io::Error::other(
                "Cannot invoke \"org.thymeleaf.engine.ThrottledTemplateWriter$IThrottledTemplateWriterAdapter.isOverflown()\" because \"this.adapter\" is null",
            )),
        }
    }

    fn is_stopped(&mut self) -> io::Result<bool> {
        self.flush_if_needed()?;
        match self.adapter.as_ref() {
            Some(ThrottledTemplateWriterAdapterMode::Characters(adapter)) => {
                Ok(adapter.is_stopped())
            }
            Some(ThrottledTemplateWriterAdapterMode::Bytes { adapter, .. }) => {
                Ok(adapter.is_stopped())
            }
            // Rust 用 Result 承接 Java 未检查异常；错误文本保留 JDK 对 null adapter
            // 调用 isStopped 的精确诊断。
            None => Err(io::Error::other(
                "Cannot invoke \"org.thymeleaf.engine.ThrottledTemplateWriter$IThrottledTemplateWriterAdapter.isStopped()\" because \"this.adapter\" is null",
            )),
        }
    }

    fn get_written_count(&self) -> i32 {
        match self.adapter.as_ref() {
            Some(ThrottledTemplateWriterAdapterMode::Characters(adapter)) => {
                adapter.get_written_count()
            }
            Some(ThrottledTemplateWriterAdapterMode::Bytes { adapter, .. }) => {
                adapter.get_written_count()
            }
            None => panic!(
                "Cannot invoke \"org.thymeleaf.engine.ThrottledTemplateWriter$IThrottledTemplateWriterAdapter.getWrittenCount()\" because \"this.adapter\" is null"
            ),
        }
    }

    fn get_max_overflow_size(&self) -> i32 {
        match self.adapter.as_ref() {
            Some(ThrottledTemplateWriterAdapterMode::Characters(adapter)) => {
                adapter.get_max_overflow_size()
            }
            Some(ThrottledTemplateWriterAdapterMode::Bytes { adapter, .. }) => {
                adapter.get_max_overflow_size()
            }
            None => panic!(
                "Cannot invoke \"org.thymeleaf.engine.ThrottledTemplateWriter$IThrottledTemplateWriterAdapter.getMaxOverflowSize()\" because \"this.adapter\" is null"
            ),
        }
    }

    fn get_overflow_grow_count(&self) -> i32 {
        match self.adapter.as_ref() {
            Some(ThrottledTemplateWriterAdapterMode::Characters(adapter)) => {
                adapter.get_overflow_grow_count()
            }
            Some(ThrottledTemplateWriterAdapterMode::Bytes { adapter, .. }) => {
                adapter.get_overflow_grow_count()
            }
            None => panic!(
                "Cannot invoke \"org.thymeleaf.engine.ThrottledTemplateWriter$IThrottledTemplateWriterAdapter.getOverflowGrowCount()\" because \"this.adapter\" is null"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::{Arc, Mutex};

    use super::super::i_throttled_template_writer_control::IThrottledTemplateWriterControl;
    use super::super::template_flow_controller::TemplateFlowController;
    use super::ThrottledTemplateWriter;
    use crate::util::{Charset, JavaWriter};

    #[test]
    fn character_throttling_state_machine_matches_java_golden() {
        let mut uninitialized = ThrottledTemplateWriter::new(
            "template".to_owned(),
            Arc::new(Mutex::new(TemplateFlowController::new())),
        );
        assert_java_npe_message(
            "uninitializedOverflown",
            uninitialized
                .is_overflown()
                .expect_err("Java null adapter must fail")
                .to_string(),
        );
        assert_java_npe_message(
            "uninitializedStopped",
            uninitialized
                .is_stopped()
                .expect_err("Java null adapter must fail")
                .to_string(),
        );
        assert_java_npe_message(
            "uninitializedWritten",
            panic_message(|| uninitialized.get_written_count()),
        );
        assert_java_npe_message(
            "uninitializedMaxOverflow",
            panic_message(|| uninitialized.get_max_overflow_size()),
        );
        assert_java_npe_message(
            "uninitializedGrowCount",
            panic_message(|| uninitialized.get_overflow_grow_count()),
        );

        let controller = Arc::new(Mutex::new(TemplateFlowController::new()));
        let output = Arc::new(Mutex::new(Vec::new()));
        let mut writer =
            ThrottledTemplateWriter::new("template".to_owned(), Arc::clone(&controller));
        writer
            .set_output_writer(Box::new(RecordingWriter(Arc::clone(&output))))
            .expect("character output initialization");
        assert_golden("initial", &state(&mut writer, &controller, &output));

        writer.allow(2).expect("allow first chunk");
        writer
            .write_utf16(&utf16("abcd"))
            .expect("write first chunk");
        assert_golden("first", &state(&mut writer, &controller, &output));

        writer.allow(1).expect("allow overflow prefix");
        assert_golden("second", &state(&mut writer, &controller, &output));

        writer.allow(i32::MAX).expect("allow unlimited remainder");
        assert_golden("unlimited", &state(&mut writer, &controller, &output));

        writer.allow(0).expect("stop output");
        writer
            .write_utf16(&utf16("ef"))
            .expect("buffer stopped output");
        assert_golden("zero", &state(&mut writer, &controller, &output));

        let byte_controller = Arc::new(Mutex::new(TemplateFlowController::new()));
        let byte_output = Arc::new(Mutex::new(Vec::new()));
        let mut byte_writer =
            ThrottledTemplateWriter::new("template".to_owned(), Arc::clone(&byte_controller));
        byte_writer
            .set_output_stream(
                Box::new(RecordingByteStream(Arc::clone(&byte_output))),
                &Charset::for_name("UTF-8").expect("UTF-8 charset"),
                2,
            )
            .expect("byte output initialization");
        byte_writer.allow(2).expect("allow UTF-8 first chunk");
        byte_writer
            .write_utf16(&utf16("éx"))
            .expect("buffer UTF-8 characters");
        assert_golden(
            "bytesFirst",
            &byte_state(&mut byte_writer, &byte_controller, &byte_output),
        );
        byte_writer.allow(1).expect("allow UTF-8 overflow");
        assert_golden(
            "bytesSecond",
            &byte_state(&mut byte_writer, &byte_controller, &byte_output),
        );

        let mut char_first = ThrottledTemplateWriter::new(
            "template".to_owned(),
            Arc::new(Mutex::new(TemplateFlowController::new())),
        );
        char_first
            .set_output_writer(Box::new(RecordingWriter(Arc::new(Mutex::new(Vec::new())))))
            .expect("character output initialization");
        let char_then_bytes = char_first
            .set_output_stream(
                Box::new(RecordingByteStream(Arc::new(Mutex::new(Vec::new())))),
                &Charset::for_name("UTF-8").expect("UTF-8 charset"),
                1,
            )
            .expect_err("Java locks writer mode after character initialization");
        assert_golden(
            "charThenBytes",
            &format!("TemplateOutputException:{char_then_bytes}"),
        );

        let mut bytes_first = ThrottledTemplateWriter::new(
            "template".to_owned(),
            Arc::new(Mutex::new(TemplateFlowController::new())),
        );
        bytes_first
            .set_output_stream(
                Box::new(RecordingByteStream(Arc::new(Mutex::new(Vec::new())))),
                &Charset::for_name("UTF-8").expect("UTF-8 charset"),
                1,
            )
            .expect("byte output initialization");
        let bytes_then_char = bytes_first
            .set_output_writer(Box::new(RecordingWriter(Arc::new(Mutex::new(Vec::new())))))
            .expect_err("Java locks writer mode after byte initialization");
        assert_golden(
            "bytesThenChar",
            &format!("TemplateOutputException:{bytes_then_char}"),
        );

        let mut failing = ThrottledTemplateWriter::new(
            "template".to_owned(),
            Arc::new(Mutex::new(TemplateFlowController::new())),
        );
        failing
            .set_output_writer(Box::new(FailOnSecondWriteWriter { writes: 0 }))
            .expect("character output initialization");
        failing.allow(2).expect("allow initial output");
        failing
            .write_utf16(&utf16("abc"))
            .expect("first underlying write succeeds");
        let overflow_io = failing
            .allow(1)
            .expect_err("overflow flush must wrap the sink failure");
        assert_golden(
            "overflowIo",
            &format!("TemplateOutputException:{overflow_io}"),
        );

        let bulk_controller = Arc::new(Mutex::new(TemplateFlowController::new()));
        let bulk_output = Arc::new(Mutex::new(Vec::new()));
        let mut bulk =
            ThrottledTemplateWriter::new("template".to_owned(), Arc::clone(&bulk_controller));
        bulk.set_output_writer(Box::new(RecordingWriter(Arc::clone(&bulk_output))))
            .expect("character output initialization");
        bulk.allow(0).expect("stop bulk output");
        bulk.write_utf16(&vec![u16::from(b'a'); 600])
            .expect("buffer first bulk segment");
        bulk.write_utf16(&vec![u16::from(b'b'); 200])
            .expect("buffer second bulk segment");
        assert_golden("bulkBuffered", &bulk_state(&mut bulk, &bulk_output));
        bulk.allow(i32::MAX).expect("drain bulk output");
        assert_golden("bulkDrained", &bulk_state(&mut bulk, &bulk_output));

        let mut resource_failures = ThrottledTemplateWriter::new(
            "template".to_owned(),
            Arc::new(Mutex::new(TemplateFlowController::new())),
        );
        resource_failures
            .set_output_writer(Box::new(FlushCloseFailingWriter))
            .expect("character output initialization");
        let flush_error = resource_failures
            .flush()
            .expect_err("flush errors must propagate without template wrapping");
        assert_golden("flushIo", &format!("IOException:{flush_error}"));
        let close_error = resource_failures
            .close()
            .expect_err("close errors must propagate without template wrapping");
        assert_golden("closeIo", &format!("IOException:{close_error}"));

        let overload_controller = Arc::new(Mutex::new(TemplateFlowController::new()));
        let overload_output = Arc::new(Mutex::new(Vec::new()));
        let mut overloads =
            ThrottledTemplateWriter::new("template".to_owned(), Arc::clone(&overload_controller));
        overloads
            .set_output_writer(Box::new(RecordingWriter(Arc::clone(&overload_output))))
            .expect("character output initialization");
        overloads.allow(i32::MAX).expect("allow unbounded output");
        // Rust 的单一 UTF-16 slice 写入入口分别承接 Java Writer 的 char、String
        // offset/len、char[] 和 char[] offset/len 四个重载。
        overloads
            .write_utf16(&utf16("x"))
            .expect("single Java char equivalent");
        overloads
            .write_utf16(&utf16("abcdef")[1..4])
            .expect("String offset/len equivalent");
        overloads
            .write_utf16(&utf16("qrs"))
            .expect("char array equivalent");
        overloads
            .write_utf16(&utf16("qrs")[1..3])
            .expect("char array offset/len equivalent");
        let overload_text =
            String::from_utf16(&overload_output.lock().expect("overload output lock"))
                .expect("test output must be valid UTF-16");
        assert_golden(
            "overloads",
            &format!("{overload_text},{}", overloads.get_written_count()),
        );
    }

    struct RecordingWriter(Arc<Mutex<Vec<u16>>>);

    impl JavaWriter for RecordingWriter {
        fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
            self.0
                .lock()
                .expect("recording writer lock")
                .extend_from_slice(characters);
            Ok(())
        }
    }

    struct FailOnSecondWriteWriter {
        writes: usize,
    }

    impl JavaWriter for FailOnSecondWriteWriter {
        fn write_utf16(&mut self, _characters: &[u16]) -> io::Result<()> {
            self.writes += 1;
            if self.writes > 1 {
                return Err(io::Error::other("overflow sink failure"));
            }
            Ok(())
        }
    }

    struct FlushCloseFailingWriter;

    impl JavaWriter for FlushCloseFailingWriter {
        fn write_utf16(&mut self, _characters: &[u16]) -> io::Result<()> {
            Ok(())
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("flush sink failure"))
        }

        fn close(&mut self) -> io::Result<()> {
            Err(io::Error::other("close sink failure"))
        }
    }

    struct RecordingByteStream(Arc<Mutex<Vec<u8>>>);

    impl Write for RecordingByteStream {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0
                .lock()
                .expect("recording byte stream lock")
                .extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn state(
        writer: &mut ThrottledTemplateWriter,
        controller: &Arc<Mutex<TemplateFlowController>>,
        output: &Arc<Mutex<Vec<u16>>>,
    ) -> String {
        let content = String::from_utf16(&output.lock().expect("output lock"))
            .expect("test output must be valid UTF-16");
        let control: &mut dyn IThrottledTemplateWriterControl = writer;
        let written = control.get_written_count();
        let overflown = control.is_overflown().expect("is overflown");
        let stopped = control.is_stopped().expect("is stopped");
        let controller_stopped = controller
            .lock()
            .expect("flow controller lock")
            .stop_processing;
        format!(
            "{content},{written},{overflown},{stopped},{controller_stopped},{},{}",
            control.get_max_overflow_size(),
            control.get_overflow_grow_count()
        )
    }

    fn assert_golden(key: &str, actual: &str) {
        let expected = include_str!("../../tests/fixtures/throttled_template_writer_golden.txt")
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}=")))
            .expect("Java Golden record");
        assert_eq!(actual, expected, "Java Golden key {key}");
    }

    fn assert_java_npe_message(key: &str, actual: String) {
        let expected = include_str!("../../tests/fixtures/throttled_template_writer_golden.txt")
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key}=")))
            .and_then(|record| record.split_once(':'))
            .map(|(_, message)| message)
            .expect("Java Golden NPE record");
        assert_eq!(actual, expected, "Java Golden key {key}");
    }

    fn panic_message<T>(operation: impl FnOnce() -> T) -> String {
        let payload = match catch_unwind(AssertUnwindSafe(operation)) {
            Ok(_) => panic!("Java null adapter getter must panic"),
            Err(payload) => payload,
        };
        match payload.downcast::<String>() {
            Ok(message) => *message,
            Err(payload) => match payload.downcast::<&'static str>() {
                Ok(message) => (*message).to_owned(),
                Err(_) => panic!("unexpected panic payload type"),
            },
        }
    }

    fn byte_state(
        writer: &mut ThrottledTemplateWriter,
        controller: &Arc<Mutex<TemplateFlowController>>,
        output: &Arc<Mutex<Vec<u8>>>,
    ) -> String {
        let content = output
            .lock()
            .expect("byte output lock")
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let control: &mut dyn IThrottledTemplateWriterControl = writer;
        let written = control.get_written_count();
        let overflown = control.is_overflown().expect("is overflown");
        let stopped = control.is_stopped().expect("is stopped");
        let controller_stopped = controller
            .lock()
            .expect("flow controller lock")
            .stop_processing;
        format!(
            "{content},{written},{overflown},{stopped},{controller_stopped},{},{}",
            control.get_max_overflow_size(),
            control.get_overflow_grow_count()
        )
    }

    fn bulk_state(writer: &mut ThrottledTemplateWriter, output: &Arc<Mutex<Vec<u16>>>) -> String {
        let output_len = output.lock().expect("bulk output lock").len();
        let control: &mut dyn IThrottledTemplateWriterControl = writer;
        let written = control.get_written_count();
        let overflown = control.is_overflown().expect("is overflown");
        let stopped = control.is_stopped().expect("is stopped");
        format!(
            "{output_len},{written},{overflown},{stopped},{},{}",
            control.get_max_overflow_size(),
            control.get_overflow_grow_count()
        )
    }

    fn utf16(value: &str) -> Vec<u16> {
        value.encode_utf16().collect()
    }
}
