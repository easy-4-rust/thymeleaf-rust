use std::fmt::{Display, Formatter};
use std::io;
use std::sync::Arc;

use crate::model::{ICloseElementTag, IElementTag, IModelVisitor, ITemplateEvent};
use crate::templatemode::TemplateMode;
use crate::util::{FastStringWriter, TemplateWriter, Utf16String};

use super::{
    AbstractElementTag, ElementDefinition, ElementDefinitionValue, IEngineTemplateEvent,
    ITemplateHandler,
};

/// 引擎内部的不可变关闭元素标签。
///
/// 对应 Java: `org.thymeleaf.engine.CloseElementTag`。
pub struct CloseElementTag {
    element_tag: AbstractElementTag,
    trailing_white_space: Option<Utf16String>,
    unmatched: bool,
}

impl CloseElementTag {
    /// 创建没有原模板位置的关闭标签。
    ///
    /// 对应 Java:
    /// `CloseElementTag#CloseElementTag(TemplateMode,ElementDefinition,String,String,boolean,boolean)`。
    #[must_use]
    pub fn new(
        template_mode: TemplateMode,
        element_definition: ElementDefinitionValue,
        element_complete_name: Utf16String,
        trailing_white_space: Option<Utf16String>,
        synthetic: bool,
        unmatched: bool,
    ) -> Self {
        Self {
            element_tag: AbstractElementTag::new(
                template_mode,
                element_definition,
                element_complete_name,
                synthetic,
            ),
            trailing_white_space,
            unmatched,
        }
    }

    /// 创建携带模板名称、行和列的关闭标签。
    ///
    /// 对应 Java:
    /// `CloseElementTag#CloseElementTag(TemplateMode,ElementDefinition,String,String,boolean,boolean,String,int,int)`。
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn with_location(
        template_mode: TemplateMode,
        element_definition: ElementDefinitionValue,
        element_complete_name: Utf16String,
        trailing_white_space: Option<Utf16String>,
        synthetic: bool,
        unmatched: bool,
        template_name: Option<Utf16String>,
        line: i32,
        col: i32,
    ) -> Self {
        Self {
            element_tag: AbstractElementTag::with_location(
                template_mode,
                element_definition,
                element_complete_name,
                synthetic,
                template_name,
                line,
                col,
            ),
            trailing_white_space,
            unmatched,
        }
    }

    /// 返回完整 UTF-16 输出表示；synthetic 标签返回空字符串。
    #[must_use]
    /// 对应 Java 语义：`CloseElementTag` 的 `to_utf16_string` 行为（Rust 侧辅助/私有路径）。
    pub fn to_utf16_string(&self) -> Utf16String {
        let mut writer = FastStringWriter::new();
        self.write(&mut writer)
            .expect("FastStringWriter must accept complete UTF-16 slices");
        writer.to_string()
    }
}

impl ICloseElementTag for CloseElementTag {
    fn is_unmatched(&self) -> bool {
        self.unmatched
    }
}

impl IElementTag for CloseElementTag {
    fn get_template_mode(&self) -> TemplateMode {
        self.element_tag.get_template_mode()
    }

    fn get_element_complete_name(&self) -> &Utf16String {
        self.element_tag.get_element_complete_name()
    }

    fn get_element_definition(&self) -> &ElementDefinition {
        self.element_tag.get_element_definition()
    }

    fn is_synthetic(&self) -> bool {
        self.element_tag.is_synthetic()
    }
}

impl ITemplateEvent for CloseElementTag {
    fn has_location(&self) -> bool {
        self.element_tag.as_template_event().has_location()
    }

    fn get_template_name(&self) -> Option<&Utf16String> {
        self.element_tag.as_template_event().get_template_name()
    }

    fn get_line(&self) -> i32 {
        self.element_tag.as_template_event().get_line()
    }

    fn get_col(&self) -> i32 {
        self.element_tag.as_template_event().get_col()
    }

    fn accept(&self, visitor: &mut dyn IModelVisitor) {
        visitor.visit_close_element_tag(self);
    }

    fn be_handled(
        self: Arc<Self>,
        handler: &mut dyn ITemplateHandler,
    ) -> Result<(), Box<dyn crate::exceptions::TemplateEngineException>> {
        handler.handle_close_element(self)
    }

    fn as_close_element_tag(&self) -> Option<&dyn ICloseElementTag> {
        Some(self)
    }

    fn write(&self, writer: &mut dyn TemplateWriter) -> io::Result<()> {
        if self.element_tag.is_synthetic() {
            return Ok(());
        }
        if self.element_tag.get_template_mode().is_text() {
            writer.write_utf16(&[u16::from(b'['), u16::from(b'/')])?;
            writer.write_utf16(self.element_tag.get_element_complete_name().as_utf16())?;
            if let Some(spaces) = self.trailing_white_space.as_ref() {
                writer.write_utf16(spaces.as_utf16())?;
            }
            return writer.write_utf16(&[u16::from(b']')]);
        }
        writer.write_utf16(&[u16::from(b'<'), u16::from(b'/')])?;
        writer.write_utf16(self.element_tag.get_element_complete_name().as_utf16())?;
        if let Some(spaces) = self.trailing_white_space.as_ref() {
            writer.write_utf16(spaces.as_utf16())?;
        }
        writer.write_utf16(&[u16::from(b'>')])
    }
}

impl IEngineTemplateEvent for CloseElementTag {}

impl Display for CloseElementTag {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.to_utf16_string().to_string_lossy())
    }
}
