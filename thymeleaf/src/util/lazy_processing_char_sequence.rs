use std::io;
use std::sync::{Arc, Mutex, RwLock};

use crate::context::ITemplateContext;
use crate::model::IModel;

use super::{IWritableCharSequence, JavaCharSequence, JavaWriter, TextUtilsError, Utf16String};

/// 延迟执行 TemplateModel 并直接写入最终输出的字符序列。
///
/// 对应 Java: `org.thymeleaf.util.LazyProcessingCharSequence`。
pub struct LazyProcessingCharSequence {
    context: Arc<dyn ITemplateContext>,
    template_model: Arc<dyn IModel>,
    resolved_text: RwLock<Option<Utf16String>>,
}

impl LazyProcessingCharSequence {
    /// 创建尚未执行模板模型的延迟字符序列。
    ///
    /// 对应 Java: `LazyProcessingCharSequence#LazyProcessingCharSequence`。
    #[must_use]
    pub fn new(context: Arc<dyn ITemplateContext>, template_model: Arc<dyn IModel>) -> Self {
        Self {
            context,
            template_model,
            resolved_text: RwLock::new(None),
        }
    }

    fn resolve_text(&self) -> Result<Utf16String, TextUtilsError> {
        if let Some(text) = self.read_resolved_text()? {
            return Ok(text.clone());
        }
        let output = Arc::new(Mutex::new(Vec::new()));
        let writer = SharedUtf16Writer {
            output: Arc::clone(&output),
        };
        self.context
            .get_configuration()
            .get_template_manager()
            .process(
                self.template_model.as_ref(),
                self.context.as_ref(),
                Box::new(writer),
            )
            .map_err(processing_text_error)?;
        let text = Utf16String::from_utf16(
            output
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        );
        *self
            .resolved_text
            .write()
            .map_err(|_| processing_lock_error())? = Some(text.clone());
        Ok(text)
    }

    fn read_resolved_text(&self) -> Result<Option<Utf16String>, TextUtilsError> {
        self.resolved_text
            .read()
            .map(|guard| guard.clone())
            .map_err(|_| processing_lock_error())
    }
}

impl JavaCharSequence for LazyProcessingCharSequence {
    fn as_utf16_string(&self) -> Option<&Utf16String> {
        None
    }

    fn java_sequence_class_name(&self) -> &str {
        "org.thymeleaf.util.LazyProcessingCharSequence"
    }

    fn java_length(&self) -> Result<i32, TextUtilsError> {
        Ok(self.resolve_text()?.len() as i32)
    }

    fn java_char_at(&self, index: i32) -> Result<u16, TextUtilsError> {
        self.resolve_text()?.java_char_at(index)
    }

    fn java_to_string(&self) -> Result<Utf16String, TextUtilsError> {
        self.resolve_text()
    }

    fn java_sub_sequence(&self, start: i32, end: i32) -> Result<Utf16String, TextUtilsError> {
        self.resolve_text()?.java_sub_sequence(start, end)
    }

    fn write_direct(&self, writer: &mut dyn JavaWriter) -> Option<io::Result<()>> {
        Some(IWritableCharSequence::write(self, writer))
    }
}

impl IWritableCharSequence for LazyProcessingCharSequence {
    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
        if let Some(text) = self
            .read_resolved_text()
            .map_err(|error| io::Error::other(error.to_string()))?
        {
            return writer.write_utf16(text.as_utf16());
        }
        let output = Arc::new(Mutex::new(Vec::new()));
        let processing_writer = SharedUtf16Writer {
            output: Arc::clone(&output),
        };
        self.context
            .get_configuration()
            .get_template_manager()
            .process(
                self.template_model.as_ref(),
                self.context.as_ref(),
                Box::new(processing_writer),
            )
            // `Writer` 只能返回 `io::Error`，但不能把模板异常压扁为字符串；把原异常
            // 作为 payload 保留下来，Java 式 cause 链才能继续暴露解析消息和模板位置。
            .map_err(io::Error::other)?;
        writer.write_utf16(
            &output
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }
}

struct SharedUtf16Writer {
    output: Arc<Mutex<Vec<u16>>>,
}

impl JavaWriter for SharedUtf16Writer {
    fn write_utf16(&mut self, characters: &[u16]) -> io::Result<()> {
        self.output
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(characters);
        Ok(())
    }
}

fn processing_text_error(error: crate::exceptions::TemplateProcessingException) -> TextUtilsError {
    TextUtilsError::SequenceAccess {
        class_name: "org.thymeleaf.exceptions.TemplateProcessingException".to_owned(),
        message: Some(Utf16String::from_rust_str(&error.get_message())),
    }
}

fn processing_lock_error() -> TextUtilsError {
    TextUtilsError::SequenceAccess {
        class_name: "java.lang.IllegalStateException".to_owned(),
        message: Some(Utf16String::from_rust_str(
            "LazyProcessingCharSequence resolved text lock is poisoned",
        )),
    }
}
