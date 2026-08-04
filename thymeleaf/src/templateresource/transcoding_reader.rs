use std::io::{self, Read};

use super::charset_decoder::CharsetDecoder;

const INPUT_BUFFER_SIZE: usize = 8 * 1024;

/// 把 Java 字符流语义适配为 Rust UTF-8 `Read` 的增量读取器。
///
/// 对应 Java: `java.io.InputStreamReader`，由 Thymeleaf 资源对象持有而不对外发布。
pub(crate) struct TranscodingReader {
    input: Box<dyn Read>,
    decoder: CharsetDecoder,
    decoded: Vec<u8>,
    decoded_position: usize,
    finished: bool,
}

impl TranscodingReader {
    /// 使用已打开输入流和已选择字符集创建增量读取器。
    ///
    /// 对应 Java: `java.io.InputStreamReader#InputStreamReader(InputStream,Charset)`。
    ///
    /// # 参数
    /// - `input`：原始字节输入流。
    /// - `decoder`：按 Java 字符集规则创建的解码器。
    ///
    /// # 返回值
    /// 返回从输入流当前位置开始读取的新适配器。
    pub(crate) fn new(input: Box<dyn Read>, decoder: CharsetDecoder) -> Self {
        Self {
            input,
            decoder,
            decoded: Vec::new(),
            decoded_position: 0,
            finished: false,
        }
    }
}

impl Read for TranscodingReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }

        loop {
            if self.decoded_position < self.decoded.len() {
                let remaining = &self.decoded[self.decoded_position..];
                let length = remaining.len().min(output.len());
                output[..length].copy_from_slice(&remaining[..length]);
                self.decoded_position += length;
                return Ok(length);
            }
            if self.finished {
                return Ok(0);
            }

            let mut input_buffer = [0_u8; INPUT_BUFFER_SIZE];
            let read = self.input.read(&mut input_buffer)?;
            let last = read == 0;
            self.decoded = self.decoder.decode_chunk(&input_buffer[..read], last);
            self.decoded_position = 0;
            self.finished = last;
        }
    }
}
