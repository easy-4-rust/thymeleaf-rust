use std::fmt::{Display, Formatter};
use std::io;
use std::sync::Arc;

use crate::model::{IModelVisitor, ITemplateEvent, IText};
use crate::util::{JavaCharSequence, JavaString, JavaWriter, TextUtilsError};

use super::{AbstractTextualTemplateEvent, IEngineTemplateEvent, ITemplateHandler};

/// 引擎内部的不可变文本事件。
///
/// 对应 Java: `org.thymeleaf.engine.Text`。
pub struct Text {
    textual_event: AbstractTextualTemplateEvent,
}

impl Text {
    /// 创建不带模板位置的文本事件。
    ///
    /// 对应 Java: `Text#Text(CharSequence)`。
    #[must_use]
    pub fn new(text: Option<Arc<dyn JavaCharSequence>>) -> Self {
        Self {
            textual_event: AbstractTextualTemplateEvent::new(text),
        }
    }

    /// 创建带模板名称、行和列的文本事件。
    ///
    /// 对应 Java: `Text#Text(CharSequence,String,int,int)`。
    #[must_use]
    pub fn with_location(
        text: Option<Arc<dyn JavaCharSequence>>,
        template_name: Option<JavaString>,
        line: i32,
        col: i32,
    ) -> Self {
        Self {
            textual_event: AbstractTextualTemplateEvent::with_location(
                text,
                template_name,
                line,
                col,
            ),
        }
    }

    /// 判断文本内容是否全部为 Java whitespace。
    pub fn is_whitespace(&self) -> Result<bool, TextUtilsError> {
        self.textual_event.is_whitespace()
    }

    /// 判断文本是否含可执行内联表达式边界。
    pub fn is_inlineable(&self) -> Result<bool, TextUtilsError> {
        self.textual_event.is_inlineable()
    }
}

impl JavaCharSequence for Text {
    fn java_sequence_class_name(&self) -> &str {
        "org.thymeleaf.engine.Text"
    }

    fn java_length(&self) -> Result<i32, TextUtilsError> {
        self.textual_event.get_content_length()
    }

    fn java_char_at(&self, index: i32) -> Result<u16, TextUtilsError> {
        self.textual_event.char_at_content(index)
    }

    fn as_java_string(&self) -> Option<&JavaString> {
        None
    }

    fn java_to_string(&self) -> Result<JavaString, TextUtilsError> {
        self.textual_event
            .get_content_text()?
            .ok_or(TextUtilsError::NullPointer)
    }

    fn java_sub_sequence(&self, start: i32, end: i32) -> Result<JavaString, TextUtilsError> {
        self.textual_event.content_sub_sequence(start, end)
    }
}

impl IText for Text {
    fn get_text(&self) -> Result<Option<JavaString>, TextUtilsError> {
        self.textual_event.get_content_text()
    }
}

impl ITemplateEvent for Text {
    fn has_location(&self) -> bool {
        self.textual_event.as_template_event().has_location()
    }

    fn get_template_name(&self) -> Option<&JavaString> {
        self.textual_event.as_template_event().get_template_name()
    }

    fn get_line(&self) -> i32 {
        self.textual_event.as_template_event().get_line()
    }

    fn get_col(&self) -> i32 {
        self.textual_event.as_template_event().get_col()
    }

    fn accept(&self, visitor: &mut dyn IModelVisitor) {
        visitor.visit_text(self);
    }

    fn be_handled(
        self: Arc<Self>,
        handler: &mut dyn ITemplateHandler,
    ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
        handler.handle_text(self)
    }

    fn as_text(&self) -> Option<&dyn IText> {
        Some(self)
    }

    fn into_text(self: Arc<Self>) -> Option<Arc<dyn IText>> {
        Some(self)
    }

    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
        self.textual_event.write_content(writer)
    }
}

impl IEngineTemplateEvent for Text {}

impl Display for Text {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let text = self
            .textual_event
            .get_content_text()
            .map_err(|_| std::fmt::Error)?
            .ok_or(std::fmt::Error)?;
        formatter.write_str(&text.to_string_lossy())
    }
}
