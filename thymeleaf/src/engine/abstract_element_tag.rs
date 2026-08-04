use crate::templatemode::TemplateMode;
use crate::util::Utf16String;

use super::{AbstractTemplateEvent, ElementDefinition, ElementDefinitionValue};

/// 元素标签事件共享的模板模式、Definition、名称、位置和 synthetic 状态。
///
/// 对应 Java: `org.thymeleaf.engine.AbstractElementTag`。
///
/// Rust 使用组合代替 Java 抽象类继承；具体 open/close/standalone 标签负责 Visitor
/// 分派及输出，公共只读状态保持在本对象中。
pub struct AbstractElementTag {
    template_event: AbstractTemplateEvent,
    template_mode: TemplateMode,
    element_definition: ElementDefinitionValue,
    element_complete_name: Utf16String,
    synthetic: bool,
}

impl AbstractElementTag {
    /// 创建没有原模板位置的元素标签基础状态。
    ///
    /// 对应 Java:
    /// `AbstractElementTag#AbstractElementTag(TemplateMode,ElementDefinition,String,boolean)`。
    #[must_use]
    pub fn new(
        template_mode: TemplateMode,
        element_definition: ElementDefinitionValue,
        element_complete_name: Utf16String,
        synthetic: bool,
    ) -> Self {
        Self {
            template_event: AbstractTemplateEvent::new(),
            template_mode,
            element_definition,
            element_complete_name,
            synthetic,
        }
    }

    /// 创建携带模板名称、行和列的元素标签基础状态。
    ///
    /// 对应 Java:
    /// `AbstractElementTag#AbstractElementTag(TemplateMode,ElementDefinition,String,boolean,String,int,int)`。
    #[must_use]
    pub fn with_location(
        template_mode: TemplateMode,
        element_definition: ElementDefinitionValue,
        element_complete_name: Utf16String,
        synthetic: bool,
        template_name: Option<Utf16String>,
        line: i32,
        col: i32,
    ) -> Self {
        Self {
            template_event: AbstractTemplateEvent::with_location(template_name, line, col),
            template_mode,
            element_definition,
            element_complete_name,
            synthetic,
        }
    }

    /// 返回模板事件位置状态。
    #[must_use]
    pub const fn as_template_event(&self) -> &AbstractTemplateEvent {
        &self.template_event
    }

    /// 返回标签的模板模式。
    #[must_use]
    pub const fn get_template_mode(&self) -> TemplateMode {
        self.template_mode
    }

    /// 返回模板中原样书写的完整元素名。
    #[must_use]
    pub const fn get_element_complete_name(&self) -> &Utf16String {
        &self.element_complete_name
    }

    /// 返回元素元数据 Definition。
    #[must_use]
    /// 对应 Java: `AbstractElementTag#getElementDefinition()`。
    pub fn get_element_definition(&self) -> &ElementDefinition {
        self.element_definition.as_element_definition()
    }

    /// 返回具体 Definition 包装值，供引擎派生事件时保留对象身份。
    #[must_use]
    pub const fn element_definition_value(&self) -> &ElementDefinitionValue {
        &self.element_definition
    }

    /// 判断标签是否由平衡/修复过程合成。
    #[must_use]
    pub const fn is_synthetic(&self) -> bool {
        self.synthetic
    }
}
