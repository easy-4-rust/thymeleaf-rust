use std::any::TypeId;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::ExecutionAttributeValue;
use crate::cache::ICacheManager;
use crate::cdatasection::ICDATASectionProcessor;
use crate::comment::ICommentProcessor;
use crate::context::IEngineContextFactory;
use crate::decoupled::IDecoupledTemplateLogicResolver;
use crate::dialect::IDialect;
use crate::doctype::IDocTypeProcessor;
use crate::element::IElementProcessor;
use crate::engine::{AttributeDefinitions, ElementDefinitions, ITemplateManager};
use crate::expression::IExpressionObjectFactory;
use crate::linkbuilder::ILinkBuilder;
use crate::messageresolver::IMessageResolver;
use crate::model::IModelFactory;
use crate::postprocessor::IPostProcessor;
use crate::preprocessor::IPreProcessor;
use crate::processinginstruction::IProcessingInstructionProcessor;
use crate::templateboundaries::ITemplateBoundariesProcessor;
use crate::templateresolver::ITemplateResolver;
use crate::text::ITextProcessor;
use crate::util::Utf16String;
use crate::xmldeclaration::IXMLDeclarationProcessor;
use crate::{DialectConfiguration, TemplateMode};

/// 决定 TemplateEngine 全部解析、处理、表达式及缓存行为的配置合同。
///
/// 实现必须可在线程间安全共享。返回的集合是初始化完成后的稳定、只读执行快照；
/// 调用方不能借这些返回值修改引擎配置。
///
/// 对应 Java: `org.thymeleaf.IEngineConfiguration`。
///
/// # 另请参阅
///
/// [`crate::EngineConfiguration`]。
///
/// # 起始版本
///
/// 上游自 Thymeleaf 3.0.0 起提供该接口。
pub trait IEngineConfiguration: Send + Sync {
    /// 返回按执行顺序排列的模板 Resolver。
    ///
    /// 对应 Java: `IEngineConfiguration#getTemplateResolvers()`。
    ///
    /// # 返回值
    ///
    /// 返回稳定排序、不可变配置对应的借用快照；未设置 order 的 Resolver 排在最后。
    fn get_template_resolvers(&self) -> Vec<&dyn ITemplateResolver>;

    /// 返回按执行顺序排列的消息 Resolver。
    ///
    /// 对应 Java: `IEngineConfiguration#getMessageResolvers()`。
    ///
    /// # 返回值
    ///
    /// 返回稳定排序的消息 Resolver 借用快照。
    fn get_message_resolvers(&self) -> Vec<&dyn IMessageResolver>;

    /// 返回按执行顺序排列的 LinkBuilder。
    ///
    /// 对应 Java: `IEngineConfiguration#getLinkBuilders()`。
    ///
    /// # 返回值
    ///
    /// 返回稳定排序的链接构建器借用快照。
    fn get_link_builders(&self) -> Vec<&dyn ILinkBuilder>;

    /// 返回配置的缓存管理器。
    ///
    /// 对应 Java: `IEngineConfiguration#getCacheManager()`。
    ///
    /// # 返回值
    ///
    /// 缓存启用时返回共享管理器；`None` 对应 Java `null`，表示没有缓存。
    fn get_cache_manager(&self) -> Option<&dyn ICacheManager>;

    /// 返回 EngineContext 工厂。
    ///
    /// 对应 Java: `IEngineConfiguration#getEngineContextFactory()`。
    ///
    /// # 返回值
    ///
    /// 返回初始化时冻结的同一工厂实例。
    fn get_engine_context_factory(&self) -> &dyn IEngineContextFactory;

    /// 返回解耦模板逻辑 Resolver。
    ///
    /// 对应 Java: `IEngineConfiguration#getDecoupledTemplateLogicResolver()`。
    ///
    /// # 返回值
    ///
    /// 返回初始化时冻结的同一 Resolver 实例。
    fn get_decoupled_template_logic_resolver(&self) -> &dyn IDecoupledTemplateLogicResolver;

    /// 返回全部方言配置。
    ///
    /// 对应 Java: `IEngineConfiguration#getDialectConfigurations()`。
    ///
    /// # 返回值
    ///
    /// 返回保留配置顺序和显式前缀状态的借用快照。
    fn get_dialect_configurations(&self) -> Vec<&DialectConfiguration>;

    /// 返回去重后的全部方言实例。
    ///
    /// 对应 Java: `IEngineConfiguration#getDialects()`。
    ///
    /// # 返回值
    ///
    /// 返回按方言配置顺序排列的身份去重借用快照。
    fn get_dialects(&self) -> Vec<&dyn IDialect>;

    /// 按运行时具体类或方言能力接口筛选方言。
    ///
    /// 对应 Java: `IEngineConfiguration#getDialectsOfType(Class<T>)`。
    ///
    /// # 参数
    ///
    /// - `type_id`：具体方言类型，或 `dyn IProcessorDialect` 等已知能力接口的
    ///   [`TypeId`]。
    ///
    /// # 返回值
    ///
    /// 返回满足 Java `Class#isInstance` 等价判断的方言，保持原配置顺序。
    fn get_dialects_of_type(&self, type_id: TypeId) -> Vec<&dyn IDialect>;

    /// 判断 StandardDialect 是否存在。
    ///
    /// 对应 Java: `IEngineConfiguration#isStandardDialectPresent()`。
    ///
    /// # 返回值
    ///
    /// 配置包含 StandardDialect 时返回 `true`。
    fn is_standard_dialect_present(&self) -> bool;

    /// 返回 StandardDialect 的生效前缀。
    ///
    /// 对应 Java: `IEngineConfiguration#getStandardDialectPrefix()`。
    ///
    /// # 返回值
    ///
    /// 返回显式或默认前缀；无 StandardDialect 或无前缀时返回 `None`。
    fn get_standard_dialect_prefix(&self) -> Option<&Utf16String>;

    /// 返回全局元素定义仓库。
    ///
    /// 对应 Java: `IEngineConfiguration#getElementDefinitions()`。
    ///
    /// # 返回值
    ///
    /// 返回方言聚合阶段构建并冻结的元素定义。
    fn get_element_definitions(&self) -> &ElementDefinitions;

    /// 返回全局属性定义仓库。
    ///
    /// 对应 Java: `IEngineConfiguration#getAttributeDefinitions()`。
    ///
    /// # 返回值
    ///
    /// 返回方言聚合阶段构建并冻结的属性定义。
    fn get_attribute_definitions(&self) -> &AttributeDefinitions;

    /// 返回指定模式的模板边界 Processor。
    ///
    /// 对应 Java: `IEngineConfiguration#getTemplateBoundariesProcessors(TemplateMode)`。
    ///
    /// # 参数
    ///
    /// - `template_mode`：待查询的非空模板模式。
    ///
    /// # 返回值
    ///
    /// 返回按方言和 Processor precedence 排序的借用快照。
    fn get_template_boundaries_processors(
        &self,
        template_mode: TemplateMode,
    ) -> Vec<&dyn ITemplateBoundariesProcessor>;

    /// 返回指定模式的 CDATA Processor。
    ///
    /// 对应 Java: `IEngineConfiguration#getCDATASectionProcessors(TemplateMode)`。
    ///
    /// # 参数
    ///
    /// - `template_mode`：待查询的非空模板模式。
    ///
    /// # 返回值
    ///
    /// 返回稳定排序的 CDATA Processor。
    fn get_cdata_section_processors(
        &self,
        template_mode: TemplateMode,
    ) -> Vec<&dyn ICDATASectionProcessor>;

    /// 返回指定模式的 Comment Processor。
    ///
    /// 对应 Java: `IEngineConfiguration#getCommentProcessors(TemplateMode)`。
    ///
    /// # 参数
    ///
    /// - `template_mode`：待查询的非空模板模式。
    ///
    /// # 返回值
    ///
    /// 返回稳定排序的 Comment Processor。
    fn get_comment_processors(&self, template_mode: TemplateMode) -> Vec<&dyn ICommentProcessor>;

    /// 返回指定模式的 DOCTYPE Processor。
    ///
    /// 对应 Java: `IEngineConfiguration#getDocTypeProcessors(TemplateMode)`。
    ///
    /// # 参数
    ///
    /// - `template_mode`：待查询的非空模板模式。
    ///
    /// # 返回值
    ///
    /// 返回稳定排序的 DOCTYPE Processor。
    fn get_doc_type_processors(&self, template_mode: TemplateMode) -> Vec<&dyn IDocTypeProcessor>;

    /// 返回指定模式的 Element Processor。
    ///
    /// 对应 Java: `IEngineConfiguration#getElementProcessors(TemplateMode)`。
    ///
    /// # 参数
    ///
    /// - `template_mode`：待查询的非空模板模式。
    ///
    /// # 返回值
    ///
    /// 返回 Tag 与 Model Element Processor 的统一稳定序列。
    fn get_element_processors(&self, template_mode: TemplateMode) -> Vec<&dyn IElementProcessor>;

    /// 返回指定模式的 Text Processor。
    ///
    /// 对应 Java: `IEngineConfiguration#getTextProcessors(TemplateMode)`。
    ///
    /// # 参数
    ///
    /// - `template_mode`：待查询的非空模板模式。
    ///
    /// # 返回值
    ///
    /// 返回稳定排序的 Text Processor。
    fn get_text_processors(&self, template_mode: TemplateMode) -> Vec<&dyn ITextProcessor>;

    /// 返回指定模式的 ProcessingInstruction Processor。
    ///
    /// 对应 Java:
    /// `IEngineConfiguration#getProcessingInstructionProcessors(TemplateMode)`。
    ///
    /// # 参数
    ///
    /// - `template_mode`：待查询的非空模板模式。
    ///
    /// # 返回值
    ///
    /// 返回稳定排序的 ProcessingInstruction Processor。
    fn get_processing_instruction_processors(
        &self,
        template_mode: TemplateMode,
    ) -> Vec<&dyn IProcessingInstructionProcessor>;

    /// 返回指定模式的 XMLDeclaration Processor。
    ///
    /// 对应 Java: `IEngineConfiguration#getXMLDeclarationProcessors(TemplateMode)`。
    ///
    /// # 参数
    ///
    /// - `template_mode`：待查询的非空模板模式。
    ///
    /// # 返回值
    ///
    /// 返回稳定排序的 XMLDeclaration Processor。
    fn get_xml_declaration_processors(
        &self,
        template_mode: TemplateMode,
    ) -> Vec<&dyn IXMLDeclarationProcessor>;

    /// 返回指定模式的 PreProcessor。
    ///
    /// 对应 Java: `IEngineConfiguration#getPreProcessors(TemplateMode)`。
    ///
    /// # 参数
    ///
    /// - `template_mode`：待查询的非空模板模式。
    ///
    /// # 返回值
    ///
    /// 返回方言级与对象级 precedence 排序后的 PreProcessor。
    fn get_pre_processors(&self, template_mode: TemplateMode) -> Vec<&dyn IPreProcessor>;

    /// 返回指定模式的 PostProcessor。
    ///
    /// 对应 Java: `IEngineConfiguration#getPostProcessors(TemplateMode)`。
    ///
    /// # 参数
    ///
    /// - `template_mode`：待查询的非空模板模式。
    ///
    /// # 返回值
    ///
    /// 返回方言级与对象级 precedence 排序后的 PostProcessor。
    fn get_post_processors(&self, template_mode: TemplateMode) -> Vec<&dyn IPostProcessor>;

    /// 返回不可修改的执行属性。
    ///
    /// 对应 Java: `IEngineConfiguration#getExecutionAttributes()`。
    ///
    /// # 返回值
    ///
    /// 返回保留可空键、可空值和方言聚合顺序的同一只读映射。
    fn get_execution_attributes(
        &self,
    ) -> &IndexMap<Option<Utf16String>, Option<Arc<ExecutionAttributeValue>>>;

    /// 返回聚合表达式对象工厂。
    ///
    /// 对应 Java: `IEngineConfiguration#getExpressionObjectFactory()`。
    ///
    /// # 返回值
    ///
    /// 返回按方言顺序委派名称、构建和缓存判断的共享工厂。
    fn get_expression_object_factory(&self) -> Arc<dyn IExpressionObjectFactory>;

    /// 返回模板管理器。
    ///
    /// 对应 Java: `IEngineConfiguration#getTemplateManager()`。
    ///
    /// # 返回值
    ///
    /// 返回二阶段初始化时创建、持有当前配置身份的唯一模板管理器。
    fn get_template_manager(&self) -> &dyn ITemplateManager;

    /// 返回指定模式的唯一模型工厂。
    ///
    /// 对应 Java: `IEngineConfiguration#getModelFactory(TemplateMode)`。
    ///
    /// # 参数
    ///
    /// - `template_mode`：模型适用的非空模板模式。
    ///
    /// # 返回值
    ///
    /// 首次访问时并发安全地创建工厂；同一模式后续始终返回同一实例。
    fn get_model_factory(&self, template_mode: TemplateMode) -> &dyn IModelFactory;
}
