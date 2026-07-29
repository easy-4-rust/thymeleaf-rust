use std::io;
use std::sync::Arc;

use crate::IEngineConfiguration;
use crate::TemplateMode;
use crate::util::JavaWriter;

use super::{IModelVisitor, ITemplateEvent};

/// 模板模型的事件序列合同。
///
/// 对应 Java: `org.thymeleaf.model.IModel`。
pub trait IModel {
    /// 返回创建模型时使用的同一引擎配置。
    fn get_configuration(&self) -> &dyn IEngineConfiguration;
    /// 返回模型的模板模式。
    fn get_template_mode(&self) -> TemplateMode;
    /// 返回事件数量。
    fn size(&self) -> usize;
    /// 返回指定位置的事件。
    fn get(&self, pos: usize) -> Arc<dyn ITemplateEvent>;
    /// 在末尾添加事件；`None` 对应 Java null 并保持不变。
    fn add(&mut self, event: Option<Arc<dyn ITemplateEvent>>) -> Result<(), IModelError>;
    /// 在指定位置插入事件。
    fn insert(
        &mut self,
        pos: usize,
        event: Option<Arc<dyn ITemplateEvent>>,
    ) -> Result<(), IModelError>;
    /// 替换指定位置的事件。
    fn replace(
        &mut self,
        pos: usize,
        event: Option<Arc<dyn ITemplateEvent>>,
    ) -> Result<(), IModelError>;
    /// 追加另一个模型的事件。
    fn add_model(&mut self, model: Option<&dyn IModel>) -> Result<(), IModelError>;
    /// 在指定位置插入另一个模型。
    fn insert_model(&mut self, pos: usize, model: Option<&dyn IModel>) -> Result<(), IModelError>;
    /// 删除指定位置的事件。
    fn remove(&mut self, pos: usize) -> Result<(), IModelError>;
    /// 清空事件序列。
    fn reset(&mut self) -> Result<(), IModelError>;
    /// 创建可变模型副本；事件保持同一不可变对象身份。
    fn clone_model(&self) -> Box<dyn IModel>;
    /// 依次把事件分派给 Visitor。
    fn accept(&self, visitor: &mut dyn IModelVisitor);
    /// 依次写出所有事件。
    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()>;
}

/// 模型变更或索引访问错误。
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum IModelError {
    /// 不可变 TemplateModel 被要求修改。
    #[error(
        "Modifications are not allowed on immutable model objects. This model object is an \
         immutable implementation of the org.thymeleaf.model.IModel interface"
    )]
    ImmutableModel,
    /// 事件索引越界。
    #[error("Model event index out of bounds: {0}")]
    IndexOutOfBounds(usize),
    /// TemplateStart/TemplateEnd 只能由解析器放入完整 TemplateModel。
    #[error(
        "Cannot insert event of type TemplateStart/TemplateEnd. These events can only be added \
         to models internally during template parsing."
    )]
    TemplateBoundaryInsertion,
    /// 两个模型来自不同引擎配置。
    #[error("Cannot add model created using a different Template Engine Configuration.")]
    DifferentConfiguration,
    /// 两个模型使用不同模板模式。
    #[error("Cannot add model created using a different Template Mode.")]
    DifferentTemplateMode,
    /// 自定义事件不是 Thymeleaf 支持的事件子接口。
    #[error("Cannot handle template event type")]
    UnsupportedEvent,
}
