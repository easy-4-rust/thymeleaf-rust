use std::any::TypeId;
use std::sync::{Arc, Weak};

use indexmap::IndexMap;

use crate::exceptions::TemplateProcessingException;
use crate::expression::{ExpressionObjects, IExpressionObjects, TemplateValue};
use crate::model::IModelFactory;
use crate::util::{JavaLocale, JavaString, ValidateError};
use crate::{IEngineConfiguration, TemplateResolutionAttributes};

use super::{IExpressionContext, ITemplateContext, IdentifierSequences};

/// 引擎上下文的配置、解析属性与公共服务实现。
///
/// 该对象刻意不管理变量；变量层级由 `EngineContext` 和后续
/// `WebEngineContext` 负责。表达式对象仍按名称惰性创建，标识序列按上下文独立。
///
/// 对应 Java: `org.thymeleaf.context.AbstractEngineContext`。
pub struct AbstractEngineContext {
    configuration: Arc<dyn IEngineConfiguration>,
    template_resolution_attributes: Option<TemplateResolutionAttributes>,
    locale: JavaLocale,
    expression_objects: ExpressionObjects,
    identifier_sequences: IdentifierSequences,
}

impl AbstractEngineContext {
    /// 创建公共引擎上下文状态。
    ///
    /// # 参数
    ///
    /// - `configuration`：当前不可空引擎配置。
    /// - `template_resolution_attributes`：可空模板解析属性快照。
    /// - `locale`：当前不可空 Locale。
    /// - `context`：最终表达式上下文的弱引用。
    ///
    /// 对应 Java: `AbstractEngineContext#AbstractEngineContext`。
    pub fn new(
        configuration: Arc<dyn IEngineConfiguration>,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
        locale: JavaLocale,
        context: Weak<dyn IExpressionContext>,
    ) -> Result<Self, ValidateError> {
        let expression_object_factory = configuration.get_expression_object_factory();
        let expression_objects =
            ExpressionObjects::new(Some(context), Some(expression_object_factory)).map_err(
                |error| ValidateError::IllegalArgument {
                    message: Some(error.to_string()),
                },
            )?;
        Ok(Self {
            configuration,
            template_resolution_attributes: template_resolution_attributes.cloned(),
            locale,
            expression_objects,
            identifier_sequences: IdentifierSequences::new(),
        })
    }

    /// 返回当前引擎配置。
    ///
    /// 对应 Java: `AbstractEngineContext#getConfiguration()`。
    #[must_use]
    pub fn get_configuration(&self) -> &dyn IEngineConfiguration {
        self.configuration.as_ref()
    }

    /// 返回当前引擎配置的共享身份。
    #[must_use]
    pub fn get_configuration_arc(&self) -> Arc<dyn IEngineConfiguration> {
        Arc::clone(&self.configuration)
    }

    /// 返回模板解析属性快照。
    ///
    /// 对应 Java: `AbstractEngineContext#getTemplateResolutionAttributes()`。
    #[must_use]
    pub fn get_template_resolution_attributes(&self) -> Option<&TemplateResolutionAttributes> {
        self.template_resolution_attributes.as_ref()
    }

    /// 返回模板处理 Locale。
    ///
    /// 对应 Java: `AbstractEngineContext#getLocale()`。
    #[must_use]
    pub fn get_locale(&self) -> JavaLocale {
        self.locale.clone()
    }

    /// 返回按需创建成员的表达式对象容器。
    ///
    /// 对应 Java: `AbstractEngineContext#getExpressionObjects()`。
    #[must_use]
    pub const fn get_expression_objects(&self) -> &dyn IExpressionObjects {
        &self.expression_objects
    }

    /// 返回指定模板模式的稳定模型工厂。
    ///
    /// 对应 Java: `AbstractEngineContext#getModelFactory()`。
    #[must_use]
    pub fn get_model_factory(&self, template_context: &dyn ITemplateContext) -> &dyn IModelFactory {
        self.configuration
            .get_model_factory(template_context.get_template_mode())
    }

    /// 依配置顺序解析消息，并按需生成缺失消息表示。
    ///
    /// 对应 Java: `AbstractEngineContext#getMessage`。
    pub fn get_message(
        &self,
        template_context: &dyn ITemplateContext,
        origin: Option<TypeId>,
        key: &JavaString,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
        use_absent_message_representation: bool,
    ) -> crate::messageresolver::MessageResolutionResult<Option<JavaString>> {
        let message_resolvers = self.configuration.get_message_resolvers();
        for message_resolver in &message_resolvers {
            if let Some(message) = message_resolver.resolve_message(
                template_context,
                origin,
                key,
                message_parameters,
            )? {
                return Ok(Some(message));
            }
        }
        if use_absent_message_representation {
            for message_resolver in &message_resolvers {
                if let Some(representation) = message_resolver
                    .create_absent_message_representation(
                        template_context,
                        origin,
                        key,
                        message_parameters,
                    )?
                {
                    return Ok(Some(representation));
                }
            }
        }
        Ok(None)
    }

    /// 依配置顺序构建链接；全部构建器拒绝时返回处理异常。
    ///
    /// 对应 Java: `AbstractEngineContext#buildLink`。
    pub fn build_link(
        &self,
        expression_context: &dyn IExpressionContext,
        base: Option<&JavaString>,
        parameters: Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
    ) -> Result<JavaString, TemplateProcessingException> {
        for link_builder in self.configuration.get_link_builders() {
            if let Some(link) = link_builder.build_link(expression_context, base, parameters)? {
                return Ok(link);
            }
        }
        let base = base.map_or_else(|| "null".to_owned(), JavaString::to_string_lossy);
        Err(TemplateProcessingException::new(Some(format!(
            "No configured link builder instance was able to build link with base \"{base}\" and \
             parameters {}",
            format_parameters(parameters)
        ))))
    }

    /// 返回本次模板执行独立的标识符序列。
    ///
    /// 对应 Java: `AbstractEngineContext#getIdentifierSequences()`。
    #[must_use]
    pub const fn get_identifier_sequences(&self) -> &IdentifierSequences {
        &self.identifier_sequences
    }
}

fn format_parameters(
    parameters: Option<&IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
) -> String {
    let Some(parameters) = parameters else {
        return "null".to_owned();
    };
    let mut result = String::from("{");
    for (index, (name, value)) in parameters.iter().enumerate() {
        if index != 0 {
            result.push_str(", ");
        }
        result.push_str(
            &name
                .as_ref()
                .map_or_else(|| "null".to_owned(), JavaString::to_string_lossy),
        );
        result.push('=');
        result.push_str(
            &value
                .as_deref()
                .and_then(TemplateValue::to_java_string)
                .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy()),
        );
    }
    result.push('}');
    result
}
