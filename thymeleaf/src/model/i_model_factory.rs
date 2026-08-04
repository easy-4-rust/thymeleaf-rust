use std::sync::Arc;

use indexmap::IndexMap;

use crate::engine::{AttributeName, TemplateData};
use crate::util::Utf16String;

use super::{
    AttributeValueQuotes, ICDATASection, ICloseElementTag, IComment, IDocType, IModel,
    IOpenElementTag, IProcessableElementTag, IProcessingInstruction, IStandaloneElementTag,
    ITemplateEvent, IText, IXMLDeclaration,
};

/// 创建和不可变修改模板事件与模型的工厂合同。
///
/// 对应 Java: `org.thymeleaf.model.IModelFactory`。
pub trait IModelFactory: Send + Sync {
    /// 创建空的可变模型。
    fn create_model(&self) -> Box<dyn IModel>;
    /// 创建包含一个事件的可变模型。
    fn create_model_with_event(
        &self,
        event: Arc<dyn ITemplateEvent>,
    ) -> Result<Box<dyn IModel>, crate::model::IModelError>;
    /// 按指定 owner 模板解析模型片段。
    fn parse(
        &self,
        owner_template: &TemplateData,
        template: &Utf16String,
    ) -> Result<Box<dyn IModel>, crate::exceptions::TemplateProcessingException>;
    /// 创建 CDATA 事件。
    fn create_cdata_section(
        &self,
        content: Utf16String,
    ) -> Result<Arc<dyn ICDATASection>, crate::exceptions::TemplateProcessingException>;
    /// 创建注释事件。
    fn create_comment(
        &self,
        content: Utf16String,
    ) -> Result<Arc<dyn IComment>, crate::exceptions::TemplateProcessingException>;
    /// 创建 HTML5 DOCTYPE。
    fn create_html5_doc_type(
        &self,
    ) -> Result<Arc<dyn IDocType>, crate::exceptions::TemplateProcessingException>;
    /// 创建带 public/system ID 的 DOCTYPE。
    fn create_doc_type(
        &self,
        public_id: Option<Utf16String>,
        system_id: Option<Utf16String>,
    ) -> Result<Arc<dyn IDocType>, crate::exceptions::TemplateProcessingException>;
    /// 创建完整 DOCTYPE。
    fn create_full_doc_type(
        &self,
        keyword: Utf16String,
        element_name: Utf16String,
        public_id: Option<Utf16String>,
        system_id: Option<Utf16String>,
        internal_subset: Option<Utf16String>,
    ) -> Result<Arc<dyn IDocType>, crate::exceptions::TemplateProcessingException>;
    /// 创建 processing instruction。
    fn create_processing_instruction(
        &self,
        target: Utf16String,
        content: Utf16String,
    ) -> Result<Arc<dyn IProcessingInstruction>, crate::exceptions::TemplateProcessingException>;
    /// 创建文本事件。
    fn create_text(&self, text: Utf16String) -> Arc<dyn IText>;
    /// 创建 XML declaration。
    fn create_xml_declaration(
        &self,
        version: Option<Utf16String>,
        encoding: Option<Utf16String>,
        standalone: Option<Utf16String>,
    ) -> Result<Arc<dyn IXMLDeclaration>, crate::exceptions::TemplateProcessingException>;
    /// 创建独立标签。
    fn create_standalone_element_tag(
        &self,
        element_name: Utf16String,
        attributes: Option<&IndexMap<Utf16String, Option<Utf16String>>>,
        attribute_value_quotes: AttributeValueQuotes,
        synthetic: bool,
        minimized: bool,
    ) -> Result<Arc<dyn IStandaloneElementTag>, crate::exceptions::TemplateProcessingException>;
    /// 创建开始标签。
    fn create_open_element_tag(
        &self,
        element_name: Utf16String,
        attributes: Option<&IndexMap<Utf16String, Option<Utf16String>>>,
        attribute_value_quotes: AttributeValueQuotes,
        synthetic: bool,
    ) -> Result<Arc<dyn IOpenElementTag>, crate::exceptions::TemplateProcessingException>;
    /// 创建结束标签。
    fn create_close_element_tag(
        &self,
        element_name: Utf16String,
        synthetic: bool,
        unmatched: bool,
    ) -> Result<Arc<dyn ICloseElementTag>, crate::exceptions::TemplateProcessingException>;
    /// 返回设置属性后的同类型新标签。
    fn set_attribute(
        &self,
        tag: Arc<dyn IProcessableElementTag>,
        attribute_name: Utf16String,
        attribute_value: Option<Utf16String>,
        attribute_value_quotes: Option<AttributeValueQuotes>,
    ) -> Result<Arc<dyn IProcessableElementTag>, crate::exceptions::TemplateProcessingException>;
    /// 返回替换属性后的同类型新标签。
    fn replace_attribute(
        &self,
        tag: Arc<dyn IProcessableElementTag>,
        old_attribute_name: &AttributeName,
        attribute_name: Utf16String,
        attribute_value: Option<Utf16String>,
        attribute_value_quotes: Option<AttributeValueQuotes>,
    ) -> Result<Arc<dyn IProcessableElementTag>, crate::exceptions::TemplateProcessingException>;
    /// 返回删除属性后的同类型标签；属性不存在时允许返回原对象身份。
    fn remove_attribute(
        &self,
        tag: Arc<dyn IProcessableElementTag>,
        attribute_name: &AttributeName,
    ) -> Result<Arc<dyn IProcessableElementTag>, crate::exceptions::TemplateProcessingException>;
}
