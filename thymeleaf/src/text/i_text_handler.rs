use super::TextParseException;

/// 文本模板解析事件处理器。
///
/// 对应 Java: `org.thymeleaf.templateparser.text.ITextHandler`。
///
/// 解析器按文档、文本、注释、元素边界和属性的出现顺序同步回调本接口。所有
/// `buffer` 都是同一 Java `char[]` 的 UTF-16 代码单元视图；`None` 精确表示
/// Java `null`。实现可以修改数组，该修改会被调用方继续观察。实现返回的
/// [`TextParseException`] 原样向上传播。
pub trait ITextHandler {
    /// 处理文档开始事件。对应 Java: `ITextHandler#handleDocumentStart`。
    ///
    /// `start_time_nanos` 为开始纳秒时间，`line`、`col` 为当前位置。
    fn handle_document_start(
        &mut self,
        start_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>>;

    /// 处理文档结束事件。对应 Java: `ITextHandler#handleDocumentEnd`。
    ///
    /// `end_time_nanos` 为结束时间，`total_time_nanos` 为总耗时；成功时返回空值。
    fn handle_document_end(
        &mut self,
        end_time_nanos: i64,
        total_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>>;

    /// 处理普通文本片段。对应 Java: `ITextHandler#handleText`。
    ///
    /// `offset` 和 `len` 指定 `buffer` 中的原始片段，`line`、`col` 为片段位置。
    fn handle_text(
        &mut self,
        buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>>;

    /// 处理注释。对应 Java: `ITextHandler#handleComment`。
    ///
    /// content 范围排除 `/*` 与 `*/`，outer 范围包含它们；成功时返回空值。
    #[allow(clippy::too_many_arguments)]
    fn handle_comment(
        &mut self,
        buffer: Option<&mut [u16]>,
        content_offset: i32,
        content_len: i32,
        outer_offset: i32,
        outer_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>>;

    /// 处理独立元素开始边界。对应 Java: `ITextHandler#handleStandaloneElementStart`。
    #[allow(clippy::too_many_arguments)]
    fn handle_standalone_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        minimized: bool,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>>;

    /// 处理独立元素结束边界。对应 Java: `ITextHandler#handleStandaloneElementEnd`。
    #[allow(clippy::too_many_arguments)]
    fn handle_standalone_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        minimized: bool,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>>;

    /// 处理开放元素开始边界。对应 Java: `ITextHandler#handleOpenElementStart`。
    fn handle_open_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>>;

    /// 处理开放元素结束边界。对应 Java: `ITextHandler#handleOpenElementEnd`。
    fn handle_open_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>>;

    /// 处理关闭元素开始边界。对应 Java: `ITextHandler#handleCloseElementStart`。
    fn handle_close_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>>;

    /// 处理关闭元素结束边界。对应 Java: `ITextHandler#handleCloseElementEnd`。
    fn handle_close_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), Box<TextParseException>>;

    /// 处理属性的名称、运算符和值范围。对应 Java: `ITextHandler#handleAttribute`。
    ///
    /// 所有 offset/len 均指向同一 `buffer`；名称、运算符和值分别携带源位置，
    /// value content 排除定界符，value outer 包含定界符。
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<(), Box<TextParseException>>;
}
