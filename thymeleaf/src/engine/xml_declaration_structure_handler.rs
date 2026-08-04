use std::sync::Arc;

use crate::model::IModel;
use crate::util::{Utf16String, Validate, ValidateError};
use crate::xmldeclaration::IXMLDeclarationStructureHandler;

/// 引擎内部 XMLDeclaration 结构动作状态机。
///
/// 对应 Java: `org.thymeleaf.engine.XMLDeclarationStructureHandler`。
pub(crate) struct XMLDeclarationStructureHandler {
    pub(crate) set_xml_declaration: bool,
    pub(crate) set_xml_declaration_keyword: Option<Utf16String>,
    pub(crate) set_xml_declaration_version: Option<Utf16String>,
    pub(crate) set_xml_declaration_encoding: Option<Utf16String>,
    pub(crate) set_xml_declaration_standalone: Option<Utf16String>,
    pub(crate) replace_with_model: bool,
    pub(crate) replace_with_model_value: Option<Arc<dyn IModel>>,
    pub(crate) replace_with_model_processable: bool,
    pub(crate) remove_xml_declaration: bool,
}

impl XMLDeclarationStructureHandler {
    /// 创建无待执行动作的处理器。
    /// 对应 Java 语义：`XMLDeclarationStructureHandler` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new() -> Self {
        let mut handler = Self {
            set_xml_declaration: false,
            set_xml_declaration_keyword: None,
            set_xml_declaration_version: None,
            set_xml_declaration_encoding: None,
            set_xml_declaration_standalone: None,
            replace_with_model: false,
            replace_with_model_value: None,
            replace_with_model_processable: false,
            remove_xml_declaration: false,
        };
        handler.reset();
        handler
    }

    /// 设置 XML declaration 的全部组成部分。
    ///
    /// 对应 Java:
    /// `XMLDeclarationStructureHandler#setXMLDeclaration(String, String, String, String)`。
    /// 方法先重置，再校验 keyword；其余三个属性允许为空。
    pub(crate) fn set_xml_declaration_nullable(
        &mut self,
        keyword: Option<Utf16String>,
        version: Option<Utf16String>,
        encoding: Option<Utf16String>,
        standalone: Option<Utf16String>,
    ) -> Result<(), ValidateError> {
        self.reset();
        Validate::not_null(keyword.as_ref(), Some("Keyword cannot be null"))?;
        self.set_xml_declaration = true;
        self.set_xml_declaration_keyword = keyword;
        self.set_xml_declaration_version = version;
        self.set_xml_declaration_encoding = encoding;
        self.set_xml_declaration_standalone = standalone;
        Ok(())
    }

    /// 使用模型替换 XML declaration。对应 Java:
    /// `XMLDeclarationStructureHandler#replaceWith(IModel, boolean)`。
    pub(crate) fn replace_with_nullable(
        &mut self,
        model: Option<Arc<dyn IModel>>,
        processable: bool,
    ) -> Result<(), ValidateError> {
        self.reset();
        Validate::not_null(model.as_deref(), Some("Model cannot be null"))?;
        self.replace_with_model = true;
        self.replace_with_model_value = model;
        self.replace_with_model_processable = processable;
        Ok(())
    }
}

impl IXMLDeclarationStructureHandler for XMLDeclarationStructureHandler {
    fn reset(&mut self) {
        self.set_xml_declaration = false;
        self.set_xml_declaration_keyword = None;
        self.set_xml_declaration_version = None;
        self.set_xml_declaration_encoding = None;
        self.set_xml_declaration_standalone = None;
        self.replace_with_model = false;
        self.replace_with_model_value = None;
        self.replace_with_model_processable = false;
        self.remove_xml_declaration = false;
    }

    fn set_xml_declaration(
        &mut self,
        keyword: Utf16String,
        version: Option<Utf16String>,
        encoding: Option<Utf16String>,
        standalone: Option<Utf16String>,
    ) {
        self.set_xml_declaration_nullable(Some(keyword), version, encoding, standalone)
            .expect("Rust non-null XML-declaration boundary must satisfy Java validation");
    }

    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.replace_with_nullable(Some(model), processable)
            .expect("Rust non-null model boundary must satisfy Java validation");
    }

    fn remove_xml_declaration(&mut self) {
        self.reset();
        self.remove_xml_declaration = true;
    }
}
