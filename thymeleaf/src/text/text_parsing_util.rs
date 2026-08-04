use std::error::Error;
use std::fmt::{Display, Formatter};

use super::{ParsingLocatorUtil, parsing_locator_util::ParsingLocatorError};
use crate::util::Utf16String;

/// 通用文本扫描中的 Java 运行时异常适配。
///
/// 对应 Java: `org.thymeleaf.templateparser.text.TextParsingUtil` 的数组访问失败。
///
/// Java 17 对本对象的 `text` null 数组访问不生成消息；前五个方法直接访问第四个
/// 参数 locator，增强 NPE 指向 `<parameter4>`；后五个方法经
/// `ParsingLocatorUtil#countChar` 访问 locator，消息指向 `<parameter1>`。
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TextParsingUtilError {
    /// `text` 为 null；Java `getMessage()` 为 null。
    NullText,
    /// 前五个扫描入口直接访问 null locator。
    NullDirectLocator,
    /// 后五个扫描入口经 `ParsingLocatorUtil` 访问 null locator。
    NullCountCharLocator,
    /// `char[]` 下标越界。
    TextArrayIndex {
        /// 实际访问下标。
        index: i32,
        /// 数组长度。
        length: usize,
    },
    /// `int[]` 下标越界。
    LocatorArrayIndex {
        /// 实际访问下标。
        index: usize,
        /// 数组长度。
        length: usize,
    },
}

impl TextParsingUtilError {
    /// 返回对应 Java 异常全限定名。
    pub(crate) const fn class_name(&self) -> &'static str {
        match self {
            Self::NullText | Self::NullDirectLocator | Self::NullCountCharLocator => {
                "java.lang.NullPointerException"
            }
            Self::TextArrayIndex { .. } | Self::LocatorArrayIndex { .. } => {
                "java.lang.ArrayIndexOutOfBoundsException"
            }
        }
    }

    /// 返回 Java `Throwable#getMessage()`。
    ///
    /// `None` 仅对应 null text 的无消息 NPE。
    /// 对应 Java 语义：`TextParsingUtil` 的 `message` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn message(&self) -> Option<Utf16String> {
        let message = match self {
            Self::NullText => return None,
            Self::NullDirectLocator => {
                "Cannot load from int array because \"<parameter4>\" is null".to_owned()
            }
            Self::NullCountCharLocator => {
                "Cannot load from int array because \"<parameter1>\" is null".to_owned()
            }
            Self::TextArrayIndex { index, length } => {
                format!("Index {index} out of bounds for length {length}")
            }
            Self::LocatorArrayIndex { index, length } => {
                format!("Index {index} out of bounds for length {length}")
            }
        };
        Some(Utf16String::from_rust_str(&message))
    }
}

impl Display for TextParsingUtilError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(
            &self
                .message()
                .map_or_else(|| "null".to_owned(), |message| message.to_string_lossy()),
        )
    }
}

impl Error for TextParsingUtilError {}

/// 文本模板解析器的通用 UTF-16 扫描工具。
///
/// 对应 Java: `org.thymeleaf.templateparser.text.TextParsingUtil`。
///
/// 本对象逐语句迁移结构、注释、字面量、空白、运算符和引号范围扫描。所有下标、
/// 计数与定位运算使用 Java `int` 回绕；返回命中位置或 `-1`，且严格保留命中字符
/// 是否计入 locator、换行更新顺序、引号切换和反斜杠奇偶判定。
pub(crate) struct TextParsingUtil;

impl TextParsingUtil {
    /// 查找不在双引号或单引号中的下一个 `]`。
    ///
    /// 对应 Java: `TextParsingUtil#findNextStructureEndAvoidQuotes`。
    pub(crate) fn find_next_structure_end_avoid_quotes(
        text: Option<&[u16]>,
        offset: i32,
        maxi: i32,
        locator: Option<&mut [i32]>,
    ) -> Result<i32, TextParsingUtilError> {
        let mut locator = locator;
        let mut in_quotes = false;
        let mut in_apos = false;
        let mut col_index = offset;
        let mut index = offset;
        let mut remaining = maxi.wrapping_sub(offset);

        while remaining != 0 {
            remaining = remaining.wrapping_sub(1);
            let character = text_unit(text, index)?.character;
            if character == u16::from(b'\n') {
                col_index = index;
                count_line_feed_direct(&mut locator)?;
            } else if character == u16::from(b'"') && !in_apos {
                in_quotes = !in_quotes;
            } else if character == u16::from(b'\'') && !in_quotes {
                in_apos = !in_apos;
            } else if character == u16::from(b']') && !in_quotes && !in_apos {
                locator_add_direct(&mut locator, 1, index.wrapping_sub(col_index))?;
                return Ok(index);
            }
            index = index.wrapping_add(1);
        }

        locator_add_direct(&mut locator, 1, maxi.wrapping_sub(col_index))?;
        Ok(-1)
    }

    /// 查找块注释闭合 `*/` 中 `/` 的位置。
    ///
    /// 对应 Java: `TextParsingUtil#findNextCommentBlockEnd`。
    pub(crate) fn find_next_comment_block_end(
        text: Option<&[u16]>,
        offset: i32,
        maxi: i32,
        locator: Option<&mut [i32]>,
    ) -> Result<i32, TextParsingUtilError> {
        let mut locator = locator;
        let mut col_index = offset;
        let mut index = offset;
        let mut remaining = maxi.wrapping_sub(offset);

        while remaining != 0 {
            remaining = remaining.wrapping_sub(1);
            let unit = text_unit(text, index)?;
            let character = unit.character;
            if character == u16::from(b'\n') {
                col_index = index;
                count_line_feed_direct(&mut locator)?;
            } else if index > offset
                && character == u16::from(b'/')
                && unit.text[index.wrapping_sub(1) as usize] == u16::from(b'*')
            {
                locator_add_direct(&mut locator, 1, index.wrapping_sub(col_index))?;
                return Ok(index);
            }
            index = index.wrapping_add(1);
        }

        locator_add_direct(&mut locator, 1, maxi.wrapping_sub(col_index))?;
        Ok(-1)
    }

    /// 查找行注释结束 LF，不消费该 LF。
    ///
    /// 对应 Java: `TextParsingUtil#findNextCommentLineEnd`。
    pub(crate) fn find_next_comment_line_end(
        text: Option<&[u16]>,
        offset: i32,
        maxi: i32,
        locator: Option<&mut [i32]>,
    ) -> Result<i32, TextParsingUtilError> {
        let mut locator = locator;
        let col_index = offset;
        let mut index = offset;
        let mut remaining = maxi.wrapping_sub(offset);

        while remaining != 0 {
            remaining = remaining.wrapping_sub(1);
            if text_unit(text, index)?.character == u16::from(b'\n') {
                locator_add_direct(&mut locator, 1, index.wrapping_sub(col_index))?;
                return Ok(index);
            }
            index = index.wrapping_add(1);
        }

        locator_add_direct(&mut locator, 1, maxi.wrapping_sub(col_index))?;
        Ok(-1)
    }

    /// 查找未被奇数个反斜杠转义的字面量结束标记。
    ///
    /// 对应 Java: `TextParsingUtil#findNextLiteralEnd`。
    pub(crate) fn find_next_literal_end(
        text: Option<&[u16]>,
        offset: i32,
        maxi: i32,
        locator: Option<&mut [i32]>,
        literal_marker: u16,
    ) -> Result<i32, TextParsingUtilError> {
        let mut locator = locator;
        let mut col_index = offset;
        let mut index = offset;
        let mut remaining = maxi.wrapping_sub(offset);

        while remaining != 0 {
            remaining = remaining.wrapping_sub(1);
            let unit = text_unit(text, index)?;
            let character = unit.character;
            if character == u16::from(b'\n') {
                col_index = index;
                count_line_feed_direct(&mut locator)?;
            } else if index > offset
                && character == literal_marker
                && is_literal_delimiter(unit.text, offset as usize, index as usize)
            {
                locator_add_direct(&mut locator, 1, index.wrapping_sub(col_index))?;
                return Ok(index);
            }
            index = index.wrapping_add(1);
        }

        locator_add_direct(&mut locator, 1, maxi.wrapping_sub(col_index))?;
        Ok(-1)
    }

    /// 查找元素开始 `[` 或启用时的注释/字面量标记。
    ///
    /// 对应 Java: `TextParsingUtil#findNextStructureStartOrLiteralMarker`。
    pub(crate) fn find_next_structure_start_or_literal_marker(
        text: Option<&[u16]>,
        offset: i32,
        maxi: i32,
        locator: Option<&mut [i32]>,
        process_comments_and_literals: bool,
    ) -> Result<i32, TextParsingUtilError> {
        let mut locator = locator;
        let mut col_index = offset;
        let mut index = offset;
        let mut remaining = maxi.wrapping_sub(offset);

        while remaining != 0 {
            remaining = remaining.wrapping_sub(1);
            let unit = text_unit(text, index)?;
            let character = unit.character;
            if character == u16::from(b'\n') {
                col_index = index;
                count_line_feed_direct(&mut locator)?;
            } else if character == u16::from(b'[') {
                locator_add_direct(&mut locator, 1, index.wrapping_sub(col_index))?;
                return Ok(index);
            } else if process_comments_and_literals {
                if character == u16::from(b'/') {
                    locator_add_direct(&mut locator, 1, index.wrapping_sub(col_index))?;
                    return Ok(index);
                }
                if matches!(character, 0x0027 | 0x0022 | 0x0060)
                    && is_literal_delimiter(unit.text, offset as usize, index as usize)
                {
                    locator_add_direct(&mut locator, 1, index.wrapping_sub(col_index))?;
                    return Ok(index);
                }
            }
            index = index.wrapping_add(1);
        }

        locator_add_direct(&mut locator, 1, maxi.wrapping_sub(col_index))?;
        Ok(-1)
    }

    /// 查找下一个空白字符，可选择忽略引号内部空白。
    ///
    /// 对应 Java: `TextParsingUtil#findNextWhitespaceCharWildcard`。
    pub(crate) fn find_next_whitespace_char_wildcard(
        text: Option<&[u16]>,
        offset: i32,
        maxi: i32,
        avoid_quotes: bool,
        locator: Option<&mut [i32]>,
    ) -> Result<i32, TextParsingUtilError> {
        let mut locator = locator;
        let mut in_quotes = false;
        let mut in_apos = false;
        let mut index = offset;
        let mut remaining = maxi.wrapping_sub(offset);

        while remaining != 0 {
            remaining = remaining.wrapping_sub(1);
            let character = text_unit(text, index)?.character;
            if avoid_quotes && !in_apos && character == u16::from(b'"') {
                in_quotes = !in_quotes;
            } else if avoid_quotes && !in_quotes && character == u16::from(b'\'') {
                in_apos = !in_apos;
            } else if !in_quotes && !in_apos && is_wildcard_whitespace(character) {
                return Ok(index);
            }
            count_char(&mut locator, character)?;
            index = index.wrapping_add(1);
        }
        Ok(-1)
    }

    /// 查找下一个非空白字符。
    ///
    /// 对应 Java: `TextParsingUtil#findNextNonWhitespaceCharWildcard`。
    pub(crate) fn find_next_non_whitespace_char_wildcard(
        text: Option<&[u16]>,
        offset: i32,
        maxi: i32,
        locator: Option<&mut [i32]>,
    ) -> Result<i32, TextParsingUtilError> {
        let mut locator = locator;
        let mut index = offset;
        let mut remaining = maxi.wrapping_sub(offset);
        while remaining != 0 {
            remaining = remaining.wrapping_sub(1);
            let character = text_unit(text, index)?.character;
            if !is_wildcard_whitespace(character) {
                return Ok(index);
            }
            count_char(&mut locator, character)?;
            index = index.wrapping_add(1);
        }
        Ok(-1)
    }

    /// 查找等号或空白运算符字符。
    ///
    /// 对应 Java: `TextParsingUtil#findNextOperatorCharWildcard`。
    pub(crate) fn find_next_operator_char_wildcard(
        text: Option<&[u16]>,
        offset: i32,
        maxi: i32,
        locator: Option<&mut [i32]>,
    ) -> Result<i32, TextParsingUtilError> {
        let mut locator = locator;
        let mut index = offset;
        let mut remaining = maxi.wrapping_sub(offset);
        while remaining != 0 {
            remaining = remaining.wrapping_sub(1);
            let character = text_unit(text, index)?.character;
            if character == u16::from(b'=') || is_wildcard_whitespace(character) {
                return Ok(index);
            }
            count_char(&mut locator, character)?;
            index = index.wrapping_add(1);
        }
        Ok(-1)
    }

    /// 查找既非等号也非空白的字符。
    ///
    /// 对应 Java: `TextParsingUtil#findNextNonOperatorCharWildcard`。
    pub(crate) fn find_next_non_operator_char_wildcard(
        text: Option<&[u16]>,
        offset: i32,
        maxi: i32,
        locator: Option<&mut [i32]>,
    ) -> Result<i32, TextParsingUtilError> {
        let mut locator = locator;
        let mut index = offset;
        let mut remaining = maxi.wrapping_sub(offset);
        while remaining != 0 {
            remaining = remaining.wrapping_sub(1);
            let character = text_unit(text, index)?.character;
            if character != u16::from(b'=') && !is_wildcard_whitespace(character) {
                return Ok(index);
            }
            count_char(&mut locator, character)?;
            index = index.wrapping_add(1);
        }
        Ok(-1)
    }

    /// 跳过开头的一个完整引号范围并返回其后字符。
    ///
    /// 对应 Java: `TextParsingUtil#findNextAnyCharAvoidQuotesWildcard`。
    pub(crate) fn find_next_any_char_avoid_quotes_wildcard(
        text: Option<&[u16]>,
        offset: i32,
        maxi: i32,
        locator: Option<&mut [i32]>,
    ) -> Result<i32, TextParsingUtilError> {
        let mut index = offset;
        let mut remaining = maxi.wrapping_sub(offset);
        if remaining == 0 {
            return Ok(-1);
        }

        remaining = remaining.wrapping_sub(1);
        let opening = text_unit(text, index)?.character;
        let quote = if matches!(opening, 0x0022 | 0x0027) {
            opening
        } else {
            return Ok(index);
        };

        // 首个引号的计数成功后，locator 必然至少含两个元素；后续 Java 数组访问不会失败。
        let locator = count_opening_quote_and_validate(locator)?;
        index = index.wrapping_add(1);

        while remaining != 0 {
            remaining = remaining.wrapping_sub(1);
            let character = text_unit(text, index)?.character;
            count_char_validated(locator, character);
            index = index.wrapping_add(1);
            if character == quote {
                return Ok(if index < maxi { index } else { -1 });
            }
        }
        Ok(-1)
    }
}

fn is_literal_delimiter(text: &[u16], offset: usize, index: usize) -> bool {
    let mut escapes = 0_i32;
    let mut cursor = index;
    while cursor > offset && text[cursor - 1] == u16::from(b'\\') {
        escapes = escapes.wrapping_add(1);
        cursor -= 1;
    }
    escapes % 2 == 0
}

struct TextUnit<'a> {
    text: &'a [u16],
    character: u16,
}

fn text_unit(text: Option<&[u16]>, index: i32) -> Result<TextUnit<'_>, TextParsingUtilError> {
    let text = text.ok_or(TextParsingUtilError::NullText)?;
    let character = usize::try_from(index)
        .ok()
        .and_then(|index| text.get(index).copied())
        .ok_or(TextParsingUtilError::TextArrayIndex {
            index,
            length: text.len(),
        })?;
    Ok(TextUnit { text, character })
}

fn count_line_feed_direct(locator: &mut Option<&mut [i32]>) -> Result<(), TextParsingUtilError> {
    let locator = locator
        .as_deref_mut()
        .ok_or(TextParsingUtilError::NullDirectLocator)?;
    let length = locator.len();
    let column = locator
        .get_mut(1)
        .ok_or(TextParsingUtilError::LocatorArrayIndex { index: 1, length })?;
    *column = 0;
    locator[0] = locator[0].wrapping_add(1);
    Ok(())
}

fn locator_add_direct(
    locator: &mut Option<&mut [i32]>,
    index: usize,
    amount: i32,
) -> Result<(), TextParsingUtilError> {
    let locator = locator
        .as_deref_mut()
        .ok_or(TextParsingUtilError::NullDirectLocator)?;
    let length = locator.len();
    let slot = locator
        .get_mut(index)
        .ok_or(TextParsingUtilError::LocatorArrayIndex { index, length })?;
    *slot = slot.wrapping_add(amount);
    Ok(())
}

fn count_char(
    locator: &mut Option<&mut [i32]>,
    character: u16,
) -> Result<(), TextParsingUtilError> {
    ParsingLocatorUtil::count_char(locator.as_deref_mut(), character).map_err(|error| match error {
        ParsingLocatorError::NullLocator => TextParsingUtilError::NullCountCharLocator,
        ParsingLocatorError::ArrayIndex { index, length } => {
            TextParsingUtilError::LocatorArrayIndex { index, length }
        }
    })
}

fn count_opening_quote_and_validate(
    locator: Option<&mut [i32]>,
) -> Result<&mut [i32], TextParsingUtilError> {
    let locator = locator.ok_or(TextParsingUtilError::NullCountCharLocator)?;
    let length = locator.len();
    let column = locator
        .get_mut(1)
        .ok_or(TextParsingUtilError::LocatorArrayIndex { index: 1, length })?;
    *column = column.wrapping_add(1);
    Ok(locator)
}

fn count_char_validated(locator: &mut [i32], character: u16) {
    if character == u16::from(b'\n') {
        locator[0] = locator[0].wrapping_add(1);
        locator[1] = 1;
    } else {
        locator[1] = locator[1].wrapping_add(1);
    }
}

fn is_wildcard_whitespace(character: u16) -> bool {
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

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use super::{TextParsingUtil, TextParsingUtilError};
    use crate::util::Utf16String;

    const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    const JAVA_GOLDEN: &str = include_str!("../../tests/fixtures/text_parsing_util_golden.txt");
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x100_0000_01b3;

    #[derive(Clone, Copy)]
    enum Operation {
        StructureEnd,
        CommentBlockEnd,
        CommentLineEnd,
        LiteralEnd,
        StructureStart,
        Whitespace,
        NonWhitespace,
        Operator,
        NonOperator,
        AnyAvoidQuotes,
    }

    #[test]
    fn text_parsing_util_matches_java_golden() {
        let mut output = String::new();
        emit(&mut output, "baseline", JAVA_BASELINE);
        fixed_cases(&mut output);
        runtime_cases(&mut output);
        exhaustive_cases(&mut output);
        assert_eq!(output, JAVA_GOLDEN);

        assert_eq!(TextParsingUtilError::NullText.to_string(), "null");
        assert_eq!(
            TextParsingUtilError::NullDirectLocator.to_string(),
            "Cannot load from int array because \"<parameter4>\" is null"
        );
    }

    #[test]
    fn error_propagation_preserves_java_access_order() {
        let newline = [u16::from(b'\n')];
        let plain = [u16::from(b'x')];
        let structure_end = [u16::from(b']')];
        let slash = [u16::from(b'/')];
        let quote = [u16::from(b'\'')];

        for operation in [
            Operation::StructureEnd,
            Operation::CommentBlockEnd,
            Operation::LiteralEnd,
            Operation::StructureStart,
        ] {
            let mut one = [7];
            assert_eq!(
                invoke(
                    operation,
                    Some(&newline),
                    0,
                    1,
                    Some(&mut one),
                    true,
                    u16::from(b'\''),
                ),
                Err(TextParsingUtilError::LocatorArrayIndex {
                    index: 1,
                    length: 1,
                })
            );
            assert_eq!(one, [7]);
        }
        assert_eq!(
            TextParsingUtil::find_next_structure_end_avoid_quotes(Some(&newline), 0, 1, None),
            Err(TextParsingUtilError::NullDirectLocator)
        );

        for operation in [
            Operation::StructureEnd,
            Operation::CommentBlockEnd,
            Operation::CommentLineEnd,
            Operation::LiteralEnd,
            Operation::StructureStart,
        ] {
            assert_eq!(
                invoke(operation, Some(&plain), 0, 0, None, true, u16::from(b'\'')),
                Err(TextParsingUtilError::NullDirectLocator)
            );
        }

        assert_eq!(
            TextParsingUtil::find_next_structure_start_or_literal_marker(
                Some(&slash),
                0,
                1,
                None,
                true,
            ),
            Err(TextParsingUtilError::NullDirectLocator)
        );
        assert_eq!(
            TextParsingUtil::find_next_structure_start_or_literal_marker(
                Some(&quote),
                0,
                1,
                None,
                true,
            ),
            Err(TextParsingUtilError::NullDirectLocator)
        );
        let mut empty = [];
        assert_eq!(
            TextParsingUtil::find_next_structure_end_avoid_quotes(
                Some(&structure_end),
                0,
                1,
                Some(&mut empty),
            ),
            Err(TextParsingUtilError::LocatorArrayIndex {
                index: 1,
                length: 0,
            })
        );
        let mut one = [7];
        assert_eq!(
            TextParsingUtil::find_next_any_char_avoid_quotes_wildcard(
                Some(&quote),
                0,
                1,
                Some(&mut one),
            ),
            Err(TextParsingUtilError::LocatorArrayIndex {
                index: 1,
                length: 1,
            })
        );
        assert_eq!(one, [7]);
        assert_eq!(
            TextParsingUtil::find_next_any_char_avoid_quotes_wildcard(None, 0, 0, None),
            Ok(-1)
        );
        let mut locator = [1, 1];
        assert_eq!(
            TextParsingUtil::find_next_any_char_avoid_quotes_wildcard(
                Some(&quote),
                0,
                2,
                Some(&mut locator),
            ),
            Err(TextParsingUtilError::TextArrayIndex {
                index: 1,
                length: 1,
            })
        );
        assert_eq!(locator, [1, 2]);

        assert_eq!(
            TextParsingUtilError::NullCountCharLocator.to_string(),
            "Cannot load from int array because \"<parameter1>\" is null"
        );
        assert_eq!(
            TextParsingUtilError::TextArrayIndex {
                index: -1,
                length: 1,
            }
            .to_string(),
            "Index -1 out of bounds for length 1"
        );
        assert_eq!(
            TextParsingUtilError::LocatorArrayIndex {
                index: 1,
                length: 0,
            }
            .to_string(),
            "Index 1 out of bounds for length 0"
        );
    }

    fn fixed_cases(output: &mut String) {
        for (key, text, offset, maxi, locator, operation, flag, marker) in [
            (
                "structure.basic",
                "abc]z",
                0,
                5,
                [1, 2],
                Operation::StructureEnd,
                false,
                '\'',
            ),
            (
                "structure.quotes",
                "\"a]b\"]",
                0,
                6,
                [1, 1],
                Operation::StructureEnd,
                false,
                '\'',
            ),
            (
                "structure.apos",
                "'a]b']",
                0,
                6,
                [1, 1],
                Operation::StructureEnd,
                false,
                '\'',
            ),
            (
                "structure.crossQuotes",
                "\"a']b\"]",
                0,
                7,
                [1, 1],
                Operation::StructureEnd,
                false,
                '\'',
            ),
            (
                "structure.newline",
                "a\nb]c",
                0,
                5,
                [4, 9],
                Operation::StructureEnd,
                false,
                '\'',
            ),
            (
                "structure.missing",
                "abc",
                0,
                3,
                [1, 2],
                Operation::StructureEnd,
                false,
                '\'',
            ),
            (
                "structure.range",
                "xx]yy",
                2,
                4,
                [2, 7],
                Operation::StructureEnd,
                false,
                '\'',
            ),
            (
                "block.basic",
                "ab*/c",
                0,
                5,
                [1, 1],
                Operation::CommentBlockEnd,
                false,
                '\'',
            ),
            (
                "block.firstSlash",
                "/abc",
                0,
                4,
                [1, 1],
                Operation::CommentBlockEnd,
                false,
                '\'',
            ),
            (
                "block.newline",
                "a\n*/",
                0,
                4,
                [3, 8],
                Operation::CommentBlockEnd,
                false,
                '\'',
            ),
            (
                "block.missing",
                "a*b",
                0,
                3,
                [1, 2],
                Operation::CommentBlockEnd,
                false,
                '\'',
            ),
            (
                "line.basic",
                "ab\nc",
                0,
                4,
                [1, 2],
                Operation::CommentLineEnd,
                false,
                '\'',
            ),
            (
                "line.firstLf",
                "\nabc",
                0,
                4,
                [5, 7],
                Operation::CommentLineEnd,
                false,
                '\'',
            ),
            (
                "line.missing",
                "abc",
                0,
                3,
                [1, 2],
                Operation::CommentLineEnd,
                false,
                '\'',
            ),
            (
                "literal.basic",
                "a'b",
                0,
                3,
                [1, 1],
                Operation::LiteralEnd,
                false,
                '\'',
            ),
            (
                "literal.escaped",
                "a\\'b'c",
                0,
                6,
                [1, 1],
                Operation::LiteralEnd,
                false,
                '\'',
            ),
            (
                "literal.evenEscapes",
                "a\\\\'b",
                0,
                5,
                [1, 1],
                Operation::LiteralEnd,
                false,
                '\'',
            ),
            (
                "literal.firstMarker",
                "'a'",
                0,
                3,
                [1, 1],
                Operation::LiteralEnd,
                false,
                '\'',
            ),
            (
                "literal.newline",
                "a\nb`c",
                0,
                5,
                [2, 8],
                Operation::LiteralEnd,
                false,
                '`',
            ),
            (
                "literal.missing",
                "abc",
                0,
                3,
                [1, 2],
                Operation::LiteralEnd,
                false,
                '"',
            ),
            (
                "start.element",
                "ab[c",
                0,
                4,
                [1, 1],
                Operation::StructureStart,
                false,
                '\'',
            ),
            (
                "start.slashEnabled",
                "ab/c",
                0,
                4,
                [1, 1],
                Operation::StructureStart,
                true,
                '\'',
            ),
            (
                "start.slashDisabled",
                "ab/c",
                0,
                4,
                [1, 1],
                Operation::StructureStart,
                false,
                '\'',
            ),
            (
                "start.quoteEnabled",
                "ab'c",
                0,
                4,
                [1, 1],
                Operation::StructureStart,
                true,
                '\'',
            ),
            (
                "start.quoteEscaped",
                "a\\'c",
                0,
                4,
                [1, 1],
                Operation::StructureStart,
                true,
                '\'',
            ),
            (
                "start.quoteEvenEscapes",
                "a\\\\'c",
                0,
                5,
                [1, 1],
                Operation::StructureStart,
                true,
                '\'',
            ),
            (
                "start.backtick",
                "ab`c",
                0,
                4,
                [1, 1],
                Operation::StructureStart,
                true,
                '\'',
            ),
            (
                "start.newline",
                "a\n/b",
                0,
                4,
                [2, 8],
                Operation::StructureStart,
                true,
                '\'',
            ),
            (
                "start.missing",
                "abc",
                0,
                3,
                [1, 2],
                Operation::StructureStart,
                true,
                '\'',
            ),
            (
                "whitespace.basic",
                "ab cd",
                0,
                5,
                [1, 1],
                Operation::Whitespace,
                false,
                '\'',
            ),
            (
                "whitespace.quoted",
                "\"a b\" c",
                0,
                7,
                [1, 1],
                Operation::Whitespace,
                true,
                '\'',
            ),
            (
                "whitespace.apos",
                "'a b' c",
                0,
                7,
                [1, 1],
                Operation::Whitespace,
                true,
                '\'',
            ),
            (
                "whitespace.noAvoid",
                "\"a b\"",
                0,
                5,
                [1, 1],
                Operation::Whitespace,
                false,
                '\'',
            ),
            (
                "whitespace.unclosed",
                "\"a b",
                0,
                4,
                [1, 1],
                Operation::Whitespace,
                true,
                '\'',
            ),
            (
                "whitespace.unicode",
                "a\u{3000}b",
                0,
                3,
                [1, 1],
                Operation::Whitespace,
                false,
                '\'',
            ),
            (
                "whitespace.nbsp",
                "a\u{00a0}b",
                0,
                3,
                [1, 1],
                Operation::Whitespace,
                false,
                '\'',
            ),
            (
                "whitespace.missing",
                "abc",
                0,
                3,
                [1, 2],
                Operation::Whitespace,
                false,
                '\'',
            ),
            (
                "nonWhitespace.basic",
                " \t\nx",
                0,
                4,
                [3, 7],
                Operation::NonWhitespace,
                false,
                '\'',
            ),
            (
                "nonWhitespace.first",
                "x ",
                0,
                2,
                [1, 1],
                Operation::NonWhitespace,
                false,
                '\'',
            ),
            (
                "nonWhitespace.missing",
                " \t",
                0,
                2,
                [1, 1],
                Operation::NonWhitespace,
                false,
                '\'',
            ),
            (
                "operator.equal",
                "ab=c",
                0,
                4,
                [1, 1],
                Operation::Operator,
                false,
                '\'',
            ),
            (
                "operator.space",
                "ab c",
                0,
                4,
                [1, 1],
                Operation::Operator,
                false,
                '\'',
            ),
            (
                "operator.missing",
                "abc",
                0,
                3,
                [1, 2],
                Operation::Operator,
                false,
                '\'',
            ),
            (
                "nonOperator.basic",
                "= \tx",
                0,
                4,
                [1, 1],
                Operation::NonOperator,
                false,
                '\'',
            ),
            (
                "nonOperator.first",
                "x=",
                0,
                2,
                [1, 1],
                Operation::NonOperator,
                false,
                '\'',
            ),
            (
                "nonOperator.missing",
                "= \t",
                0,
                3,
                [1, 1],
                Operation::NonOperator,
                false,
                '\'',
            ),
            (
                "any.basic",
                "abc",
                0,
                3,
                [1, 1],
                Operation::AnyAvoidQuotes,
                false,
                '\'',
            ),
            (
                "any.quotes",
                "\"ab\"c",
                0,
                5,
                [1, 1],
                Operation::AnyAvoidQuotes,
                false,
                '\'',
            ),
            (
                "any.apos",
                "'ab'c",
                0,
                5,
                [1, 1],
                Operation::AnyAvoidQuotes,
                false,
                '\'',
            ),
            (
                "any.quoteAtEnd",
                "\"ab\"",
                0,
                4,
                [1, 1],
                Operation::AnyAvoidQuotes,
                false,
                '\'',
            ),
            (
                "any.unclosed",
                "\"ab",
                0,
                3,
                [1, 1],
                Operation::AnyAvoidQuotes,
                false,
                '\'',
            ),
            (
                "any.newline",
                "\"a\nb\"c",
                0,
                6,
                [2, 7],
                Operation::AnyAvoidQuotes,
                false,
                '\'',
            ),
        ] {
            emit_outcome(
                output,
                key,
                operation,
                Some(text),
                offset,
                maxi,
                Some(locator.to_vec()),
                flag,
                marker as u16,
            );
        }
    }

    fn runtime_cases(output: &mut String) {
        let operations = [
            Operation::StructureEnd,
            Operation::CommentBlockEnd,
            Operation::CommentLineEnd,
            Operation::LiteralEnd,
            Operation::StructureStart,
            Operation::Whitespace,
            Operation::NonWhitespace,
            Operation::Operator,
            Operation::NonOperator,
            Operation::AnyAvoidQuotes,
        ];
        for (index, operation) in operations.into_iter().enumerate() {
            emit_outcome(
                output,
                &format!("runtime.nullText.{index}"),
                operation,
                None,
                0,
                1,
                Some(vec![1, 1]),
                false,
                u16::from(b'\''),
            );
            emit_outcome(
                output,
                &format!("runtime.negativeOffset.{index}"),
                operation,
                Some("x"),
                -1,
                0,
                Some(vec![1, 1]),
                false,
                u16::from(b'\''),
            );
        }

        for (key, text, operation, flag) in [
            (
                "runtime.structureNullLocator",
                "]",
                Operation::StructureEnd,
                false,
            ),
            (
                "runtime.blockNullLocator",
                "*/",
                Operation::CommentBlockEnd,
                false,
            ),
            (
                "runtime.lineNullLocator",
                "\n",
                Operation::CommentLineEnd,
                false,
            ),
            (
                "runtime.literalNullLocator",
                "a'",
                Operation::LiteralEnd,
                false,
            ),
            (
                "runtime.startNullLocator",
                "[",
                Operation::StructureStart,
                true,
            ),
            (
                "runtime.whitespaceNullLocator",
                "x",
                Operation::Whitespace,
                false,
            ),
            (
                "runtime.nonWhitespaceNullLocator",
                " ",
                Operation::NonWhitespace,
                false,
            ),
            (
                "runtime.operatorNullLocator",
                "x",
                Operation::Operator,
                false,
            ),
            (
                "runtime.nonOperatorNullLocator",
                "=",
                Operation::NonOperator,
                false,
            ),
            (
                "runtime.anyNullLocator",
                "\"",
                Operation::AnyAvoidQuotes,
                false,
            ),
        ] {
            emit_outcome(
                output,
                key,
                operation,
                Some(text),
                0,
                text.encode_utf16().count() as i32,
                None,
                flag,
                u16::from(b'\''),
            );
        }

        emit_outcome(
            output,
            "runtime.structureOneLocator",
            Operation::StructureEnd,
            Some("\n]"),
            0,
            2,
            Some(vec![i32::MAX]),
            false,
            u16::from(b'\''),
        );
        emit_outcome(
            output,
            "runtime.whitespaceEmptyLocator",
            Operation::Whitespace,
            Some("x"),
            0,
            1,
            Some(Vec::new()),
            false,
            u16::from(b'\''),
        );
    }

    fn exhaustive_cases(output: &mut String) {
        let mut whitespace_hash = FNV_OFFSET;
        for unit in u16::MIN..=u16::MAX {
            let text = [unit];
            for operation in [
                Operation::Whitespace,
                Operation::NonWhitespace,
                Operation::Operator,
                Operation::NonOperator,
            ] {
                let mut locator = [1, 1];
                let result = invoke(
                    operation,
                    Some(&text),
                    0,
                    1,
                    Some(&mut locator),
                    false,
                    u16::from(b'\''),
                )
                .expect("单代码单元与双元素 locator 不会产生运行时异常");
                whitespace_hash = mix(whitespace_hash, result);
                whitespace_hash = mix(whitespace_hash, locator[0]);
                whitespace_hash = mix(whitespace_hash, locator[1]);
            }
        }
        emit(
            output,
            "exhaustive.whitespaceHash",
            format!("{whitespace_hash:016x}"),
        );

        let mut delimiter_hash = FNV_OFFSET;
        for slashes in 0..=12 {
            let mut text = vec![u16::from(b'a')];
            text.extend(std::iter::repeat_n(u16::from(b'\\'), slashes));
            text.extend([u16::from(b'\''), u16::from(b'z'), u16::from(b'\'')]);
            for operation in [Operation::LiteralEnd, Operation::StructureStart] {
                let mut locator = [1, 1];
                let result = invoke(
                    operation,
                    Some(&text),
                    0,
                    text.len() as i32,
                    Some(&mut locator),
                    true,
                    u16::from(b'\''),
                )
                .expect("valid delimiter case");
                delimiter_hash = mix(delimiter_hash, result);
                delimiter_hash = mix(delimiter_hash, locator[1]);
            }
        }
        emit(
            output,
            "exhaustive.delimiterHash",
            format!("{delimiter_hash:016x}"),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_outcome(
        output: &mut String,
        key: &str,
        operation: Operation,
        text: Option<&str>,
        offset: i32,
        maxi: i32,
        mut locator: Option<Vec<i32>>,
        flag: bool,
        marker: u16,
    ) {
        let text = text.map(|text| text.encode_utf16().collect::<Vec<_>>());
        let result = invoke(
            operation,
            text.as_deref(),
            offset,
            maxi,
            locator.as_deref_mut(),
            flag,
            marker,
        );
        let locator = describe_locator(locator.as_deref());
        let value = match result {
            Ok(index) => format!("OK:{index}:{locator}"),
            Err(error) => format!(
                "ERR:{}:{}:{locator}",
                error.class_name(),
                to_utf16_hex(
                    &error
                        .message()
                        .unwrap_or_else(|| Utf16String::from_rust_str("null"))
                )
            ),
        };
        emit(output, key, value);
    }

    #[allow(clippy::too_many_arguments)]
    fn invoke(
        operation: Operation,
        text: Option<&[u16]>,
        offset: i32,
        maxi: i32,
        locator: Option<&mut [i32]>,
        flag: bool,
        marker: u16,
    ) -> Result<i32, TextParsingUtilError> {
        match operation {
            Operation::StructureEnd => {
                TextParsingUtil::find_next_structure_end_avoid_quotes(text, offset, maxi, locator)
            }
            Operation::CommentBlockEnd => {
                TextParsingUtil::find_next_comment_block_end(text, offset, maxi, locator)
            }
            Operation::CommentLineEnd => {
                TextParsingUtil::find_next_comment_line_end(text, offset, maxi, locator)
            }
            Operation::LiteralEnd => {
                TextParsingUtil::find_next_literal_end(text, offset, maxi, locator, marker)
            }
            Operation::StructureStart => {
                TextParsingUtil::find_next_structure_start_or_literal_marker(
                    text, offset, maxi, locator, flag,
                )
            }
            Operation::Whitespace => TextParsingUtil::find_next_whitespace_char_wildcard(
                text, offset, maxi, flag, locator,
            ),
            Operation::NonWhitespace => {
                TextParsingUtil::find_next_non_whitespace_char_wildcard(text, offset, maxi, locator)
            }
            Operation::Operator => {
                TextParsingUtil::find_next_operator_char_wildcard(text, offset, maxi, locator)
            }
            Operation::NonOperator => {
                TextParsingUtil::find_next_non_operator_char_wildcard(text, offset, maxi, locator)
            }
            Operation::AnyAvoidQuotes => TextParsingUtil::find_next_any_char_avoid_quotes_wildcard(
                text, offset, maxi, locator,
            ),
        }
    }

    fn describe_locator(locator: Option<&[i32]>) -> String {
        locator.map_or_else(
            || "null".to_owned(),
            |locator| {
                locator
                    .iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            },
        )
    }

    fn to_utf16_hex(value: &Utf16String) -> String {
        value
            .as_utf16()
            .iter()
            .map(|unit| format!("{unit:04x}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn mix(hash: u64, value: i32) -> u64 {
        (hash ^ value as i64 as u64).wrapping_mul(FNV_PRIME)
    }

    fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
        writeln!(output, "{key}={value}").expect("write to string");
    }
}
