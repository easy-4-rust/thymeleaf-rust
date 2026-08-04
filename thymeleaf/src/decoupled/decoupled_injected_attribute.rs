use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::util::Utf16String;

/// 创建解耦逻辑注入属性时的 Java 数组错误。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.markup.decoupled.DecoupledInjectedAttribute#createAttribute`
/// 的数组分配和 `System.arraycopy` 失败。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecoupledInjectedAttributeError {
    /// 输入 `char[]` 为 null。
    NullBuffer,
    /// 新数组长度按 Java `int` 回绕后为负。
    NegativeArraySize(i32),
    /// 任一源范围或目标范围越界。
    ArrayIndexOutOfBounds,
    /// getter 使用的内部 `String(char[], offset, count)` 范围非法。
    StringIndexOutOfBounds {
        /// UTF-16 起始位置。
        offset: i32,
        /// UTF-16 长度。
        length: i32,
        /// 内部数组长度。
        buffer_length: usize,
    },
}

impl DecoupledInjectedAttributeError {
    /// 返回对应 Java 异常类名。
    #[must_use]
    pub const fn class_name(&self) -> &'static str {
        match self {
            Self::NullBuffer => "java.lang.NullPointerException",
            Self::NegativeArraySize(_) => "java.lang.NegativeArraySizeException",
            Self::ArrayIndexOutOfBounds => "java.lang.ArrayIndexOutOfBoundsException",
            Self::StringIndexOutOfBounds { .. } => "java.lang.StringIndexOutOfBoundsException",
        }
    }
}

impl Display for DecoupledInjectedAttributeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullBuffer => formatter.write_str("null"),
            Self::NegativeArraySize(length) => write!(formatter, "{length}"),
            Self::ArrayIndexOutOfBounds => {
                formatter.write_str("arraycopy: source index out of bounds")
            }
            Self::StringIndexOutOfBounds {
                offset,
                length,
                buffer_length,
            } => write!(
                formatter,
                "offset {offset}, count {length}, length {buffer_length}"
            ),
        }
    }
}

impl Error for DecoupledInjectedAttributeError {}

/// 解耦模板逻辑在解析阶段注入的独立属性值。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.markup.decoupled.DecoupledInjectedAttribute`。
///
/// 工厂只复制属性名称、运算符和 outer value 三段到私有 UTF-16 buffer，之后所有
/// offset 都相对该私有副本。原始 parser buffer 的后续修改不会影响本对象。
pub struct DecoupledInjectedAttribute {
    buffer: Vec<u16>,
    name_offset: i32,
    name_len: i32,
    operator_offset: i32,
    operator_len: i32,
    value_content_offset: i32,
    value_content_len: i32,
    value_outer_offset: i32,
    value_outer_len: i32,
}

impl DecoupledInjectedAttribute {
    /// 从 parser UTF-16 buffer 的三个范围创建独立属性。
    ///
    /// 对应 Java: `DecoupledInjectedAttribute#createAttribute(char[],...)`。
    ///
    /// # 参数
    ///
    /// 所有 offset/len 都是 Java `int` UTF-16 code unit 范围；`buffer=None`
    /// 对应 Java null。
    ///
    /// # 返回
    ///
    /// 保存最小私有 buffer 及重新基准化 offset 的属性。
    ///
    /// # 错误
    ///
    /// 保留 Java 数组长度回绕、null buffer 及 `System.arraycopy` 越界类别。
    #[allow(clippy::too_many_arguments)]
    pub fn create_attribute(
        buffer: Option<&[u16]>,
        name_offset: i32,
        name_len: i32,
        operator_offset: i32,
        operator_len: i32,
        value_content_offset: i32,
        value_content_len: i32,
        value_outer_offset: i32,
        value_outer_len: i32,
    ) -> Result<Self, DecoupledInjectedAttributeError> {
        let new_length = name_len
            .wrapping_add(operator_len)
            .wrapping_add(value_outer_len);
        if new_length < 0 {
            return Err(DecoupledInjectedAttributeError::NegativeArraySize(
                new_length,
            ));
        }
        let buffer = buffer.ok_or(DecoupledInjectedAttributeError::NullBuffer)?;
        let mut new_buffer = vec![0; new_length as usize];

        copy_java_range(buffer, name_offset, name_len, &mut new_buffer, 0)?;
        copy_java_range(
            buffer,
            operator_offset,
            operator_len,
            &mut new_buffer,
            name_len,
        )?;
        copy_java_range(
            buffer,
            value_outer_offset,
            value_outer_len,
            &mut new_buffer,
            name_len.wrapping_add(operator_len),
        )?;

        Ok(Self {
            buffer: new_buffer,
            name_offset: 0,
            name_len,
            operator_offset: operator_offset.wrapping_sub(name_offset),
            operator_len,
            value_content_offset: value_content_offset.wrapping_sub(name_offset),
            value_content_len,
            value_outer_offset: value_outer_offset.wrapping_sub(name_offset),
            value_outer_len,
        })
    }

    /// 返回属性完整名称。对应 Java `getName()`。
    ///
    /// # 错误
    ///
    /// 工厂收到彼此不一致的合法复制范围时，保留 Java `String` 构造器的范围异常。
    pub fn get_name(&self) -> Result<Utf16String, DecoupledInjectedAttributeError> {
        self.slice(self.name_offset, self.name_len)
    }

    /// 返回属性运算符。对应 Java `getOperator()`。
    ///
    /// # 错误
    ///
    /// 内部重基准化范围越界时返回 `StringIndexOutOfBoundsException` 对应错误。
    pub fn get_operator(&self) -> Result<Utf16String, DecoupledInjectedAttributeError> {
        self.slice(self.operator_offset, self.operator_len)
    }

    /// 返回不含引号的属性值内容。对应 Java `getValueContent()`。
    ///
    /// # 错误
    ///
    /// 内部重基准化范围越界时返回 `StringIndexOutOfBoundsException` 对应错误。
    pub fn get_value_content(&self) -> Result<Utf16String, DecoupledInjectedAttributeError> {
        self.slice(self.value_content_offset, self.value_content_len)
    }

    /// 返回包含引号的 outer 属性值。对应 Java `getValueOuter()`。
    ///
    /// # 错误
    ///
    /// 内部重基准化范围越界时返回 `StringIndexOutOfBoundsException` 对应错误。
    pub fn get_value_outer(&self) -> Result<Utf16String, DecoupledInjectedAttributeError> {
        self.slice(self.value_outer_offset, self.value_outer_len)
    }

    /// 返回名称、运算符与 outer value 拼接后的完整属性。
    ///
    /// 对应 Java: `DecoupledInjectedAttribute#toString()`。
    #[must_use]
    pub fn to_utf16_string(&self) -> Utf16String {
        Utf16String::from_utf16(self.buffer.clone())
    }

    /// 返回向标记 parser 注入属性所需的私有 UTF-16 buffer 与全部范围。
    ///
    /// 对应 Java 同包
    /// `DecoupledTemplateLogicMarkupHandler#processInjectedAttributes` 对 package-private
    /// 字段的直接读取；Rust 将该可见性收窄为 crate 内部。
    #[allow(clippy::type_complexity)]
    pub(crate) fn parser_parts(&self) -> (&[u16], i32, i32, i32, i32, i32, i32, i32, i32) {
        (
            &self.buffer,
            self.name_offset,
            self.name_len,
            self.operator_offset,
            self.operator_len,
            self.value_content_offset,
            self.value_content_len,
            self.value_outer_offset,
            self.value_outer_len,
        )
    }

    fn slice(&self, offset: i32, len: i32) -> Result<Utf16String, DecoupledInjectedAttributeError> {
        let range_is_invalid = offset < 0
            || len < 0
            || usize::try_from(offset)
                .ok()
                .zip(usize::try_from(len).ok())
                .is_none_or(|(start, length)| {
                    start > self.buffer.len().saturating_sub(length) || length > self.buffer.len()
                });
        if range_is_invalid {
            return Err(DecoupledInjectedAttributeError::StringIndexOutOfBounds {
                offset,
                length: len,
                buffer_length: self.buffer.len(),
            });
        }
        let start = offset as usize;
        let end = start + len as usize;
        Ok(Utf16String::from_utf16(self.buffer[start..end].to_vec()))
    }
}

fn copy_java_range(
    source: &[u16],
    source_offset: i32,
    length: i32,
    target: &mut [u16],
    target_offset: i32,
) -> Result<(), DecoupledInjectedAttributeError> {
    let source_end = source_offset.wrapping_add(length);
    let target_end = target_offset.wrapping_add(length);
    if source_offset < 0
        || target_offset < 0
        || length < 0
        || source_end < 0
        || target_end < 0
        || source_end as usize > source.len()
        || target_end as usize > target.len()
    {
        return Err(DecoupledInjectedAttributeError::ArrayIndexOutOfBounds);
    }
    target[target_offset as usize..target_end as usize]
        .copy_from_slice(&source[source_offset as usize..source_end as usize]);
    Ok(())
}
