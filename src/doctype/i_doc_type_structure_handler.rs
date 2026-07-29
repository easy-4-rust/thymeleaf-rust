use std::sync::Arc;

use crate::model::IModel;
use crate::util::JavaString;

/// DOCTYPE Processor 的结构变更合同。
///
/// 对应 Java: `org.thymeleaf.processor.doctype.IDocTypeStructureHandler`。
pub trait IDocTypeStructureHandler {
    /// 清除已指定动作。
    fn reset(&mut self);
    /// 设置 DOCTYPE 的全部组成部分。
    fn set_doc_type(
        &mut self,
        keyword: JavaString,
        element_name: JavaString,
        public_id: Option<JavaString>,
        system_id: Option<JavaString>,
        internal_subset: Option<JavaString>,
    );
    /// 使用模型替换当前事件。
    fn replace_with(&mut self, model: Arc<dyn IModel>, processable: bool);
    /// 删除当前 DOCTYPE。
    fn remove_doc_type(&mut self);
}
