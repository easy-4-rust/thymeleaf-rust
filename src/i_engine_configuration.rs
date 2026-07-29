use std::any::TypeId;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::cache::ICacheManager;
use crate::cdatasection::ICDATASectionProcessor;
use crate::comment::ICommentProcessor;
use crate::context::IEngineContextFactory;
use crate::decoupled::IDecoupledTemplateLogicResolver;
use crate::dialect::IDialect;
use crate::doctype::IDocTypeProcessor;
use crate::element::IElementProcessor;
use crate::engine::{AttributeDefinitions, ElementDefinitions, ITemplateManager};
use crate::expression::{IExpressionObjectFactory, TemplateValue};
use crate::linkbuilder::ILinkBuilder;
use crate::messageresolver::IMessageResolver;
use crate::model::IModelFactory;
use crate::postprocessor::IPostProcessor;
use crate::preprocessor::IPreProcessor;
use crate::processinginstruction::IProcessingInstructionProcessor;
use crate::templateboundaries::ITemplateBoundariesProcessor;
use crate::templateresolver::ITemplateResolver;
use crate::text::ITextProcessor;
use crate::util::JavaString;
use crate::xmldeclaration::IXMLDeclarationProcessor;
use crate::{DialectConfiguration, TemplateMode};

/// 决定 TemplateEngine 全部解析、处理、表达式及缓存行为的配置合同。
///
/// 对应 Java: `org.thymeleaf.IEngineConfiguration`。
///
/// 实现必须可在线程间安全共享；返回的集合按配置完成后的稳定执行顺序暴露，不允许
/// 调用方修改配置内部集合。
pub trait IEngineConfiguration: Send + Sync {
    /// 返回按执行顺序排列的模板 Resolver。
    fn get_template_resolvers(&self) -> Vec<&dyn ITemplateResolver>;
    /// 返回按执行顺序排列的消息 Resolver。
    fn get_message_resolvers(&self) -> Vec<&dyn IMessageResolver>;
    /// 返回按执行顺序排列的 LinkBuilder。
    fn get_link_builders(&self) -> Vec<&dyn ILinkBuilder>;
    /// 返回可空缓存管理器。
    fn get_cache_manager(&self) -> Option<&dyn ICacheManager>;
    /// 返回 EngineContext 工厂。
    fn get_engine_context_factory(&self) -> &dyn IEngineContextFactory;
    /// 返回解耦模板逻辑 Resolver。
    fn get_decoupled_template_logic_resolver(&self) -> &dyn IDecoupledTemplateLogicResolver;
    /// 返回全部方言配置。
    fn get_dialect_configurations(&self) -> Vec<&DialectConfiguration>;
    /// 返回去重后的方言实例。
    fn get_dialects(&self) -> Vec<&dyn IDialect>;
    /// 按运行时类型筛选方言。
    fn get_dialects_of_type(&self, type_id: TypeId) -> Vec<&dyn IDialect>;
    /// 判断 StandardDialect 是否存在。
    fn is_standard_dialect_present(&self) -> bool;
    /// 返回 StandardDialect 前缀；不存在时为 `None`。
    fn get_standard_dialect_prefix(&self) -> Option<&JavaString>;
    /// 返回元素定义仓库。
    fn get_element_definitions(&self) -> &ElementDefinitions;
    /// 返回属性定义仓库。
    fn get_attribute_definitions(&self) -> &AttributeDefinitions;
    /// 返回指定模式的模板边界 Processor。
    fn get_template_boundaries_processors(
        &self,
        template_mode: TemplateMode,
    ) -> Vec<&dyn ITemplateBoundariesProcessor>;
    /// 返回指定模式的 CDATA Processor。
    fn get_cdata_section_processors(
        &self,
        template_mode: TemplateMode,
    ) -> Vec<&dyn ICDATASectionProcessor>;
    /// 返回指定模式的 Comment Processor。
    fn get_comment_processors(&self, template_mode: TemplateMode) -> Vec<&dyn ICommentProcessor>;
    /// 返回指定模式的 DOCTYPE Processor。
    fn get_doc_type_processors(&self, template_mode: TemplateMode) -> Vec<&dyn IDocTypeProcessor>;
    /// 返回指定模式的 Element Processor。
    fn get_element_processors(&self, template_mode: TemplateMode) -> Vec<&dyn IElementProcessor>;
    /// 返回指定模式的 Text Processor。
    fn get_text_processors(&self, template_mode: TemplateMode) -> Vec<&dyn ITextProcessor>;
    /// 返回指定模式的 ProcessingInstruction Processor。
    fn get_processing_instruction_processors(
        &self,
        template_mode: TemplateMode,
    ) -> Vec<&dyn IProcessingInstructionProcessor>;
    /// 返回指定模式的 XMLDeclaration Processor。
    fn get_xml_declaration_processors(
        &self,
        template_mode: TemplateMode,
    ) -> Vec<&dyn IXMLDeclarationProcessor>;
    /// 返回指定模式的预处理器。
    fn get_pre_processors(&self, template_mode: TemplateMode) -> Vec<&dyn IPreProcessor>;
    /// 返回指定模式的后处理器。
    fn get_post_processors(&self, template_mode: TemplateMode) -> Vec<&dyn IPostProcessor>;
    /// 返回不可修改的执行属性。
    fn get_execution_attributes(&self)
    -> &IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>;
    /// 返回聚合表达式对象工厂。
    fn get_expression_object_factory(&self) -> &dyn IExpressionObjectFactory;
    /// 返回模板管理器。
    fn get_template_manager(&self) -> &dyn ITemplateManager;
    /// 返回指定模式的稳定模型工厂。
    fn get_model_factory(&self, template_mode: TemplateMode) -> &dyn IModelFactory;
}
