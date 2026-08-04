use std::error::Error;
use std::fmt::{Display, Formatter};
use std::panic::panic_any;
use std::rc::Rc;

use super::{AbstractChainedTextHandler, ITextHandler, TextParseException};
use crate::util::Utf16String;

const DEFAULT_STACK_LEN: usize = 10;
const DEFAULT_ATTRIBUTE_NAMES_LEN: usize = 3;
const REPOSITORY_INITIAL_LEN: usize = 20;
const REPOSITORY_INITIAL_INC: usize = 5;

/// 事件预处理器中 Java 未检查异常的精确适配。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.EventProcessorTextHandler` 调用
/// `TextUtils`、数组读取、数组创建和 `System.arraycopy` 时可能抛出的异常。
///
/// 解析结构错误仍通过 [`TextParseException`] 的 checked 通道返回；本类型只作为
/// panic payload 保存 Java 异常类名与 UTF-16 消息。
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EventProcessorTextHandlerRuntimeError {
    class_name: &'static str,
    java_message: Utf16String,
}

impl EventProcessorTextHandlerRuntimeError {
    fn illegal_argument(message: &'static str) -> Self {
        Self {
            class_name: "java.lang.IllegalArgumentException",
            java_message: Utf16String::from_rust_str(message),
        }
    }

    fn negative_array_size(len: i32) -> Self {
        Self {
            class_name: "java.lang.NegativeArraySizeException",
            java_message: Utf16String::from_rust_str(&len.to_string()),
        }
    }

    fn array_index(index: i32, length: usize) -> Self {
        Self {
            class_name: "java.lang.ArrayIndexOutOfBoundsException",
            java_message: Utf16String::from_rust_str(&format!(
                "Index {index} out of bounds for length {length}"
            )),
        }
    }

    fn arraycopy_source_index(index: i32, length: usize) -> Self {
        Self {
            class_name: "java.lang.ArrayIndexOutOfBoundsException",
            java_message: Utf16String::from_rust_str(&format!(
                "arraycopy: source index {index} out of bounds for char[{length}]"
            )),
        }
    }

    fn arraycopy_last_source(index: i64, length: usize) -> Self {
        Self {
            class_name: "java.lang.ArrayIndexOutOfBoundsException",
            java_message: Utf16String::from_rust_str(&format!(
                "arraycopy: last source index {index} out of bounds for char[{length}]"
            )),
        }
    }

    /// 返回对应 Java 异常全限定名。
    ///
    /// # 返回
    /// `IllegalArgumentException`、`ArrayIndexOutOfBoundsException` 或
    /// `NegativeArraySizeException`。
    #[must_use]
    pub(crate) const fn class_name(&self) -> &'static str {
        self.class_name
    }

    /// 返回对应 Java 异常的 UTF-16 消息。
    ///
    /// # 返回
    /// 与固定 JDK Oracle 对齐的消息。
    #[must_use]
    pub(crate) const fn java_message(&self) -> &Utf16String {
        &self.java_message
    }
}

impl Display for EventProcessorTextHandlerRuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.java_message.to_string_lossy())
    }
}

impl Error for EventProcessorTextHandlerRuntimeError {}

/// 对解析器事件执行元素嵌套校验、属性名称去重和结构名称驻留。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.EventProcessorTextHandler`。
///
/// 该对象是单次解析执行内部的非线程安全处理器。开放标签在下游成功后入栈，
/// 关闭标签在转发前完成匹配和出栈，属性在转发前登记；这些先后次序完整保留
/// Java 在下游失败时可观察到的部分状态变化。
pub(crate) struct EventProcessorTextHandler {
    chained: AbstractChainedTextHandler,
    structure_names_repository: StructureNamesRepository,
    element_stack: Vec<Rc<[u16]>>,
    element_stack_len: usize,
    current_element_attribute_names: Option<Vec<Rc<[u16]>>>,
    current_element_attribute_names_len: usize,
}

impl EventProcessorTextHandler {
    /// 创建事件预处理器。
    ///
    /// 对应 Java:
    /// `EventProcessorTextHandler#EventProcessorTextHandler(ITextHandler)`。
    ///
    /// # 参数
    /// - `handler`：下游文本事件处理器；`None` 保留 Java null 的延迟失败语义。
    #[must_use]
    pub(crate) fn new(handler: Option<Box<dyn ITextHandler>>) -> Self {
        Self {
            chained: AbstractChainedTextHandler::new(handler),
            structure_names_repository: StructureNamesRepository::new(),
            element_stack: Vec::with_capacity(DEFAULT_STACK_LEN),
            element_stack_len: DEFAULT_STACK_LEN,
            current_element_attribute_names: None,
            current_element_attribute_names_len: 0,
        }
    }

    fn clear_attribute_names(&mut self) {
        self.current_element_attribute_names = None;
        self.current_element_attribute_names_len = 0;
    }

    fn check_stack_for_element(
        &mut self,
        buffer: Option<&[u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) -> Result<bool, Box<TextParseException>> {
        let Some(peek) = self.peek_from_stack().cloned() else {
            return Err(text_parse_at(
                "Malformed template: unnamed closing element is never opened",
                line,
                col,
            ));
        };

        if exact_equals(
            Some(peek.as_ref()),
            0,
            peek.len() as i32,
            buffer,
            offset,
            len,
        ) {
            self.pop_from_stack();
            return Ok(true);
        }

        let message = if peek.is_empty() {
            Utf16String::from_rust_str("Malformed template: unnamed element is never closed")
        } else {
            quoted_message(
                "Malformed template: element \"",
                peek.as_ref(),
                "\" is never closed",
            )
        };
        Err(Box::new(TextParseException::with_message_at(
            Some(&message),
            line,
            col,
        )))
    }

    fn push_to_stack(&mut self, buffer: Option<&[u16]>, offset: i32, len: i32) {
        if self.element_stack.len() == self.element_stack_len {
            self.grow_stack();
        }
        let name = self
            .structure_names_repository
            .get_structure_name(buffer, offset, len);
        self.element_stack.push(name);
    }

    fn peek_from_stack(&self) -> Option<&Rc<[u16]>> {
        self.element_stack.last()
    }

    fn pop_from_stack(&mut self) -> Option<Rc<[u16]>> {
        self.element_stack.pop()
    }

    fn grow_stack(&mut self) {
        self.element_stack_len += DEFAULT_STACK_LEN;
        self.element_stack.reserve_exact(DEFAULT_STACK_LEN);
    }
}

impl ITextHandler for EventProcessorTextHandler {
    fn handle_document_start(
        &mut self,
        start_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.chained
            .handle_document_start(start_time_nanos, line, col)
    }

    fn handle_document_end(
        &mut self,
        end_time_nanos: i64,
        total_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        if let Some(popped) = self.pop_from_stack() {
            let message = quoted_message(
                "Malformed template: element \"",
                popped.as_ref(),
                "\" is never closed (no closing tag at the end of document)",
            );
            return Err(Box::new(TextParseException::with_message(Some(message))));
        }
        self.chained
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
        self.chained.handle_text(buffer, offset, len, line, col)
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
        self.chained.handle_comment(
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
        self.clear_attribute_names();
        self.chained.handle_standalone_element_start(
            buffer,
            name_offset,
            name_len,
            minimized,
            line,
            col,
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
        self.chained.handle_standalone_element_end(
            buffer,
            name_offset,
            name_len,
            minimized,
            line,
            col,
        )
    }

    fn handle_open_element_start(
        &mut self,
        mut buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.clear_attribute_names();
        self.chained.handle_open_element_start(
            buffer.as_deref_mut(),
            name_offset,
            name_len,
            line,
            col,
        )?;
        self.push_to_stack(buffer.as_deref(), name_offset, name_len);
        Ok(())
    }

    fn handle_open_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        self.chained
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
        if !self.check_stack_for_element(buffer.as_deref(), name_offset, name_len, line, col)? {
            let name = copy_java_range(buffer.as_deref(), name_offset, name_len);
            let message = quoted_message(
                "Malformed text: element \"",
                name.as_ref(),
                "\" is never closed",
            );
            return Err(Box::new(TextParseException::with_message_at(
                Some(&message),
                line,
                col,
            )));
        }
        self.clear_attribute_names();
        self.chained
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
        self.chained
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
        if self.current_element_attribute_names.is_none() {
            self.current_element_attribute_names =
                Some(Vec::with_capacity(DEFAULT_ATTRIBUTE_NAMES_LEN));
            self.current_element_attribute_names_len = DEFAULT_ATTRIBUTE_NAMES_LEN;
        }

        if self
            .current_element_attribute_names
            .as_ref()
            .expect("attribute array initialized")
            .iter()
            .any(|current| {
                exact_equals(
                    Some(current.as_ref()),
                    0,
                    current.len() as i32,
                    buffer.as_deref(),
                    name_offset,
                    name_len,
                )
            })
        {
            let name = copy_java_range(buffer.as_deref(), name_offset, name_len);
            let message = quoted_message(
                "Malformed text: Attribute \"",
                name.as_ref(),
                "\" appears more than once in element",
            );
            return Err(Box::new(TextParseException::with_message_at(
                Some(&message),
                name_line,
                name_col,
            )));
        }

        let attributes = self
            .current_element_attribute_names
            .as_mut()
            .expect("attribute array initialized");
        if attributes.len() == self.current_element_attribute_names_len {
            self.current_element_attribute_names_len += DEFAULT_ATTRIBUTE_NAMES_LEN;
            attributes.reserve_exact(DEFAULT_ATTRIBUTE_NAMES_LEN);
        }

        let name = self.structure_names_repository.get_structure_name(
            buffer.as_deref(),
            name_offset,
            name_len,
        );
        attributes.push(name);

        self.chained.handle_attribute(
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

/// 元素名和属性名的单次解析驻留仓库。
///
/// 对应 Java:
/// `EventProcessorTextHandler.StructureNamesRepository`。
///
/// 名称按 UTF-16 代码单元精确、有序保存；相同名称返回同一个 [`Rc`] 分配，
/// 从而保留 Java 返回同一 `char[]` 的身份语义。该对象不是线程安全的。
pub(crate) struct StructureNamesRepository {
    repository: Vec<Rc<[u16]>>,
    repository_len: usize,
}

impl StructureNamesRepository {
    /// 创建初始逻辑长度为 20 的空名称仓库。
    ///
    /// 对应 Java: `StructureNamesRepository#StructureNamesRepository()`。
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            repository: Vec::with_capacity(REPOSITORY_INITIAL_LEN),
            repository_len: REPOSITORY_INITIAL_LEN,
        }
    }

    /// 获取精确匹配的驻留名称，或复制给定范围并按序插入。
    ///
    /// 对应 Java: `StructureNamesRepository#getStructureName(char[],int,int)`。
    ///
    /// # 参数
    /// - `text`：Java UTF-16 `char[]`，`None` 对应 null。
    /// - `offset`：名称起点。
    /// - `len`：名称代码单元数。
    ///
    /// # 返回
    /// 已缓存或新建的同一身份名称。
    pub(crate) fn get_structure_name(
        &mut self,
        text: Option<&[u16]>,
        offset: i32,
        len: i32,
    ) -> Rc<[u16]> {
        let Some(text) = text else {
            panic_runtime(EventProcessorTextHandlerRuntimeError::illegal_argument(
                "Text cannot be null",
            ));
        };

        // 精确复现 TextUtils.binarySearch 的 low/high/mid 探测顺序；非法范围也会
        // 因为被哪个候选项先读取而产生不同的 JVM 异常，不能交给标准库决定。
        let mut low = 0_i32;
        let mut high = self.repository.len() as i32 - 1;
        while low <= high {
            let midpoint = ((low + high) as u32 >> 1) as i32;
            match compare_java_range(&self.repository[midpoint as usize], text, offset, len) {
                std::cmp::Ordering::Less => low = midpoint + 1,
                std::cmp::Ordering::Greater => high = midpoint - 1,
                std::cmp::Ordering::Equal => {
                    return Rc::clone(&self.repository[midpoint as usize]);
                }
            }
        }
        self.store_structure_name(-(low + 1), text, offset, len)
    }

    fn store_structure_name(
        &mut self,
        index: i32,
        text: &[u16],
        offset: i32,
        len: i32,
    ) -> Rc<[u16]> {
        if self.repository.len() == self.repository_len {
            self.repository_len += REPOSITORY_INITIAL_INC;
            self.repository.reserve_exact(REPOSITORY_INITIAL_INC);
        }
        if len < 0 {
            panic_runtime(EventProcessorTextHandlerRuntimeError::negative_array_size(
                len,
            ));
        }

        let structure_name = Rc::<[u16]>::from(copy_array_range(text, offset, len));
        let insertion_index = (-(index + 1)) as usize;
        self.repository
            .insert(insertion_index, Rc::clone(&structure_name));
        structure_name
    }
}

fn exact_equals(
    first: Option<&[u16]>,
    first_offset: i32,
    first_len: i32,
    second: Option<&[u16]>,
    second_offset: i32,
    second_len: i32,
) -> bool {
    let Some(first) = first else {
        panic_runtime(EventProcessorTextHandlerRuntimeError::illegal_argument(
            "First text buffer being compared cannot be null",
        ));
    };
    let Some(second) = second else {
        panic_runtime(EventProcessorTextHandlerRuntimeError::illegal_argument(
            "Second text buffer being compared cannot be null",
        ));
    };
    if first_len != second_len {
        return false;
    }
    for index in 0..first_len {
        if java_array_get(first, first_offset.wrapping_add(index))
            != java_array_get(second, second_offset.wrapping_add(index))
        {
            return false;
        }
    }
    true
}

fn compare_java_range(
    candidate: &[u16],
    text: &[u16],
    offset: i32,
    len: i32,
) -> std::cmp::Ordering {
    let mut count = i32::try_from(candidate.len())
        .expect("Java char[] length fits i32")
        .min(len);
    let mut index = 0_i32;
    loop {
        let before_decrement = count;
        count = count.wrapping_sub(1);
        if before_decrement == 0 {
            break;
        }
        let first = java_array_get(candidate, index);
        let second = java_array_get(text, offset.wrapping_add(index));
        match first.cmp(&second) {
            std::cmp::Ordering::Equal => index = index.wrapping_add(1),
            ordering => return ordering,
        }
    }
    (candidate.len() as i32).cmp(&len)
}

fn java_array_get(text: &[u16], index: i32) -> u16 {
    if index < 0 || index as usize >= text.len() {
        panic_runtime(EventProcessorTextHandlerRuntimeError::array_index(
            index,
            text.len(),
        ));
    }
    text[index as usize]
}

fn copy_java_range(text: Option<&[u16]>, offset: i32, len: i32) -> Rc<[u16]> {
    let Some(text) = text else {
        panic_runtime(EventProcessorTextHandlerRuntimeError::illegal_argument(
            "Text cannot be null",
        ));
    };
    if len < 0 {
        panic_runtime(EventProcessorTextHandlerRuntimeError::negative_array_size(
            len,
        ));
    }
    Rc::from(copy_array_range(text, offset, len))
}

fn copy_array_range(text: &[u16], offset: i32, len: i32) -> Vec<u16> {
    if offset < 0 {
        panic_runtime(
            EventProcessorTextHandlerRuntimeError::arraycopy_source_index(offset, text.len()),
        );
    }
    let last = i64::from(offset) + i64::from(len);
    if last > text.len() as i64 {
        panic_runtime(
            EventProcessorTextHandlerRuntimeError::arraycopy_last_source(last, text.len()),
        );
    }
    text[offset as usize..last as usize].to_vec()
}

fn quoted_message(prefix: &str, name: &[u16], suffix: &str) -> Utf16String {
    let mut message = Utf16String::from_rust_str(prefix).as_utf16().to_vec();
    message.extend_from_slice(name);
    message.extend_from_slice(Utf16String::from_rust_str(suffix).as_utf16());
    Utf16String::from_utf16(message)
}

fn text_parse_at(message: &str, line: i32, col: i32) -> Box<TextParseException> {
    Box::new(TextParseException::with_message_at(
        Some(&Utf16String::from_rust_str(message)),
        line,
        col,
    ))
}

fn panic_runtime(error: EventProcessorTextHandlerRuntimeError) -> ! {
    panic_any(error)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fmt::Display;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::rc::Rc;

    use super::{
        EventProcessorTextHandler, EventProcessorTextHandlerRuntimeError, ITextHandler,
        StructureNamesRepository, TextParseException,
    };
    use crate::util::Utf16String;

    const JAVA_GOLDEN: &str =
        include_str!("../../tests/fixtures/event_processor_text_handler_golden.txt");

    #[derive(Default)]
    struct RecordingState {
        events: String,
        fail_event: Option<&'static str>,
        mutate_attribute_name: bool,
    }

    struct RecordingHandler {
        state: Rc<RefCell<RecordingState>>,
    }

    impl RecordingHandler {
        fn record(
            &self,
            event: &'static str,
            buffer: Option<&mut [u16]>,
            offset: i32,
            len: i32,
        ) -> Result<(), Box<TextParseException>> {
            let mut state = self.state.borrow_mut();
            if !state.events.is_empty() {
                state.events.push('|');
            }
            state.events.push_str(event);
            state.events.push('@');
            match buffer.as_deref() {
                None => state.events.push_str("null"),
                Some(value)
                    if offset >= 0
                        && len >= 0
                        && (i64::from(offset) + i64::from(len)) <= value.len() as i64 =>
                {
                    state
                        .events
                        .push_str(&hex(&value[offset as usize..(offset + len) as usize]));
                }
                Some(_) => state.events.push_str(&format!("range({offset},{len})")),
            }
            if state.mutate_attribute_name && event == "attribute" && len > 0 {
                buffer.expect("attribute buffer")[offset as usize] = b'b'.into();
            }
            if state.fail_event == Some(event) {
                return Err(Box::new(TextParseException::with_message_at(
                    Some(&Utf16String::from_rust_str(&format!("downstream-{event}"))),
                    71,
                    72,
                )));
            }
            Ok(())
        }
    }

    impl ITextHandler for RecordingHandler {
        fn handle_document_start(
            &mut self,
            _start_time_nanos: i64,
            _line: i32,
            _col: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }

        fn handle_document_end(
            &mut self,
            _end_time_nanos: i64,
            _total_time_nanos: i64,
            _line: i32,
            _col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record("documentEnd", None, 0, 0)
        }

        fn handle_text(
            &mut self,
            _buffer: Option<&mut [u16]>,
            _offset: i32,
            _len: i32,
            _line: i32,
            _col: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }

        fn handle_comment(
            &mut self,
            _buffer: Option<&mut [u16]>,
            _content_offset: i32,
            _content_len: i32,
            _outer_offset: i32,
            _outer_len: i32,
            _line: i32,
            _col: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }

        fn handle_standalone_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            _minimized: bool,
            _line: i32,
            _col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record("standaloneStart", buffer, name_offset, name_len)
        }

        fn handle_standalone_element_end(
            &mut self,
            _buffer: Option<&mut [u16]>,
            _name_offset: i32,
            _name_len: i32,
            _minimized: bool,
            _line: i32,
            _col: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }

        fn handle_open_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            _line: i32,
            _col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record("openStart", buffer, name_offset, name_len)
        }

        fn handle_open_element_end(
            &mut self,
            _buffer: Option<&mut [u16]>,
            _name_offset: i32,
            _name_len: i32,
            _line: i32,
            _col: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }

        fn handle_close_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            _line: i32,
            _col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record("closeStart", buffer, name_offset, name_len)
        }

        fn handle_close_element_end(
            &mut self,
            _buffer: Option<&mut [u16]>,
            _name_offset: i32,
            _name_len: i32,
            _line: i32,
            _col: i32,
        ) -> Result<(), Box<TextParseException>> {
            Ok(())
        }

        fn handle_attribute(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            _name_line: i32,
            _name_col: i32,
            _operator_offset: i32,
            _operator_len: i32,
            _operator_line: i32,
            _operator_col: i32,
            _value_content_offset: i32,
            _value_content_len: i32,
            _value_outer_offset: i32,
            _value_outer_len: i32,
            _value_line: i32,
            _value_col: i32,
        ) -> Result<(), Box<TextParseException>> {
            self.record("attribute", buffer, name_offset, name_len)
        }
    }

    fn handler() -> (EventProcessorTextHandler, Rc<RefCell<RecordingState>>) {
        let state = Rc::new(RefCell::new(RecordingState::default()));
        let next = RecordingHandler {
            state: Rc::clone(&state),
        };
        (EventProcessorTextHandler::new(Some(Box::new(next))), state)
    }

    fn java_chars(value: &str) -> Vec<u16> {
        value.encode_utf16().collect()
    }

    fn attribute(
        handler: &mut EventProcessorTextHandler,
        name: &str,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        let mut chars = java_chars(name);
        let len = chars.len() as i32;
        attribute_buffer(handler, Some(chars.as_mut_slice()), 0, len, line, col)
    }

    fn attribute_buffer(
        handler: &mut EventProcessorTextHandler,
        chars: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>> {
        handler.handle_attribute(
            chars, offset, len, line, col, -1, 0, line, col, -1, 0, -1, 0, line, col,
        )
    }

    fn state(handler: &EventProcessorTextHandler) -> String {
        format!(
            "stack={}/{};attrs={};repo={}",
            names(&handler.element_stack),
            handler.element_stack_len,
            handler
                .current_element_attribute_names
                .as_ref()
                .map_or_else(
                    || "null".to_owned(),
                    |attributes| format!(
                        "{}/{}",
                        names(attributes),
                        handler.current_element_attribute_names_len
                    )
                ),
            repository_state(&handler.structure_names_repository)
        )
    }

    fn repository_state(repository: &StructureNamesRepository) -> String {
        format!(
            "{}/{}",
            names(&repository.repository),
            repository.repository_len
        )
    }

    fn names(values: &[Rc<[u16]>]) -> String {
        format!(
            "[{}]",
            values
                .iter()
                .map(|value| hex(value))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn hex(value: &[u16]) -> String {
        value
            .iter()
            .map(|unit| format!("{unit:04x}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn throwable(operation: &mut dyn FnMut() -> Result<(), Box<TextParseException>>) -> String {
        match catch_unwind(AssertUnwindSafe(operation)) {
            Ok(Ok(())) => "NO_ERROR".to_owned(),
            Ok(Err(error)) => format!(
                "org.thymeleaf.templateparser.text.TextParseException;message={};line={};col={}",
                error
                    .get_message()
                    .map_or_else(|| "null".to_owned(), |message| hex(message.as_utf16())),
                error
                    .get_line()
                    .map_or_else(|| "null".to_owned(), |line| line.to_string()),
                error
                    .get_col()
                    .map_or_else(|| "null".to_owned(), |col| col.to_string())
            ),
            Err(payload) => {
                let error = payload
                    .downcast::<EventProcessorTextHandlerRuntimeError>()
                    .expect("known Java runtime payload");
                format!(
                    "{};message={}",
                    error.class_name(),
                    hex(error.java_message().as_utf16())
                )
            }
        }
    }

    fn emit(lines: &mut Vec<String>, key: &str, value: impl Display) {
        lines.push(format!("{key}={value}"));
    }

    fn generate_golden() -> String {
        let mut lines = Vec::new();
        emit(
            &mut lines,
            "baseline",
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add",
        );

        let (mut happy, happy_state) = handler();
        let mut root = java_chars("root");
        let mut id = java_chars("id");
        let mut child = java_chars("child");
        happy
            .handle_open_element_start(Some(&mut root), 0, 4, 1, 2)
            .unwrap();
        attribute_buffer(&mut happy, Some(&mut id), 0, 2, 3, 4).unwrap();
        happy
            .handle_open_element_start(Some(&mut child), 0, 5, 5, 6)
            .unwrap();
        happy
            .handle_close_element_start(Some(&mut child), 0, 5, 7, 8)
            .unwrap();
        let mut single = java_chars("single");
        happy
            .handle_standalone_element_start(Some(&mut single), 0, 6, true, 9, 10)
            .unwrap();
        happy
            .handle_close_element_start(Some(&mut root), 0, 4, 11, 12)
            .unwrap();
        happy.handle_document_end(13, 14, 15, 16).unwrap();
        emit(
            &mut lines,
            "happy.events",
            happy_state.borrow().events.clone(),
        );
        emit(&mut lines, "happy.state", state(&happy));

        let (mut empty, _) = handler();
        emit(
            &mut lines,
            "stack.closeEmpty",
            throwable(&mut || empty.handle_close_element_start(None, -9, -8, 21, 22)),
        );
        emit(&mut lines, "stack.closeEmpty.state", state(&empty));

        let (mut mismatch, _) = handler();
        let mut alpha = java_chars("alpha");
        mismatch
            .handle_open_element_start(Some(&mut alpha), 0, 5, 1, 1)
            .unwrap();
        let mut beta = java_chars("beta");
        emit(
            &mut lines,
            "stack.mismatch",
            throwable(&mut || mismatch.handle_close_element_start(Some(&mut beta), 0, 4, 23, 24)),
        );
        emit(&mut lines, "stack.mismatch.state", state(&mismatch));
        emit(
            &mut lines,
            "stack.documentEnd1",
            throwable(&mut || mismatch.handle_document_end(1, 2, 3, 4)),
        );
        emit(&mut lines, "stack.documentEnd1.state", state(&mismatch));
        mismatch.handle_document_end(1, 2, 3, 4).unwrap();
        emit(&mut lines, "stack.documentEnd2.state", state(&mismatch));

        let (mut unnamed, _) = handler();
        let mut blank = Vec::new();
        unnamed
            .handle_open_element_start(Some(&mut blank), 0, 0, 1, 1)
            .unwrap();
        let mut x = java_chars("x");
        emit(
            &mut lines,
            "stack.unnamed",
            throwable(&mut || unnamed.handle_close_element_start(Some(&mut x), 0, 1, 25, 26)),
        );

        let (mut drain, _) = handler();
        for value in ["a", "b"] {
            let mut chars = java_chars(value);
            drain
                .handle_open_element_start(Some(&mut chars), 0, 1, 1, 1)
                .unwrap();
        }
        emit(
            &mut lines,
            "stack.drain1",
            throwable(&mut || drain.handle_document_end(1, 2, 3, 4)),
        );
        emit(
            &mut lines,
            "stack.drain2",
            throwable(&mut || drain.handle_document_end(1, 2, 3, 4)),
        );
        drain.handle_document_end(1, 2, 3, 4).unwrap();
        emit(&mut lines, "stack.drain.state", state(&drain));

        let (mut attributes, attribute_state) = handler();
        let mut s = java_chars("s");
        attributes
            .handle_standalone_element_start(Some(&mut s), 0, 1, false, 1, 1)
            .unwrap();
        attribute(&mut attributes, "name", 31, 32).unwrap();
        emit(
            &mut lines,
            "attribute.duplicate",
            throwable(&mut || attribute(&mut attributes, "name", 33, 34)),
        );
        for (name, line, col) in [("Name", 35, 36), ("a", 1, 1), ("b", 1, 1)] {
            attribute(&mut attributes, name, line, col).unwrap();
        }
        emit(
            &mut lines,
            "attribute.caseAndGrowth.state",
            state(&attributes),
        );
        emit(
            &mut lines,
            "attribute.caseAndGrowth.events",
            attribute_state.borrow().events.clone(),
        );

        let (mut mutation, mutation_state) = handler();
        mutation_state.borrow_mut().mutate_attribute_name = true;
        let mut s = java_chars("s");
        mutation
            .handle_standalone_element_start(Some(&mut s), 0, 1, false, 1, 1)
            .unwrap();
        let mut mutable = java_chars("a");
        attribute_buffer(&mut mutation, Some(&mut mutable), 0, 1, 1, 1).unwrap();
        mutation_state.borrow_mut().mutate_attribute_name = false;
        attribute(&mut mutation, "b", 2, 2).unwrap();
        emit(&mut lines, "attribute.mutation.buffer", hex(&mutable));
        emit(&mut lines, "attribute.mutation.state", state(&mutation));

        let (mut open, open_state) = handler();
        open_state.borrow_mut().fail_event = Some("openStart");
        let mut x = java_chars("x");
        emit(
            &mut lines,
            "ordering.open.checked",
            throwable(&mut || open.handle_open_element_start(Some(&mut x), 0, 1, 1, 2)),
        );
        emit(&mut lines, "ordering.open.state", state(&open));

        let (mut close, close_state) = handler();
        let mut x = java_chars("x");
        close
            .handle_open_element_start(Some(&mut x), 0, 1, 1, 1)
            .unwrap();
        attribute(&mut close, "old", 1, 1).unwrap();
        close_state.borrow_mut().fail_event = Some("closeStart");
        emit(
            &mut lines,
            "ordering.close.checked",
            throwable(&mut || close.handle_close_element_start(Some(&mut x), 0, 1, 3, 4)),
        );
        emit(&mut lines, "ordering.close.state", state(&close));

        let (mut failed_attribute, failed_attribute_state) = handler();
        let mut s = java_chars("s");
        failed_attribute
            .handle_standalone_element_start(Some(&mut s), 0, 1, false, 1, 1)
            .unwrap();
        failed_attribute_state.borrow_mut().fail_event = Some("attribute");
        emit(
            &mut lines,
            "ordering.attribute.checked",
            throwable(&mut || attribute(&mut failed_attribute, "x", 5, 6)),
        );
        failed_attribute_state.borrow_mut().fail_event = None;
        emit(
            &mut lines,
            "ordering.attribute.retry",
            throwable(&mut || attribute(&mut failed_attribute, "x", 7, 8)),
        );
        emit(
            &mut lines,
            "ordering.attribute.state",
            state(&failed_attribute),
        );

        let (mut standalone, standalone_state) = handler();
        let mut s = java_chars("s");
        standalone
            .handle_standalone_element_start(Some(&mut s), 0, 1, false, 1, 1)
            .unwrap();
        attribute(&mut standalone, "old", 1, 1).unwrap();
        standalone_state.borrow_mut().fail_event = Some("standaloneStart");
        let mut t = java_chars("t");
        emit(
            &mut lines,
            "ordering.standalone.checked",
            throwable(&mut || {
                standalone.handle_standalone_element_start(Some(&mut t), 0, 1, false, 1, 1)
            }),
        );
        emit(&mut lines, "ordering.standalone.state", state(&standalone));

        let (mut growth, _) = handler();
        for index in 0..11 {
            let mut name = java_chars(&format!("e{index}"));
            let len = name.len() as i32;
            growth
                .handle_open_element_start(Some(&mut name), 0, len, 1, 1)
                .unwrap();
        }
        emit(&mut lines, "growth.stack.open", state(&growth));
        for index in (0..11).rev() {
            let mut name = java_chars(&format!("e{index}"));
            let len = name.len() as i32;
            growth
                .handle_close_element_start(Some(&mut name), 0, len, 1, 1)
                .unwrap();
        }
        emit(&mut lines, "growth.stack.closed", state(&growth));

        let mut repository = StructureNamesRepository::new();
        let mut source = vec![b'x'.into(), b'b'.into(), 0, 0xD800, b'z'.into()];
        let first = repository.get_structure_name(Some(&source), 1, 3);
        let same = repository.get_structure_name(Some(&[b'b'.into(), 0, 0xD800]), 0, 3);
        source[1] = b'q'.into();
        emit(&mut lines, "repository.identity", Rc::ptr_eq(&first, &same));
        emit(&mut lines, "repository.copy", hex(&first));
        for name in ["z", "a", "m", "", "A", "\u{D7FF}", "\0"] {
            let chars = if name == "\u{D7FF}" {
                vec![0xD800]
            } else {
                java_chars(name)
            };
            repository.get_structure_name(Some(&chars), 0, chars.len() as i32);
        }
        emit(
            &mut lines,
            "repository.sorted",
            repository_state(&repository),
        );
        for index in 0..20 {
            let chars = java_chars(&format!("n{index}"));
            repository.get_structure_name(Some(&chars), 0, chars.len() as i32);
        }
        emit(
            &mut lines,
            "repository.grown",
            repository_state(&repository),
        );

        let mut invalid_repository = StructureNamesRepository::new();
        emit(
            &mut lines,
            "invalid.repository.null",
            throwable(&mut || {
                invalid_repository.get_structure_name(None, 0, 0);
                Ok(())
            }),
        );
        emit(
            &mut lines,
            "invalid.repository.negativeOffset",
            throwable(&mut || {
                invalid_repository.get_structure_name(Some(&[b'a'.into()]), -1, 1);
                Ok(())
            }),
        );
        emit(
            &mut lines,
            "invalid.repository.longRange",
            throwable(&mut || {
                invalid_repository.get_structure_name(Some(&[b'a'.into()]), 0, 2);
                Ok(())
            }),
        );
        emit(
            &mut lines,
            "invalid.repository.negativeLen",
            throwable(&mut || {
                invalid_repository.get_structure_name(Some(&[b'a'.into()]), 0, -1);
                Ok(())
            }),
        );
        emit(
            &mut lines,
            "invalid.repository.state",
            repository_state(&invalid_repository),
        );

        let mut populated = StructureNamesRepository::new();
        populated.get_structure_name(Some(&[b'a'.into()]), 0, 1);
        emit(
            &mut lines,
            "invalid.repository.populatedNegativeLenDifferent",
            throwable(&mut || {
                populated.get_structure_name(Some(&[b'b'.into()]), 0, -1);
                Ok(())
            }),
        );
        emit(
            &mut lines,
            "invalid.repository.populatedNegativeLenEqualPrefix",
            throwable(&mut || {
                populated.get_structure_name(Some(&[b'a'.into()]), 0, -1);
                Ok(())
            }),
        );
        emit(
            &mut lines,
            "invalid.repository.populatedState",
            repository_state(&populated),
        );

        let (mut invalid_open, _) = handler();
        emit(
            &mut lines,
            "invalid.open.null",
            throwable(&mut || invalid_open.handle_open_element_start(None, 0, 0, 1, 1)),
        );
        emit(&mut lines, "invalid.open.state", state(&invalid_open));

        let (mut invalid_close, _) = handler();
        let mut x = java_chars("x");
        invalid_close
            .handle_open_element_start(Some(&mut x), 0, 1, 1, 1)
            .unwrap();
        emit(
            &mut lines,
            "invalid.close.null",
            throwable(&mut || invalid_close.handle_close_element_start(None, 0, 0, 1, 1)),
        );
        emit(&mut lines, "invalid.close.state", state(&invalid_close));

        let (mut invalid_attribute, _) = handler();
        emit(
            &mut lines,
            "invalid.attribute.null",
            throwable(&mut || attribute_buffer(&mut invalid_attribute, None, 0, 0, 1, 1)),
        );
        emit(
            &mut lines,
            "invalid.attribute.state",
            state(&invalid_attribute),
        );

        lines.join("\n") + "\n"
    }

    #[test]
    fn java_golden_matches_all_observable_event_processor_semantics() {
        assert_eq!(generate_golden(), JAVA_GOLDEN);
    }

    #[test]
    fn inherited_events_and_runtime_helpers_are_covered() {
        let (mut handler, _) = handler();
        handler.handle_document_start(1, 2, 3).unwrap();
        let mut buffer = java_chars("x");
        handler.handle_text(Some(&mut buffer), 0, 1, 1, 1).unwrap();
        handler
            .handle_comment(Some(&mut buffer), 0, 1, 0, 1, 1, 1)
            .unwrap();
        handler
            .handle_standalone_element_end(Some(&mut buffer), 0, 1, false, 1, 1)
            .unwrap();
        handler
            .handle_open_element_end(Some(&mut buffer), 0, 1, 1, 1)
            .unwrap();
        handler
            .handle_close_element_end(Some(&mut buffer), 0, 1, 1, 1)
            .unwrap();

        let first_null = catch_unwind(AssertUnwindSafe(|| {
            super::exact_equals(None, 0, 0, Some(&[]), 0, 0);
        }))
        .unwrap_err()
        .downcast::<EventProcessorTextHandlerRuntimeError>()
        .unwrap();
        assert_eq!(
            first_null.java_message().to_string_lossy(),
            "First text buffer being compared cannot be null"
        );
        assert_eq!(
            first_null.to_string(),
            "First text buffer being compared cannot be null"
        );
        assert!(!super::exact_equals(Some(&[1]), 0, 1, Some(&[1, 2]), 0, 2));
        assert!(!super::exact_equals(Some(&[1]), 0, 1, Some(&[2]), 0, 1));
        let index = catch_unwind(AssertUnwindSafe(|| {
            super::exact_equals(Some(&[1]), -1, 1, Some(&[1]), 0, 1);
        }))
        .unwrap_err()
        .downcast::<EventProcessorTextHandlerRuntimeError>()
        .unwrap();
        assert_eq!(
            index.java_message().to_string_lossy(),
            "Index -1 out of bounds for length 1"
        );

        for (buffer, len, expected) in [
            (None, 0, "Text cannot be null"),
            (Some(&[1_u16][..]), -1, "-1"),
        ] {
            let error = catch_unwind(AssertUnwindSafe(|| {
                super::copy_java_range(buffer, 0, len);
            }))
            .unwrap_err()
            .downcast::<EventProcessorTextHandlerRuntimeError>()
            .unwrap();
            assert_eq!(error.java_message().to_string_lossy(), expected);
        }

        let probe_state = Rc::new(RefCell::new(RecordingState::default()));
        let probe = RecordingHandler {
            state: Rc::clone(&probe_state),
        };
        probe.record("range", Some(&mut [1]), -1, 1).unwrap();
        assert_eq!(probe_state.borrow().events, "range@range(-1,1)");
        assert_eq!(throwable(&mut || Ok(())), "NO_ERROR");
        assert_eq!(
            throwable(&mut || Err(Box::new(TextParseException::new()))),
            "org.thymeleaf.templateparser.text.TextParseException;message=null;line=null;col=null"
        );

        let no_next = EventProcessorTextHandler::new(None);
        assert_eq!(no_next.element_stack.len(), 0);
    }
}
