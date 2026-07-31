use std::sync::Arc;

use indexmap::IndexMap;

use crate::engine::TemplateData;
use crate::{IEngineConfiguration, TemplateResolutionAttributes};

use super::{EngineContext, IContext, IEngineContext, IEngineContextFactory, WebEngineContext};

/// 标准 Engine Context 工厂。
///
/// 工厂检查原始 Context 是否提供 [`super::IWebContext`] capability：Web Context
/// 创建 [`WebEngineContext`] 并保持 exchange 共享身份，普通 Context 创建
/// [`EngineContext`]。这是 `TemplateEngine` 使用的默认实现。
///
/// 对应 Java: `org.thymeleaf.context.StandardEngineContextFactory`。
#[derive(Clone, Copy, Debug, Default)]
pub struct StandardEngineContextFactory;

impl StandardEngineContextFactory {
    /// 创建无状态、可在线程间共享的标准工厂。
    ///
    /// 对应 Java: `StandardEngineContextFactory#StandardEngineContextFactory()`。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl IEngineContextFactory for StandardEngineContextFactory {
    /// 复制调用方变量并根据 Web capability 创建对应 Engine Context。
    ///
    /// 名称集合先形成一次稳定快照，再严格按该顺序调用 `get_variable`。空集合不会
    /// 调用任何变量 getter。Java 为避免昂贵的 Servlet attribute-name 枚举，会在
    /// 到达本工厂前优先复用已有 Engine Context；本方法仍只读取一次名称快照。
    ///
    /// # 参数
    ///
    /// - `configuration`：当前冻结引擎配置。
    /// - `template_data`：根层模板数据。
    /// - `template_resolution_attributes`：可空解析属性。
    /// - `context`：不可空的调用方 Context。
    ///
    /// # 返回值
    ///
    /// Web Context 返回 `WebEngineContext`，否则返回 `EngineContext`；两者均为新
    /// 共享对象且层级仍为 0。
    ///
    /// 对应 Java: `StandardEngineContextFactory#createEngineContext`。
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
        if let Some(web_context) = context.as_web_context() {
            return WebEngineContext::new(
                configuration,
                template_data,
                template_resolution_attributes,
                web_context.get_exchange_arc(),
                web_context.get_locale(),
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
