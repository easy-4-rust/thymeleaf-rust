use std::io;
use std::sync::{Arc, RwLock};

use crate::util::{JavaCharSequence, JavaString, JavaWriter, TextUtilsError};

use super::{AbstractTemplateEvent, engine_event_utils::compute_inlineable};

/// 文本型模板事件共享的延迟字符串、长度和内容分类逻辑。
///
/// 对应 Java: `org.thymeleaf.engine.AbstractTextualTemplateEvent`。
pub struct AbstractTextualTemplateEvent {
    template_event: AbstractTemplateEvent,
    content: Option<Arc<dyn JavaCharSequence>>,
    content_string: Option<JavaString>,
    content_length: i32,
    computed_content_string: RwLock<Option<JavaString>>,
    computed_content_length: RwLock<i32>,
    computed_whitespace: RwLock<Option<bool>>,
    computed_inlineable: RwLock<Option<bool>>,
}

impl AbstractTextualTemplateEvent {
    /// 创建不带位置的文本事件。
    #[must_use]
    pub fn new(content: Option<Arc<dyn JavaCharSequence>>) -> Self {
        Self::with_location(content, None, -1, -1)
    }

    /// 创建带原模板位置的文本事件。
    #[must_use]
    pub fn with_location(
        content: Option<Arc<dyn JavaCharSequence>>,
        template_name: Option<JavaString>,
        line: i32,
        col: i32,
    ) -> Self {
        let content_string = content
            .as_ref()
            .and_then(|value| value.as_java_string())
            .cloned();
        let content_length = content_string
            .as_ref()
            .map_or(-1, |value| value.len() as i32);
        Self {
            template_event: AbstractTemplateEvent::with_location(template_name, line, col),
            content,
            content_string,
            content_length,
            computed_content_string: RwLock::new(None),
            computed_content_length: RwLock::new(-1),
            computed_whitespace: RwLock::new(None),
            computed_inlineable: RwLock::new(None),
        }
    }

    /// 返回事件位置基类。
    #[must_use]
    pub const fn as_template_event(&self) -> &AbstractTemplateEvent {
        &self.template_event
    }

    /// 按 Java 缓存语义取得可空内容字符串。
    pub fn get_content_text(&self) -> Result<Option<JavaString>, TextUtilsError> {
        if self.content_string.is_some() || self.content.is_none() {
            return Ok(self.content_string.clone());
        }
        if let Some(value) = read_lock(&self.computed_content_string).as_ref() {
            return Ok(Some(value.clone()));
        }
        let value = sequence(self)?.java_to_string()?;
        *write_lock(&self.computed_content_string) = Some(value.clone());
        Ok(Some(value))
    }

    /// 返回内容 UTF-16 长度；null 内容返回 `-1`。
    pub fn get_content_length(&self) -> Result<i32, TextUtilsError> {
        if self.content_length >= 0 || self.content.is_none() {
            return Ok(self.content_length);
        }
        let cached = *read_lock(&self.computed_content_length);
        if cached >= 0 {
            return Ok(cached);
        }
        let length = sequence(self)?.java_length()?;
        *write_lock(&self.computed_content_length) = length;
        Ok(length)
    }

    /// 返回指定 UTF-16 代码单元。
    pub fn char_at_content(&self, index: i32) -> Result<u16, TextUtilsError> {
        if let Some(value) = self.content_string.as_ref() {
            return value.java_char_at(index);
        }
        if let Some(value) = read_lock(&self.computed_content_string).as_ref() {
            return value.java_char_at(index);
        }
        sequence(self)?.java_char_at(index)
    }

    /// 返回指定 UTF-16 子序列。
    pub fn content_sub_sequence(&self, start: i32, end: i32) -> Result<JavaString, TextUtilsError> {
        if let Some(value) = self.content_string.as_ref() {
            return value.java_sub_sequence(start, end);
        }
        if let Some(value) = read_lock(&self.computed_content_string).as_ref() {
            return value.java_sub_sequence(start, end);
        }
        sequence(self)?.java_sub_sequence(start, end)
    }

    /// 判断非空内容是否全部为 Java whitespace。
    pub fn is_whitespace(&self) -> Result<bool, TextUtilsError> {
        if let Some(value) = *read_lock(&self.computed_whitespace) {
            return Ok(value);
        }
        let mut remaining = self.get_content_length()?;
        let result = if remaining == 0 {
            false
        } else {
            let mut whitespace = true;
            while remaining != 0 {
                remaining -= 1;
                let character = self.char_at_content(remaining)?;
                if character != u16::from(b' ')
                    && character != u16::from(b'\n')
                    && !java_is_whitespace(character)
                {
                    whitespace = false;
                    break;
                }
            }
            whitespace
        };
        *write_lock(&self.computed_whitespace) = Some(result);
        Ok(result)
    }

    /// 判断内容是否包含 `[[...]]` 或 `[(...)]` 内联标记对。
    ///
    /// 对应 Java `AbstractTextualTemplateEvent#isInlineable()`，算法与
    /// `EngineEventUtils::compute_inlineable` 一致（右向左闭包支配扫描）。
    pub fn is_inlineable(&self) -> Result<bool, TextUtilsError> {
        if let Some(value) = *read_lock(&self.computed_inlineable) {
            return Ok(value);
        }
        let text = self
            .get_content_text()?
            .unwrap_or_else(|| JavaString::from_utf16(Vec::new()));
        let result = compute_inlineable(&text)?;
        *write_lock(&self.computed_inlineable) = Some(result);
        Ok(result)
    }

    /// 将内容写出，并在序列适配器声明 `IWritableCharSequence` 能力时避免整串分配。
    pub fn write_content(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
        if let Some(content) = self.content_string.as_ref() {
            return writer.write_utf16(content.as_utf16());
        }
        if let Some(content) = read_lock(&self.computed_content_string).as_ref() {
            return writer.write_utf16(content.as_utf16());
        }
        if let Some(content) = self.content.as_ref()
            && let Some(result) = content.write_direct(writer)
        {
            return result;
        }
        let content = self
            .get_content_text()
            .map_err(|error| io::Error::other(error.to_string()))?;
        if let Some(content) = content {
            writer.write_utf16(content.as_utf16())?;
        }
        Ok(())
    }
}

fn sequence(event: &AbstractTextualTemplateEvent) -> Result<&dyn JavaCharSequence, TextUtilsError> {
    event.content.as_deref().ok_or(TextUtilsError::NullPointer)
}

fn java_is_whitespace(character: u16) -> bool {
    matches!(
        char::from_u32(u32::from(character)),
        Some(
            '\u{0009}'
            | '\u{000A}'
            | '\u{000B}'
            | '\u{000C}'
            | '\u{000D}'
            | '\u{001C}'
            | '\u{001D}'
            | '\u{001E}'
            | '\u{001F}'
            | '\u{1680}'
            | '\u{2000}'..='\u{2006}'
            | '\u{2008}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{205F}'
            | '\u{3000}',
        )
    )
}

fn read_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
