use std::fmt::{Display, Formatter};
use std::io;

use crate::model::{IModelVisitor, IProcessingInstruction, ITemplateEvent};
use crate::util::{JavaString, JavaWriter};

use super::{AbstractTemplateEvent, IEngineTemplateEvent, ITemplateHandler};

/// 引擎内部的不可变 XML processing instruction 事件。
///
/// 对应 Java: `org.thymeleaf.engine.ProcessingInstruction`。
pub struct ProcessingInstruction {
    template_event: AbstractTemplateEvent,
    target: Option<JavaString>,
    content: Option<JavaString>,
    processing_instruction: JavaString,
}

impl ProcessingInstruction {
    /// 从 target 与可空内容计算完整 processing instruction。
    ///
    /// 对应 Java: `ProcessingInstruction#ProcessingInstruction(String,String)`。
    #[must_use]
    pub fn new(target: Option<JavaString>, content: Option<JavaString>) -> Self {
        let processing_instruction =
            compute_processing_instruction(target.as_ref(), content.as_ref());
        Self {
            template_event: AbstractTemplateEvent::new(),
            target,
            content,
            processing_instruction,
        }
    }

    /// 从 parser 保留的完整文本、分解字段和位置创建事件。
    ///
    /// 对应 Java:
    /// `ProcessingInstruction#ProcessingInstruction(String,String,String,String,int,int)`。
    /// 完整文本为 null 时按 target/content 重新计算。
    #[must_use]
    pub fn with_location(
        processing_instruction: Option<JavaString>,
        target: Option<JavaString>,
        content: Option<JavaString>,
        template_name: Option<JavaString>,
        line: i32,
        col: i32,
    ) -> Self {
        let processing_instruction = processing_instruction
            .unwrap_or_else(|| compute_processing_instruction(target.as_ref(), content.as_ref()));
        Self {
            template_event: AbstractTemplateEvent::with_location(template_name, line, col),
            target,
            content,
            processing_instruction,
        }
    }
}

impl IProcessingInstruction for ProcessingInstruction {
    fn get_target(&self) -> Option<&JavaString> {
        self.target.as_ref()
    }

    fn get_content(&self) -> Option<&JavaString> {
        self.content.as_ref()
    }

    fn get_processing_instruction(&self) -> Option<&JavaString> {
        Some(&self.processing_instruction)
    }
}

impl ITemplateEvent for ProcessingInstruction {
    fn has_location(&self) -> bool {
        self.template_event.has_location()
    }

    fn get_template_name(&self) -> Option<&JavaString> {
        self.template_event.get_template_name()
    }

    fn get_line(&self) -> i32 {
        self.template_event.get_line()
    }

    fn get_col(&self) -> i32 {
        self.template_event.get_col()
    }

    fn accept(&self, visitor: &mut dyn IModelVisitor) {
        visitor.visit_processing_instruction(self);
    }

    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
        writer.write_utf16(self.processing_instruction.as_utf16())
    }
}

impl IEngineTemplateEvent for ProcessingInstruction {
    fn be_handled(&self, handler: &mut dyn ITemplateHandler) {
        handler.handle_processing_instruction(self);
    }
}

impl Display for ProcessingInstruction {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.processing_instruction.to_string_lossy())
    }
}

fn compute_processing_instruction(
    target: Option<&JavaString>,
    content: Option<&JavaString>,
) -> JavaString {
    let mut result = Vec::with_capacity(100);
    result.extend("<?".encode_utf16());
    append_nullable(&mut result, target);
    if let Some(content) = content {
        result.push(u16::from(b' '));
        result.extend_from_slice(content.as_utf16());
    }
    result.extend("?>".encode_utf16());
    JavaString::from_utf16(result)
}

fn append_nullable(result: &mut Vec<u16>, value: Option<&JavaString>) {
    match value {
        Some(value) => result.extend_from_slice(value.as_utf16()),
        None => result.extend("null".encode_utf16()),
    }
}
