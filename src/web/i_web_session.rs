use std::sync::Arc;

use indexmap::IndexMap;

use crate::expression::TemplateValue;
use crate::util::JavaString;

/// Web 会话作用域合同。
///
/// 对应 Java: `org.thymeleaf.web.IWebSession`。
pub trait IWebSession {
    /// 判断底层会话是否已经存在。
    fn exists(&self) -> bool;
    /// 判断会话是否包含指定属性。
    fn contains_attribute(&self, name: Option<&JavaString>) -> bool;
    /// 返回会话属性数量。
    fn get_attribute_count(&self) -> i32;
    /// 返回会话属性名称快照。
    fn get_all_attribute_names(&self) -> Vec<Option<JavaString>>;
    /// 返回会话属性 Map 快照。
    fn get_attribute_map(&self) -> IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>;
    /// 返回会话属性值。
    fn get_attribute_value(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>>;
    /// 新增或替换会话属性。
    fn set_attribute_value(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>);
    /// 删除会话属性。
    fn remove_attribute(&self, name: Option<&JavaString>);
}
