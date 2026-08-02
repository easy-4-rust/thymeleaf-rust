use std::sync::Arc;

use crate::engine::TemplateData;
use crate::{IEngineConfiguration, TemplateResolutionAttributes};

use super::{IContext, IEngineContext};

/// 为模板执行创建引擎内部上下文的工厂合同。
///
/// 模板引擎实际向 Processor 暴露的是 [`IEngineContext`]，而调用方通常只提供更简单
/// 的 [`IContext`]。工厂负责从根模板数据、解析属性和调用方上下文创建前者。每次根
/// 模板处理只调用一次工厂；嵌套的 `th:insert`、`th:replace` 等操作复用已创建的
/// Engine Context，通过增加层级并切换该层的 TemplateData 表达嵌套处理。
///
/// 实现必须线程安全；同一个工厂实例可能由多个并发模板执行共享。具体工厂可通过
/// `TemplateEngine#getEngineContextFactory()` 和 `setEngineContextFactory(...)`
/// 获取或替换。
///
/// 对应 Java: `org.thymeleaf.context.IEngineContextFactory`。
pub trait IEngineContextFactory: Send + Sync {
    /// 为一个根模板处理创建全新的 Engine Context。
    ///
    /// 此方法只接收根层 TemplateData。嵌套模板不会再次调用工厂，而会由
    /// `EngineContextManager` 复用返回对象。Rust 的非空引用和拥有值在类型层面排除
    /// Java 调用可能传入的 null configuration、templateData 与 context。
    ///
    /// # 参数
    ///
    /// - `configuration`：当前模板引擎的冻结配置。
    /// - `template_data`：应用于层级 0 的根模板数据。
    /// - `template_resolution_attributes`：本次模板处理的可空解析属性快照。
    /// - `context`：调用模板引擎时提供的原始 Context。
    ///
    /// # 返回值
    ///
    /// 返回新创建、尚未由管理器增加执行层级的 Engine Context。
    ///
    /// 对应 Java: `IEngineContextFactory#createEngineContext`。
    fn create_engine_context(
        &self,
        configuration: Arc<dyn IEngineConfiguration>,
        template_data: TemplateData,
        template_resolution_attributes: Option<&TemplateResolutionAttributes>,
        context: &dyn IContext,
    ) -> Arc<dyn IEngineContext>;
}
