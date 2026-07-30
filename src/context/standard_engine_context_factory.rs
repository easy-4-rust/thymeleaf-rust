use std::sync::Arc;

use indexmap::IndexMap;

use crate::engine::TemplateData;
use crate::{IEngineConfiguration, TemplateResolutionAttributes};

use super::{EngineContext, IContext, IEngineContext, IEngineContextFactory, WebEngineContext};

/// 根据用户上下文 capability 创建标准 EngineContext。
///
/// Web 上下文创建 `WebEngineContext` 并保持 exchange 身份，普通上下文创建
/// `EngineContext`。变量在创建前只读取一次名称快照。
///
/// 对应 Java: `org.thymeleaf.context.StandardEngineContextFactory`。
#[derive(Clone, Copy, Debug, Default)]
pub struct StandardEngineContextFactory;

impl StandardEngineContextFactory {
    /// 创建无状态标准工厂。
    ///
    /// 对应 Java: `StandardEngineContextFactory#StandardEngineContextFactory()`。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl IEngineContextFactory for StandardEngineContextFactory {
    fn create_engine_context(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        template_data: TemplateData,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
        context: &dyn IContext,
    ) -> Arc<dyn IEngineContext> {
        let variable_names = context.get_variable_names().snapshot();
        let mut variables = IndexMap::with_capacity(variable_names.len());
        for variable_name in variable_names {
            variables.insert(
                variable_name.clone(),
                context.get_variable(variable_name.as_ref()),
            );
        }
        if let Some(web_exchange) = context.get_web_exchange_arc() {
            return WebEngineContext::new(
                configuration,
                template_data,
                template_resolution_attributes,
                web_exchange,
                context.get_locale(),
                Some(&variables),
            );
        }
        EngineContext::new(
            configuration,
            template_data,
            template_resolution_attributes,
            context.get_locale(),
            Some(&variables),
        )
    }
}
