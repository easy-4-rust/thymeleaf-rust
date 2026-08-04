use std::any::Any;
use std::fmt::{Display, Formatter};
use std::io;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::model::{AttributeValueQuotes, IAttribute};
use crate::util::{FastStringWriter, JavaWriter, Utf16String};

use super::{AttributeDefinition, AttributeDefinitionValue};

const DEFAULT_OPERATOR: &str = "=";

/// 引擎内部使用的不可变模板属性。
///
/// 对应 Java: `org.thymeleaf.engine.Attribute`。
///
/// 本对象只保存模板中已经出现或将要输出的原始属性形态，不负责把任意对象转换为
/// 字符串，也不负责计算 HTML 布尔属性；这些职责与 Java 版一致，属于 Processor。
pub struct Attribute {
    definition: AttributeDefinitionValue,
    complete_name: Utf16String,
    operator: Option<Utf16String>,
    value: Option<Utf16String>,
    value_quotes: Option<AttributeValueQuotes>,
    template_name: Option<Utf16String>,
    line: i32,
    col: i32,
    // IStandardExpression 的完整执行合同将在标准表达式闭包中接入。这里保存同一
    // 对象身份，等价于 Java volatile 引用的缓存/替换语义。
    standard_expression: RwLock<Option<Arc<dyn Any + Send + Sync>>>,
}

impl Attribute {
    /// 创建原始属性并规范化操作符与引号。
    ///
    /// 对应 Java: `Attribute#Attribute(...)`。
    ///
    /// 值为 null 时操作符和引号必为 null；非 null 值缺省使用 `=` 与双引号；
    /// 空字符串不能使用无引号形态。
    #[expect(
        clippy::too_many_arguments,
        reason = "参数逐项对齐 Java Attribute 构造器，不引入失真的参数对象"
    )]
    /// 对应 Java 语义：`Attribute` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub(crate) fn new(
        definition: AttributeDefinitionValue,
        complete_name: Utf16String,
        operator: Option<Utf16String>,
        value: Option<Utf16String>,
        value_quotes: Option<AttributeValueQuotes>,
        template_name: Option<Utf16String>,
        line: i32,
        col: i32,
    ) -> Self {
        let operator = value
            .as_ref()
            .map(|_| operator.unwrap_or_else(|| Utf16String::from_rust_str(DEFAULT_OPERATOR)));
        let value_quotes = value.as_ref().map(|value| match value_quotes {
            None => AttributeValueQuotes::DOUBLE,
            Some(AttributeValueQuotes::NONE) if value.is_empty() => AttributeValueQuotes::DOUBLE,
            Some(value_quotes) => value_quotes,
        });
        Self {
            definition,
            complete_name,
            operator,
            value,
            value_quotes,
            template_name,
            line,
            col,
            standard_expression: RwLock::new(None),
        }
    }

    /// 返回缓存的标准表达式对象。
    ///
    /// 对应 Java: `Attribute#getCachedStandardExpression()`。
    pub fn get_cached_standard_expression(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        read_lock(&self.standard_expression).clone()
    }

    /// 替换缓存的标准表达式对象；`None` 对应 Java null。
    ///
    /// 对应 Java: `Attribute#setCachedStandardExpression(IStandardExpression)`。
    pub fn set_cached_standard_expression(
        &self,
        standard_expression: Option<Arc<dyn Any + Send + Sync>>,
    ) {
        *write_lock(&self.standard_expression) = standard_expression;
    }

    /// 派生新属性，并保留未显式替换的定义、名称、操作符、位置和引号。
    ///
    /// 对应 Java: `Attribute#modify(...)`。属性值不可保留，调用者传入的
    /// `None` 明确表示 Java null；新对象不会继承标准表达式缓存。
    pub(crate) fn modify(
        &self,
        definition: Option<AttributeDefinitionValue>,
        complete_name: Option<Utf16String>,
        value: Option<Utf16String>,
        value_quotes: Option<AttributeValueQuotes>,
    ) -> Self {
        Self::new(
            definition.unwrap_or_else(|| self.definition.clone()),
            complete_name.unwrap_or_else(|| self.complete_name.clone()),
            self.operator.clone(),
            value,
            value_quotes.or(self.value_quotes),
            self.template_name.clone(),
            self.line,
            self.col,
        )
    }

    /// 返回 Java `toString()` 对应的 UTF-16 属性表示。
    #[must_use]
    /// 对应 Java 语义：`Attribute` 的 `to_utf16_string` 行为（Rust 侧辅助/私有路径）。
    pub fn to_utf16_string(&self) -> Utf16String {
        let mut writer = FastStringWriter::new();
        // 内存 Writer 的完整切片写入不可能失败；Java 版同样把此分支视为不可达。
        self.write(&mut writer)
            .expect("FastStringWriter must accept complete UTF-16 slices");
        writer.to_string()
    }
}

impl IAttribute for Attribute {
    fn get_attribute_complete_name(&self) -> &Utf16String {
        &self.complete_name
    }

    fn get_attribute_definition(&self) -> &AttributeDefinition {
        self.definition.as_attribute_definition()
    }

    fn get_operator(&self) -> Option<&Utf16String> {
        self.operator.as_ref()
    }

    fn get_value(&self) -> Option<&Utf16String> {
        self.value.as_ref()
    }

    fn get_value_quotes(&self) -> Option<AttributeValueQuotes> {
        self.value_quotes
    }

    fn has_location(&self) -> bool {
        self.template_name.is_some() && self.line != -1 && self.col != -1
    }

    fn get_template_name(&self) -> Option<&Utf16String> {
        self.template_name.as_ref()
    }

    fn get_line(&self) -> i32 {
        self.line
    }

    fn get_col(&self) -> i32 {
        self.col
    }

    fn write(&self, writer: &mut dyn JavaWriter) -> io::Result<()> {
        writer.write_utf16(self.complete_name.as_utf16())?;
        let Some(value) = self.value.as_ref() else {
            return Ok(());
        };

        if let Some(operator) = self.operator.as_ref() {
            writer.write_utf16(operator.as_utf16())?;
        }
        match self.value_quotes {
            Some(AttributeValueQuotes::DOUBLE) => {
                writer.write_utf16(&[u16::from(b'"')])?;
                writer.write_utf16(value.as_utf16())?;
                writer.write_utf16(&[u16::from(b'"')])
            }
            Some(AttributeValueQuotes::SINGLE) => {
                writer.write_utf16(&[u16::from(b'\'')])?;
                writer.write_utf16(value.as_utf16())?;
                writer.write_utf16(&[u16::from(b'\'')])
            }
            Some(AttributeValueQuotes::NONE) | None => writer.write_utf16(value.as_utf16()),
        }
    }
}

impl Display for Attribute {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.to_utf16_string().to_string_lossy())
    }
}

fn read_lock<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_lock<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
