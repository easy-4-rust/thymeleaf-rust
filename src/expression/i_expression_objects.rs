use std::sync::Arc;

use crate::util::JavaString;

use super::TemplateValue;

/// 模板执行期间表达式工具对象的容器合同。
///
/// 对应 Java: `org.thymeleaf.expression.IExpressionObjects`。
pub trait IExpressionObjects {
    /// 返回工厂声明的表达式对象名称数量。
    fn size(&self) -> i32;

    /// 判断指定可空名称是否由容器声明。
    fn contains_object(&self, name: Option<&JavaString>) -> bool;

    /// 返回对象名称集合的只读快照。
    fn get_object_names(&self) -> Vec<Option<JavaString>>;

    /// 返回指定名称对应的对象；未声明或工厂返回 null 时为 `None`。
    fn get_object(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>>;
}
