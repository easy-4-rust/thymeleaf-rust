use std::sync::Arc;

use crate::doctype::IDocTypeStructureHandler;
use crate::model::IModel;
use crate::util::{JavaString, Validate, ValidateError};

/// 引擎内部 DocType 结构动作状态机。
///
/// 对应 Java: `org.thymeleaf.engine.DocTypeStructureHandler`。
pub(crate) struct DocTypeStructureHandler {
    pub(crate) set_doc_type: bool,
    pub(crate) set_doc_type_keyword: Option<JavaString>,
    pub(crate) set_doc_type_element_name: Option<JavaString>,
    pub(crate) set_doc_type_public_id: Option<JavaString>,
    pub(crate) set_doc_type_system_id: Option<JavaString>,
    pub(crate) set_doc_type_internal_subset: Option<JavaString>,
    pub(crate) replace_with_model: bool,
    pub(crate) replace_with_model_value: Option<Arc<dyn IModel>>,
    pub(crate) replace_with_model_processable: bool,
    pub(crate) remove_doc_type: bool,
}

impl DocTypeStructureHandler {
    /// 创建无待执行动作的处理器。
    /// 对应 Java 语义：`DocTypeStructureHandler` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new() -> Self {
        let mut handler = Self {
            set_doc_type: false,
            set_doc_type_keyword: None,
            set_doc_type_element_name: None,
            set_doc_type_public_id: None,
            set_doc_type_system_id: None,
            set_doc_type_internal_subset: None,
            replace_with_model: false,
            replace_with_model_value: None,
            replace_with_model_processable: false,
            remove_doc_type: false,
        };
        handler.reset();
        handler
    }

    /// 设置 DOCTYPE 的全部组成部分。
    ///
    /// 对应 Java: `DocTypeStructureHandler#setDocType(String, String, String,
    /// String, String)`。方法先重置，再依次校验 `keyword`、`element_name`；
    /// `public_id`、`system_id` 与 `internal_subset` 均允许为空。
    pub(crate) fn set_doc_type_nullable(
        &mut self,
        keyword: Option<JavaString>,
        element_name: Option<JavaString>,
        public_id: Option<JavaString>,
        system_id: Option<JavaString>,
        internal_subset: Option<JavaString>,
    ) -> Result<(), ValidateError> {
        self.reset();
        Validate::not_null(keyword.as_ref(), Some("Keyword cannot be null"))?;
        Validate::not_null(element_name.as_ref(), Some("Element name cannot be null"))?;
        self.set_doc_type = true;
        self.set_doc_type_keyword = keyword;
        self.set_doc_type_element_name = element_name;
        self.set_doc_type_public_id = public_id;
        self.set_doc_type_system_id = system_id;
        self.set_doc_type_internal_subset = internal_subset;
        Ok(())
    }

    /// 使用模型替换 DOCTYPE。对应 Java:
    /// `DocTypeStructureHandler#replaceWith(IModel, boolean)`。
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

impl IDocTypeStructureHandler for DocTypeStructureHandler {
    fn reset(&mut self) {
        self.set_doc_type = false;
        self.set_doc_type_keyword = None;
        self.set_doc_type_element_name = None;
        self.set_doc_type_public_id = None;
        self.set_doc_type_system_id = None;
        self.set_doc_type_internal_subset = None;
        self.replace_with_model = false;
        self.replace_with_model_value = None;
        self.replace_with_model_processable = false;
        self.remove_doc_type = false;
    }

    fn set_doc_type(
        &mut self,
        keyword: JavaString,
        element_name: JavaString,
        public_id: Option<JavaString>,
        system_id: Option<JavaString>,
        internal_subset: Option<JavaString>,
    ) {
        self.set_doc_type_nullable(
            Some(keyword),
            Some(element_name),
            public_id,
            system_id,
            internal_subset,
        )
        .expect("Rust non-null DOCTYPE boundary must satisfy Java validation");
    }

    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.replace_with_nullable(Some(model), processable)
            .expect("Rust non-null model boundary must satisfy Java validation");
    }

    fn remove_doc_type(&mut self) {
        self.reset();
        self.remove_doc_type = true;
    }
}
