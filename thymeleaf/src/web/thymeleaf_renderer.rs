use std::io;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context as TaskContext, Poll};
use std::time::Duration;

use bytes::Bytes;
use encoding_rs::{CoderResult, Encoding};
use futures_channel::mpsc::{Receiver, Sender, channel};
use futures_core::Stream;
use futures_executor::block_on;
use futures_util::SinkExt;
use http::HeaderMap;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE, HeaderValue};
use http_body::Frame;

use crate::context::IContext;
use crate::engine::{DataDrivenTemplateIterator, DataDrivenTemplateSignal};
use crate::expression::TemplateValue;
use crate::util::{Charset, ContentTypeUtils, JavaString};
use crate::{IEngineConfiguration, ITemplateEngine, TemplateMode, TemplateSpec};

use super::{RenderError, RenderedTemplate, RenderedTemplateBody};

const DEFAULT_CHUNK_SIZE: i32 = 8 * 1024;

/// 把框架中立 Thymeleaf Engine 转换为统一 HTTP 渲染结果。
///
/// 完整渲染直接返回 `Bytes`；流式渲染在线程内创建并驱动 Java 语义节流处理器，
/// 只把 `http-body` 数据帧跨线程发送，避免把请求级非并发处理状态暴露给宿主。
pub struct ThymeleafRenderer {
    engine: Arc<dyn ITemplateEngine>,
    chunk_size: i32,
}

impl ThymeleafRenderer {
    /// 使用默认 8 KiB 流式分块大小创建渲染器。
    #[must_use]
    pub fn new(engine: Arc<dyn ITemplateEngine>) -> Self {
        Self {
            engine,
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }

    /// 设置每次节流处理允许写出的最大字节数。
    ///
    /// 小于一的值会归一化为一，避免产生不能推进处理状态的空循环。
    #[must_use]
    pub fn with_chunk_size(mut self, chunk_size: i32) -> Self {
        self.chunk_size = chunk_size.max(1);
        self
    }

    /// 完整渲染模板，并按响应 `Content-Type` 的字符集编码 HTTP Body。
    pub fn render_full(
        &self,
        template_spec: &TemplateSpec,
        context: &dyn IContext,
    ) -> Result<RenderedTemplate, RenderError> {
        let output = self
            .engine
            .process(template_spec, context)
            .map_err(|error| RenderError::new(error.to_string()))?;
        let mut metadata = render_metadata(template_spec, self.engine.as_ref())?;
        let bytes = encode_java_string(&output, &metadata.charset)?;
        metadata.headers.insert(
            CONTENT_LENGTH,
            HeaderValue::from_str(&bytes.len().to_string())
                .map_err(|error| RenderError::new(error.to_string()))?,
        );
        Ok(RenderedTemplate::new(
            metadata.headers,
            RenderedTemplateBody::Full(bytes),
        ))
    }

    /// 在后台线程中按 Thymeleaf 节流语义生成 HTTP 数据帧。
    ///
    /// `context` 必须由调用方以 `Arc` 交出共享生命周期；处理器只在工作线程内部
    /// 创建和使用，因此仍遵守 Java “同一处理器不得并发 process” 的合同。
    pub fn render_stream(
        &self,
        template_spec: TemplateSpec,
        context: Arc<dyn IContext>,
    ) -> Result<RenderedTemplate, RenderError> {
        let metadata = render_metadata(&template_spec, self.engine.as_ref())?;
        let engine = Arc::clone(&self.engine);
        let chunk_size = self.chunk_size;
        let charset = metadata.charset;
        // 只允许一个待消费帧，消费端变慢时工作线程会停在发送处，形成真正背压。
        let (sender, receiver) = channel(1);
        std::thread::Builder::new()
            .name("thymeleaf-render".to_owned())
            .spawn(move || {
                render_chunks(
                    engine,
                    template_spec,
                    context,
                    chunk_size,
                    charset,
                    None,
                    sender,
                );
            })
            .map_err(|error| RenderError::new(error.to_string()))?;
        Ok(RenderedTemplate::new(
            metadata.headers,
            RenderedTemplateBody::Stream(Box::pin(RenderFrameStream { receiver })),
        ))
    }

    /// 渲染由宿主持续喂入数据的背压响应流。
    ///
    /// `data_driven_iterator` 必须与模板 Context 中的迭代值来自同一次
    /// [`DataDrivenTemplateIterator::shared_template_value`] 调用。工作线程会绑定
    /// Thymeleaf Writer 控制器，并在队列暂空时休眠，直到调用方喂入数据或标记结束。
    pub fn render_data_stream(
        &self,
        template_spec: TemplateSpec,
        context: Arc<dyn IContext>,
        data_driven_iterator: Arc<Mutex<DataDrivenTemplateIterator<Arc<TemplateValue>>>>,
    ) -> Result<RenderedTemplate, RenderError> {
        let metadata = render_metadata(&template_spec, self.engine.as_ref())?;
        let engine = Arc::clone(&self.engine);
        let chunk_size = self.chunk_size;
        let charset = metadata.charset;
        let (sender, receiver) = channel(1);
        std::thread::Builder::new()
            .name("thymeleaf-data-render".to_owned())
            .spawn(move || {
                render_chunks(
                    engine,
                    template_spec,
                    context,
                    chunk_size,
                    charset,
                    Some(data_driven_iterator),
                    sender,
                );
            })
            .map_err(|error| RenderError::new(error.to_string()))?;
        Ok(RenderedTemplate::new(
            metadata.headers,
            RenderedTemplateBody::Stream(Box::pin(RenderFrameStream { receiver })),
        ))
    }
}

fn render_chunks(
    engine: Arc<dyn ITemplateEngine>,
    template_spec: TemplateSpec,
    context: Arc<dyn IContext>,
    chunk_size: i32,
    charset: Charset,
    data_driven_iterator: Option<Arc<Mutex<DataDrivenTemplateIterator<Arc<TemplateValue>>>>>,
    mut sender: Sender<Result<Frame<Bytes>, RenderError>>,
) {
    let mut processor = match engine.process_throttled(&template_spec, context.as_ref()) {
        Ok(processor) => processor,
        Err(error) => {
            let _ = send_render_item(&mut sender, Err(RenderError::new(error.to_string())));
            return;
        }
    };
    let data_signal = data_driven_iterator.as_ref().map(|iterator| {
        let mut iterator = iterator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        iterator.set_writer_control(processor.get_throttled_template_writer_control());
        iterator.get_signal()
    });
    while !processor.is_finished() {
        let signal_revision = data_signal.as_ref().map(DataDrivenTemplateSignal::revision);
        let output_stream = ChunkOutputStream::new(sender.clone());
        match processor.process_output_stream(chunk_size, Box::new(output_stream), &charset) {
            Ok(0) if !processor.is_finished() => {
                if let (Some(signal), Some(revision)) = (data_signal.as_ref(), signal_revision) {
                    while !signal.wait_for_change(revision, Duration::from_millis(100)) {
                        if sender.is_closed() {
                            return;
                        }
                    }
                    continue;
                }
                let _ = send_render_item(
                    &mut sender,
                    Err(RenderError::new(
                        "Throttled rendering paused without producing output; data-driven rendering \
                     must be driven through its iterator control",
                    )),
                );
                return;
            }
            Ok(_) => {}
            Err(error) => {
                let _ = send_render_item(&mut sender, Err(RenderError::new(error.to_string())));
                return;
            }
        }
    }
    // 节流处理器会保留最后一个 OutputStream，其中仍持有 Sender 克隆。仅依赖析构
    // 顺序会使接收端在已收到完整正文后继续等待 EOF；显式关闭通道，确保所有宿主
    // 的响应流在模板完成时立即结束。对应 Spring Reactive 测试对完成信号的要求。
    sender.close_channel();
}

struct RenderMetadata {
    headers: HeaderMap,
    charset: Charset,
}

fn render_metadata(
    template_spec: &TemplateSpec,
    engine: &dyn ITemplateEngine,
) -> Result<RenderMetadata, RenderError> {
    // 初始化发生在工作开始前，使配置错误同步返回给框架 handler。
    let configuration: Arc<dyn IEngineConfiguration> = engine
        .get_configuration()
        .map_err(|error| RenderError::new(error.to_string()))?;
    drop(configuration);
    let explicit_content_type = template_spec.get_output_content_type();
    let charset = explicit_content_type
        .and_then(|content_type| content_type.parse::<mime::Mime>().ok())
        .and_then(|content_type| {
            content_type
                .get_param(mime::CHARSET)
                .map(|value| value.as_str().to_owned())
        })
        .map_or_else(
            || Charset::for_name("UTF-8"),
            |charset_name| Charset::for_name(&charset_name),
        )
        .map_err(|error| RenderError::new(error.to_string()))?;
    let content_type = match explicit_content_type {
        Some(content_type) => {
            ContentTypeUtils::combine_content_type_and_charset(Some(content_type), Some(&charset))
                .map_err(|error| RenderError::new(error.to_string()))?
                .ok_or_else(|| RenderError::new("output Content-Type cannot be blank"))?
        }
        None => ContentTypeUtils::compute_content_type_for_template_name(
            Some(template_spec.get_template()),
            Some(&charset),
        )
        .unwrap_or_else(|| default_content_type(template_spec.get_template_mode(), &charset)),
    };
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(&content_type)
            .map_err(|error| RenderError::new(error.to_string()))?,
    );
    Ok(RenderMetadata { headers, charset })
}

fn default_content_type(template_mode: Option<TemplateMode>, charset: &Charset) -> String {
    let mime_type = match template_mode.unwrap_or(TemplateMode::HTML) {
        TemplateMode::HTML => "text/html",
        TemplateMode::XML => "application/xml",
        TemplateMode::TEXT => "text/plain",
        TemplateMode::JAVASCRIPT => "application/javascript",
        TemplateMode::CSS => "text/css",
        TemplateMode::RAW => "application/octet-stream",
    };
    format!("{mime_type};charset={charset}")
}

fn encode_java_string(value: &JavaString, charset: &Charset) -> Result<Bytes, RenderError> {
    let encoding = Encoding::for_label(charset.name().as_bytes())
        .ok_or_else(|| RenderError::new(format!("Unsupported charset: {charset}")))?;
    let mut encoder = encoding.new_encoder();
    let mut source_offset = 0;
    let mut output = Vec::with_capacity(value.len().saturating_mul(4).saturating_add(32));
    loop {
        let mut buffer = [0_u8; 4096];
        let (result, read, written, _) =
            encoder.encode_from_utf16(&value.as_utf16()[source_offset..], &mut buffer, true);
        output.extend_from_slice(&buffer[..written]);
        source_offset += read;
        if result == CoderResult::InputEmpty {
            break;
        }
    }
    Ok(Bytes::from(output))
}

struct ChunkOutputStream {
    sender: Sender<Result<Frame<Bytes>, RenderError>>,
    bytes: Vec<u8>,
}

impl ChunkOutputStream {
    fn new(sender: Sender<Result<Frame<Bytes>, RenderError>>) -> Self {
        Self {
            sender,
            bytes: Vec::new(),
        }
    }

    fn send_chunk(&mut self) -> io::Result<()> {
        if self.bytes.is_empty() {
            return Ok(());
        }
        let bytes = Bytes::from(std::mem::take(&mut self.bytes));
        send_render_item(&mut self.sender, Ok(Frame::data(bytes)))
    }
}

impl io::Write for ChunkOutputStream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.send_chunk()
    }
}

impl Drop for ChunkOutputStream {
    fn drop(&mut self) {
        let _ = self.send_chunk();
    }
}

struct RenderFrameStream {
    receiver: Receiver<Result<Frame<Bytes>, RenderError>>,
}

impl Stream for RenderFrameStream {
    type Item = Result<Frame<Bytes>, RenderError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        context: &mut TaskContext<'_>,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.receiver).poll_next(context)
    }
}

fn send_render_item(
    sender: &mut Sender<Result<Frame<Bytes>, RenderError>>,
    item: Result<Frame<Bytes>, RenderError>,
) -> io::Result<()> {
    block_on(sender.send(item))
        .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "render stream receiver closed"))
}
