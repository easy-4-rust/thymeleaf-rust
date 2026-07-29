#![cfg_attr(
    not(test),
    expect(dead_code, reason = "text parser 消费者对象将在后续切片中迁移")
)]

/// 文本模板解析器的可变扫描状态。
///
/// 对应 Java: `org.thymeleaf.templateparser.text.TextParseStatus`。
///
/// 上游类型、构造器和全部字段均为 package-private。Rust 保持 crate 内可见性，
/// 供后续 text parser 对象直接按扫描进度修改，不增加公共 getter/setter。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct TextParseStatus {
    /// 当前输入 offset；对应 Java `offset`。
    pub(crate) offset: i32,
    /// 当前行号；对应 Java `line`。
    pub(crate) line: i32,
    /// 当前列号；对应 Java `col`。
    pub(crate) col: i32,
    /// 当前是否处于结构内部；对应 Java `inStructure`。
    pub(crate) in_structure: bool,
    /// 当前是否处于行注释内部；对应 Java `inCommentLine`。
    pub(crate) in_comment_line: bool,
    /// 当前字面量定界符；对应 Java UTF-16 `literalMarker`。
    pub(crate) literal_marker: u16,
}

impl TextParseStatus {
    /// 创建所有字段均为 Java 默认零值的新状态。
    ///
    /// 对应 Java: `TextParseStatus#TextParseStatus()`。
    ///
    /// # 返回
    /// offset/line/col 为 0、两个布尔值为 false、literal marker 为 NUL 的独立状态。
    pub(crate) const fn new() -> Self {
        Self {
            offset: 0,
            line: 0,
            col: 0,
            in_structure: false,
            in_comment_line: false,
            literal_marker: 0,
        }
    }
}
