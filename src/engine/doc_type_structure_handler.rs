use std::sync::Arc;

use crate::doctype::IDocTypeStructureHandler;
use crate::model::IModel;
use crate::util::JavaString;

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
    pub(crate) fn new() -> Self {
        Self {
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
        }
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
        self.reset();
        self.set_doc_type = true;
        self.set_doc_type_keyword = Some(keyword);
        self.set_doc_type_element_name = Some(element_name);
        self.set_doc_type_public_id = public_id;
        self.set_doc_type_system_id = system_id;
        self.set_doc_type_internal_subset = internal_subset;
    }

    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool) {
        self.reset();
        self.replace_with_model = true;
        self.replace_with_model_value = Some(model);
        self.replace_with_model_processable = processable;
    }

    fn remove_doc_type(&mut self) {
        self.reset();
        self.remove_doc_type = true;
    }
}
