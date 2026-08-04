use std::io;

use crate::engine::AttributeDefinition;
use crate::util::{TemplateWriter, Utf16String};

use super::AttributeValueQuotes;

/// 标签中不可变属性的公共合同。
///
/// 对应 Java: `org.thymeleaf.model.IAttribute`。
pub trait IAttribute {
    /// 返回模板中原样书写的完整属性名。
    fn get_attribute_complete_name(&self) -> &Utf16String;

    /// 返回属性元数据定义。
    fn get_attribute_definition(&self) -> &AttributeDefinition;

    /// 返回包含原始空白的可空等号操作符。
    fn get_operator(&self) -> Option<&Utf16String>;

    /// 返回可空属性值；无值属性仅在 HTML 模式合法。
    fn get_value(&self) -> Option<&Utf16String>;

    /// 返回可空属性值引号类型。
    fn get_value_quotes(&self) -> Option<AttributeValueQuotes>;

    /// 判断属性是否携带原模板位置。
    fn has_location(&self) -> bool;

    /// 返回可空原模板名称。
    fn get_template_name(&self) -> Option<&Utf16String>;

    /// 返回一基行号。
    fn get_line(&self) -> i32;

    /// 返回一基列号。
    fn get_col(&self) -> i32;

    /// 将属性完整写入输出。
    fn write(&self, writer: &mut dyn TemplateWriter) -> io::Result<()>;
}
