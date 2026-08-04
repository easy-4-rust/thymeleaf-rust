use std::fmt::{Display, Formatter};
use std::io;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::model::{IComment, IModelVisitor, ITemplateEvent};
use crate::util::{JavaCharSequence, JavaWriter, TextUtilsError, Utf16String};

use super::{AbstractTextualTemplateEvent, IEngineTemplateEvent, ITemplateHandler};

const COMMENT_PREFIX: &str = "<!--";
const COMMENT_SUFFIX: &str = "-->";

/// 引擎内部的不可变标记注释事件。
///
/// 对应 Java: `org.thymeleaf.engine.Comment`。
pub struct Comment {
    textual_event: AbstractTextualTemplateEvent,
    prefix: Utf16String,
    suffix: Utf16String,
    computed_comment: RwLock<Option<Utf16String>>,
}

impl Comment {
    /// 使用标准 HTML/XML 注释边界创建事件。
    ///
    /// 对应 Java: `Comment#Comment(CharSequence)`。
    #[must_use]
    pub fn new(content: Option<Arc<dyn JavaCharSequence>>) -> Self {
        Self::with_boundaries(
            Utf16String::from_rust_str(COMMENT_PREFIX),
            content,
            Utf16String::from_rust_str(COMMENT_SUFFIX),
        )
    }

    /// 使用 parser 保留的自定义前后缀创建事件。
    ///
    /// 对应 Java: `Comment#Comment(String,CharSequence,String)`。
    #[must_use]
    pub fn with_boundaries(
        prefix: Utf16String,
        content: Option<Arc<dyn JavaCharSequence>>,
        suffix: Utf16String,
    ) -> Self {
        Self {
            textual_event: AbstractTextualTemplateEvent::new(content),
            prefix,
            suffix,
            computed_comment: RwLock::new(None),
        }
    }

    /// 使用标准边界和原模板位置创建事件。
    ///
    /// 对应 Java: `Comment#Comment(CharSequence,String,int,int)`。
    #[must_use]
    pub fn with_location(
        content: Option<Arc<dyn JavaCharSequence>>,
        template_name: Option<Utf16String>,
        line: i32,
        col: i32,
    ) -> Self {
        Self::with_boundaries_and_location(
            Utf16String::from_rust_str(COMMENT_PREFIX),
            content,
            Utf16String::from_rust_str(COMMENT_SUFFIX),
            template_name,
            line,
            col,
        )
    }

    /// 使用 parser 保留的边界和原模板位置创建事件。
    ///
    /// 对应 Java: `Comment#Comment(String,CharSequence,String,String,int,int)`。
    #[must_use]
    pub fn with_boundaries_and_location(
        prefix: Utf16String,
        content: Option<Arc<dyn JavaCharSequence>>,
        suffix: Utf16String,
        template_name: Option<Utf16String>,
        line: i32,
        col: i32,
    ) -> Self {
        Self {
            textual_event: AbstractTextualTemplateEvent::with_location(
                content,
                template_name,
                line,
                col,
            ),
            prefix,
            suffix,
            computed_comment: RwLock::new(None),
        }
    }

    fn compute_comment(&self) -> Result<Utf16String, TextUtilsError> {
        if let Some(value) = read_lock(&self.computed_comment).as_ref() {
            return Ok(value.clone());
        }
        let content = self
            .textual_event
            .get_content_text()?
            .ok_or(TextUtilsError::NullPointer)?;
        let mut result = Vec::with_capacity(self.prefix.len() + content.len() + self.suffix.len());
        result.extend_from_slice(self.prefix.as_utf16());
        result.extend_from_slice(content.as_utf16());
        result.extend_from_slice(self.suffix.as_utf16());
        let result = Utf16String::from_utf16(result);
        *write_lock(&self.computed_comment) = Some(result.clone());
        Ok(result)
    }

    /// 返回 parser 保留的注释前缀。
    pub(crate) const fn prefix(&self) -> &Utf16String {
        &self.prefix
    }

    /// 返回 parser 保留的注释后缀。
    pub(crate) const fn suffix(&self) -> &Utf16String {
        &self.suffix
    }
}

impl JavaCharSequence for Comment {
    fn java_sequence_class_name(&self) -> &str {
        "org.thymeleaf.engine.Comment"
    }

    fn java_length(&self) -> Result<i32, TextUtilsError> {
        Ok((self.prefix.len() as i32)
            .wrapping_add(self.textual_event.get_content_length()?)
            .wrapping_add(self.suffix.len() as i32))
    }

    fn java_char_at(&self, index: i32) -> Result<u16, TextUtilsError> {
        let prefix_length = self.prefix.len() as i32;
        if index < prefix_length {
            return self.prefix.java_char_at(index);
        }
        let content_end = prefix_length.wrapping_add(self.textual_event.get_content_length()?);
        if index >= content_end {
            return self.suffix.java_char_at(index.wrapping_sub(content_end));
        }
        self.textual_event
            .char_at_content(index.wrapping_sub(prefix_length))
    }

    fn as_utf16_string(&self) -> Option<&Utf16String> {
        None
    }

    fn java_to_string(&self) -> Result<Utf16String, TextUtilsError> {
        self.compute_comment()
    }

    fn java_sub_sequence(&self, start: i32, end: i32) -> Result<Utf16String, TextUtilsError> {
        let prefix_length = self.prefix.len() as i32;
        let content_end = prefix_length.wrapping_add(self.textual_event.get_content_length()?);
        if start >= prefix_length && end < content_end {
            return self.textual_event.content_sub_sequence(
                start.wrapping_sub(prefix_length),
                end.wrapping_sub(prefix_length),
            );
        }
        self.compute_comment()?.java_sub_sequence(start, end)
    }
}

impl IComment for Comment {
    fn as_engine_comment(&self) -> Option<&Self> {
        Some(self)
    }

    fn get_comment(&self) -> Result<Option<Utf16String>, TextUtilsError> {
        self.compute_comment().map(Some)
    }

    fn get_content(&self) -> Result<Option<Utf16String>, TextUtilsError> {
        self.textual_event.get_content_text()
    }
}

impl ITemplateEvent for Comment {
    fn has_location(&self) -> bool {
        self.textual_event.as_template_event().has_location()
    }

    fn get_template_name(&self) -> Option<&Utf16String> {
        self.textual_event.as_template_event().get_template_name()
    }

    fn get_line(&self) -> i32 {
        self.textual_event.as_template_event().get_line()
    }

    fn get_col(&self) -> i32 {
        self.textual_event.as_template_event().get_col()
    }

    fn accept(&self, visitor: &mut dyn IModelVisitor) {
        visitor.visit_comment(self);
    }

    fn be_handled(
        self: Arc<Self>,
        handler: &mut dyn ITemplateHandler,
    ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
        handler.handle_comment(self)
    }

    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
        writer.write_utf16(self.prefix.as_utf16())?;
        self.textual_event.write_content(writer)?;
        writer.write_utf16(self.suffix.as_utf16())
    }
}

impl IEngineTemplateEvent for Comment {}

impl Display for Comment {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let comment = self.compute_comment().map_err(|_| std::fmt::Error)?;
        formatter.write_str(&comment.to_string_lossy())
    }
}

fn read_lock<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_lock<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
