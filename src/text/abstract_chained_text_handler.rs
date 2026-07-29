use std::error::Error;
use std::fmt::{Display, Formatter};
use std::panic::panic_any;

use super::{ITextHandler, TextParseException};
use crate::util::JavaString;

const HANDLER_CLASS: &str = "org.thymeleaf.templateparser.text.ITextHandler";

/// handler 链的 Java 运行时异常适配。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.AbstractChainedTextHandler` 在 `next == null`
/// 时由 JVM 调用指令产生的 `NullPointerException`。
///
/// checked [`TextParseException`] 仍由 [`ITextHandler`] 的 `Result` 通道原对象返回；
/// 此类型只作为 panic payload 保留 Java 未检查异常的类名和增强 NPE 消息。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainedTextHandlerRuntimeError {
    method_signature: &'static str,
}

impl ChainedTextHandlerRuntimeError {
    fn null_next(method_signature: &'static str) -> Self {
        Self { method_signature }
    }

    /// 返回对应 Java 异常全限定名。
    ///
    /// # 返回
    /// 固定为 `java.lang.NullPointerException`。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        "java.lang.NullPointerException"
    }

    /// 返回 Java 17 增强 NPE 的 UTF-16 消息。
    ///
    /// # 返回
    /// 精确保留失败回调签名及字段表达式 `this.next`。
    #[must_use]
    pub fn java_message(&self) -> JavaString {
        JavaString::from_rust_str(&format!(
            "Cannot invoke \"{HANDLER_CLASS}.{}\" because \"this.next\" is null",
            self.method_signature
        ))
    }
}

impl Display for ChainedTextHandlerRuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.java_message().to_string_lossy())
    }
}

impl Error for ChainedTextHandlerRuntimeError {}

/// 将所有文本解析事件同步转发给下一个处理器的组合基类。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.AbstractChainedTextHandler`。
///
/// 上游通过继承复用十一个转发实现；Rust 以拥有
/// `Option<Box<dyn ITextHandler>>` 的组合对象表达同一链。每个回调把原始参数和同一
/// 可变 UTF-16 buffer 直接传给 `next`，不复制、不校验、不捕获异常：buffer 修改、
/// checked exception 的 Box 身份以及 runtime panic payload 均原样越过本层。
/// `None` 保留 Java 构造器接受 null、实际回调时才触发增强 NPE 的时序。
pub struct AbstractChainedTextHandler {
    next: Option<Box<dyn ITextHandler>>,
}

impl AbstractChainedTextHandler {
    /// 创建指向可空下游处理器的 handler 链节点。
    ///
    /// 对应 Java:
    /// `AbstractChainedTextHandler#AbstractChainedTextHandler(ITextHandler)`。
    ///
    /// # 参数
    /// - `next`：Java 参数 `next`；`None` 对应 Java null，构造阶段不校验。
    #[must_use]
    pub fn new(next: Option<Box<dyn ITextHandler>>) -> Self {
        Self { next }
    }

    /// 返回下游处理器的可变借用。
    ///
    /// 对应 Java:
    /// `AbstractChainedTextHandler#getNext()`。
    ///
    /// # 返回
    /// `Some` 指向构造时传入的同一对象；`None` 保留 Java null。可变借用是 Rust
    /// 对同步 handler 回调所需可变接收者的所有权映射。
    pub fn get_next(&mut self) -> Option<&mut (dyn ITextHandler + '_)> {
        match self.next {
            Some(ref mut next) => Some(next.as_mut()),
            None => None,
        }
    }

    fn require_next(&mut self, method_signature: &'static str) -> &mut (dyn ITextHandler + '_) {
        match self.next {
            Some(ref mut next) => next.as_mut(),
            None => panic_any(ChainedTextHandlerRuntimeError::null_next(method_signature)),
        }
    }
}

impl ITextHandler for AbstractChainedTextHandler {
    fn handle_document_start(
        &mut self,
        start_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.require_next("handleDocumentStart(long, int, int)")
            .handle_document_start(start_time_nanos, line, col)
    }

    fn handle_document_end(
        &mut self,
        end_time_nanos: i64,
        total_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.require_next("handleDocumentEnd(long, long, int, int)")
            .handle_document_end(end_time_nanos, total_time_nanos, line, col)
    }

    fn handle_text(
        &mut self,
        buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.require_next("handleText(char[], int, int, int, int)")
            .handle_text(buffer, offset, len, line, col)
    }

    fn handle_comment(
        &mut self,
        buffer: Option<&mut [u16]>,
        content_offset: i32,
        content_len: i32,
        outer_offset: i32,
        outer_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.require_next("handleComment(char[], int, int, int, int, int, int)")
            .handle_comment(
                buffer,
                content_offset,
                content_len,
                outer_offset,
                outer_len,
                line,
                col,
            )
    }

    fn handle_standalone_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        minimized: bool,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.require_next("handleStandaloneElementStart(char[], int, int, boolean, int, int)")
            .handle_standalone_element_start(buffer, name_offset, name_len, minimized, line, col)
    }

    fn handle_standalone_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        minimized: bool,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.require_next("handleStandaloneElementEnd(char[], int, int, boolean, int, int)")
            .handle_standalone_element_end(buffer, name_offset, name_len, minimized, line, col)
    }

    fn handle_open_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.require_next("handleOpenElementStart(char[], int, int, int, int)")
            .handle_open_element_start(buffer, name_offset, name_len, line, col)
    }

    fn handle_open_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.require_next("handleOpenElementEnd(char[], int, int, int, int)")
            .handle_open_element_end(buffer, name_offset, name_len, line, col)
    }

    fn handle_close_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.require_next("handleCloseElementStart(char[], int, int, int, int)")
            .handle_close_element_start(buffer, name_offset, name_len, line, col)
    }

    fn handle_close_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.require_next("handleCloseElementEnd(char[], int, int, int, int)")
            .handle_close_element_end(buffer, name_offset, name_len, line, col)
    }

    fn handle_attribute(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        name_line: i32,
        name_col: i32,
        operator_offset: i32,
        operator_len: i32,
        operator_line: i32,
        operator_col: i32,
        value_content_offset: i32,
        value_content_len: i32,
        value_outer_offset: i32,
        value_outer_len: i32,
        value_line: i32,
        value_col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.require_next(
            "handleAttribute(char[], int, int, int, int, int, int, int, int, int, int, int, int, int, int)",
        )
        .handle_attribute(
            buffer,
            name_offset,
            name_len,
            name_line,
            name_col,
            operator_offset,
            operator_len,
            operator_line,
            operator_col,
            value_content_offset,
            value_content_len,
            value_outer_offset,
            value_outer_len,
            value_line,
            value_col,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fmt::{Display, Write};
    use std::panic::{AssertUnwindSafe, catch_unwind, panic_any};
    use std::rc::Rc;
    use std::sync::Arc;

    use super::{
        AbstractChainedTextHandler, ChainedTextHandlerRuntimeError, ITextHandler,
        TextParseException,
    };
    use crate::text::AbstractTextHandler;
    use crate::util::JavaString;

    const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    const JAVA_GOLDEN: &str = include_str!("../../tests/fixtures/text_handler_adapters_golden.txt");
    const EVENTS: [Event; 11] = [
        Event::DocumentStart,
        Event::DocumentEnd,
        Event::Text,
        Event::Comment,
        Event::StandaloneStart,
        Event::StandaloneEnd,
        Event::OpenStart,
        Event::OpenEnd,
        Event::CloseStart,
        Event::CloseEnd,
        Event::Attribute,
    ];

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Event {
        DocumentStart,
        DocumentEnd,
        Text,
        Comment,
        StandaloneStart,
        StandaloneEnd,
        OpenStart,
        OpenEnd,
        CloseStart,
        CloseEnd,
        Attribute,
    }

    impl Event {
        const fn key(self) -> &'static str {
            match self {
                Self::DocumentStart => "documentStart",
                Self::DocumentEnd => "documentEnd",
                Self::Text => "text",
                Self::Comment => "comment",
                Self::StandaloneStart => "standaloneStart",
                Self::StandaloneEnd => "standaloneEnd",
                Self::OpenStart => "openStart",
                Self::OpenEnd => "openEnd",
                Self::CloseStart => "closeStart",
                Self::CloseEnd => "closeEnd",
                Self::Attribute => "attribute",
            }
        }
    }

    #[derive(Debug)]
    struct RuntimeFailure {
        event: Event,
        identity: Arc<()>,
    }

    struct RecordingHandler {
        events: Rc<RefCell<String>>,
        fail_event: Option<Event>,
        checked: Option<Box<TextParseException>>,
        runtime: Option<RuntimeFailure>,
    }

    impl RecordingHandler {
        fn success(events: Rc<RefCell<String>>) -> Self {
            Self {
                events,
                fail_event: None,
                checked: None,
                runtime: None,
            }
        }

        fn checked(
            events: Rc<RefCell<String>>,
            fail_event: Event,
            checked: Box<TextParseException>,
        ) -> Self {
            Self {
                events,
                fail_event: Some(fail_event),
                checked: Some(checked),
                runtime: None,
            }
        }

        fn runtime(
            events: Rc<RefCell<String>>,
            fail_event: Event,
            runtime: RuntimeFailure,
        ) -> Self {
            Self {
                events,
                fail_event: Some(fail_event),
                checked: None,
                runtime: Some(runtime),
            }
        }

        fn record(
            &mut self,
            event: Event,
            buffer: Option<&mut [u16]>,
            arguments: &str,
        ) -> Result<(), Box<TextParseException>> {
            {
                let mut events = self.events.borrow_mut();
                if !events.is_empty() {
                    events.push('|');
                }
                write!(
                    events,
                    "{}({arguments})@{}",
                    event.key(),
                    describe_buffer(buffer.as_deref())
                )
                .expect("write event");
            }
            if let Some(buffer) = buffer
                && let Some(first) = buffer.first_mut()
            {
                *first = first.wrapping_add(1);
            }
            if self.fail_event == Some(event) {
                if let Some(checked) = self.checked.take() {
                    return Err(checked);
                }
                panic_any(self.runtime.take().expect("runtime failure"));
            }
            Ok(())
        }
    }

    impl ITextHandler for RecordingHandler {
        fn handle_document_start(
            &mut self,
            start_time_nanos: i64,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                Event::DocumentStart,
                None,
                &format!("{start_time_nanos},{line},{col}"),
            )
        }

        fn handle_document_end(
            &mut self,
            end_time_nanos: i64,
            total_time_nanos: i64,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                Event::DocumentEnd,
                None,
                &format!("{end_time_nanos},{total_time_nanos},{line},{col}"),
            )
        }

        fn handle_text(
            &mut self,
            buffer: Option<&mut [u16]>,
            offset: i32,
            len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(Event::Text, buffer, &format!("{offset},{len},{line},{col}"))
        }

        fn handle_comment(
            &mut self,
            buffer: Option<&mut [u16]>,
            content_offset: i32,
            content_len: i32,
            outer_offset: i32,
            outer_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                Event::Comment,
                buffer,
                &format!("{content_offset},{content_len},{outer_offset},{outer_len},{line},{col}"),
            )
        }

        fn handle_standalone_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            minimized: bool,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                Event::StandaloneStart,
                buffer,
                &format!("{name_offset},{name_len},{minimized},{line},{col}"),
            )
        }

        fn handle_standalone_element_end(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            minimized: bool,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                Event::StandaloneEnd,
                buffer,
                &format!("{name_offset},{name_len},{minimized},{line},{col}"),
            )
        }

        fn handle_open_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                Event::OpenStart,
                buffer,
                &format!("{name_offset},{name_len},{line},{col}"),
            )
        }

        fn handle_open_element_end(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                Event::OpenEnd,
                buffer,
                &format!("{name_offset},{name_len},{line},{col}"),
            )
        }

        fn handle_close_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                Event::CloseStart,
                buffer,
                &format!("{name_offset},{name_len},{line},{col}"),
            )
        }

        fn handle_close_element_end(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                Event::CloseEnd,
                buffer,
                &format!("{name_offset},{name_len},{line},{col}"),
            )
        }

        fn handle_attribute(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            name_line: i32,
            name_col: i32,
            operator_offset: i32,
            operator_len: i32,
            operator_line: i32,
            operator_col: i32,
            value_content_offset: i32,
            value_content_len: i32,
            value_outer_offset: i32,
            value_outer_len: i32,
            value_line: i32,
            value_col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record(
                Event::Attribute,
                buffer,
                &format!(
                    "{name_offset},{name_len},{name_line},{name_col},\
                     {operator_offset},{operator_len},{operator_line},{operator_col},\
                     {value_content_offset},{value_content_len},{value_outer_offset},\
                     {value_outer_len},{value_line},{value_col}"
                ),
            )
        }
    }

    #[test]
    fn handler_adapters_match_java_golden() {
        let mut output = String::new();
        emit(&mut output, "baseline", JAVA_BASELINE);
        abstract_no_op_cases(&mut output);
        chained_success_cases(&mut output);
        chained_checked_cases(&mut output);
        chained_runtime_cases(&mut output);
        chained_null_cases(&mut output);
        assert_eq!(output, JAVA_GOLDEN);
    }

    #[test]
    fn runtime_error_display_and_value_semantics_are_covered() {
        let mut handler = AbstractChainedTextHandler::new(None);
        let panic = catch_unwind(AssertUnwindSafe(|| {
            handler
                .handle_document_start(0, 0, 0)
                .expect("null next panics first");
        }))
        .expect_err("null next");
        let error = panic
            .downcast::<ChainedTextHandlerRuntimeError>()
            .expect("runtime adapter");
        let cloned = error.as_ref().clone();
        assert_eq!(*error, cloned);
        assert_eq!(error.java_class_name(), "java.lang.NullPointerException");
        assert_eq!(error.to_string(), error.java_message().to_string_lossy());
        assert_eq!(describe_java_string(None), "null");
    }

    fn abstract_no_op_cases(output: &mut String) {
        let mut handler = AbstractTextHandler::new();
        let _default = AbstractTextHandler::default();
        let mut buffer = [u16::from(b'a'), 0xD800, u16::from(b'z')];
        for event in EVENTS {
            invoke(&mut handler, event, None).expect("null no-op event");
            invoke(&mut handler, event, Some(&mut buffer)).expect("buffer no-op event");
        }
        emit(
            output,
            "abstract.allEvents",
            format!("OK:{}", describe_buffer(Some(&buffer))),
        );
    }

    fn chained_success_cases(output: &mut String) {
        let events = Rc::new(RefCell::new(String::new()));
        let next = RecordingHandler::success(Rc::clone(&events));
        let mut handler = AbstractChainedTextHandler::new(Some(Box::new(next)));
        let mut buffer = [u16::from(b'a'), 0xD800, u16::from(b'z')];
        for event in EVENTS {
            invoke(&mut handler, event, Some(&mut buffer)).expect("forwarded event");
        }
        emit(output, "chained.identity", handler.get_next().is_some());
        emit(output, "chained.success.events", events.borrow().as_str());
        emit(
            output,
            "chained.success.buffer",
            describe_buffer(Some(&buffer)),
        );
    }

    fn chained_checked_cases(output: &mut String) {
        for event in EVENTS {
            let checked = Box::new(TextParseException::with_message_at(
                Some(&JavaString::from_rust_str(&format!(
                    "checked-{}",
                    event.key()
                ))),
                101,
                202,
            ));
            let expected_pointer = std::ptr::from_ref::<TextParseException>(checked.as_ref());
            let events = Rc::new(RefCell::new(String::new()));
            let next = RecordingHandler::checked(Rc::clone(&events), event, checked);
            let mut handler = AbstractChainedTextHandler::new(Some(Box::new(next)));
            let mut buffer = [u16::from(b'a'), u16::from(b'b')];
            let error = invoke(&mut handler, event, Some(&mut buffer))
                .expect_err("checked exception is forwarded");
            emit(
                output,
                &format!("chained.checked.{}", event.key()),
                format!(
                    "same={};class=org.thymeleaf.templateparser.text.TextParseException;\
                     message={};line={};col={};buffer={}",
                    std::ptr::eq(error.as_ref(), expected_pointer),
                    describe_java_string(error.get_message()),
                    error.get_line().expect("line"),
                    error.get_col().expect("col"),
                    describe_buffer(Some(&buffer))
                ),
            );
        }
    }

    fn chained_runtime_cases(output: &mut String) {
        for event in EVENTS {
            let identity = Arc::new(());
            let runtime = RuntimeFailure {
                event,
                identity: Arc::clone(&identity),
            };
            let events = Rc::new(RefCell::new(String::new()));
            let next = RecordingHandler::runtime(Rc::clone(&events), event, runtime);
            let mut handler = AbstractChainedTextHandler::new(Some(Box::new(next)));
            let mut buffer = [u16::from(b'a'), u16::from(b'b')];
            let panic = catch_unwind(AssertUnwindSafe(|| {
                invoke(&mut handler, event, Some(&mut buffer))
                    .expect("runtime failure does not use checked channel");
            }))
            .expect_err("runtime panic is forwarded");
            let runtime = panic.downcast::<RuntimeFailure>().expect("runtime payload");
            emit(
                output,
                &format!("chained.runtime.{}", event.key()),
                format!(
                    "same={};class=java.lang.IllegalStateException;message={};buffer={}",
                    runtime.event == event && Arc::ptr_eq(&runtime.identity, &identity),
                    describe_java_string(Some(&JavaString::from_rust_str(&format!(
                        "runtime-{}",
                        event.key()
                    )))),
                    describe_buffer(Some(&buffer))
                ),
            );
        }
    }

    fn chained_null_cases(output: &mut String) {
        let mut handler = AbstractChainedTextHandler::new(None);
        emit(
            output,
            "chained.null.identity",
            handler.get_next().is_none(),
        );
        for event in EVENTS {
            let mut buffer = [u16::from(b'a'), u16::from(b'b')];
            let panic = catch_unwind(AssertUnwindSafe(|| {
                invoke(&mut handler, event, Some(&mut buffer))
                    .expect("null next does not use checked channel");
            }))
            .expect_err("null next panic");
            let error = panic
                .downcast::<ChainedTextHandlerRuntimeError>()
                .expect("null next runtime adapter");
            emit(
                output,
                &format!("chained.null.{}", event.key()),
                format!(
                    "class={};message={};buffer={}",
                    error.java_class_name(),
                    describe_java_string(Some(&error.java_message())),
                    describe_buffer(Some(&buffer))
                ),
            );
        }
    }

    fn invoke(
        handler: &mut dyn ITextHandler,
        event: Event,
        buffer: Option<&mut [u16]>,
    ) -> Result<(), Box<TextParseException>> {
        match event {
            Event::DocumentStart => handler.handle_document_start(i64::MIN, i32::MIN, i32::MAX),
            Event::DocumentEnd => handler.handle_document_end(i64::MAX, -7, i32::MAX, i32::MIN),
            Event::Text => handler.handle_text(buffer, -1, 7, 11, 13),
            Event::Comment => handler.handle_comment(buffer, 1, 2, 3, 4, 5, 6),
            Event::StandaloneStart => {
                handler.handle_standalone_element_start(buffer, 7, 8, true, 9, 10)
            }
            Event::StandaloneEnd => {
                handler.handle_standalone_element_end(buffer, 11, 12, false, 13, 14)
            }
            Event::OpenStart => handler.handle_open_element_start(buffer, 15, 16, 17, 18),
            Event::OpenEnd => handler.handle_open_element_end(buffer, 19, 20, 21, 22),
            Event::CloseStart => handler.handle_close_element_start(buffer, 23, 24, 25, 26),
            Event::CloseEnd => handler.handle_close_element_end(buffer, 27, 28, 29, 30),
            Event::Attribute => handler.handle_attribute(
                buffer, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44,
            ),
        }
    }

    fn describe_buffer(buffer: Option<&[u16]>) -> String {
        buffer.map_or_else(
            || "null".to_owned(),
            |buffer| {
                buffer
                    .iter()
                    .map(|unit| format!("{unit:04x}"))
                    .collect::<Vec<_>>()
                    .join(",")
            },
        )
    }

    fn describe_java_string(value: Option<&JavaString>) -> String {
        value.map_or_else(
            || "null".to_owned(),
            |value| describe_buffer(Some(value.as_utf16())),
        )
    }

    fn emit(output: &mut String, key: &str, value: impl Display) {
        writeln!(output, "{key}={value}").expect("write golden output");
    }
}
