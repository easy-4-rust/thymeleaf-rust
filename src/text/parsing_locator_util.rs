#![cfg_attr(
    not(test),
    expect(dead_code, reason = "text parser 消费者对象将在后续切片中迁移")
)]

use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::util::JavaString;

const NULL_LOCATOR_MESSAGE: &str = "Cannot load from int array because \"<parameter1>\" is null";

/// `ParsingLocatorUtil` 更新定位数组失败。
///
/// 对应 Java: `org.thymeleaf.templateparser.text.ParsingLocatorUtil#countChar`
/// 读取 null 或过短 `int[]` 时抛出的运行时异常。
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ParsingLocatorError {
    /// locator 为 Java null。
    NullLocator,
    /// locator 不包含当前分支要读写的数组下标。
    ArrayIndex {
        /// Java 数组访问下标。
        index: usize,
        /// locator 实际长度。
        length: usize,
    },
}

impl ParsingLocatorError {
    /// 返回对应 Java 异常全限定名。
    ///
    /// # 返回
    /// Java `Throwable#getClass().getName()` 的精确结果。
    pub(crate) const fn java_class_name(&self) -> &'static str {
        match self {
            Self::NullLocator => "java.lang.NullPointerException",
            Self::ArrayIndex { .. } => "java.lang.ArrayIndexOutOfBoundsException",
        }
    }

    /// 返回对应 Java 异常消息。
    ///
    /// # 返回
    /// Java 17 增强 NPE 或数组访问消息的 UTF-16 值。
    pub(crate) fn message(&self) -> JavaString {
        match self {
            Self::NullLocator => JavaString::from_rust_str(NULL_LOCATOR_MESSAGE),
            Self::ArrayIndex { index, length } => JavaString::from_rust_str(&format!(
                "Index {index} out of bounds for length {length}"
            )),
        }
    }
}

impl Display for ParsingLocatorError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message().to_string_lossy())
    }
}

impl Error for ParsingLocatorError {}

/// 文本解析期间更新行号和列号的内部工具。
///
/// 对应 Java: `org.thymeleaf.templateparser.text.ParsingLocatorUtil`。
///
/// locator 下标 0 为行号、下标 1 为列号。只有 LF 会让行号加一并把列号重置为
/// 1；CR、NUL、代理项及其他 UTF-16 代码单元都只让列号加一。加法按 Java `int`
/// 回绕，短数组的检查和部分写入顺序保持不变。
pub(crate) struct ParsingLocatorUtil;

impl ParsingLocatorUtil {
    /// 计入一个 UTF-16 代码单元并更新 locator。
    ///
    /// 对应 Java: `ParsingLocatorUtil#countChar(int[],char)`。
    ///
    /// # 参数
    /// - `locator`：可空定位数组；下标 0/1 分别为 line/column。
    /// - `character`：待计入的 Java UTF-16 `char`。
    ///
    /// # 错误
    /// null 返回增强 NPE。LF 分支先更新下标 0，再访问下标 1，因此单元素数组会在
    /// 返回越界错误前保留已递增的行号；其他字符直接访问下标 1，不修改下标 0。
    pub(crate) fn count_char(
        locator: Option<&mut [i32]>,
        character: u16,
    ) -> Result<(), ParsingLocatorError> {
        let locator = locator.ok_or(ParsingLocatorError::NullLocator)?;
        if character == u16::from(b'\n') {
            let length = locator.len();
            let line = locator
                .get_mut(0)
                .ok_or(ParsingLocatorError::ArrayIndex { index: 0, length })?;
            *line = line.wrapping_add(1);
            let column = locator
                .get_mut(1)
                .ok_or(ParsingLocatorError::ArrayIndex { index: 1, length })?;
            *column = 1;
            return Ok(());
        }
        let length = locator.len();
        let column = locator
            .get_mut(1)
            .ok_or(ParsingLocatorError::ArrayIndex { index: 1, length })?;
        *column = column.wrapping_add(1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use super::{NULL_LOCATOR_MESSAGE, ParsingLocatorError, ParsingLocatorUtil};
    use crate::text::TextParseStatus;
    use crate::util::JavaString;

    const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    const JAVA_GOLDEN: &str =
        include_str!("../../tests/fixtures/text_parser_foundation_golden.txt");

    #[test]
    fn text_parser_foundation_matches_java_golden() {
        let mut output = String::new();
        emit(&mut output, "baseline", JAVA_BASELINE);
        status_cases(&mut output);
        locator_cases(&mut output);
        assert_eq!(output, JAVA_GOLDEN);
    }

    #[test]
    fn error_display_preserves_java_message() {
        assert_eq!(
            ParsingLocatorError::NullLocator.to_string(),
            NULL_LOCATOR_MESSAGE
        );
    }

    fn status_cases(output: &mut String) {
        let mut first = TextParseStatus::new();
        let second = TextParseStatus::default();
        emit(output, "status.default", describe_status(&first));
        first.offset = -1;
        first.line = i32::MAX;
        first.col = i32::MIN;
        first.in_structure = true;
        first.in_comment_line = true;
        first.literal_marker = 0xD800;
        emit(output, "status.mutated", describe_status(&first));
        emit(output, "status.independent", describe_status(&second));
    }

    fn locator_cases(output: &mut String) {
        let mut locator = [0, 0];
        count_and_emit(output, "locator.ascii", &mut locator, u16::from(b'A'));
        count_and_emit(output, "locator.lf", &mut locator, u16::from(b'\n'));
        count_and_emit(output, "locator.cr", &mut locator, u16::from(b'\r'));
        count_and_emit(output, "locator.nul", &mut locator, 0);
        count_and_emit(output, "locator.surrogate", &mut locator, 0xD800);

        let mut line_overflow = [i32::MAX, 7];
        count_and_emit(
            output,
            "locator.lineOverflow",
            &mut line_overflow,
            u16::from(b'\n'),
        );
        let mut column_overflow = [9, i32::MAX];
        count_and_emit(
            output,
            "locator.columnOverflow",
            &mut column_overflow,
            u16::from(b'x'),
        );

        emit_locator_outcome(output, "locator.nullLf", None, u16::from(b'\n'));
        emit_locator_outcome(output, "locator.nullAscii", None, u16::from(b'x'));
        let mut empty_lf = [];
        emit_locator_outcome(
            output,
            "locator.emptyLf",
            Some(&mut empty_lf),
            u16::from(b'\n'),
        );
        let mut empty_ascii = [];
        emit_locator_outcome(
            output,
            "locator.emptyAscii",
            Some(&mut empty_ascii),
            u16::from(b'x'),
        );
        let mut one_lf = [5];
        emit_locator_outcome(output, "locator.oneLf", Some(&mut one_lf), u16::from(b'\n'));
        let mut one_ascii = [5];
        emit_locator_outcome(
            output,
            "locator.oneAscii",
            Some(&mut one_ascii),
            u16::from(b'x'),
        );
        let mut extra = [2, 3, 99];
        emit_locator_outcome(output, "locator.extra", Some(&mut extra), u16::from(b'\n'));
    }

    fn count_and_emit(output: &mut String, key: &str, locator: &mut [i32], character: u16) {
        ParsingLocatorUtil::count_char(Some(locator), character).expect("valid locator");
        emit(output, key, describe_locator(locator));
    }

    fn emit_locator_outcome(
        output: &mut String,
        key: &str,
        locator: Option<&mut [i32]>,
        character: u16,
    ) {
        match locator {
            Some(locator) => {
                let result = ParsingLocatorUtil::count_char(Some(locator), character);
                match result {
                    Ok(()) => emit(output, key, format!("OK:{}", describe_locator(locator))),
                    Err(error) => emit(
                        output,
                        key,
                        format!(
                            "ERR:{}:{}:{}",
                            error.java_class_name(),
                            to_utf16_hex(&error.message()),
                            describe_locator(locator)
                        ),
                    ),
                }
            }
            None => {
                let error = ParsingLocatorUtil::count_char(None, character)
                    .expect_err("null locator error");
                emit(
                    output,
                    key,
                    format!(
                        "ERR:{}:{}:null",
                        error.java_class_name(),
                        to_utf16_hex(&error.message())
                    ),
                );
            }
        }
    }

    fn describe_status(status: &TextParseStatus) -> String {
        format!(
            "{},{},{},{},{},{:04x}",
            status.offset,
            status.line,
            status.col,
            status.in_structure,
            status.in_comment_line,
            status.literal_marker
        )
    }

    fn describe_locator(locator: &[i32]) -> String {
        locator
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn to_utf16_hex(value: &JavaString) -> String {
        value
            .as_utf16()
            .iter()
            .map(|unit| format!("{unit:04x}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
        writeln!(output, "{key}={value}").expect("write to string");
    }
}
