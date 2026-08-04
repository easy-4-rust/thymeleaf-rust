use super::text_parsing_comment_util::{TextParsingCommentError, TextParsingCommentUtil};

/// 文本模式正则表达式字面量起点判定工具。
///
/// 对应 Java: `org.thymeleaf.templateparser.text.TextParsingLiteralUtil`。
///
/// 本无状态对象通过前一个非 Java 空白 `char` 判断 `/` 是否开启正则字面量，并排除
/// `/*`、`//` 注释；空白语义严格采用 `Character.isWhitespace(char)` 的 BMP 规则。
pub(crate) struct TextParsingLiteralUtil;

impl TextParsingLiteralUtil {
    /// 判断 `offset` 的 `/` 是否为正则字面量起点。
    ///
    /// 对应 Java: `TextParsingLiteralUtil#isRegexLiteralStart`。
    pub(crate) fn is_regex_literal_start(
        buffer: Option<&[u16]>,
        offset: i32,
        maxi: i32,
    ) -> Result<bool, TextParsingCommentError> {
        if offset == 0 {
            return Ok(false);
        }
        let buffer_value = buffer.ok_or(TextParsingCommentError::NullArrayLoad)?;
        if array_unit(buffer_value, offset)? != u16::from(b'/') {
            return Ok(false);
        }
        if TextParsingCommentUtil::is_comment_block_start(buffer, offset, maxi)? {
            return Ok(false);
        }
        // 到达此处已证明 offset 有效，且下一代码单元已由块注释谓词成功读取。
        // 直接执行与 isCommentLineStart 相同的比较，消除 Java 控制流不可达的
        // Rust Result 伪错误分支。
        if maxi.wrapping_sub(offset) > 1
            && buffer_value[offset.wrapping_add(1) as usize] == u16::from(b'/')
        {
            return Ok(false);
        }

        let mut index = offset.wrapping_sub(1);
        while index >= 0 {
            // index 从已验证 offset 的前一位单调递减，该访问不会产生新数组异常。
            let character = buffer_value[index as usize];
            if !is_java_whitespace(character) {
                return Ok(matches!(character, 0x0028 | 0x003D | 0x002C));
            }
            index = index.wrapping_sub(1);
        }
        Ok(false)
    }
}

fn array_unit(buffer: &[u16], index: i32) -> Result<u16, TextParsingCommentError> {
    usize::try_from(index)
        .ok()
        .and_then(|index| buffer.get(index).copied())
        .ok_or(TextParsingCommentError::ArrayIndex {
            index,
            size: buffer.len(),
        })
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

#[cfg(test)]
mod tests {
    use super::TextParsingLiteralUtil;

    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x100_0000_01b3;

    #[test]
    fn matches_context_comment_and_runtime_cases() {
        for (value, offset, maxi, expected) in [
            ("(/", 1, 2, true),
            ("= \t/", 3, 4, true),
            (",\u{3000}/", 2, 3, true),
            ("a/", 1, 2, false),
            ("/", 0, 1, false),
            ("(x", 1, 2, false),
            ("(/*", 1, 3, false),
            ("(//", 1, 3, false),
            (" \t/", 2, 3, false),
        ] {
            let buffer: Vec<u16> = value.encode_utf16().collect();
            assert_eq!(
                TextParsingLiteralUtil::is_regex_literal_start(Some(&buffer), offset, maxi)
                    .expect("valid case"),
                expected
            );
        }
        assert!(!TextParsingLiteralUtil::is_regex_literal_start(None, 0, 1).unwrap());
        let error =
            TextParsingLiteralUtil::is_regex_literal_start(None, 1, 2).expect_err("null positive");
        assert_eq!(error.class_name(), "java.lang.NullPointerException");
        let buffer = vec![u16::from(b'/')];
        for offset in [-1, 1] {
            let error = TextParsingLiteralUtil::is_regex_literal_start(Some(&buffer), offset, 2)
                .expect_err("bounds");
            assert_eq!(
                error.class_name(),
                "java.lang.ArrayIndexOutOfBoundsException"
            );
        }
        let truncated = vec![u16::from(b'('), u16::from(b'/')];
        let error = TextParsingLiteralUtil::is_regex_literal_start(Some(&truncated), 1, 3)
            .expect_err("comment lookahead bounds");
        assert_eq!(
            error.java_message().to_string_lossy(),
            "Index 2 out of bounds for length 2"
        );
    }

    #[test]
    fn matches_exhaustive_java_character_whitespace_golden() {
        let mut hash = FNV_OFFSET;
        for unit in u16::MIN..=u16::MAX {
            let buffer = [u16::from(b'('), unit, u16::from(b'/')];
            let result = TextParsingLiteralUtil::is_regex_literal_start(Some(&buffer), 2, 3)
                .expect("valid exhaustive case");
            hash = (hash ^ u64::from(result)).wrapping_mul(FNV_PRIME);
        }
        assert_eq!(hash, 0x1d79_c3a7_9cb8_fc65);
    }
}
