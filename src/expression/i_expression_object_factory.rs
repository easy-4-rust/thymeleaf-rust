use std::sync::Arc;

use crate::context::IExpressionContext;
use crate::util::JavaString;

use super::TemplateValue;

/// 表达式工具对象的名称、构建及缓存策略合同。
///
/// 对应 Java: `org.thymeleaf.expression.IExpressionObjectFactory`。
pub trait IExpressionObjectFactory {
    /// 返回工厂支持的全部对象名称。
    ///
    /// Java `Set<String>` 理论上允许 `null`，因此名称元素使用 `Option`，不在 SPI
    /// 边界擅自过滤第三方工厂返回值。
    fn get_all_expression_object_names(&self) -> Vec<Option<JavaString>>;
    /// 按名称构建对象；Java null 映射为 `None`。
    fn build_object(
        &self,
        context: &dyn IExpressionContext,
        expression_object_name: Option<&JavaString>,
    ) -> Option<Arc<TemplateValue>>;
    /// 判断指定名称的构建结果是否应由容器缓存。
    fn is_cacheable(&self, expression_object_name: Option<&JavaString>) -> bool;
}
