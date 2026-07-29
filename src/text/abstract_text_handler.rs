use super::{ITextHandler, TextParseException};

/// 文本解析事件的默认空操作处理器。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.text.AbstractTextHandler`。
///
/// 上游抽象基类为 [`ITextHandler`] 的全部十一个回调提供空实现，供子类只覆盖
/// 关心的事件。Rust 不模拟继承；本零状态对象保留所有默认行为，而定制处理器直接
/// 实现 [`ITextHandler`]。回调不会读取参数，因此允许 Java `null` 对应的
/// `None`、非法 offset/len、极值时间与位置，并始终成功返回。私有标记让不同
/// 实例保持普通对象的独立地址语义，不把 Java 引用对象压缩成 Rust 零大小单例。
#[derive(Debug)]
pub struct AbstractTextHandler {
    _object_marker: bool,
}

impl AbstractTextHandler {
    /// 创建默认空操作处理器。
    ///
    /// 对应 Java:
    /// `AbstractTextHandler#AbstractTextHandler()`。
    ///
    /// Java 构造器为 `protected` 且类型为抽象类；Rust 将继承用途映射为可组合的
    /// 零状态值，调用方通过 [`ITextHandler`] trait 实现覆盖行为。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _object_marker: false,
        }
    }
}

impl Default for AbstractTextHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ITextHandler for AbstractTextHandler {
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
        Ok(())
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
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        _minimized: bool,
        _line: i32,
        _col: i32,
    ) -> Result<(), Box<TextParseException>> {
        Ok(())
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
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        _line: i32,
        _col: i32,
    ) -> Result<(), Box<TextParseException>> {
        Ok(())
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
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
        _line: i32,
        _col: i32,
    ) -> Result<(), Box<TextParseException>> {
        Ok(())
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
        _buffer: Option<&mut [u16]>,
        _name_offset: i32,
        _name_len: i32,
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
        Ok(())
    }
}
