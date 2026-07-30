use std::any::TypeId;
use std::sync::{Arc, OnceLock, Weak};

use indexmap::IndexMap;

use crate::cache::ICacheManager;
use crate::cdatasection::ICDATASectionProcessor;
use crate::comment::ICommentProcessor;
use crate::context::IEngineContextFactory;
use crate::decoupled::IDecoupledTemplateLogicResolver;
use crate::doctype::IDocTypeProcessor;
use crate::element::IElementProcessor;
use crate::engine::{
    AttributeDefinitions, ElementDefinitions, ITemplateManager, StandardModelFactory,
    TemplateManager,
};
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
use crate::util::JavaString;
use crate::xmldeclaration::IXMLDeclarationProcessor;
use crate::{
    DialectConfiguration, DialectSetConfiguration, ExecutionAttributeValue, IEngineConfiguration,
    TemplateMode,
};

/// 默认不可变引擎配置，聚合 Resolver、Dialect、Cache 和运行时工厂。
///
/// 对应 Java: `org.thymeleaf.EngineConfiguration`。
pub struct EngineConfiguration {
    template_resolvers: Vec<Arc<dyn ITemplateResolver>>,
    message_resolvers: Vec<Arc<dyn IMessageResolver>>,
    link_builders: Vec<Arc<dyn ILinkBuilder>>,
    cache_manager: Option<Arc<dyn ICacheManager>>,
    engine_context_factory: Arc<dyn IEngineContextFactory>,
    decoupled_template_logic_resolver: Arc<dyn IDecoupledTemplateLogicResolver>,
    dialect_set_configuration: DialectSetConfiguration,
    self_weak: OnceLock<Weak<EngineConfiguration>>,
    template_manager: OnceLock<TemplateManager>,
    html_model_factory: OnceLock<StandardModelFactory>,
    xml_model_factory: OnceLock<StandardModelFactory>,
    text_model_factory: OnceLock<StandardModelFactory>,
    javascript_model_factory: OnceLock<StandardModelFactory>,
    css_model_factory: OnceLock<StandardModelFactory>,
    raw_model_factory: OnceLock<StandardModelFactory>,
}

impl EngineConfiguration {
    /// 构建配置、稳定排序 Resolver，并在配置对象建立后初始化 TemplateManager。
    ///
    /// 对应 Java: package 构造器及 `EngineConfiguration#initialize()`。
    pub fn new(
        mut template_resolvers: Vec<Arc<dyn ITemplateResolver>>,
        mut message_resolvers: Vec<Arc<dyn IMessageResolver>>,
        mut link_builders: Vec<Arc<dyn ILinkBuilder>>,
        dialect_configurations: Vec<DialectConfiguration>,
        cache_manager: Option<Arc<dyn ICacheManager>>,
        engine_context_factory: Arc<dyn IEngineContextFactory>,
        decoupled_template_logic_resolver: Arc<dyn IDecoupledTemplateLogicResolver>,
    ) -> Result<Arc<Self>, crate::exceptions::ConfigurationException> {
        if template_resolvers.is_empty() {
            return Err(crate::exceptions::ConfigurationException::new(Some(
                "Template Resolver set cannot be empty".to_owned(),
            )));
        }
        template_resolvers.sort_by(TemplateResolverComparator::compare);
        message_resolvers.sort_by(MessageResolverComparator::compare);
        link_builders.sort_by(LinkBuilderComparator::compare);
        let dialect_set_configuration = DialectSetConfiguration::build(dialect_configurations)?;

        let configuration = Arc::new(Self {
            template_resolvers,
            message_resolvers,
            link_builders,
            cache_manager,
            engine_context_factory,
            decoupled_template_logic_resolver,
            dialect_set_configuration,
            self_weak: OnceLock::new(),
            template_manager: OnceLock::new(),
            html_model_factory: OnceLock::new(),
            xml_model_factory: OnceLock::new(),
            text_model_factory: OnceLock::new(),
            javascript_model_factory: OnceLock::new(),
            css_model_factory: OnceLock::new(),
            raw_model_factory: OnceLock::new(),
        });
        configuration
            .self_weak
            .set(Arc::downgrade(&configuration))
            .expect("self weak reference is initialized exactly once");
        let shared: Arc<dyn IEngineConfiguration> = configuration.clone();
        configuration
            .template_manager
            .set(TemplateManager::new(shared))
            .map_err(|_| {
                crate::exceptions::ConfigurationException::new(Some(
                    "Template Manager has already been initialized".to_owned(),
                ))
            })?;
        Ok(configuration)
    }

    fn shared_configuration(&self) -> Arc<dyn IEngineConfiguration> {
        self.self_weak
            .get()
            .and_then(Weak::upgrade)
            .expect("published EngineConfiguration retains a strong owner")
    }

    fn model_factory_slot(&self, mode: TemplateMode) -> &OnceLock<StandardModelFactory> {
        match mode {
            TemplateMode::HTML => &self.html_model_factory,
            TemplateMode::XML => &self.xml_model_factory,
            TemplateMode::TEXT => &self.text_model_factory,
            TemplateMode::JAVASCRIPT => &self.javascript_model_factory,
            TemplateMode::CSS => &self.css_model_factory,
            TemplateMode::RAW => &self.raw_model_factory,
        }
    }

    /// 判断指定模式的已解析模型能否安全执行结构重塑优化。
    ///
    /// 对应 Java: `EngineConfiguration#isModelReshapeable(TemplateMode)`。
    #[must_use]
    pub fn is_model_reshapeable(&self, template_mode: TemplateMode) -> bool {
        if !self.dialect_set_configuration.is_standard_dialect_present()
            || self
                .dialect_set_configuration
                .get_text_processors(template_mode)
                .len()
                > 1
        {
            return false;
        }
        if template_mode.is_markup() {
            let allowed_comment_processors = if template_mode == TemplateMode::HTML {
                2
            } else {
                1
            };
            if self
                .dialect_set_configuration
                .get_comment_processors(template_mode)
                .len()
                > allowed_comment_processors
                || self
                    .dialect_set_configuration
                    .get_cdata_section_processors(template_mode)
                    .len()
                    > 1
            {
                return false;
            }
        }
        self.dialect_set_configuration
            .get_pre_processors(template_mode)
            .is_empty()
            && self
                .dialect_set_configuration
                .get_post_processors(template_mode)
                .is_empty()
    }
}

impl IEngineConfiguration for EngineConfiguration {
    fn get_template_resolvers(&self) -> Vec<&dyn ITemplateResolver> {
        self.template_resolvers.iter().map(Arc::as_ref).collect()
    }
    fn get_message_resolvers(&self) -> Vec<&dyn IMessageResolver> {
        self.message_resolvers.iter().map(Arc::as_ref).collect()
    }
    fn get_link_builders(&self) -> Vec<&dyn ILinkBuilder> {
        self.link_builders.iter().map(Arc::as_ref).collect()
    }
    fn get_cache_manager(&self) -> Option<&dyn ICacheManager> {
        self.cache_manager.as_deref()
    }
    fn get_engine_context_factory(&self) -> &dyn IEngineContextFactory {
        self.engine_context_factory.as_ref()
    }
    fn get_decoupled_template_logic_resolver(&self) -> &dyn IDecoupledTemplateLogicResolver {
        self.decoupled_template_logic_resolver.as_ref()
    }
    fn get_dialect_configurations(&self) -> Vec<&DialectConfiguration> {
        self.dialect_set_configuration
            .get_dialect_configurations()
            .iter()
            .collect()
    }
    fn get_dialects(&self) -> Vec<&dyn crate::IDialect> {
        self.dialect_set_configuration
            .get_dialects()
            .iter()
            .map(Arc::as_ref)
            .collect()
    }
    fn get_dialects_of_type(&self, type_id: TypeId) -> Vec<&dyn crate::IDialect> {
        self.get_dialects()
            .into_iter()
            .filter(|dialect| dialect.dialect_type_id() == type_id)
            .collect()
    }
    fn is_standard_dialect_present(&self) -> bool {
        self.dialect_set_configuration.is_standard_dialect_present()
    }
    fn get_standard_dialect_prefix(&self) -> Option<&JavaString> {
        self.dialect_set_configuration.get_standard_dialect_prefix()
    }
    fn get_element_definitions(&self) -> &ElementDefinitions {
        self.dialect_set_configuration.get_element_definitions()
    }
    fn get_attribute_definitions(&self) -> &AttributeDefinitions {
        self.dialect_set_configuration.get_attribute_definitions()
    }
    fn get_template_boundaries_processors(
        &self,
        mode: TemplateMode,
    ) -> Vec<&dyn ITemplateBoundariesProcessor> {
        self.dialect_set_configuration
            .get_template_boundaries_processors(mode)
    }
    fn get_cdata_section_processors(&self, mode: TemplateMode) -> Vec<&dyn ICDATASectionProcessor> {
        self.dialect_set_configuration
            .get_cdata_section_processors(mode)
    }
    fn get_comment_processors(&self, mode: TemplateMode) -> Vec<&dyn ICommentProcessor> {
        self.dialect_set_configuration.get_comment_processors(mode)
    }
    fn get_doc_type_processors(&self, mode: TemplateMode) -> Vec<&dyn IDocTypeProcessor> {
        self.dialect_set_configuration.get_doc_type_processors(mode)
    }
    fn get_element_processors(&self, mode: TemplateMode) -> Vec<&dyn IElementProcessor> {
        self.dialect_set_configuration.get_element_processors(mode)
    }
    fn get_text_processors(&self, mode: TemplateMode) -> Vec<&dyn ITextProcessor> {
        self.dialect_set_configuration.get_text_processors(mode)
    }
    fn get_processing_instruction_processors(
        &self,
        mode: TemplateMode,
    ) -> Vec<&dyn IProcessingInstructionProcessor> {
        self.dialect_set_configuration
            .get_processing_instruction_processors(mode)
    }
    fn get_xml_declaration_processors(
        &self,
        mode: TemplateMode,
    ) -> Vec<&dyn IXMLDeclarationProcessor> {
        self.dialect_set_configuration
            .get_xml_declaration_processors(mode)
    }
    fn get_pre_processors(&self, mode: TemplateMode) -> Vec<&dyn IPreProcessor> {
        self.dialect_set_configuration.get_pre_processors(mode)
    }
    fn get_post_processors(&self, mode: TemplateMode) -> Vec<&dyn IPostProcessor> {
        self.dialect_set_configuration.get_post_processors(mode)
    }
    fn get_execution_attributes(
        &self,
    ) -> &IndexMap<Option<JavaString>, Option<Arc<ExecutionAttributeValue>>> {
        self.dialect_set_configuration.get_execution_attributes()
    }
    fn get_expression_object_factory(&self) -> Arc<dyn IExpressionObjectFactory> {
        self.dialect_set_configuration
            .get_expression_object_factory()
    }
    fn get_template_manager(&self) -> &dyn ITemplateManager {
        self.template_manager
            .get()
            .expect("EngineConfiguration is initialized before publication")
    }
    fn get_model_factory(&self, mode: TemplateMode) -> &dyn IModelFactory {
        self.model_factory_slot(mode).get_or_init(|| {
            StandardModelFactory::new(
                self.shared_configuration(),
                mode,
                self.dialect_set_configuration.get_element_definitions_arc(),
                self.dialect_set_configuration
                    .get_attribute_definitions_arc(),
            )
        })
    }
}

fn compare_optional_order(left: Option<i32>, right: Option<i32>) -> std::cmp::Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

/// 模板解析器的 Java 空值安全顺序比较器。
///
/// 对应 Java: `EngineConfiguration.TemplateResolverComparator`。
struct TemplateResolverComparator;

impl TemplateResolverComparator {
    fn compare(
        left: &Arc<dyn ITemplateResolver>,
        right: &Arc<dyn ITemplateResolver>,
    ) -> std::cmp::Ordering {
        compare_optional_order(left.get_order(), right.get_order())
    }
}

/// 消息解析器的 Java 空值安全顺序比较器。
///
/// 对应 Java: `EngineConfiguration.MessageResolverComparator`。
struct MessageResolverComparator;

impl MessageResolverComparator {
    fn compare(
        left: &Arc<dyn IMessageResolver>,
        right: &Arc<dyn IMessageResolver>,
    ) -> std::cmp::Ordering {
        compare_optional_order(left.get_order(), right.get_order())
    }
}

/// 链接构建器的 Java 空值安全顺序比较器。
///
/// 对应 Java: `EngineConfiguration.LinkBuilderComparator`。
struct LinkBuilderComparator;

impl LinkBuilderComparator {
    fn compare(left: &Arc<dyn ILinkBuilder>, right: &Arc<dyn ILinkBuilder>) -> std::cmp::Ordering {
        compare_optional_order(left.get_order(), right.get_order())
    }
}
