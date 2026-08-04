//! 引擎过程对象差分证据。
//!
//! 覆盖对象（对象表编号）：`TemplateManager`（120）、`TemplateModelController`
//! （122，含 `SkipBody`）、`OutputTemplateHandler`（102）、
//! `ThrottledTemplateProcessor`（130）。
//!
//! 上游无这四个对象的独立 JUnit 测试：`TemplateManager` 与
//! `TemplateModelController` 由 `TemplateEngine` 每次 `process` 驱动
//! （corpus 2,595 个 `.thtest` 期望即为它们全链路的 Java 输出），
//! `ThrottledTemplateProcessor` 由 `ThrottledWebTestExecutorArgumentsProvider`
//! 以 7 档节流步长驱动 corpus 执行。本文件给出直接对象级证据：
//!
//! 1. `TemplateManager`：模板解析/缓存/处理器链组装 —— 重复 process、
//!    多模板交替、处理器链输出正确性；
//! 2. `TemplateModelController`：模型层级与 skip 语义 —— 经
//!    `th:if`/`th:remove` 等处理器验证事件流；
//! 3. `OutputTemplateHandler`：直接事件写入（与解析器批
//!    `HtmlBlockSelectorMarkupHandlerTest` 的终端输出路径一致）；
//! 4. `ThrottledTemplateProcessor`：分块/全量/字节输出与普通 process
//!    输出逐字节一致（Java 节流执行器保证同一输出，这里把三条路径
//!    拼接到同一上游期望）。

use std::io;
use std::sync::{Arc, Mutex};

use thymeleaf::context::Context;
use thymeleaf::engine::{ITemplateHandler, OutputTemplateHandler, Text};
use thymeleaf::expression::TemplateValue;
use thymeleaf::model::IText;
use thymeleaf::util::{Charset, JavaWriter, Utf16String};
use thymeleaf::{ITemplateResolver, TemplateEngine};

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

fn string_engine() -> TemplateEngine {
    let engine = TemplateEngine::new();
    let resolver = thymeleaf::templateresolver::StringTemplateResolver::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("set resolver");
    engine
}

fn context_with(variables: &[(&str, &str)]) -> Context {
    let context = Context::new();
    for (name, value) in variables {
        context.set_variable(
            Some(js(name)),
            Some(Arc::new(TemplateValue::string(js(value)))),
        );
    }
    context
}

/// 捕获 UTF-16 输出的 Writer（对应 Java StringWriter）。
#[derive(Clone)]
struct CapturedWriter {
    buffer: Arc<Mutex<Vec<u16>>>,
}

impl JavaWriter for CapturedWriter {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        self.buffer
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .extend_from_slice(characters);
        Ok(())
    }
}

/// 持有 UTF-8 字节的 Writer（对应 Java OutputStream 写出）。
#[derive(Clone)]
struct ByteSink {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl io::Write for ByteSink {
    fn write(&mut self, chunk: &[u8]) -> io::Result<usize> {
        self.buffer
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .extend_from_slice(chunk);
        Ok(chunk.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn writer_text(buffer: &Arc<Mutex<Vec<u16>>>) -> String {
    String::from_utf16_lossy(&buffer.lock().unwrap_or_else(|error| error.into_inner()))
}

// ===========================================================================
// 1. TemplateManager：解析缓存与处理器链组装
// ===========================================================================

/// 同一模板重复 process（第二次命中解析缓存）、不同变量、多模板交替，
/// 输出与 StandardDialect 语义一致（corpus 已对同类输出做 Java 期望差分）。
#[test]
fn template_manager_reprocess_and_cache_paths() {
    let engine = string_engine();

    // 第一次 process：解析并写入缓存
    let first = engine
        .process_template(
            "<p th:text='${name}'>a</p>",
            &context_with(&[("name", "one")]),
        )
        .expect("first process")
        .to_string_lossy();
    assert_eq!("<p>one</p>", first);

    // 第二次 process：命中缓存，输出仍按新变量重新计算
    let second = engine
        .process_template(
            "<p th:text='${name}'>a</p>",
            &context_with(&[("name", "two")]),
        )
        .expect("cached process")
        .to_string_lossy();
    assert_eq!("<p>two</p>", second);

    // 多模板交替：不同模板名互不干扰
    let other = engine
        .process_template(
            "<span th:text='${name}'>x</span>",
            &context_with(&[("name", "three")]),
        )
        .expect("other template")
        .to_string_lossy();
    assert_eq!("<span>three</span>", other);

    let third = engine
        .process_template(
            "<p th:text='${name}'>a</p>",
            &context_with(&[("name", "four")]),
        )
        .expect("third process")
        .to_string_lossy();
    assert_eq!("<p>four</p>", third);
}

// ===========================================================================
// 2. TemplateModelController：模型层级与 skip 语义
// ===========================================================================

/// `th:if`（条件分支模型层）与 `th:remove`（移除模型）经
/// TemplateModelController 的事件流输出。
#[test]
fn template_model_controller_skip_and_conditional_flows() {
    let engine = string_engine();

    // th:if=false 时 body 跳过（SkipBody 语义）
    let skipped = engine
        .process_template(
            "<div th:if='${show}'><p>hidden</p></div>",
            &context_with(&[("show", "false")]),
        )
        .expect("skipped body")
        .to_string_lossy();
    assert_eq!("", skipped);

    let shown = engine
        .process_template(
            "<div th:if='${show}'><p th:text='${name}'>hidden</p></div>",
            &context_with(&[("show", "true"), ("name", "visible")]),
        )
        .expect("shown body")
        .to_string_lossy();
    assert_eq!("<div><p>visible</p></div>", shown);

    // th:remove="tag"：移除当前元素但保留 body
    let removed = engine
        .process_template("<div th:remove='tag'><p>kept</p></div>", &Context::new())
        .expect("removed tag")
        .to_string_lossy();
    assert_eq!("<p>kept</p>", removed);
}

// ===========================================================================
// 3. OutputTemplateHandler：直接事件写入
// ===========================================================================

/// 构造 `Text` 事件直接喂给 `OutputTemplateHandler`，验证终端写出
/// （与 Java `OutputTemplateHandler(Writer)` 的直接使用一致）。
#[test]
fn output_template_handler_writes_text_events() {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = CapturedWriter {
        buffer: buffer.clone(),
    };
    let mut handler = OutputTemplateHandler::new(Box::new(writer));

    let text: Arc<dyn IText> = Arc::new(Text::new(Some(Arc::new(Utf16String::from_rust_str(
        "hello ",
    )))));
    let next: Arc<dyn IText> = Arc::new(Text::new(Some(Arc::new(Utf16String::from_rust_str(
        "world",
    )))));
    handler.handle_text(text).expect("write text");
    handler.handle_text(next).expect("write text");
    assert_eq!("hello world", writer_text(&buffer));

    // 空文本事件写出空串
    let empty: Arc<dyn IText> = Arc::new(Text::new(Some(Arc::new(Utf16String::from_rust_str("")))));
    handler.handle_text(empty).expect("write empty");
    assert_eq!("hello world", writer_text(&buffer));
}

// ===========================================================================
// 4. ThrottledTemplateProcessor：分块/全量/字节与普通 process 一致
// ===========================================================================

const THROTTLED_TEMPLATES: &[(&str, &[(&str, &str)])] = &[
    ("<p th:text='${name}'>a</p>", &[("name", "throttled")]),
    (
        "<ul><li th:each='i : ${items}' th:text='${i}'>x</li></ul>",
        &[("items", "a|b|c")],
    ),
    ("<div th:if='${show}'>visible</div>", &[("show", "true")]),
];

/// 普通 process、`process_all_writer` 与 `process_writer` 分块三条路径
/// 输出逐字节一致（对应 Java 节流执行器以 7 档步长跑出与普通执行相同输出）。
#[test]
fn throttled_processing_matches_plain_processing() {
    let engine = string_engine();
    for (template, variables) in THROTTLED_TEMPLATES {
        let context = context_with(variables);

        let plain = engine
            .process_template(template, &context)
            .expect("plain process")
            .to_string_lossy();

        let mut all_processor = engine
            .process_throttled_template(template, &context)
            .expect("throttled processor");
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = CapturedWriter {
            buffer: buffer.clone(),
        };
        let written = all_processor
            .process_all_writer(Box::new(writer))
            .expect("process all");
        assert_eq!(plain, writer_text(&buffer));
        assert_eq!(plain.len() as i32, written, "process_all 返回写出单元数");

        // 分块路径：每步 1 个 UTF-16 单元直到完成
        let mut step_processor = engine
            .process_throttled_template(template, &context)
            .expect("step throttled processor");
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = CapturedWriter {
            buffer: buffer.clone(),
        };
        let mut steps = 0;
        let max_steps = plain.len() as i64 * 2 + 64;
        while !step_processor.is_finished() {
            let written = step_processor
                .process_writer(1, Box::new(writer.clone()))
                .expect("step process");
            assert!(written <= 1, "每步最多写出 1 个单元");
            steps += 1;
            assert!(
                steps < max_steps,
                "分块步进超过安全上限（模板 {template:?}），疑似死循环"
            );
        }
        assert_eq!(plain, writer_text(&buffer));
        assert!(
            steps >= plain.len() as i64,
            "分块步数（{steps}）不得少于输出长度（{}）",
            plain.len()
        );
    }
}

/// `process_output_stream`（UTF-8 字节）与 `process_writer`（UTF-16）
/// 输出编码一致。
#[test]
fn throttled_bytes_matches_chars_encoding() {
    let engine = string_engine();
    let template = "<p th:text='${name}'>a</p>";
    let context = context_with(&[("name", "throttled")]);

    let mut char_processor = engine
        .process_throttled_template(template, &context)
        .expect("char processor");
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let written_chars = char_processor
        .process_all_writer(Box::new(CapturedWriter {
            buffer: buffer.clone(),
        }))
        .expect("chars");
    let text = writer_text(&buffer);
    assert_eq!("<p>throttled</p>", text);
    assert_eq!(text.len() as i32, written_chars);

    let mut byte_processor = engine
        .process_throttled_template(template, &context)
        .expect("byte processor");
    let charset = Charset::for_name("UTF-8").expect("utf-8 charset");
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let written_bytes = byte_processor
        .process_output_stream(
            i32::MAX,
            Box::new(ByteSink {
                buffer: bytes.clone(),
            }),
            &charset,
        )
        .expect("bytes");
    assert_eq!(
        text.as_bytes(),
        bytes
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_slice()
    );
    assert_eq!(
        text.len() as i32,
        written_bytes,
        "UTF-8 单字节字符与 UTF-16 单元数一致"
    );
}

/// 节流处理器观察面：processor identifier、template spec 与 writer 控制。
#[test]
fn throttled_processor_observers() {
    let engine = string_engine();
    let template = "<p>obs</p>";
    let context = Context::new();

    let mut processor = engine
        .process_throttled_template(template, &context)
        .expect("throttled processor");

    assert!(
        !processor.get_processor_identifier().is_empty(),
        "processor identifier 必须非空"
    );
    assert_eq!(
        template,
        processor.get_template_spec().get_template(),
        "template spec 必须保留模板名"
    );
    assert!(!processor.is_finished(), "未处理时不得完成");

    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = CapturedWriter {
        buffer: buffer.clone(),
    };
    processor
        .process_all_writer(Box::new(writer))
        .expect("process all");
    assert_eq!("<p>obs</p>", writer_text(&buffer));

    // 对应 Java：writer 初始化（首次写出）后 isOverflown/isStopped 才可查询；
    // 未初始化时 Java 抛 adapter null NPE（Rust 保留同样错误）。
    let mut control = processor.get_throttled_template_writer_control();
    assert!(
        !control.is_overflown().expect("overflow check"),
        "无上限写出后不得溢出"
    );
    assert!(
        !control.is_stopped().expect("stopped check"),
        "无上限写出后不得停止"
    );
    assert!(processor.is_finished(), "处理后必须完成");
    assert_eq!(
        0,
        processor
            .process_all_writer(Box::new(CapturedWriter {
                buffer: Arc::new(Mutex::new(Vec::new()))
            }))
            .expect("finished process returns zero"),
        "完成后继续处理返回零"
    );
}
