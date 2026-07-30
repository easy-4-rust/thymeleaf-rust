use std::collections::HashMap;
use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};

use crate::cdatasection::ICDATASectionProcessor;
use crate::comment::ICommentProcessor;
use crate::context::IExpressionContext;
use crate::doctype::IDocTypeProcessor;
use crate::element::IElementProcessor;
use crate::engine::{
    AttributeDefinitions, AttributeDefinitionsError, ElementDefinitions, ElementDefinitionsError,
    ElementProcessorsByTemplateMode,
};
use crate::exceptions::ConfigurationException;
use crate::expression::{IExpressionObjectFactory, StandardExpressionResult, TemplateValue};
use crate::postprocessor::IPostProcessor;
use crate::preprocessor::IPreProcessor;
use crate::processinginstruction::IProcessingInstructionProcessor;
use crate::processor::IProcessor;
use crate::templateboundaries::ITemplateBoundariesProcessor;
use crate::text::ITextProcessor;
use crate::util::{JavaString, ProcessorComparators, ProcessorConfigurationUtils};
use crate::xmldeclaration::IXMLDeclarationProcessor;
use crate::{DialectConfiguration, ExecutionAttributeValue, IDialect, TemplateMode};

/// 聚合全部 Dialect 的 Processor、执行属性和表达式对象工厂。
///
/// 对应 Java: `org.thymeleaf.DialectSetConfiguration`。
pub struct DialectSetConfiguration {
    dialect_configurations: Vec<DialectConfiguration>,
    dialects: Vec<Arc<dyn IDialect>>,
    standard_dialect_present: bool,
    standard_dialect_prefix: Option<JavaString>,
    execution_attributes: IndexMap<Option<JavaString>, Option<Arc<ExecutionAttributeValue>>>,
    expression_object_factory: Arc<AggregateExpressionObjectFactory>,
    element_definitions: Arc<ElementDefinitions>,
    attribute_definitions: Arc<AttributeDefinitions>,
    template_boundaries_processors: ProcessorMap<dyn ITemplateBoundariesProcessor>,
    cdata_section_processors: ProcessorMap<dyn ICDATASectionProcessor>,
    comment_processors: ProcessorMap<dyn ICommentProcessor>,
    doc_type_processors: ProcessorMap<dyn IDocTypeProcessor>,
    element_processors: ProcessorMap<dyn IElementProcessor>,
    processing_instruction_processors: ProcessorMap<dyn IProcessingInstructionProcessor>,
    text_processors: ProcessorMap<dyn ITextProcessor>,
    xml_declaration_processors: ProcessorMap<dyn IXMLDeclarationProcessor>,
    pre_processors: HashMap<TemplateMode, Vec<Arc<dyn IPreProcessor>>>,
    post_processors: HashMap<TemplateMode, Vec<Arc<dyn IPostProcessor>>>,
}

type ProcessorMap<T> = HashMap<TemplateMode, Vec<Arc<T>>>;

impl DialectSetConfiguration {
    /// 构建不可变方言聚合快照。
    ///
    /// 对应 Java: `DialectSetConfiguration#build(Set<DialectConfiguration>)`。
    pub fn build(
        dialect_configurations: Vec<DialectConfiguration>,
    ) -> Result<Self, ConfigurationException> {
        let mut dialects = Vec::<Arc<dyn IDialect>>::new();
        let mut standard_dialect_present = false;
        let mut standard_dialect_prefix = None;
        let mut execution_attributes = IndexMap::new();
        let mut expression_factories = Vec::new();
        let mut template_boundaries_processors = HashMap::new();
        let mut cdata_section_processors = HashMap::new();
        let mut comment_processors = HashMap::new();
        let mut doc_type_processors = HashMap::new();
        let mut element_processors = HashMap::new();
        let mut processing_instruction_processors = HashMap::new();
        let mut text_processors = HashMap::new();
        let mut xml_declaration_processors = HashMap::new();
        let mut pre_processors = HashMap::<TemplateMode, Vec<Arc<dyn IPreProcessor>>>::new();
        let mut post_processors = HashMap::<TemplateMode, Vec<Arc<dyn IPostProcessor>>>::new();

        for configuration in &dialect_configurations {
            let dialect = configuration.get_dialect_arc();
            if !dialects
                .iter()
                .any(|existing| Arc::ptr_eq(existing, &dialect))
            {
                dialects.push(Arc::clone(&dialect));
            }

            if let Some(processor_dialect) = dialect.as_processor_dialect() {
                let prefix = if configuration.is_prefix_specified() {
                    configuration.get_prefix()
                } else {
                    processor_dialect.get_prefix()
                };
                if dialect.is_standard_dialect() {
                    standard_dialect_present = true;
                    standard_dialect_prefix = prefix.map(JavaString::from_rust_str);
                }
                let processor_set = processor_dialect.get_processors(prefix).ok_or_else(|| {
                    configuration_error(format!(
                        "Dialect should not return null processor set: {}",
                        dialect.get_name().unwrap_or("")
                    ))
                })?;
                for processor in processor_set.iter() {
                    let processor = processor.cloned().ok_or_else(|| {
                        configuration_error(format!(
                            "Dialect should not return null processor in processor set: {}",
                            dialect.get_name().unwrap_or("")
                        ))
                    })?;
                    let mode = processor.get_template_mode().ok_or_else(|| {
                        configuration_error(format!(
                            "Template mode cannot be null (processor: {})",
                            processor.java_class_name()
                        ))
                    })?;
                    let dialect_precedence = processor_dialect.get_dialect_processor_precedence();
                    classify_processor(
                        processor,
                        mode,
                        dialect_precedence,
                        &mut template_boundaries_processors,
                        &mut cdata_section_processors,
                        &mut comment_processors,
                        &mut doc_type_processors,
                        &mut element_processors,
                        &mut processing_instruction_processors,
                        &mut text_processors,
                        &mut xml_declaration_processors,
                    )?;
                }
            }

            if let Some(attribute_dialect) = dialect.as_execution_attribute_dialect() {
                if let Some(attributes) = attribute_dialect.get_execution_attributes() {
                    for (name, value) in attributes {
                        let key = name.as_deref().map(JavaString::from_rust_str);
                        if execution_attributes.contains_key(&key) {
                            return Err(configuration_error(format!(
                                "Conflicting execution attribute. Two or more dialects specify an execution attribute with the same name \"{}\".",
                                key.as_ref()
                                    .map(JavaString::to_string_lossy)
                                    .unwrap_or_else(|| "null".to_owned())
                            )));
                        }
                        execution_attributes.insert(key, value);
                    }
                }
            }

            if let Some(expression_dialect) = dialect.as_expression_object_dialect() {
                expression_factories.push(expression_dialect.get_expression_object_factory());
            }

            if let Some(pre_processor_dialect) = dialect.as_pre_processor_dialect() {
                let dialect_precedence =
                    pre_processor_dialect.get_dialect_pre_processor_precedence();
                for processor in pre_processor_dialect.get_pre_processors() {
                    pre_processors
                        .entry(processor.get_template_mode())
                        .or_default()
                        .push(ProcessorConfigurationUtils::wrap_pre_processor(
                            processor,
                            dialect_precedence,
                        ));
                }
            }

            if let Some(post_processor_dialect) = dialect.as_post_processor_dialect() {
                let dialect_precedence =
                    post_processor_dialect.get_dialect_post_processor_precedence();
                for processor in post_processor_dialect.get_post_processors() {
                    post_processors
                        .entry(processor.get_template_mode())
                        .or_default()
                        .push(ProcessorConfigurationUtils::wrap_post_processor(
                            processor,
                            dialect_precedence,
                        ));
                }
            }
        }

        sort_processor_map(&mut template_boundaries_processors);
        sort_processor_map(&mut cdata_section_processors);
        sort_processor_map(&mut comment_processors);
        sort_processor_map(&mut doc_type_processors);
        sort_processor_map(&mut element_processors);
        sort_processor_map(&mut processing_instruction_processors);
        sort_processor_map(&mut text_processors);
        sort_processor_map(&mut xml_declaration_processors);
        for processors in pre_processors.values_mut() {
            processors.sort_by(|left, right| {
                ProcessorComparators::compare_pre_processors(left.as_ref(), right.as_ref())
            });
        }
        for processors in post_processors.values_mut() {
            processors.sort_by(|left, right| {
                ProcessorComparators::compare_post_processors(left.as_ref(), right.as_ref())
            });
        }

        let element_definition_processors = clone_element_processor_map(&element_processors);
        let element_definitions = Arc::new(
            ElementDefinitions::new(element_definition_processors.clone())
                .map_err(element_definitions_error)?,
        );
        let attribute_definitions = Arc::new(
            AttributeDefinitions::new(element_definition_processors)
                .map_err(attribute_definitions_error)?,
        );

        initialize_definitions(
            &template_boundaries_processors,
            &element_definitions,
            &attribute_definitions,
        );
        initialize_definitions(
            &cdata_section_processors,
            &element_definitions,
            &attribute_definitions,
        );
        initialize_definitions(
            &comment_processors,
            &element_definitions,
            &attribute_definitions,
        );
        initialize_definitions(
            &doc_type_processors,
            &element_definitions,
            &attribute_definitions,
        );
        initialize_definitions(
            &element_processors,
            &element_definitions,
            &attribute_definitions,
        );
        initialize_definitions(
            &processing_instruction_processors,
            &element_definitions,
            &attribute_definitions,
        );
        initialize_definitions(
            &text_processors,
            &element_definitions,
            &attribute_definitions,
        );
        initialize_definitions(
            &xml_declaration_processors,
            &element_definitions,
            &attribute_definitions,
        );
        initialize_pre_processor_definitions(
            &pre_processors,
            &element_definitions,
            &attribute_definitions,
        );
        initialize_post_processor_definitions(
            &post_processors,
            &element_definitions,
            &attribute_definitions,
        );

        Ok(Self {
            dialect_configurations,
            dialects,
            standard_dialect_present,
            standard_dialect_prefix,
            execution_attributes,
            expression_object_factory: Arc::new(AggregateExpressionObjectFactory::new(
                expression_factories,
            )),
            element_definitions,
            attribute_definitions,
            template_boundaries_processors,
            cdata_section_processors,
            comment_processors,
            doc_type_processors,
            element_processors,
            processing_instruction_processors,
            text_processors,
            xml_declaration_processors,
            pre_processors,
            post_processors,
        })
    }

    /// 返回方言配置快照。
    pub fn get_dialect_configurations(&self) -> &[DialectConfiguration] {
        &self.dialect_configurations
    }

    /// 返回按首次出现顺序去重的方言。
    pub fn get_dialects(&self) -> &[Arc<dyn IDialect>] {
        &self.dialects
    }

    /// 判断是否注册 StandardDialect。
    pub const fn is_standard_dialect_present(&self) -> bool {
        self.standard_dialect_present
    }

    /// 返回 StandardDialect 实际前缀。
    pub fn get_standard_dialect_prefix(&self) -> Option<&JavaString> {
        self.standard_dialect_prefix.as_ref()
    }

    /// 返回不可变执行属性。
    pub fn get_execution_attributes(
        &self,
    ) -> &IndexMap<Option<JavaString>, Option<Arc<ExecutionAttributeValue>>> {
        &self.execution_attributes
    }

    /// 返回指定名称的执行属性；名称缺失或值为 Java null 时返回 `None`。
    pub fn get_execution_attribute(
        &self,
        execution_attribute_name: Option<&JavaString>,
    ) -> Option<Arc<ExecutionAttributeValue>> {
        self.execution_attributes
            .get(&execution_attribute_name.cloned())
            .cloned()
            .flatten()
    }

    /// 判断执行属性 Map 是否包含指定键，显式 Java null 值仍计为存在。
    pub fn has_execution_attribute(&self, execution_attribute_name: Option<&JavaString>) -> bool {
        self.execution_attributes
            .contains_key(&execution_attribute_name.cloned())
    }

    /// 返回聚合表达式对象工厂。
    pub fn get_expression_object_factory(&self) -> Arc<dyn IExpressionObjectFactory> {
        self.expression_object_factory.clone()
    }

    /// 返回元素定义仓库。
    pub fn get_element_definitions(&self) -> &ElementDefinitions {
        self.element_definitions.as_ref()
    }

    /// 返回属性定义仓库。
    pub fn get_attribute_definitions(&self) -> &AttributeDefinitions {
        self.attribute_definitions.as_ref()
    }

    /// 返回共享元素定义仓库。
    pub fn get_element_definitions_arc(&self) -> Arc<ElementDefinitions> {
        Arc::clone(&self.element_definitions)
    }

    /// 返回共享属性定义仓库。
    pub fn get_attribute_definitions_arc(&self) -> Arc<AttributeDefinitions> {
        Arc::clone(&self.attribute_definitions)
    }

    /// 返回指定模式的 TemplateBoundaries Processor。
    pub fn get_template_boundaries_processors(
        &self,
        mode: TemplateMode,
    ) -> Vec<&dyn ITemplateBoundariesProcessor> {
        processor_refs(&self.template_boundaries_processors, mode)
    }

    /// 返回指定模式的 CDATA Processor。
    pub fn get_cdata_section_processors(
        &self,
        mode: TemplateMode,
    ) -> Vec<&dyn ICDATASectionProcessor> {
        processor_refs(&self.cdata_section_processors, mode)
    }

    /// 返回指定模式的 Comment Processor。
    pub fn get_comment_processors(&self, mode: TemplateMode) -> Vec<&dyn ICommentProcessor> {
        processor_refs(&self.comment_processors, mode)
    }

    /// 返回指定模式的 DOCTYPE Processor。
    pub fn get_doc_type_processors(&self, mode: TemplateMode) -> Vec<&dyn IDocTypeProcessor> {
        processor_refs(&self.doc_type_processors, mode)
    }

    /// 返回指定模式的 Element Processor。
    pub fn get_element_processors(&self, mode: TemplateMode) -> Vec<&dyn IElementProcessor> {
        processor_refs(&self.element_processors, mode)
    }

    /// 返回指定模式的 ProcessingInstruction Processor。
    pub fn get_processing_instruction_processors(
        &self,
        mode: TemplateMode,
    ) -> Vec<&dyn IProcessingInstructionProcessor> {
        processor_refs(&self.processing_instruction_processors, mode)
    }

    /// 返回指定模式的 Text Processor。
    pub fn get_text_processors(&self, mode: TemplateMode) -> Vec<&dyn ITextProcessor> {
        processor_refs(&self.text_processors, mode)
    }

    /// 返回指定模式的 XMLDeclaration Processor。
    pub fn get_xml_declaration_processors(
        &self,
        mode: TemplateMode,
    ) -> Vec<&dyn IXMLDeclarationProcessor> {
        processor_refs(&self.xml_declaration_processors, mode)
    }

    /// 返回指定模式的 PreProcessor，保持方言出现顺序并按 Processor precedence、
    /// handler 类名和身份排序。
    pub fn get_pre_processors(&self, mode: TemplateMode) -> Vec<&dyn IPreProcessor> {
        self.pre_processors
            .get(&mode)
            .map(|processors| processors.iter().map(AsRef::as_ref).collect())
            .unwrap_or_default()
    }

    /// 返回指定模式的 PostProcessor。
    pub fn get_post_processors(&self, mode: TemplateMode) -> Vec<&dyn IPostProcessor> {
        self.post_processors
            .get(&mode)
            .map(|processors| processors.iter().map(AsRef::as_ref).collect())
            .unwrap_or_default()
    }
}

/// 聚合多个方言表达式对象工厂，后注册的工厂覆盖同名对象。
///
/// 对应 Java: `DialectSetConfiguration.AggregateExpressionObjectFactory`。
struct AggregateExpressionObjectFactory {
    factories: Vec<Arc<dyn IExpressionObjectFactory>>,
}

impl AggregateExpressionObjectFactory {
    fn new(factories: Vec<Arc<dyn IExpressionObjectFactory>>) -> Self {
        Self { factories }
    }

    fn factory_for(&self, name: Option<&JavaString>) -> Option<&Arc<dyn IExpressionObjectFactory>> {
        self.factories.iter().rev().find(|factory| {
            factory
                .get_all_expression_object_names()
                .iter()
                .any(|candidate| candidate.as_ref() == name)
        })
    }
}

impl IExpressionObjectFactory for AggregateExpressionObjectFactory {
    fn get_all_expression_object_names(&self) -> Vec<Option<JavaString>> {
        if self.factories.len() == 1 {
            return self.factories[0].get_all_expression_object_names();
        }
        if self.factories.is_empty() {
            return Vec::new();
        }
        let mut names = IndexSet::new();
        for factory in self.factories.iter().rev() {
            names.extend(factory.get_all_expression_object_names());
        }
        names.into_iter().collect()
    }

    fn build_object(
        &self,
        context: Arc<dyn IExpressionContext>,
        expression_object_name: Option<&JavaString>,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let Some(factory) = self.factory_for(expression_object_name) else {
            return Ok(None);
        };
        factory.build_object(context, expression_object_name)
    }

    fn is_cacheable(&self, expression_object_name: Option<&JavaString>) -> bool {
        self.factory_for(expression_object_name)
            .is_some_and(|factory| factory.is_cacheable(expression_object_name))
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_processor(
    processor: Arc<dyn IProcessor>,
    mode: TemplateMode,
    dialect_precedence: i32,
    boundaries: &mut ProcessorMap<dyn ITemplateBoundariesProcessor>,
    cdata: &mut ProcessorMap<dyn ICDATASectionProcessor>,
    comments: &mut ProcessorMap<dyn ICommentProcessor>,
    doctypes: &mut ProcessorMap<dyn IDocTypeProcessor>,
    elements: &mut ProcessorMap<dyn IElementProcessor>,
    instructions: &mut ProcessorMap<dyn IProcessingInstructionProcessor>,
    texts: &mut ProcessorMap<dyn ITextProcessor>,
    declarations: &mut ProcessorMap<dyn IXMLDeclarationProcessor>,
) -> Result<(), ConfigurationException> {
    if processor.as_element_processor().is_some() {
        elements
            .entry(mode)
            .or_default()
            .push(ProcessorConfigurationUtils::wrap_element(
                processor,
                dialect_precedence,
            )?);
    } else if processor.as_template_boundaries_processor().is_some() {
        boundaries.entry(mode).or_default().push(
            ProcessorConfigurationUtils::wrap_template_boundaries(processor, dialect_precedence)?,
        );
    } else if processor.as_cdata_section_processor().is_some() {
        cdata
            .entry(mode)
            .or_default()
            .push(ProcessorConfigurationUtils::wrap_cdata_section(
                processor,
                dialect_precedence,
            )?);
    } else if processor.as_comment_processor().is_some() {
        comments
            .entry(mode)
            .or_default()
            .push(ProcessorConfigurationUtils::wrap_comment(
                processor,
                dialect_precedence,
            )?);
    } else if processor.as_doc_type_processor().is_some() {
        doctypes
            .entry(mode)
            .or_default()
            .push(ProcessorConfigurationUtils::wrap_doc_type(
                processor,
                dialect_precedence,
            )?);
    } else if processor.as_processing_instruction_processor().is_some() {
        instructions.entry(mode).or_default().push(
            ProcessorConfigurationUtils::wrap_processing_instruction(
                processor,
                dialect_precedence,
            )?,
        );
    } else if processor.as_text_processor().is_some() {
        texts
            .entry(mode)
            .or_default()
            .push(ProcessorConfigurationUtils::wrap_text(
                processor,
                dialect_precedence,
            )?);
    } else if processor.as_xml_declaration_processor().is_some() {
        declarations.entry(mode).or_default().push(
            ProcessorConfigurationUtils::wrap_xml_declaration(processor, dialect_precedence)?,
        );
    }
    Ok(())
}

fn sort_processor_map<T>(processors: &mut ProcessorMap<T>)
where
    T: IProcessor + ?Sized,
{
    for values in processors.values_mut() {
        values.sort_by(|left, right| {
            ProcessorComparators::compare_processors(left.as_ref(), right.as_ref())
        });
    }
}

fn processor_refs<T>(processors: &ProcessorMap<T>, mode: TemplateMode) -> Vec<&T>
where
    T: ?Sized,
{
    processors
        .get(&mode)
        .map(|values| values.iter().map(Arc::as_ref).collect())
        .unwrap_or_default()
}

fn clone_element_processor_map(
    processors: &ProcessorMap<dyn IElementProcessor>,
) -> ElementProcessorsByTemplateMode {
    processors
        .iter()
        .map(|(mode, values)| (*mode, values.clone()))
        .collect()
}

fn initialize_definitions<T>(
    processors: &ProcessorMap<T>,
    element_definitions: &Arc<ElementDefinitions>,
    attribute_definitions: &Arc<AttributeDefinitions>,
) where
    T: IProcessor + ?Sized,
{
    for processor in processors.values().flatten() {
        if processor.is_element_definitions_aware() {
            processor.set_element_definitions(Arc::clone(element_definitions));
        }
        if processor.is_attribute_definitions_aware() {
            processor.set_attribute_definitions(Arc::clone(attribute_definitions));
        }
    }
}

fn initialize_pre_processor_definitions(
    processors: &HashMap<TemplateMode, Vec<Arc<dyn IPreProcessor>>>,
    element_definitions: &Arc<ElementDefinitions>,
    attribute_definitions: &Arc<AttributeDefinitions>,
) {
    for processor in processors.values().flatten() {
        if processor.is_element_definitions_aware() {
            processor.set_element_definitions(Arc::clone(element_definitions));
        }
        if processor.is_attribute_definitions_aware() {
            processor.set_attribute_definitions(Arc::clone(attribute_definitions));
        }
    }
}

fn initialize_post_processor_definitions(
    processors: &HashMap<TemplateMode, Vec<Arc<dyn IPostProcessor>>>,
    element_definitions: &Arc<ElementDefinitions>,
    attribute_definitions: &Arc<AttributeDefinitions>,
) {
    for processor in processors.values().flatten() {
        if processor.is_element_definitions_aware() {
            processor.set_element_definitions(Arc::clone(element_definitions));
        }
        if processor.is_attribute_definitions_aware() {
            processor.set_attribute_definitions(Arc::clone(attribute_definitions));
        }
    }
}

fn configuration_error(message: String) -> ConfigurationException {
    ConfigurationException::new(Some(message))
}

fn element_definitions_error(error: ElementDefinitionsError) -> ConfigurationException {
    configuration_error(error.to_string())
}

fn attribute_definitions_error(error: AttributeDefinitionsError) -> ConfigurationException {
    configuration_error(error.to_string())
}
