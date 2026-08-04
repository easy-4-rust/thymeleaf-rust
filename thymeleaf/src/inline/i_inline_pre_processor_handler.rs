/// 内联表达式预处理事件处理器。
///
/// 对应 Java: `org.thymeleaf.standard.inline.IInlinePreProcessorHandler`。
///
/// 该接口在模板解析阶段同步接收文本、元素边界和属性事件。`buffer` 使用 UTF-16
/// 代码单元保存 Java `char[]`，`None` 精确表示 Java `null`；实现可以修改缓冲区，
/// 后续调用方将观察到同一数组上的修改。
///
/// 此接口有意不定义 CDATA 和注释事件：内联处理可能把文本拆成多个事件，而拆分
/// 带定界符的 CDATA 或注释会产生无效语法。因此其中允许的内联表达式必须留到
/// Processor 执行阶段处理。
pub trait IInlinePreProcessorHandler {
    /// 处理普通文本片段。对应 Java: `IInlinePreProcessorHandler#handleText`。
    ///
    /// `offset`、`len` 指定 `buffer` 中的片段，`line`、`col` 是起始位置。
    fn handle_text(
        &mut self,
        buffer: Option<&mut [u16]>,
        offset: i32,
        len: i32,
        line: i32,
        col: i32,
    );

    /// 处理独立元素开始边界。
    ///
    /// 对应 Java: `IInlinePreProcessorHandler#handleStandaloneElementStart`。
    #[allow(clippy::too_many_arguments)]
    fn handle_standalone_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        minimized: bool,
        line: i32,
        col: i32,
    );

    /// 处理独立元素结束边界。
    ///
    /// 对应 Java: `IInlinePreProcessorHandler#handleStandaloneElementEnd`。
    #[allow(clippy::too_many_arguments)]
    fn handle_standalone_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        minimized: bool,
        line: i32,
        col: i32,
    );

    /// 处理开放元素开始边界。
    ///
    /// 对应 Java: `IInlinePreProcessorHandler#handleOpenElementStart`。
    fn handle_open_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    );

    /// 处理开放元素结束边界。
    ///
    /// 对应 Java: `IInlinePreProcessorHandler#handleOpenElementEnd`。
    fn handle_open_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    );

    /// 处理解析器自动补出的开放元素开始边界。
    ///
    /// 对应 Java: `IInlinePreProcessorHandler#handleAutoOpenElementStart`。
    fn handle_auto_open_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    );

    /// 处理解析器自动补出的开放元素结束边界。
    ///
    /// 对应 Java: `IInlinePreProcessorHandler#handleAutoOpenElementEnd`。
    fn handle_auto_open_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    );

    /// 处理关闭元素开始边界。
    ///
    /// 对应 Java: `IInlinePreProcessorHandler#handleCloseElementStart`。
    fn handle_close_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    );

    /// 处理关闭元素结束边界。
    ///
    /// 对应 Java: `IInlinePreProcessorHandler#handleCloseElementEnd`。
    fn handle_close_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    );

    /// 处理解析器自动补出的关闭元素开始边界。
    ///
    /// 对应 Java: `IInlinePreProcessorHandler#handleAutoCloseElementStart`。
    fn handle_auto_close_element_start(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    );

    /// 处理解析器自动补出的关闭元素结束边界。
    ///
    /// 对应 Java: `IInlinePreProcessorHandler#handleAutoCloseElementEnd`。
    fn handle_auto_close_element_end(
        &mut self,
        buffer: Option<&mut [u16]>,
        name_offset: i32,
        name_len: i32,
        line: i32,
        col: i32,
    );

    /// 处理属性名称、运算符和值的全部源范围。
    ///
    /// 对应 Java: `IInlinePreProcessorHandler#handleAttribute`。所有 offset/len 都指向
    /// 同一 `buffer`；value content 排除定界符，value outer 包含定界符。
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
    );
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fmt::Write;

    use super::IInlinePreProcessorHandler;

    const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    const JAVA_GOLDEN: &str =
        include_str!("../../tests/fixtures/inline_pre_processor_handler_golden.txt");

    #[derive(Clone, Copy)]
    enum Event {
        Text,
        StandaloneStart,
        StandaloneEnd,
        OpenStart,
        OpenEnd,
        AutoOpenStart,
        AutoOpenEnd,
        CloseStart,
        CloseEnd,
        AutoCloseStart,
        AutoCloseEnd,
        Attribute,
    }

    impl Event {
        const ALL: [Self; 12] = [
            Self::Text,
            Self::StandaloneStart,
            Self::StandaloneEnd,
            Self::OpenStart,
            Self::OpenEnd,
            Self::AutoOpenStart,
            Self::AutoOpenEnd,
            Self::CloseStart,
            Self::CloseEnd,
            Self::AutoCloseStart,
            Self::AutoCloseEnd,
            Self::Attribute,
        ];

        const fn key(self) -> &'static str {
            match self {
                Self::Text => "text",
                Self::StandaloneStart => "standaloneStart",
                Self::StandaloneEnd => "standaloneEnd",
                Self::OpenStart => "openStart",
                Self::OpenEnd => "openEnd",
                Self::AutoOpenStart => "autoOpenStart",
                Self::AutoOpenEnd => "autoOpenEnd",
                Self::CloseStart => "closeStart",
                Self::CloseEnd => "closeEnd",
                Self::AutoCloseStart => "autoCloseStart",
                Self::AutoCloseEnd => "autoCloseEnd",
                Self::Attribute => "attribute",
            }
        }
    }

    #[derive(Default)]
    struct RecordingHandler {
        event: String,
    }

    impl RecordingHandler {
        fn record(&mut self, name: &str, mut buffer: Option<&mut [u16]>, arguments: &str) {
            if let Some(value) = buffer.as_deref_mut().and_then(|value| value.first_mut()) {
                *value = value.wrapping_add(1);
            }
            self.event = format!("{name}({arguments})@{}", hex(buffer.as_deref()));
        }
    }

    impl IInlinePreProcessorHandler for RecordingHandler {
        fn handle_text(
            &mut self,
            buffer: Option<&mut [u16]>,
            offset: i32,
            len: i32,
            line: i32,
            col: i32,
        ) {
            self.record("text", buffer, &format!("{offset},{len},{line},{col}"));
        }

        fn handle_standalone_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            minimized: bool,
            line: i32,
            col: i32,
        ) {
            self.record(
                "standaloneStart",
                buffer,
                &format!("{name_offset},{name_len},{minimized},{line},{col}"),
            );
        }

        fn handle_standalone_element_end(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            minimized: bool,
            line: i32,
            col: i32,
        ) {
            self.record(
                "standaloneEnd",
                buffer,
                &format!("{name_offset},{name_len},{minimized},{line},{col}"),
            );
        }

        fn handle_open_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) {
            self.record(
                "openStart",
                buffer,
                &format!("{name_offset},{name_len},{line},{col}"),
            );
        }

        fn handle_open_element_end(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) {
            self.record(
                "openEnd",
                buffer,
                &format!("{name_offset},{name_len},{line},{col}"),
            );
        }

        fn handle_auto_open_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) {
            self.record(
                "autoOpenStart",
                buffer,
                &format!("{name_offset},{name_len},{line},{col}"),
            );
        }

        fn handle_auto_open_element_end(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) {
            self.record(
                "autoOpenEnd",
                buffer,
                &format!("{name_offset},{name_len},{line},{col}"),
            );
        }

        fn handle_close_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) {
            self.record(
                "closeStart",
                buffer,
                &format!("{name_offset},{name_len},{line},{col}"),
            );
        }

        fn handle_close_element_end(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) {
            self.record(
                "closeEnd",
                buffer,
                &format!("{name_offset},{name_len},{line},{col}"),
            );
        }

        fn handle_auto_close_element_start(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) {
            self.record(
                "autoCloseStart",
                buffer,
                &format!("{name_offset},{name_len},{line},{col}"),
            );
        }

        fn handle_auto_close_element_end(
            &mut self,
            buffer: Option<&mut [u16]>,
            name_offset: i32,
            name_len: i32,
            line: i32,
            col: i32,
        ) {
            self.record(
                "autoCloseEnd",
                buffer,
                &format!("{name_offset},{name_len},{line},{col}"),
            );
        }

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
        ) {
            self.record(
                "attribute",
                buffer,
                &format!(
                    "{name_offset},{name_len},{name_line},{name_col},{operator_offset},\
                     {operator_len},{operator_line},{operator_col},{value_content_offset},\
                     {value_content_len},{value_outer_offset},{value_outer_len},{value_line},\
                     {value_col}"
                )
                .replace(' ', ""),
            );
        }
    }

    fn invoke(
        handler: &mut dyn IInlinePreProcessorHandler,
        event: Event,
        buffer: Option<&mut [u16]>,
    ) {
        match event {
            Event::Text => handler.handle_text(buffer, -1, 2, 3, 4),
            Event::StandaloneStart => {
                handler.handle_standalone_element_start(buffer, 5, 6, true, 7, 8);
            }
            Event::StandaloneEnd => {
                handler.handle_standalone_element_end(buffer, 9, 10, false, 11, 12);
            }
            Event::OpenStart => handler.handle_open_element_start(buffer, 13, 14, 15, 16),
            Event::OpenEnd => handler.handle_open_element_end(buffer, 17, 18, 19, 20),
            Event::AutoOpenStart => {
                handler.handle_auto_open_element_start(buffer, 21, 22, 23, 24);
            }
            Event::AutoOpenEnd => {
                handler.handle_auto_open_element_end(buffer, 25, 26, 27, 28);
            }
            Event::CloseStart => handler.handle_close_element_start(buffer, 29, 30, 31, 32),
            Event::CloseEnd => handler.handle_close_element_end(buffer, 33, 34, 35, 36),
            Event::AutoCloseStart => {
                handler.handle_auto_close_element_start(buffer, 37, 38, 39, 40);
            }
            Event::AutoCloseEnd => {
                handler.handle_auto_close_element_end(buffer, 41, 42, 43, 44);
            }
            Event::Attribute => handler.handle_attribute(
                buffer, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58,
            ),
        }
    }

    fn hex(value: Option<&[u16]>) -> String {
        let Some(value) = value else {
            return "null".to_owned();
        };
        let mut output = String::new();
        for (index, code_unit) in value.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            write!(output, "{code_unit:04x}").expect("writing to String cannot fail");
        }
        output
    }

    fn fixture() -> BTreeMap<&'static str, &'static str> {
        JAVA_GOLDEN
            .lines()
            .map(|line| line.split_once('=').expect("golden line must contain '='"))
            .collect()
    }

    #[test]
    fn golden_covers_all_callbacks_and_nullable_mutable_buffers() {
        let expected = fixture();
        assert_eq!(expected["baseline"], JAVA_BASELINE);
        assert_eq!(expected.len(), 1 + Event::ALL.len() * 2);

        for event in Event::ALL {
            let mut handler = RecordingHandler::default();
            let mut buffer = [0x0061, 0xd800, 0x007a];
            invoke(&mut handler, event, Some(&mut buffer));
            let buffer_key = format!("{}.buffer", event.key());
            assert_eq!(handler.event, expected[buffer_key.as_str()]);

            invoke(&mut handler, event, None);
            let null_key = format!("{}.null", event.key());
            assert_eq!(handler.event, expected[null_key.as_str()]);
        }
    }
}
