use super::{CharSequenceValue, TextUtilsError};

/// IE 条件注释解析结果。
///
/// 对应 Java:
/// `org.thymeleaf.standard.util.StandardConditionalCommentUtils.ConditionalCommentParsingResult`。
///
/// 所有 offset/len 均以原始 Java `CharSequence` 的 UTF-16 code unit 为单位。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalCommentParsingResult {
    start_expression_offset: i32,
    start_expression_len: i32,
    content_offset: i32,
    content_len: i32,
    end_expression_offset: i32,
    end_expression_len: i32,
}

impl ConditionalCommentParsingResult {
    /// 使用六个 UTF-16 范围值创建解析结果。
    ///
    /// 对应 Java:
    /// `ConditionalCommentParsingResult#ConditionalCommentParsingResult(int,int,int,int,int,int)`。
    ///
    /// # 参数
    ///
    /// 参数按 Java 构造器顺序分别表示起始表达式、内容和结束表达式的 offset/len。
    ///
    /// # 返回
    ///
    /// 不执行额外校验、原样保存全部 Java `int` 的结果。
    #[must_use]
    pub const fn new(
        start_expression_offset: i32,
        start_expression_len: i32,
        content_offset: i32,
        content_len: i32,
        end_expression_offset: i32,
        end_expression_len: i32,
    ) -> Self {
        Self {
            start_expression_offset,
            start_expression_len,
            content_offset,
            content_len,
            end_expression_offset,
            end_expression_len,
        }
    }

    /// 返回起始条件表达式 offset。对应 Java `getStartExpressionOffset()`。
    #[must_use]
    pub const fn get_start_expression_offset(&self) -> i32 {
        self.start_expression_offset
    }

    /// 返回起始条件表达式长度。对应 Java `getStartExpressionLen()`。
    #[must_use]
    pub const fn get_start_expression_len(&self) -> i32 {
        self.start_expression_len
    }

    /// 返回条件注释内容 offset。对应 Java `getContentOffset()`。
    #[must_use]
    pub const fn get_content_offset(&self) -> i32 {
        self.content_offset
    }

    /// 返回条件注释内容长度。对应 Java `getContentLen()`。
    #[must_use]
    pub const fn get_content_len(&self) -> i32 {
        self.content_len
    }

    /// 返回结束条件表达式 offset。对应 Java `getEndExpressionOffset()`。
    #[must_use]
    pub const fn get_end_expression_offset(&self) -> i32 {
        self.end_expression_offset
    }

    /// 返回结束条件表达式长度。对应 Java `getEndExpressionLen()`。
    #[must_use]
    pub const fn get_end_expression_len(&self) -> i32 {
        self.end_expression_len
    }
}

/// Thymeleaf Standard Dialect 的 IE 条件注释解析工具。
///
/// 对应 Java: `org.thymeleaf.standard.util.StandardConditionalCommentUtils`。
///
/// 解析器不先验证完整 `<!--...-->` 外壳，而是严格从 UTF-16 offset 4 和
/// `length - 4` 两端按上游顺序扫描。格式不匹配返回 `None`；动态
/// `CharSequence#length/charAt` 的异常与调用顺序原样传播。
pub struct StandardConditionalCommentUtils {
    _private: (),
}

impl StandardConditionalCommentUtils {
    /// 尝试把文本解析为 IE 条件注释。
    ///
    /// 对应 Java:
    /// `StandardConditionalCommentUtils#parseConditionalComment(CharSequence)`。
    ///
    /// # 参数
    ///
    /// - `text`：原始动态 Java `CharSequence`；`None` 对应 Java `null`。
    ///
    /// # 返回
    ///
    /// 格式有效时返回三个 UTF-16 范围，无效格式返回 `None`。
    ///
    /// # 错误
    ///
    /// `text` 为 `None` 时返回 Java `NullPointerException` 等价错误；自定义
    /// `CharSequence` 的 length/charAt 异常原样传播。
    pub fn parse_conditional_comment(
        text: Option<&dyn CharSequenceValue>,
    ) -> Result<Option<ConditionalCommentParsingResult>, TextUtilsError> {
        let text = text.ok_or(TextUtilsError::NullPointer)?;
        let len = text.java_length()?;
        let mut i = 4;

        while i < len && is_java_whitespace(text.java_char_at(i)?) {
            i += 1;
        }
        if i >= len || text.java_char_at(i)? != u16::from(b'[') {
            return Ok(None);
        }
        i += 1;
        let start_expression_offset = i;

        while i < len && text.java_char_at(i)? != u16::from(b']') {
            i += 1;
        }
        if i >= len {
            return Ok(None);
        }
        let start_expression_len = i - start_expression_offset;
        i += 1;

        while i < len && is_java_whitespace(text.java_char_at(i)?) {
            i += 1;
        }
        if i >= len || text.java_char_at(i)? != u16::from(b'>') {
            return Ok(None);
        }
        i += 1;
        let content_offset = i;

        i = len.wrapping_sub(4);
        while i > content_offset && is_java_whitespace(text.java_char_at(i)?) {
            i -= 1;
        }
        if i <= content_offset || text.java_char_at(i)? != u16::from(b']') {
            return Ok(None);
        }
        i -= 1;
        let end_expression_last_pos = i + 1;

        while i > content_offset && text.java_char_at(i)? != u16::from(b'[') {
            i -= 1;
        }
        if i <= content_offset {
            return Ok(None);
        }
        let end_expression_offset = i + 1;
        let end_expression_len = end_expression_last_pos - end_expression_offset;
        i -= 1;

        while i >= content_offset && is_java_whitespace(text.java_char_at(i)?) {
            i -= 1;
        }
        if i <= content_offset || text.java_char_at(i)? != u16::from(b'!') {
            return Ok(None);
        }
        i -= 1;
        if i <= content_offset || text.java_char_at(i)? != u16::from(b'<') {
            return Ok(None);
        }
        i -= 1;

        let content_len = (i + 1) - content_offset;
        if content_len <= 0 || start_expression_len <= 0 || end_expression_len <= 0 {
            return Ok(None);
        }

        Ok(Some(ConditionalCommentParsingResult::new(
            start_expression_offset,
            start_expression_len,
            content_offset,
            content_len,
            end_expression_offset,
            end_expression_len,
        )))
    }
}

fn is_java_whitespace(character: u16) -> bool {
    matches!(
        character,
        0x0009..=0x000D
            | 0x001C..=0x0020
            | 0x1680
            | 0x2000..=0x2006
            | 0x2008..=0x200A
            | 0x2028
            | 0x2029
            | 0x205F
            | 0x3000
    )
}
