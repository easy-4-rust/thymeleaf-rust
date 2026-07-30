//! Spring View/Reactive 测试迁入中立 Rust Web 合同后的合并证据。

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use futures_executor::block_on;
use futures_util::StreamExt;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use thymeleaf::context::{Context, IContext};
use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::util::Charset;
use thymeleaf::web::{RenderedTemplateBody, ThymeleafRenderer};
use thymeleaf::{ITemplateEngine, ITemplateResolver, TemplateEngine, TemplateMode, TemplateSpec};

fn configured_engine() -> Arc<TemplateEngine> {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(TemplateMode::HTML);
    let engine = Arc::new(TemplateEngine::new());
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("template resolver must be configurable before rendering");
    engine
}

fn renderer(chunk_size: i32) -> ThymeleafRenderer {
    let engine = configured_engine();
    ThymeleafRenderer::new(engine as Arc<dyn ITemplateEngine>).with_chunk_size(chunk_size)
}

#[derive(Clone, Default)]
struct SharedOutput(Arc<Mutex<Vec<u8>>>);

impl Write for SharedOutput {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("output lock")
            .extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn neutral_throttled_processor_reaches_completion_and_preserves_output() {
    let template = "<p>throttled</p>";
    let template_spec = TemplateSpec::with_template_mode(Some(template), Some(TemplateMode::HTML))
        .expect("template spec");
    let engine = configured_engine();
    let mut processor = engine
        .process_throttled(&template_spec, &Context::new())
        .expect("throttled processor");
    let output = SharedOutput::default();
    let charset = Charset::for_name("UTF-8").expect("UTF-8");

    for _ in 0..32 {
        if processor.is_finished() {
            break;
        }
        processor
            .process_output_stream(8 * 1024, Box::new(output.clone()), &charset)
            .expect("throttled process");
    }

    assert!(
        processor.is_finished(),
        "processor must emit a completion signal"
    );
    assert_eq!(
        output.0.lock().expect("output lock").as_slice(),
        template.as_bytes()
    );
}

#[test]
fn neutral_full_view_preserves_body_content_type_and_length() {
    let template = "<p>Spring-neutral view</p>";
    let template_spec = TemplateSpec::with_template_mode(Some(template), Some(TemplateMode::HTML))
        .expect("template spec");
    let rendered = renderer(8)
        .render_full(&template_spec, &Context::new())
        .expect("full render");
    let (_, headers, body) = rendered.into_parts();

    assert_eq!(
        headers
            .get(CONTENT_TYPE)
            .expect("content type")
            .to_str()
            .expect("ASCII content type"),
        "text/html;charset=UTF-8"
    );
    assert_eq!(
        headers
            .get(CONTENT_LENGTH)
            .expect("content length")
            .to_str()
            .expect("ASCII content length"),
        template.len().to_string()
    );
    let RenderedTemplateBody::Full(body) = body else {
        panic!("full rendering must produce a full body");
    };
    assert_eq!(body.as_ref(), template.as_bytes());
}

#[test]
fn neutral_reactive_view_preserves_chunked_output_and_reports_charset_errors() {
    let template = "<p>reactive</p>";
    let template_spec = TemplateSpec::with_template_mode(Some(template), Some(TemplateMode::HTML))
        .expect("template spec");
    let context = Arc::new(Context::new()) as Arc<dyn IContext>;
    let rendered = renderer(2)
        .render_stream(template_spec, context)
        .expect("stream render");
    let (_, headers, body) = rendered.into_parts();

    assert!(headers.get(CONTENT_LENGTH).is_none());
    let RenderedTemplateBody::Stream(stream) = body else {
        panic!("stream rendering must produce a stream body");
    };
    let frames = block_on(stream.collect::<Vec<_>>());
    assert!(frames.len() > 1, "chunk size two must split the response");
    let output = frames
        .into_iter()
        .flat_map(|frame| {
            frame
                .expect("render frame")
                .into_data()
                .expect("data frame")
        })
        .collect::<Vec<_>>();
    assert_eq!(output, template.as_bytes());

    let invalid_charset = TemplateSpec::with_output_content_type(
        Some(template),
        Some("text/html;charset=NOT-A-CHARSET"),
    )
    .expect("template spec accepts MIME metadata before rendering");
    let error = match renderer(2).render_full(&invalid_charset, &Context::new()) {
        Ok(_) => panic!("unknown charset must fail before producing a response"),
        Err(error) => error,
    };
    assert!(
        error.to_string().to_ascii_lowercase().contains("charset"),
        "charset failure must remain observable"
    );
}
