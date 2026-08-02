use super::RawParseException;

/// RAW parser 产生文档与文本事件时调用的处理器合同。
///
/// 对应 Java: `org.thymeleaf.templateparser.raw.IRawHandler`。
///
/// 所有位置均直接沿用 parser 的一基行列；文本 buffer 是 UTF-16 `char[]` 的借用，
/// 不要求处理器预先创建中间字符串。
#[expect(
    clippy::result_large_err,
    reason = "公开 SPI 保留具体 RawParseException，避免破坏 Java checked exception 对照"
)]
/// 对应 Java: `IRawHandler`。
pub trait IRawHandler {
    /// 处理文档开始事件。
    ///
    /// # 参数
    ///
    /// - `start_time_nanos`：解析开始的纳秒时间戳。
    /// - `line`：事件行号。
    /// - `col`：事件列号。
    ///
    /// # 错误
    ///
    /// 处理失败时返回 RAW 解析异常。
    fn handle_document_start(
        &mut self,
        start_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), RawParseException>;

    /// 处理文档结束事件。
    ///
    /// # 参数
    ///
    /// - `end_time_nanos`：解析结束的纳秒时间戳。
    /// - `total_time_nanos`：解析总耗时。
    /// - `line`：事件行号。
    /// - `col`：事件列号。
    ///
    /// # 错误
    ///
    /// 处理失败时返回 RAW 解析异常。
    fn handle_document_end(
        &mut self,
        end_time_nanos: i64,
        total_time_nanos: i64,
        line: i32,
        col: i32,
    ) -> Result<(), RawParseException>;

    /// 处理 UTF-16 文本片段。
    ///
    /// # 参数
    ///
    /// - `buffer`：Java `char[]`；`None` 保留调用方传入 null 的可能性。
    /// - `offset`：片段起始下标。
    /// - `len`：片段长度。
    /// - `line`：片段起始行号。
    /// - `col`：片段起始列号。
    ///
    /// # 错误
    ///
    /// 处理失败或实现拒绝输入范围时返回 RAW 解析异常。
    fn handle_text(
        &mut self,
        buffer: Option<&[u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    ) -> Result<(), RawParseException>;
}
