use std::io::Read;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::expression::TemplateValue;
use crate::util::JavaString;

/// Web 应用全局属性与资源访问合同。
///
/// 对应 Java: `org.thymeleaf.web.IWebApplication`。
///
/// 该 SPI 不绑定 Servlet 或任一 Rust Web 框架；具体集成负责提供属性作用域和资源
/// 读取实现。
pub trait IWebApplication {
    /// 判断应用作用域是否包含指定可空名称。
    fn contains_attribute(&self, name: Option<&JavaString>) -> bool;
    /// 返回应用属性数量。
    fn get_attribute_count(&self) -> i32;
    /// 返回应用属性名称的迭代快照。
    fn get_all_attribute_names(&self) -> Vec<Option<JavaString>>;
    /// 返回应用属性 Map 快照。
    fn get_attribute_map(&self) -> IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>;
    /// 返回应用属性值；不存在与 Java null 均按 Java API 折叠为 `None`。
    fn get_attribute_value(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>>;
    /// 新增或替换应用属性。
    fn set_attribute_value(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>);
    /// 删除应用属性。
    fn remove_attribute(&self, name: Option<&JavaString>);
    /// 判断给定路径的应用资源是否存在。
    fn resource_exists(&self, path: Option<&JavaString>) -> bool;
    /// 打开给定路径的应用资源；不存在时返回 `None`。
    fn get_resource_as_stream(&self, path: Option<&JavaString>) -> Option<Box<dyn Read + Send>>;
}
