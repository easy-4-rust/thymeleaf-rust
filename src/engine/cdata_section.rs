use std::fmt::{Display, Formatter};
use std::io;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::model::{ICDATASection, IModelVisitor, ITemplateEvent};
use crate::util::{JavaCharSequence, JavaString, JavaWriter, TextUtilsError};

use super::{AbstractTextualTemplateEvent, IEngineTemplateEvent, ITemplateHandler};

const CDATA_PREFIX: &str = "<![CDATA[";
const CDATA_SUFFIX: &str = "]]>";

/// 引擎内部的不可变 CDATA section 事件。
///
/// 对应 Java: `org.thymeleaf.engine.CDATASection`。
pub struct CDATASection {
    textual_event: AbstractTextualTemplateEvent,
    prefix: JavaString,
    suffix: JavaString,
    computed_cdata_section: RwLock<Option<JavaString>>,
}

impl CDATASection {
    /// 使用标准 CDATA 边界创建事件。
    ///
    /// 对应 Java: `CDATASection#CDATASection(CharSequence)`。
    #[must_use]
    pub fn new(content: Option<Arc<dyn JavaCharSequence>>) -> Self {
        Self::with_boundaries(
            JavaString::from_rust_str(CDATA_PREFIX),
            content,
            JavaString::from_rust_str(CDATA_SUFFIX),
        )
    }

    /// 使用 parser 保留的自定义前后缀创建事件。
    ///
    /// 对应 Java: `CDATASection#CDATASection(String,CharSequence,String)`。
    #[must_use]
    pub fn with_boundaries(
        prefix: JavaString,
        content: Option<Arc<dyn JavaCharSequence>>,
        suffix: JavaString,
    ) -> Self {
        Self {
            textual_event: AbstractTextualTemplateEvent::new(content),
            prefix,
            suffix,
            computed_cdata_section: RwLock::new(None),
        }
    }

    /// 使用标准边界和模板位置创建事件。
    ///
    /// 对应 Java: `CDATASection#CDATASection(CharSequence,String,int,int)`。
    #[must_use]
    pub fn with_location(
        content: Option<Arc<dyn JavaCharSequence>>,
        template_name: Option<JavaString>,
        line: i32,
        col: i32,
    ) -> Self {
        Self::with_boundaries_and_location(
            JavaString::from_rust_str(CDATA_PREFIX),
            content,
            JavaString::from_rust_str(CDATA_SUFFIX),
            template_name,
            line,
            col,
        )
    }

    /// 使用 parser 保留的边界和模板位置创建事件。
    ///
    /// 对应 Java:
    /// `CDATASection#CDATASection(String,CharSequence,String,String,int,int)`。
    #[must_use]
    pub fn with_boundaries_and_location(
        prefix: JavaString,
        content: Option<Arc<dyn JavaCharSequence>>,
        suffix: JavaString,
        template_name: Option<JavaString>,
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
            computed_cdata_section: RwLock::new(None),
        }
    }

    fn compute_cdata_section(&self) -> Result<JavaString, TextUtilsError> {
        if let Some(value) = read_lock(&self.computed_cdata_section).as_ref() {
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
        let result = JavaString::from_utf16(result);
        *write_lock(&self.computed_cdata_section) = Some(result.clone());
        Ok(result)
    }
}

impl JavaCharSequence for CDATASection {
    fn java_sequence_class_name(&self) -> &str {
        "org.thymeleaf.engine.CDATASection"
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

    fn as_java_string(&self) -> Option<&JavaString> {
        None
    }

    fn java_to_string(&self) -> Result<JavaString, TextUtilsError> {
        self.compute_cdata_section()
    }

    fn java_sub_sequence(&self, start: i32, end: i32) -> Result<JavaString, TextUtilsError> {
        let prefix_length = self.prefix.len() as i32;
        let content_end = prefix_length.wrapping_add(self.textual_event.get_content_length()?);
        if start >= prefix_length && end < content_end {
            return self.textual_event.content_sub_sequence(
                start.wrapping_sub(prefix_length),
                end.wrapping_sub(prefix_length),
            );
        }
        self.compute_cdata_section()?.java_sub_sequence(start, end)
    }
}

impl ICDATASection for CDATASection {
    fn get_cdata_section(&self) -> Result<Option<JavaString>, TextUtilsError> {
        self.compute_cdata_section().map(Some)
    }

    fn get_content(&self) -> Result<Option<JavaString>, TextUtilsError> {
        self.textual_event.get_content_text()
    }
}

impl ITemplateEvent for CDATASection {
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
        visitor.visit_cdata_section(self);
    }

    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
        writer.write_utf16(self.prefix.as_utf16())?;
        self.textual_event.write_content(writer)?;
        writer.write_utf16(self.suffix.as_utf16())
    }
}

impl IEngineTemplateEvent for CDATASection {
    fn be_handled(&self, handler: &mut dyn ITemplateHandler) {
        handler.handle_cdata_section(self);
    }
}

impl Display for CDATASection {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let cdata = self.compute_cdata_section().map_err(|_| std::fmt::Error)?;
        formatter.write_str(&cdata.to_string_lossy())
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
