use crate::text::{TextParserReader, TextParserReaderError};

/// 块容器被识别后采用的处理动作。
///
/// 对应 Java: `org.thymeleaf.templateparser.reader.BlockAwareReader.BlockAction`。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BlockAction {
    /// 丢弃块容器及其全部内容。
    DiscardAll,
    /// 只丢弃块容器，保留容器内的内容。
    DiscardContainer,
}

/// 跨任意 Reader 缓冲区边界识别并移除成对块结构的状态机。
///
/// 该对象保留 Java 实现的 UTF-16 `char`、溢出缓冲、未闭合块异常和委托关闭语义。
/// 对应 Java: `org.thymeleaf.templateparser.reader.BlockAwareReader`。
pub(crate) struct BlockAwareReader {
    reader: Box<dyn TextParserReader>,
    action: BlockAction,
    prefix: Vec<u16>,
    suffix: Vec<u16>,
    p0: u16,
    s0: u16,
    overflow_buffer: Option<Vec<u16>>,
    overflow_buffer_len: usize,
    inside_comment: bool,
    index: usize,
    discard_from: i32,
}

impl BlockAwareReader {
    /// 创建块感知 Reader。
    ///
    /// `reader` 是被包装的 Java Reader 语义适配器；`action` 决定是否保留块内容；
    /// `prefix` 与 `suffix` 是 UTF-16 定界符。对应 Java:
    /// `BlockAwareReader#BlockAwareReader(Reader,BlockAction,char[],char[])`。
    pub(crate) fn new(
        reader: Box<dyn TextParserReader>,
        action: BlockAction,
        prefix: &[u16],
        suffix: &[u16],
    ) -> Self {
        Self {
            reader,
            action,
            prefix: prefix.to_vec(),
            suffix: suffix.to_vec(),
            p0: prefix[0],
            s0: suffix[0],
            overflow_buffer: None,
            overflow_buffer_len: 0,
            inside_comment: false,
            index: 0,
            discard_from: -1,
        }
    }

    /// 读取指定 UTF-16 范围，并就地移除已经识别的块结构。
    ///
    /// 返回值与 Java `Reader#read(char[],int,int)` 一致：正数表示读取数，`0`
    /// 表示本次无数据，`-1` 表示 EOF。未闭合块在 EOF 时返回 `IOException`。
    /// 对应 Java: `BlockAwareReader#read`。
    pub(crate) fn read_range(
        &mut self,
        cbuf: &mut [u16],
        off: i32,
        len: i32,
    ) -> Result<i32, TextParserReaderError> {
        let mut read = self.read_bytes(cbuf, off, len)?;
        if read <= 0 {
            if read < 0 && self.inside_comment {
                let prefix = String::from_utf16_lossy(&self.prefix);
                let suffix = String::from_utf16_lossy(&self.suffix);
                return Err(TextParserReaderError::io(&format!(
                    "Unfinished block structure {prefix}...{suffix}"
                )));
            }
            return Ok(read);
        }

        self.discard_from = if self.discard_from < 0 {
            self.discard_from
        } else {
            off.max(self.discard_from)
        };

        let mut maxi = (off + read) as usize;
        let mut i = off as usize;
        while i < maxi {
            let c = cbuf[i];
            i += 1;

            if self.index == 0 && c != self.p0 && c != self.s0 {
                continue;
            }

            if !self.inside_comment {
                if c == self.prefix[self.index] {
                    self.index += 1;
                    if self.index == self.prefix.len() {
                        let structure_len = self.prefix.len();
                        if i < maxi {
                            cbuf.copy_within(i..maxi, i - structure_len);
                        }
                        self.inside_comment = true;
                        self.index = 0;
                        read -= structure_len as i32;
                        maxi -= structure_len;
                        i -= structure_len;
                        self.discard_from = if self.action == BlockAction::DiscardAll {
                            i as i32
                        } else {
                            -1
                        };
                    }
                } else {
                    if self.index > 0 {
                        i -= self.index;
                    }
                    self.index = 0;
                }
            } else if c == self.suffix[self.index] {
                self.index += 1;
                if self.index == self.suffix.len() {
                    let structure_len = self.suffix.len();
                    if i < maxi {
                        cbuf.copy_within(i..maxi, i - structure_len);
                    }
                    self.inside_comment = false;
                    self.index = 0;
                    read -= structure_len as i32;
                    maxi -= structure_len;
                    i -= structure_len;

                    if self.discard_from >= 0 {
                        let discard_from = self.discard_from as usize;
                        if i < maxi {
                            cbuf.copy_within(i..maxi, discard_from);
                        }
                        read -= (i - discard_from) as i32;
                        maxi -= i - discard_from;
                        i = discard_from;
                        self.discard_from = -1;
                    }
                }
            } else {
                if self.index > 0 {
                    i -= self.index;
                }
                self.index = 0;
            }
        }

        if self.index > 0 {
            let overflow_count = self.index;
            self.overflow_last_bytes(cbuf, maxi, overflow_count);
            read -= overflow_count as i32;
            maxi -= overflow_count;

            let structure = if self.inside_comment {
                self.suffix.clone()
            } else {
                self.prefix.clone()
            };
            if self.match_overflow(&structure)? {
                self.inside_comment = !self.inside_comment;
                self.overflow_buffer_len -= structure.len();
                self.index = 0;
            } else {
                let overflow = self
                    .overflow_buffer
                    .as_mut()
                    .expect("overflow buffer is allocated before matching");
                cbuf[maxi] = overflow[0];
                read += 1;
                maxi += 1;
                overflow.copy_within(1..self.overflow_buffer_len, 0);
                self.overflow_buffer_len -= 1;
                self.index = 0;
            }
        }

        if self.discard_from >= 0 {
            read -= maxi as i32 - self.discard_from;
            self.discard_from = 0;
        }

        self.discard_from = if self.inside_comment && self.action == BlockAction::DiscardAll {
            0
        } else {
            -1
        };
        Ok(read)
    }

    /// 关闭底层 Reader，并原样传播其失败。
    ///
    /// 对应 Java: `BlockAwareReader#close`。
    pub(crate) fn close(&mut self) -> Result<(), TextParserReaderError> {
        self.reader.close()
    }

    fn read_bytes(
        &mut self,
        buffer: &mut [u16],
        off: i32,
        len: i32,
    ) -> Result<i32, TextParserReaderError> {
        if len == 0 {
            return Ok(0);
        }
        if self.overflow_buffer_len == 0 {
            return self.reader.read_range(buffer, off, len);
        }

        let requested = len as usize;
        let destination = off as usize;
        if self.overflow_buffer_len <= requested {
            let overflow_len = self.overflow_buffer_len;
            let overflow = self
                .overflow_buffer
                .as_ref()
                .expect("positive overflow length requires an overflow buffer");
            buffer[destination..destination + overflow_len]
                .copy_from_slice(&overflow[..overflow_len]);
            let mut read = overflow_len as i32;
            self.overflow_buffer_len = 0;
            if read < len {
                let delegate_read = self.reader.read_range(buffer, off + read, len - read)?;
                if delegate_read > 0 {
                    read += delegate_read;
                }
            }
            return Ok(read);
        }

        let overflow = self
            .overflow_buffer
            .as_mut()
            .expect("positive overflow length requires an overflow buffer");
        buffer[destination..destination + requested].copy_from_slice(&overflow[..requested]);
        overflow.copy_within(requested..self.overflow_buffer_len, 0);
        self.overflow_buffer_len -= requested;
        Ok(len)
    }

    fn overflow_last_bytes(&mut self, buffer: &[u16], maxi: usize, overflow_count: usize) {
        let capacity = self.prefix.len().max(self.suffix.len());
        let overflow = self
            .overflow_buffer
            .get_or_insert_with(|| vec![0; capacity]);
        if self.overflow_buffer_len > 0 {
            overflow.copy_within(0..self.overflow_buffer_len, overflow_count);
        }
        overflow[..overflow_count].copy_from_slice(&buffer[maxi - overflow_count..maxi]);
        self.overflow_buffer_len += overflow_count;
    }

    fn match_overflow(&mut self, structure: &[u16]) -> Result<bool, TextParserReaderError> {
        if self.overflow_buffer_len > 0 {
            let overflow = self
                .overflow_buffer
                .as_ref()
                .expect("positive overflow length requires an overflow buffer");
            for index in 0..self.overflow_buffer_len {
                if overflow[index] != structure[index] {
                    return Ok(false);
                }
            }
        }

        let mut overflow_read = 0;
        while overflow_read >= 0 && self.overflow_buffer_len < structure.len() {
            let overflow = self
                .overflow_buffer
                .as_mut()
                .expect("overflow matching requires an overflow buffer");
            overflow_read = self
                .reader
                .read_range(overflow, self.overflow_buffer_len as i32, 1)?;
            if overflow_read > 0 {
                self.overflow_buffer_len += 1;
                if overflow[self.overflow_buffer_len - 1] != structure[self.overflow_buffer_len - 1]
                {
                    return Ok(false);
                }
            }
        }
        Ok(self.overflow_buffer_len == structure.len())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fmt::Write;
    use std::rc::Rc;

    use super::{BlockAction, BlockAwareReader};
    use crate::reader::{ParserLevelCommentTextReader, PrototypeOnlyCommentTextReader};
    use crate::text::{TextParserReader, TextParserReaderError};
    use crate::util::Utf16String;

    const JAVA_GOLDEN: &str = include_str!("../../tests/fixtures/text_comment_reader_golden.txt");

    #[derive(Debug)]
    struct StringReader {
        value: Vec<u16>,
        position: usize,
    }

    impl StringReader {
        fn new(value: &str) -> Self {
            Self {
                value: value.encode_utf16().collect(),
                position: 0,
            }
        }
    }

    impl TextParserReader for StringReader {
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

    #[derive(Debug, Default)]
    struct TrackingState {
        position: usize,
        zero_returned: bool,
        close_count: usize,
    }

    #[derive(Debug)]
    struct TrackingReader {
        value: Vec<u16>,
        fail_at_position: Option<usize>,
        zero_once: bool,
        fail_close: bool,
        state: Rc<RefCell<TrackingState>>,
    }

    impl TrackingReader {
        fn new(
            value: &str,
            fail_at_position: Option<usize>,
            zero_once: bool,
            fail_close: bool,
        ) -> (Self, Rc<RefCell<TrackingState>>) {
            let state = Rc::new(RefCell::new(TrackingState::default()));
            (
                Self {
                    value: value.encode_utf16().collect(),
                    fail_at_position,
                    zero_once,
                    fail_close,
                    state: Rc::clone(&state),
                },
                state,
            )
        }
    }

    impl TextParserReader for TrackingReader {
        fn read_range(
            &mut self,
            buffer: &mut [u16],
            offset: i32,
            len: i32,
        ) -> Result<i32, TextParserReaderError> {
            let mut state = self.state.borrow_mut();
            if self.zero_once && !state.zero_returned {
                state.zero_returned = true;
                return Ok(0);
            }
            if self
                .fail_at_position
                .is_some_and(|position| state.position >= position)
            {
                return Err(TextParserReaderError::io("read-boom"));
            }
            if state.position >= self.value.len() {
                return Ok(-1);
            }
            let copied = (len as usize).min(self.value.len() - state.position);
            let offset = offset as usize;
            buffer[offset..offset + copied]
                .copy_from_slice(&self.value[state.position..state.position + copied]);
            state.position += copied;
            Ok(copied as i32)
        }

        fn close(&mut self) -> Result<(), TextParserReaderError> {
            self.state.borrow_mut().close_count += 1;
            if self.fail_close {
                return Err(TextParserReaderError::io("close-boom"));
            }
            Ok(())
        }
    }

    fn parser_reader(value: &str) -> Box<dyn TextParserReader> {
        Box::new(ParserLevelCommentTextReader::new(Box::new(
            StringReader::new(value),
        )))
    }

    fn prototype_reader(value: &str) -> Box<dyn TextParserReader> {
        Box::new(PrototypeOnlyCommentTextReader::new(Box::new(
            StringReader::new(value),
        )))
    }

    fn combined_reader(value: &str) -> Box<dyn TextParserReader> {
        Box::new(ParserLevelCommentTextReader::new(Box::new(
            PrototypeOnlyCommentTextReader::new(Box::new(StringReader::new(value))),
        )))
    }

    fn describe_error(error: &TextParserReaderError) -> String {
        format!(
            "{}:{}",
            error.class_name(),
            error
                .java_message()
                .map_or_else(|| "null".to_owned(), |message| message.to_string_lossy())
        )
    }

    fn escape(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('\r', "\\r")
            .replace('\n', "\\n")
            .replace('|', "\\|")
    }

    fn emit_read(
        output: &mut String,
        key: &str,
        mut reader: Box<dyn TextParserReader>,
        buffer_size: usize,
        offset: i32,
        len: i32,
    ) {
        let mut buffer = vec![0; buffer_size];
        let mut result = Vec::new();
        let mut returns = Vec::new();
        let mut throwable = "none".to_owned();
        let mut read = 0;
        let mut guard = 0;
        while read >= 0 && guard < 1000 {
            guard += 1;
            match reader.read_range(&mut buffer, offset, len) {
                Ok(count) => {
                    read = count;
                    returns.push(count.to_string());
                    if count > 0 {
                        result.extend_from_slice(
                            &buffer[offset as usize..offset as usize + count as usize],
                        );
                    }
                }
                Err(error) => {
                    throwable = describe_error(&error);
                    break;
                }
            }
        }
        writeln!(
            output,
            "{key}={}|returns={}|throwable={}",
            escape(&String::from_utf16_lossy(&result)),
            returns.join(","),
            escape(&throwable)
        )
        .expect("write reader golden");
    }

    fn generate_golden() -> String {
        let mut output = String::new();
        writeln!(output, "baseline=10f9dd2eb8cbd98515ce14b149d115e0287d0add")
            .expect("write baseline");

        let parser_cases = [
            "",
            "plain",
            "/* hello */",
            "a/*[-hidden-]*/b",
            "/*[-all-]*/",
            "/* /*[- hello -]]*/ -]*/",
            "/* /*[- hello -]]*/ -]*/ */",
            "/* /*[[- hello -]]*/ -]*/ */",
            "/***[- hello -]***/ -]*/",
            "x/*[-a-]*/y/*[-b-]*/z",
            "x/*[",
            "x/*[-a-]*",
            "中/*[-😀-]*/文",
        ];
        let prototype_cases = [
            "",
            "plain",
            "/* hello */",
            "a/*[+shown+]*/b",
            "/*[+all+]*/",
            "/* /*[+ hello +]*/ +]]]*/// */",
            "/*[+hello+]*/ +]]]*/// aa",
            "/*[[[/*[+hello +]*/ +]]]*///",
            "x/*[+a+]*/y/*[+b+]*/z",
            "x/*[",
            "x/*[+a+]*",
            "中/*[+😀+]*/文",
        ];
        let requests = [(1, 0, 1), (3, 0, 3), (7, 2, 3), (13, 4, 7)];

        for (case_index, case) in parser_cases.iter().enumerate() {
            for (request_index, &(size, offset, len)) in requests.iter().enumerate() {
                emit_read(
                    &mut output,
                    &format!("parser.{case_index}.{request_index}"),
                    parser_reader(case),
                    size,
                    offset,
                    len,
                );
            }
        }
        for (case_index, case) in prototype_cases.iter().enumerate() {
            for (request_index, &(size, offset, len)) in requests.iter().enumerate() {
                emit_read(
                    &mut output,
                    &format!("prototype.{case_index}.{request_index}"),
                    prototype_reader(case),
                    size,
                    offset,
                    len,
                );
            }
        }

        let combined_cases = [
            "a/*[+shown+]*/b/*[-hidden-]*/c",
            "/*[-x/*[+not-shown+]*/y-]*/tail",
            "head/*[+x/*[-hidden-]*/y+]*/tail",
        ];
        for (case_index, case) in combined_cases.iter().enumerate() {
            for (request_index, &(size, offset, len)) in requests.iter().enumerate() {
                emit_read(
                    &mut output,
                    &format!("combined.{case_index}.{request_index}"),
                    combined_reader(case),
                    size,
                    offset,
                    len,
                );
            }
        }

        emit_read(
            &mut output,
            "unfinished.parser",
            parser_reader("a/*[-open"),
            3,
            0,
            3,
        );
        emit_read(
            &mut output,
            "unfinished.prototype",
            prototype_reader("a/*[+open"),
            4,
            1,
            2,
        );

        let (zero_delegate, _) = TrackingReader::new("a/*[-x-]*/b", None, true, false);
        let mut zero_reader = ParserLevelCommentTextReader::new(Box::new(zero_delegate));
        let mut zero_buffer = vec![0; 4];
        let zero = zero_reader
            .read_range(&mut zero_buffer, 2, 0)
            .expect("zero-length tracking read cannot fail");
        writeln!(output, "zero.return={zero}").expect("write zero return");
        emit_read(&mut output, "zero.after", Box::new(zero_reader), 4, 1, 2);

        let (failure_delegate, _) = TrackingReader::new("abcdef", Some(2), false, false);
        emit_read(
            &mut output,
            "delegate.readFailure",
            Box::new(ParserLevelCommentTextReader::new(Box::new(
                failure_delegate,
            ))),
            3,
            0,
            3,
        );

        let (close_delegate, close_state) = TrackingReader::new("plain", None, false, true);
        let mut close_reader = PrototypeOnlyCommentTextReader::new(Box::new(close_delegate));
        let close_error = close_reader
            .close()
            .expect_err("configured close failure must propagate");
        writeln!(output, "close.throwable={}", describe_error(&close_error))
            .expect("write close failure");
        writeln!(output, "close.count={}", close_state.borrow().close_count)
            .expect("write close count");
        output
    }

    fn read_all(
        mut reader: Box<dyn TextParserReader>,
        buffer_size: usize,
        offset: usize,
        len: usize,
    ) -> Result<String, TextParserReaderError> {
        let mut buffer = vec![0; buffer_size];
        let mut output = Vec::new();
        loop {
            let read = reader.read_range(&mut buffer, offset as i32, len as i32)?;
            if read < 0 {
                return Ok(String::from_utf16_lossy(&output));
            }
            if read > 0 {
                output.extend_from_slice(&buffer[offset..offset + read as usize]);
            }
        }
    }

    fn parser_equivalent(message: &str) -> String {
        let chars: Vec<char> = message.chars().collect();
        let mut output = String::new();
        let mut in_comment = false;
        for (index, &character) in chars.iter().enumerate() {
            if !in_comment && character == '/' && index + 1 < chars.len() && chars[index + 1] == '*'
            {
                in_comment = true;
                continue;
            }
            if in_comment && character == '/' && index > 0 && chars[index - 1] == '*' {
                in_comment = false;
                continue;
            }
            if !in_comment {
                output.push(character);
            }
        }
        output
    }

    fn prototype_equivalent(message: &str) -> String {
        let chars: Vec<char> = message.chars().collect();
        let mut output = String::new();
        let mut was_open = false;
        let mut in_open_structure = false;
        let mut in_close_structure = false;
        for (index, &character) in chars.iter().enumerate() {
            if !in_open_structure
                && !in_close_structure
                && character == '/'
                && index + 1 < chars.len()
                && chars[index + 1] == '*'
            {
                in_open_structure = true;
                continue;
            }
            if !in_open_structure && !in_close_structure && was_open && character == '+' {
                in_close_structure = true;
                continue;
            }
            if in_close_structure && character == '/' && index > 0 && chars[index - 1] == '*' {
                in_close_structure = false;
                was_open = false;
                continue;
            }
            if in_open_structure && character == '+' {
                in_open_structure = false;
                was_open = true;
                continue;
            }
            if !in_open_structure && !in_close_structure {
                output.push(character);
            }
        }
        output
    }

    fn generated_messages(prefix: &str, suffix: &str) -> Vec<String> {
        let message = "0123456789";
        let mut all_messages = Vec::new();
        for i in 0..=message.len() {
            let mut first = message.to_owned();
            first.insert_str(i, suffix);
            for j in 0..=i {
                let mut second = first.clone();
                second.insert_str(j, prefix);
                for k in 0..=j {
                    let mut third = second.clone();
                    third.insert_str(k, suffix);
                    all_messages.push(third.clone());
                    for l in 0..=k {
                        let mut fourth = third.clone();
                        fourth.insert_str(l, prefix);
                        all_messages.push(fourth);
                    }
                }
            }
        }
        all_messages
    }

    /// SOURCE_PARITY：固定 Java Oracle 同时覆盖输出、每次 read 返回值、异常和 close。
    #[test]
    fn java_golden_matches_text_comment_reader_streaming_contract() {
        assert_eq!(generate_golden(), JAVA_GOLDEN);
    }

    /// SOURCE_PARITY：迁移上游两个 JUnit `test01` 的结构插入穷举；每个输入在
    /// 1..=8 的读取长度和所有合法 offset 下跨边界读取。
    #[test]
    fn generated_structure_positions_match_upstream_equivalence_algorithms() {
        for message in generated_messages("/*[-", "-]*/") {
            let expected = parser_equivalent(&message);
            for len in 1..=8 {
                for offset in 0..len {
                    let actual =
                        read_all(parser_reader(&message), len + 2, offset, len + 2 - offset)
                            .expect("generated parser-level structure is complete");
                    assert_eq!(
                        actual, expected,
                        "message={message:?}, len={len}, offset={offset}"
                    );
                }
            }
        }
        for message in generated_messages("/*[+", "+]*/") {
            let expected = prototype_equivalent(&message);
            for len in 1..=8 {
                for offset in 0..len {
                    let actual = read_all(
                        prototype_reader(&message),
                        len + 2,
                        offset,
                        len + 2 - offset,
                    )
                    .expect("generated prototype-only structure is complete");
                    assert_eq!(
                        actual, expected,
                        "message={message:?}, len={len}, offset={offset}"
                    );
                }
            }
        }
    }

    /// SOURCE_PARITY：逐项迁移上游两个 JUnit `test02` 的人工回归集合，并使用
    /// 原测试完全相同的 `(buffer_size, len, offset)` 三重循环。
    #[test]
    fn upstream_handwritten_examples_match_for_every_original_buffer_shape() {
        let parser_cases = [
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
        ];
        let prototype_cases = [
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
        ];

        for (message, expected) in parser_cases {
            for buffer_size in 1..=message.len() + 10 {
                for len in 1..=buffer_size {
                    for offset in 0..len {
                        assert_eq!(
                            read_all(parser_reader(message), buffer_size, offset, len - offset,)
                                .expect("上游人工 parser case 是完整结构"),
                            expected,
                            "message={message:?}, buffer={buffer_size}, len={len}, offset={offset}"
                        );
                    }
                }
            }
        }
        for (message, expected) in prototype_cases {
            for buffer_size in 1..=message.len() + 10 {
                for len in 1..=buffer_size {
                    for offset in 0..len {
                        assert_eq!(
                            read_all(prototype_reader(message), buffer_size, offset, len - offset,)
                                .expect("上游人工 prototype case 是完整结构"),
                            expected,
                            "message={message:?}, buffer={buffer_size}, len={len}, offset={offset}"
                        );
                    }
                }
            }
        }
    }

    /// RUST_OBLIGATION：基类的两种动作、UTF-16 code unit 与公共 Reader 默认方法
    /// 必须在 Rust 动态分派中保持 Java 行为。
    #[test]
    fn direct_block_actions_and_utf16_reader_default_method_are_preserved() {
        let value = Utf16String::from_utf16(vec![0xd800, b'a' as u16, 0xdc00]);
        let mut reader = StringReader {
            value: value.as_utf16().to_vec(),
            position: 0,
        };
        let mut buffer = vec![0; 3];
        assert_eq!(reader.read_buffer(&mut buffer), Ok(3));
        assert_eq!(buffer, value.as_utf16());

        let mut discard_all = BlockAwareReader::new(
            Box::new(StringReader::new("a<x>b")),
            BlockAction::DiscardAll,
            &"<".encode_utf16().collect::<Vec<_>>(),
            &">".encode_utf16().collect::<Vec<_>>(),
        );
        let mut all_buffer = vec![0; 8];
        assert_eq!(discard_all.read_range(&mut all_buffer, 0, 8), Ok(2));
        assert_eq!(String::from_utf16_lossy(&all_buffer[..2]), "ab");

        let mut discard_container = BlockAwareReader::new(
            Box::new(StringReader::new("a<x>b")),
            BlockAction::DiscardContainer,
            &"<".encode_utf16().collect::<Vec<_>>(),
            &">".encode_utf16().collect::<Vec<_>>(),
        );
        let mut container_buffer = vec![0; 8];
        assert_eq!(
            discard_container.read_range(&mut container_buffer, 0, 8),
            Ok(3)
        );
        assert_eq!(String::from_utf16_lossy(&container_buffer[..3]), "axb");

        let mut zero_string_reader = StringReader::new("x");
        assert_eq!(zero_string_reader.read_range(&mut [0], 0, 0), Ok(0));

        let (close_delegate, close_state) = TrackingReader::new("", None, false, false);
        let mut parser_close_reader = ParserLevelCommentTextReader::new(Box::new(close_delegate));
        assert_eq!(parser_close_reader.close(), Ok(()));
        assert_eq!(close_state.borrow().close_count, 1);

        // 人工建立 Java 状态机允许出现的“已有 overflow 又遇到尾部候选”状态，
        // 覆盖数组右移及 matchOverflow 的早期不匹配分支。
        let mut overflow_reader = BlockAwareReader::new(
            Box::new(StringReader::new("")),
            BlockAction::DiscardAll,
            &[b'a' as u16, b'b' as u16],
            &[b'c' as u16, b'd' as u16],
        );
        overflow_reader.overflow_buffer = Some(vec![b'x' as u16, 0]);
        overflow_reader.overflow_buffer_len = 1;
        overflow_reader.overflow_last_bytes(&[b'a' as u16], 1, 1);
        assert_eq!(
            overflow_reader.overflow_buffer.as_deref(),
            Some(&[b'a' as u16, b'x' as u16][..])
        );
        assert!(
            !overflow_reader
                .match_overflow(&[b'z' as u16, b'y' as u16])
                .expect("in-memory mismatch cannot fail")
        );

        let mut empty_overflow_reader = BlockAwareReader::new(
            Box::new(StringReader::new("ab")),
            BlockAction::DiscardAll,
            &[b'a' as u16, b'b' as u16],
            &[b'c' as u16, b'd' as u16],
        );
        empty_overflow_reader.overflow_buffer = Some(vec![0; 2]);
        assert!(
            empty_overflow_reader
                .match_overflow(&[b'a' as u16, b'b' as u16])
                .expect("内存 Reader 的完整匹配不能失败")
        );

        let (overflow_failure_delegate, _) =
            TrackingReader::new("/*[-x-]*/", Some(1), false, false);
        let mut overflow_failure =
            ParserLevelCommentTextReader::new(Box::new(overflow_failure_delegate));
        let error = overflow_failure
            .read_range(&mut [0], 0, 1)
            .expect_err("补全跨缓冲区前缀时的委托异常必须传播");
        assert_eq!(error.class_name(), "java.io.IOException");
        assert_eq!(
            error
                .java_message()
                .map(|message| message.to_string_lossy()),
            Some("read-boom".to_owned())
        );

        let (read_bytes_failure_delegate, _) = TrackingReader::new("", Some(0), false, false);
        let mut read_bytes_failure = BlockAwareReader::new(
            Box::new(read_bytes_failure_delegate),
            BlockAction::DiscardAll,
            &[b'a' as u16, b'b' as u16],
            &[b'c' as u16, b'd' as u16],
        );
        read_bytes_failure.overflow_buffer = Some(vec![b'x' as u16, 0]);
        read_bytes_failure.overflow_buffer_len = 1;
        assert!(read_bytes_failure.read_range(&mut [0; 2], 0, 2).is_err());

        assert_eq!(
            describe_error(&TextParserReaderError::new("example.NullMessage", None)),
            "example.NullMessage:null"
        );
        assert!(read_all(parser_reader("/*[-open"), 2, 0, 2).is_err());
    }
}
