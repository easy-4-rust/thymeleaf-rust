//! 文本注释读取器差分 —— 1:1 移植 Java：
//! - `ParserLevelCommentTextReaderTest`（`/*[-`…`-]*/` 删除块）
//! - `PrototypeOnlyCommentTextReaderTest`（`/*[+`…`+]*/` 去壳留内容）
//!
//! 两个 Java 测试类均为：`test01` 对“前缀/后缀插入 0123456789”的全组合消息按
//! Java 等价算法计算期望，`test02` 使用手写用例；每个消息在所有 `(j,k,l)`
//! 缓冲形状下逐字节比较 —— 本文件 1:1 复刻该语义。

use std::fmt::Write as _;

use thymeleaf::reader::{ParserLevelCommentTextReader, PrototypeOnlyCommentTextReader};
use thymeleaf::text::{TextParserReader, TextParserReaderError};

/// UTF-16 字符串读取器（对应 Java `StringReader`）。
struct Utf16StringReader {
    value: Vec<u16>,
    position: usize,
}

impl Utf16StringReader {
    fn new(value: &str) -> Self {
        Self {
            value: value.encode_utf16().collect(),
            position: 0,
        }
    }
}

impl TextParserReader for Utf16StringReader {
    fn read_range(
        &mut self,
        buffer: &mut [u16],
        offset: i32,
        len: i32,
    ) -> Result<i32, TextParserReaderError> {
        if len == 0 {
            return Ok(0);
        }
        if self.position >= self.value.len() {
            return Ok(-1);
        }
        let copied = (len as usize).min(self.value.len() - self.position);
        let offset = offset as usize;
        buffer[offset..offset + copied]
            .copy_from_slice(&self.value[self.position..self.position + copied]);
        self.position += copied;
        Ok(copied as i32)
    }
}

/// 读取器工厂：按读者类型构造（对应 Java 两个 Reader 类）。
enum ReaderKind {
    ParserLevel,
    PrototypeOnly,
}

impl ReaderKind {
    fn new_reader(&self, message: &str) -> Box<dyn TextParserReader> {
        match self {
            Self::ParserLevel => Box::new(ParserLevelCommentTextReader::new(Box::new(
                Utf16StringReader::new(message),
            ))),
            Self::PrototypeOnly => Box::new(PrototypeOnlyCommentTextReader::new(Box::new(
                Utf16StringReader::new(message),
            ))),
        }
    }
}

/// `testMessage(message, expected)`：所有 `(j,k,l)` 缓冲形状下读取并比较。
fn test_message(kind: &ReaderKind, message: &str, expected: &str) {
    let mut failures = String::new();
    for j in 1..=(message.encode_utf16().count() + 10) {
        for k in 1..=j {
            for l in 0..k {
                let mut reader = kind.new_reader(message);
                let mut buffer = vec![0u16; j];
                let mut result = String::new();
                loop {
                    let read = reader
                        .read_range(&mut buffer, l as i32, (k - l) as i32)
                        .expect("reader read");
                    if read < 0 {
                        break;
                    }
                    let unit = buffer[l..l + read as usize].to_vec();
                    result.push_str(&String::from_utf16_lossy(&unit));
                }
                if result != expected {
                    let _ = write!(
                        failures,
                        "checking '{message}' ({j},{k},{l}): expected {expected:?}, got {result:?}\n"
                    );
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "message 在部分缓冲形状下不一致:\n{failures}"
    );
}

/// Java `computeAllParserLevelMessages`：`/*[-`/`-]*/` 插入 0123456789 的全组合。
fn compute_all_parser_level_messages() -> Vec<String> {
    let prefix = "/*[-";
    let suffix = "-]*/";
    let message = "0123456789";
    let mut all = Vec::new();
    for i in 0..=message.len() {
        let mut msb1 = String::new();
        msb1.push_str(&message[..i]);
        msb1.push_str(suffix);
        msb1.push_str(&message[i..]);
        for j in 0..=i {
            let mut msb2 = String::new();
            msb2.push_str(&msb1[..j]);
            msb2.push_str(prefix);
            msb2.push_str(&msb1[j..]);
            for k in 0..=j {
                let mut msb3 = String::new();
                msb3.push_str(&msb2[..k]);
                msb3.push_str(suffix);
                msb3.push_str(&msb2[k..]);
                all.push(msb3.clone());
                for l in 0..=k {
                    let mut msb4 = String::new();
                    msb4.push_str(&msb3[..l]);
                    msb4.push_str(prefix);
                    msb4.push_str(&msb3[l..]);
                    all.push(msb4);
                }
            }
        }
    }
    all
}

/// Java `computeParserLevelEquivalent`：剥离 `/*...*/` 注释区间。
fn compute_parser_level_equivalent(message: &str) -> String {
    let chars = message.chars().collect::<Vec<_>>();
    let mut out = String::new();
    let mut in_comment = false;
    let mut i = 0usize;
    while i < chars.len() {
        if !in_comment && chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            in_comment = true;
            i += 1;
            continue;
        } else if in_comment && chars[i] == '/' && i > 0 && chars[i - 1] == '*' {
            in_comment = false;
            i += 1;
            continue;
        }
        if !in_comment {
            out.push(chars[i]);
        }
        i += 1;
    }
    out
}

/// Java `computeAllPrototypeOnlyMessages`：`/*[+`/`+]*/` 插入 0123456789 的全组合。
fn compute_all_prototype_only_messages() -> Vec<String> {
    let prefix = "/*[+";
    let suffix = "+]*/";
    let message = "0123456789";
    let mut all = Vec::new();
    for i in 0..=message.len() {
        let mut msb1 = String::new();
        msb1.push_str(&message[..i]);
        msb1.push_str(suffix);
        msb1.push_str(&message[i..]);
        for j in 0..=i {
            let mut msb2 = String::new();
            msb2.push_str(&msb1[..j]);
            msb2.push_str(prefix);
            msb2.push_str(&msb1[j..]);
            for k in 0..=j {
                let mut msb3 = String::new();
                msb3.push_str(&msb2[..k]);
                msb3.push_str(suffix);
                msb3.push_str(&msb2[k..]);
                all.push(msb3.clone());
                for l in 0..=k {
                    let mut msb4 = String::new();
                    msb4.push_str(&msb3[..l]);
                    msb4.push_str(prefix);
                    msb4.push_str(&msb3[l..]);
                    all.push(msb4);
                }
            }
        }
    }
    all
}

/// Java `computePrototypeOnlyEquivalent`：`/*[+`…`+]*/` 去壳、内容保留。
fn compute_prototype_only_equivalent(message: &str) -> String {
    let chars = message.chars().collect::<Vec<_>>();
    let mut out = String::new();
    let mut was_open = false;
    let mut in_open_structure = false;
    let mut in_close_structure = false;
    let mut i = 0usize;
    while i < chars.len() {
        if !in_open_structure
            && !in_close_structure
            && chars[i] == '/'
            && i + 1 < chars.len()
            && chars[i + 1] == '*'
        {
            in_open_structure = true;
            i += 1;
            continue;
        } else if !in_open_structure && !in_close_structure && was_open && chars[i] == '+' {
            in_close_structure = true;
            i += 1;
            continue;
        } else if in_close_structure && chars[i] == '/' && i > 0 && chars[i - 1] == '*' {
            in_close_structure = false;
            was_open = false;
            i += 1;
            continue;
        } else if in_open_structure && chars[i] == '+' {
            in_open_structure = false;
            was_open = true;
            i += 1;
            continue;
        }
        if !in_open_structure && !in_close_structure {
            out.push(chars[i]);
        }
        i += 1;
    }
    out
}

// ===========================================================================
// ParserLevelCommentTextReaderTest
// ===========================================================================

#[test]
fn parser_level_comment_test01_matches_java_equivalents() {
    let kind = ReaderKind::ParserLevel;
    for message in compute_all_parser_level_messages() {
        let expected = compute_parser_level_equivalent(&message);
        test_message(&kind, &message, &expected);
    }
}

#[test]
fn parser_level_comment_test02_matches_java_handwritten_cases() {
    let kind = ReaderKind::ParserLevel;
    for (message, expected) in [
        ("/* hello */", "/* hello */"),
        ("/* /*[- hello -]]*/ -]*/", "/* "),
        ("/* /*[- hello -]]*/ -]*/ */", "/*  */"),
        (
            "/* /*[[- hello -]]*/ -]*/ */",
            "/* /*[[- hello -]]*/ -]*/ */",
        ),
        (
            "/* /*[[- hello -]]*/ -]*/ /*[- -]*/*/",
            "/* /*[[- hello -]]*/ -]*/ */",
        ),
        (
            "/* /*[[--- hello ---]]*/ -]*/ */",
            "/* /*[[--- hello ---]]*/ -]*/ */",
        ),
        (
            "/* /*[[--- hello ---]]*/ -]*/ /*[- -]*/*/",
            "/* /*[[--- hello ---]]*/ -]*/ */",
        ),
        ("hello", "hello"),
        ("/*[- hello -]***/ -]*/", ""),
        ("/*[- hello -]***/ -]*/", ""),
        ("/***[- hello -]***/ -]*/", "/***[- hello -]***/ -]*/"),
        ("/***[- hello -]***/ -]*/ */", "/***[- hello -]***/ -]*/ */"),
        (
            "/***[- hello -]***/ -]*/ /*[- -]*/*/",
            "/***[- hello -]***/ -]*/ */",
        ),
        (
            "/***[- hello -]***/ -]*/ /*[- -]*/",
            "/***[- hello -]***/ -]*/ ",
        ),
    ] {
        test_message(&kind, message, expected);
    }
}

// ===========================================================================
// PrototypeOnlyCommentTextReaderTest
// ===========================================================================

#[test]
fn prototype_only_comment_test01_matches_java_equivalents() {
    let kind = ReaderKind::PrototypeOnly;
    for message in compute_all_prototype_only_messages() {
        let expected = compute_prototype_only_equivalent(&message);
        test_message(&kind, &message, &expected);
    }
}

#[test]
fn prototype_only_comment_test02_matches_java_handwritten_cases() {
    let kind = ReaderKind::PrototypeOnly;
    for (message, expected) in [
        ("/* hello */", "/* hello */"),
        ("/* /*[[[ hello +]*/ */", "/* /*[[[ hello +]*/ */"),
        (
            "/* /*[[[ hello +]*/ +]]]*///",
            "/* /*[[[ hello +]*/ +]]]*///",
        ),
        (
            "/* /*[[[ hello +]*/ +]]]*/// */",
            "/* /*[[[ hello +]*/ +]]]*/// */",
        ),
        ("/* /*[+ hello +]*/ +]]]*/// */", "/*  hello  +]]]*/// */"),
        (
            "/* /*[+ hello +]*/ +]]]*/// /*[[[ */",
            "/*  hello  +]]]*/// /*[[[ */",
        ),
        (
            "/* /*[+ hello +]*/ +]]]*/// /*[[[ +]]]*///*/",
            "/*  hello  +]]]*/// /*[[[ +]]]*///*/",
        ),
        ("hello", "hello"),
        ("/*[[[ hello +]*/", "/*[[[ hello +]*/"),
        ("/*[[[ hello +]*/ a+]]]*///a", "/*[[[ hello +]*/ a+]]]*///a"),
        ("/*[[[ hello +]*/ +]]]*///", "/*[[[ hello +]*/ +]]]*///"),
        ("/*[+ hello +]*/", " hello "),
        ("/*[+ hello +]*/ +]]]*///", " hello  +]]]*///"),
        ("/*[+ hello +]*/ aa+]]]*///bb", " hello  aa+]]]*///bb"),
        ("/*[+hello+]*/", "hello"),
        ("/*[+hello+]*/ +]]]*/// aa", "hello +]]]*/// aa"),
        ("/*[+hello+]*/ +]]]*///", "hello +]]]*///"),
        ("hey /*[+hello+]*/", "hey hello"),
        ("hey /*[+hello+]*/ +]]]*///", "hey hello +]]]*///"),
        ("hey /*[+hello+]*/ +]]]*///", "hey hello +]]]*///"),
        ("hey /*[+hello+]*/ +]*/", "hey hello +]*/"),
        ("hey /*[+hello+]*/ +]*/", "hey hello +]*/"),
        ("/*[+ hello +]*/ +]]]*///", " hello  +]]]*///"),
        ("/*[+ hello +]*/ +]]]*/// */", " hello  +]]]*/// */"),
        ("/*[+ hello +]*/ +]]]*/// /*[[[", " hello  +]]]*/// /*[[["),
        (
            "/*[+ hello +]*/ +]]]*/// /*[[[ */",
            " hello  +]]]*/// /*[[[ */",
        ),
        (
            "/*[+ hello +]*/ +]]]*/// /*[[[ +]]]*///*/",
            " hello  +]]]*/// /*[[[ +]]]*///*/",
        ),
        (
            "/*[+ hello +]*/ +]]]*/// /*[[[ +]]]*///",
            " hello  +]]]*/// /*[[[ +]]]*///",
        ),
        ("/*[+hello +]*/ +]]]*///", "hello  +]]]*///"),
        ("/*[+hello +]*/ +]]]*/// */", "hello  +]]]*/// */"),
        ("/*[+hello +]*/ +]]]*/// /*[[[", "hello  +]]]*/// /*[[["),
        (
            "/*[+hello +]*/ +]]]*/// /*[[[ */",
            "hello  +]]]*/// /*[[[ */",
        ),
        (
            "/*[+hello +]*/ +]]]*/// /*[[[ +]]]*///*/",
            "hello  +]]]*/// /*[[[ +]]]*///*/",
        ),
        (
            "/*[+hello +]*/ +]]]*/// /*[[[ +]]]*///",
            "hello  +]]]*/// /*[[[ +]]]*///",
        ),
        (
            "/*[[[/*[+hello +]*/ +]]]*/// /*[[[ +]]]*///",
            "/*[[[hello  +]]]*/// /*[[[ +]]]*///",
        ),
    ] {
        test_message(&kind, message, expected);
    }
}
