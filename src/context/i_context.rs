use std::any::Any;
use std::sync::Arc;

use crate::expression::TemplateValue;
use crate::util::{JavaLocale, JavaString};
use crate::web::IWebExchange;

/// `IContext#getVariableNames()` 返回的可变 Set 视图合同。
///
/// Java `Map#keySet()` 是由原 Map 支撑的实时视图，移除名称也会删除变量。该
/// Rust 合同保留实时查询与删除能力，同时用快照方法支持安全迭代。
pub trait IContextVariableNames {
    /// 返回当前变量名数量。
    fn len(&self) -> usize;

    /// 判断当前名称集合是否为空。
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 判断集合是否包含可空名称。
    fn contains(&self, name: Option<&JavaString>) -> bool;

    /// 返回当前迭代顺序的独立名称快照。
    fn snapshot(&self) -> Vec<Option<JavaString>>;

    /// 从支撑 Context 删除名称及其变量。
    ///
    /// # 返回
    ///
    /// 名称原先存在时返回 `true`。
    fn remove(&self, name: Option<&JavaString>) -> bool;
}

/// 模板执行所需 Locale 与变量的基础上下文合同。
///
/// 对应 Java: `org.thymeleaf.context.IContext`。
///
/// Context 刻意不继承 Map，表达式访问必须经过引擎控制层，从而避免自定义 Map
/// 实现绕过安全限制。
pub trait IContext: Any + Send + Sync {
    /// 返回 `Any` 视图，供 `Contexts` 复现 Java `instanceof`/强制转换。
    fn as_any(&self) -> &dyn Any;

    /// 返回模板处理使用的 Locale 快照。
    fn get_locale(&self) -> JavaLocale;

    /// 判断指定可空变量名是否已存在。
    fn contains_variable(&self, name: Option<&JavaString>) -> bool;

    /// 返回由 Context 变量 Map 支撑的实时名称视图。
    fn get_variable_names(&self) -> Box<dyn IContextVariableNames + '_>;

    /// 返回指定变量的可空值。
    ///
    /// `None` 表示变量不存在；显式 Java null 返回
    /// `Some(TemplateValue::Null)`，最终 Java API 边界可重新折叠两者。
    fn get_variable(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>>;

    /// 返回可选 Web exchange capability。
    ///
    /// Java 通过 `context instanceof IWebContext` 发现该能力；Rust 用显式 capability
    /// 避免丢失 trait object 的动态接口信息。普通上下文默认不具备 Web 能力。
    fn get_web_exchange(&self) -> Option<&dyn IWebExchange> {
        None
    }

    /// 返回可共享的 Web exchange 身份。
    ///
    /// EngineContext 工厂需要把 exchange 转移到整个渲染生命周期；普通上下文默认
    /// 不具备该能力。
    fn get_web_exchange_arc(&self) -> Option<Arc<dyn IWebExchange>> {
        None
    }

    /// 返回可选 Web Context capability。
    ///
    /// 对应 Java `context instanceof IWebContext` 后的安全强制转换。
    fn as_web_context(&self) -> Option<&dyn super::IWebContext> {
        None
    }

    /// 返回可选 Engine Context capability。
    ///
    /// 对应 Java `context instanceof IEngineContext` 后的安全强制转换。
    fn as_engine_context(&self) -> Option<&dyn super::IEngineContext> {
        None
    }

    /// 返回可共享的 Engine Context 身份。
    ///
    /// 嵌套模板处理用它复用现有上下文，而不是克隆变量。普通上下文默认不具备该
    /// 能力。
    fn get_engine_context_arc(&self) -> Option<Arc<dyn super::IEngineContext>> {
        None
    }

    /// 返回可选模板处理上下文 capability。
    ///
    /// Java 通过 `context instanceof ITemplateContext` 判定 Message、Link 和 Fragment
    /// 表达式是否位于模板处理链。Rust trait object 不能可靠执行横向接口强转，因此
    /// 由实现显式暴露同一能力。
    fn as_template_context(&self) -> Option<&dyn super::ITemplateContext> {
        None
    }
}
