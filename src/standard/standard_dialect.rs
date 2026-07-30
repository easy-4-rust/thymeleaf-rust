use std::sync::{Arc, RwLock};

use crate::ExecutionAttributeValue;
use crate::TemplateMode;
use crate::exceptions::TemplateProcessingException;
use crate::expression::{
    IExpressionObjectFactory, IStandardConversionService, IStandardExpressionParser,
    IStandardVariableExpressionEvaluator, NativeVariableExpressionEvaluator,
    StandardConversionService, StandardExpressionObjectFactory, StandardExpressionParser,
    StandardExpressions,
};
use crate::processor::{
    IProcessor, ProcessorSet, StandardActionTagProcessor, StandardAltTitleTagProcessor,
    StandardAssertTagProcessor, StandardAttrTagProcessor, StandardAttrappendTagProcessor,
    StandardAttrprependTagProcessor, StandardBlockTagProcessor, StandardCaseTagProcessor,
    StandardClassappendTagProcessor, StandardConditionalCommentProcessor,
    StandardConditionalFixedValueTagProcessor, StandardDOMEventAttributeTagProcessor,
    StandardDefaultAttributesTagProcessor, StandardEachTagProcessor, StandardFragmentTagProcessor,
    StandardHrefTagProcessor, StandardIfTagProcessor, StandardIncludeTagProcessor,
    StandardInlineEnablementTemplateBoundariesProcessor, StandardInlineHTMLTagProcessor,
    StandardInlineTextualTagProcessor, StandardInlineXMLTagProcessor,
    StandardInliningCDATASectionProcessor, StandardInliningCommentProcessor,
    StandardInliningTextProcessor, StandardInsertTagProcessor, StandardLangXmlLangTagProcessor,
    StandardMethodTagProcessor, StandardNonRemovableAttributeTagProcessor,
    StandardObjectTagProcessor, StandardRefAttributeTagProcessor,
    StandardRemovableAttributeTagProcessor, StandardRemoveTagProcessor,
    StandardReplaceTagProcessor, StandardSrcTagProcessor, StandardStyleappendTagProcessor,
    StandardSwitchTagProcessor, StandardTextTagProcessor, StandardTranslationDocTypeProcessor,
    StandardUnlessTagProcessor, StandardUtextTagProcessor, StandardValueTagProcessor,
    StandardWithTagProcessor, StandardXmlBaseTagProcessor, StandardXmlLangTagProcessor,
    StandardXmlNsTagProcessor, StandardXmlSpaceTagProcessor,
};
use crate::serializer::{
    IStandardCSSSerializer, IStandardJavaScriptSerializer, StandardCSSSerializer,
    StandardJavaScriptSerializer, StandardSerializers,
};
use crate::util::JavaString;

use crate::dialect::{
    AbstractProcessorDialect, ExecutionAttributeMap, IDialect, IExecutionAttributeDialect,
    IExpressionObjectDialect, IProcessorDialect,
};

/// Thymeleaf 默认 Standard Dialect，集中注册标准表达式、序列化器和全部标准 Processor。
///
/// 对应 Java: `org.thymeleaf.standard.StandardDialect`。
pub struct StandardDialect {
    base: AbstractProcessorDialect,
    variable_expression_evaluator: RwLock<Option<Arc<dyn IStandardVariableExpressionEvaluator>>>,
    expression_parser: RwLock<Option<Arc<dyn IStandardExpressionParser>>>,
    conversion_service: RwLock<Option<Arc<dyn IStandardConversionService>>>,
    java_script_serializer: RwLock<Option<Arc<dyn IStandardJavaScriptSerializer>>>,
    css_serializer: RwLock<Option<Arc<dyn IStandardCSSSerializer>>>,
    expression_object_factory: RwLock<Option<Arc<dyn IExpressionObjectFactory>>>,
}

impl StandardDialect {
    /// Standard Dialect 名称。
    pub const NAME: &'static str = "Standard";
    /// Standard Dialect 默认前缀。
    pub const PREFIX: &'static str = "th";
    /// Standard Dialect 的跨方言 Processor precedence。
    pub const PROCESSOR_PRECEDENCE: i32 = 1000;

    /// 创建使用 Java 默认组件的 Standard Dialect。
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: AbstractProcessorDialect::new(
                Some(Self::NAME),
                Some(Self::PREFIX),
                Self::PROCESSOR_PRECEDENCE,
            )
            .expect("Standard dialect constants are non-null"),
            variable_expression_evaluator: RwLock::new(None),
            expression_parser: RwLock::new(None),
            conversion_service: RwLock::new(None),
            java_script_serializer: RwLock::new(None),
            css_serializer: RwLock::new(None),
            expression_object_factory: RwLock::new(None),
        }
    }

    /// 返回变量表达式求值器；首次访问时创建 Java 默认 OGNL 求值器。
    pub fn get_variable_expression_evaluator(
        &self,
    ) -> Arc<dyn IStandardVariableExpressionEvaluator> {
        get_or_initialize(&self.variable_expression_evaluator, || {
            Arc::new(NativeVariableExpressionEvaluator::new(true))
        })
    }

    /// 替换变量表达式求值器。Rust 类型系统在调用边界排除了 Java `null`。
    pub fn set_variable_expression_evaluator(
        &self,
        evaluator: Arc<dyn IStandardVariableExpressionEvaluator>,
    ) {
        set_component(&self.variable_expression_evaluator, evaluator);
    }

    /// 返回 Standard Expression Parser。
    pub fn get_expression_parser(&self) -> Arc<dyn IStandardExpressionParser> {
        get_or_initialize(&self.expression_parser, || {
            Arc::new(StandardExpressionParser::new())
        })
    }

    /// 替换 Standard Expression Parser。
    pub fn set_expression_parser(&self, parser: Arc<dyn IStandardExpressionParser>) {
        set_component(&self.expression_parser, parser);
    }

    /// 返回 Standard Conversion Service。
    pub fn get_conversion_service(&self) -> Arc<dyn IStandardConversionService> {
        get_or_initialize(&self.conversion_service, || {
            Arc::new(StandardConversionService::new())
        })
    }

    /// 替换 Standard Conversion Service。
    pub fn set_conversion_service(&self, conversion_service: Arc<dyn IStandardConversionService>) {
        set_component(&self.conversion_service, conversion_service);
    }

    /// 返回 JavaScript Serializer。
    pub fn get_java_script_serializer(&self) -> Arc<dyn IStandardJavaScriptSerializer> {
        get_or_initialize(&self.java_script_serializer, || {
            Arc::new(StandardJavaScriptSerializer::new(true))
        })
    }

    /// 替换 JavaScript Serializer。
    pub fn set_java_script_serializer(&self, serializer: Arc<dyn IStandardJavaScriptSerializer>) {
        set_component(&self.java_script_serializer, serializer);
    }

    /// 返回 CSS Serializer。
    pub fn get_css_serializer(&self) -> Arc<dyn IStandardCSSSerializer> {
        get_or_initialize(&self.css_serializer, || {
            Arc::new(StandardCSSSerializer::new())
        })
    }

    /// 替换 CSS Serializer。
    pub fn set_css_serializer(&self, serializer: Arc<dyn IStandardCSSSerializer>) {
        set_component(&self.css_serializer, serializer);
    }

    /// 创建指定实际方言前缀对应的完整标准 Processor 集合。
    pub fn create_standard_processors_set(
        dialect_prefix: Option<&str>,
    ) -> Result<ProcessorSet, TemplateProcessingException> {
        let prefix = dialect_prefix.map(JavaString::from_rust_str);
        let mut processors = ProcessorSet::new();

        register_markup_processors(&mut processors, TemplateMode::HTML, prefix.clone(), true)?;
        register_markup_processors(&mut processors, TemplateMode::XML, prefix.clone(), false)?;
        for mode in [
            TemplateMode::TEXT,
            TemplateMode::JAVASCRIPT,
            TemplateMode::CSS,
        ] {
            register_textual_processors(&mut processors, mode, prefix.clone())?;
        }
        Ok(processors)
    }
}

impl Default for StandardDialect {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for StandardDialect {
    fn is_standard_dialect(&self) -> bool {
        true
    }

    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }

    fn as_execution_attribute_dialect(&self) -> Option<&dyn IExecutionAttributeDialect> {
        Some(self)
    }

    fn as_expression_object_dialect(&self) -> Option<&dyn IExpressionObjectDialect> {
        Some(self)
    }

    fn get_name(&self) -> Option<&str> {
        Some(self.base.get_name())
    }
}

impl IProcessorDialect for StandardDialect {
    fn get_prefix(&self) -> Option<&str> {
        self.base.get_prefix()
    }

    fn get_dialect_processor_precedence(&self) -> i32 {
        self.base.get_dialect_processor_precedence()
    }

    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        Some(
            Self::create_standard_processors_set(dialect_prefix)
                .unwrap_or_else(|error| panic!("Could not create Standard processors: {error}")),
        )
    }
}

impl IExecutionAttributeDialect for StandardDialect {
    fn get_execution_attributes(&self) -> Option<ExecutionAttributeMap> {
        Some(vec![
            execution_attribute(
                StandardExpressions::STANDARD_VARIABLE_EXPRESSION_EVALUATOR_ATTRIBUTE_NAME,
                self.get_variable_expression_evaluator(),
            ),
            execution_attribute(
                StandardExpressions::STANDARD_EXPRESSION_PARSER_ATTRIBUTE_NAME,
                self.get_expression_parser(),
            ),
            execution_attribute(
                StandardExpressions::STANDARD_CONVERSION_SERVICE_ATTRIBUTE_NAME,
                self.get_conversion_service(),
            ),
            execution_attribute(
                StandardSerializers::STANDARD_JAVASCRIPT_SERIALIZER_ATTRIBUTE_NAME,
                self.get_java_script_serializer(),
            ),
            execution_attribute(
                StandardSerializers::STANDARD_CSS_SERIALIZER_ATTRIBUTE_NAME,
                self.get_css_serializer(),
            ),
        ])
    }
}

impl IExpressionObjectDialect for StandardDialect {
    fn get_expression_object_factory(&self) -> Arc<dyn IExpressionObjectFactory> {
        get_or_initialize(&self.expression_object_factory, || {
            Arc::new(StandardExpressionObjectFactory::new())
        })
    }
}

fn get_or_initialize<T, F>(slot: &RwLock<Option<Arc<T>>>, factory: F) -> Arc<T>
where
    T: ?Sized,
    F: FnOnce() -> Arc<T>,
{
    if let Some(component) = slot.read().expect("Standard dialect lock").as_ref() {
        return Arc::clone(component);
    }
    let mut guard = slot.write().expect("Standard dialect lock");
    Arc::clone(guard.get_or_insert_with(factory))
}

fn set_component<T>(slot: &RwLock<Option<Arc<T>>>, component: Arc<T>)
where
    T: ?Sized,
{
    *slot.write().expect("Standard dialect lock") = Some(component);
}

fn execution_attribute<T>(
    name: &'static str,
    component: Arc<T>,
) -> (Option<String>, Option<Arc<ExecutionAttributeValue>>)
where
    T: ?Sized + Send + Sync + 'static,
    Arc<T>: Send + Sync + 'static,
{
    (
        Some(name.to_owned()),
        Some(Arc::new(ExecutionAttributeValue::new(component))),
    )
}

fn insert<P>(processors: &mut ProcessorSet, processor: P)
where
    P: IProcessor + 'static,
{
    let processor: Arc<dyn IProcessor> = Arc::new(processor);
    processors.insert(Some(processor));
}

fn register_markup_processors(
    processors: &mut ProcessorSet,
    mode: TemplateMode,
    prefix: Option<JavaString>,
    html: bool,
) -> Result<(), TemplateProcessingException> {
    insert(
        processors,
        StandardAssertTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardAttrTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardAttrappendTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardAttrprependTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardCaseTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardEachTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardFragmentTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardIfTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardIncludeTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardInsertTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardObjectTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardRemoveTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardReplaceTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardSwitchTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardTextTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardUnlessTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardUtextTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardWithTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardXmlNsTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardRefAttributeTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardDefaultAttributesTagProcessor::new(mode, prefix.clone())?,
    );

    if html {
        register_html_only_processors(processors, prefix.clone())?;
        insert(
            processors,
            StandardInlineHTMLTagProcessor::new(prefix.clone())?,
        );
    } else {
        insert(
            processors,
            StandardInlineXMLTagProcessor::new(prefix.clone())?,
        );
    }
    insert(
        processors,
        StandardBlockTagProcessor::new(
            mode,
            prefix,
            JavaString::from_rust_str(StandardBlockTagProcessor::ELEMENT_NAME),
        )?,
    );
    insert(processors, StandardInliningTextProcessor::new(mode)?);
    insert(
        processors,
        StandardInliningCDATASectionProcessor::new(mode)?,
    );
    insert(processors, StandardInliningCommentProcessor::new(mode)?);
    insert(
        processors,
        StandardInlineEnablementTemplateBoundariesProcessor::new(mode)?,
    );
    Ok(())
}

fn register_html_only_processors(
    processors: &mut ProcessorSet,
    prefix: Option<JavaString>,
) -> Result<(), TemplateProcessingException> {
    insert(processors, StandardActionTagProcessor::new(prefix.clone())?);
    insert(
        processors,
        StandardAltTitleTagProcessor::new(prefix.clone())?,
    );
    insert(
        processors,
        StandardClassappendTagProcessor::new(prefix.clone())?,
    );
    for name in StandardConditionalFixedValueTagProcessor::ATTR_NAMES {
        insert(
            processors,
            StandardConditionalFixedValueTagProcessor::new(
                prefix.clone(),
                JavaString::from_rust_str(name),
            )?,
        );
    }
    for name in StandardDOMEventAttributeTagProcessor::ATTR_NAMES {
        insert(
            processors,
            StandardDOMEventAttributeTagProcessor::new(
                prefix.clone(),
                JavaString::from_rust_str(name),
            )?,
        );
    }
    insert(processors, StandardHrefTagProcessor::new(prefix.clone())?);
    insert(
        processors,
        StandardLangXmlLangTagProcessor::new(prefix.clone())?,
    );
    insert(processors, StandardMethodTagProcessor::new(prefix.clone())?);
    for name in StandardNonRemovableAttributeTagProcessor::ATTR_NAMES {
        insert(
            processors,
            StandardNonRemovableAttributeTagProcessor::new(
                prefix.clone(),
                JavaString::from_rust_str(name),
            )?,
        );
    }
    for name in StandardRemovableAttributeTagProcessor::ATTR_NAMES {
        insert(
            processors,
            StandardRemovableAttributeTagProcessor::new(
                prefix.clone(),
                JavaString::from_rust_str(name),
            )?,
        );
    }
    insert(processors, StandardSrcTagProcessor::new(prefix.clone())?);
    insert(
        processors,
        StandardStyleappendTagProcessor::new(prefix.clone())?,
    );
    insert(processors, StandardValueTagProcessor::new(prefix.clone())?);
    insert(
        processors,
        StandardXmlBaseTagProcessor::new(prefix.clone())?,
    );
    insert(
        processors,
        StandardXmlLangTagProcessor::new(prefix.clone())?,
    );
    insert(processors, StandardXmlSpaceTagProcessor::new(prefix)?);
    insert(processors, StandardTranslationDocTypeProcessor::new()?);
    insert(processors, StandardConditionalCommentProcessor::new()?);
    Ok(())
}

fn register_textual_processors(
    processors: &mut ProcessorSet,
    mode: TemplateMode,
    prefix: Option<JavaString>,
) -> Result<(), TemplateProcessingException> {
    insert(
        processors,
        StandardAssertTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardCaseTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardEachTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardIfTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardInlineTextualTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardInsertTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardObjectTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardRemoveTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardReplaceTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardSwitchTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardTextTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardUnlessTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardUtextTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardWithTagProcessor::new(mode, prefix.clone())?,
    );
    insert(
        processors,
        StandardBlockTagProcessor::new(
            mode,
            prefix.clone(),
            JavaString::from_rust_str(StandardBlockTagProcessor::ELEMENT_NAME),
        )?,
    );
    insert(
        processors,
        StandardBlockTagProcessor::new(mode, None, JavaString::from_rust_str(""))?,
    );
    insert(processors, StandardInliningTextProcessor::new(mode)?);
    insert(
        processors,
        StandardInlineEnablementTemplateBoundariesProcessor::new(mode)?,
    );
    Ok(())
}
