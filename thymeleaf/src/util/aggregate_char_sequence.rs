use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::sync::{Arc, Mutex};

use super::{
    CharSequenceValue, IWritableCharSequence, TemplateWriter, TextUtilsError, Utf16String,
};

/// 可被线程安全聚合字符序列持有的组件。
pub type AggregateComponent = Arc<dyn CharSequenceValue + Send + Sync>;

/// 聚合字符序列构造错误。
/// 对应 Java 语义：`AggregateCharSequence` 的 Rust 侧类型 `AggregateCharSequenceError`。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AggregateCharSequenceError {
    /// 单组件重载收到 null。
    NullComponent,
    /// 多个独立参数中至少一个为 null。
    NullComponentArgument,
    /// 数组或 List 参数为 null。
    NullComponents,
    /// 数组或 List 中包含 null。
    NullContainedComponent,
    /// 组件访问失败。
    Sequence(TextUtilsError),
}

impl AggregateCharSequenceError {
    /// 返回对应 Java 异常全限定名。
    #[must_use]
    pub fn class_name(&self) -> &str {
        match self {
            Self::NullComponent
            | Self::NullComponentArgument
            | Self::NullComponents
            | Self::NullContainedComponent => "java.lang.IllegalArgumentException",
            Self::Sequence(error) => error.class_name(),
        }
    }
}

impl Display for AggregateCharSequenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullComponent => {
                formatter.write_str("Component argument is null, which is forbidden")
            }
            Self::NullComponentArgument => {
                formatter.write_str("At least one component argument is null, which is forbidden")
            }
            Self::NullComponents => formatter.write_str("Components argument array cannot be null"),
            Self::NullContainedComponent => formatter
                .write_str("Components argument contains at least a null, which is forbidden"),
            Self::Sequence(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for AggregateCharSequenceError {}

impl From<TextUtilsError> for AggregateCharSequenceError {
    fn from(value: TextUtilsError) -> Self {
        Self::Sequence(value)
    }
}

/// 不复制组件即可提供统一 UTF-16 视图的聚合字符序列。
///
/// 对应 Java: `org.thymeleaf.util.AggregateCharSequence`。
///
/// 组件在构造后不得修改。偏移和总长度按构造时的 Java `length()` 调用结果缓存。
pub struct AggregateCharSequence {
    values: Vec<AggregateComponent>,
    offsets: Vec<i32>,
    length: i32,
    hash: Mutex<i32>,
}

impl AggregateCharSequence {
    /// 创建单组件聚合。
    /// 对应 Java 语义：`AggregateCharSequence` 的 `from_one` 行为（Rust 侧辅助/私有路径）。
    pub fn from_one(
        component: Option<AggregateComponent>,
    ) -> Result<Self, AggregateCharSequenceError> {
        let component = component.ok_or(AggregateCharSequenceError::NullComponent)?;
        let length = component.java_length()?;
        Ok(Self::new_parts(vec![component], vec![0], length))
    }

    /// 创建双组件聚合。
    /// 对应 Java 语义：`AggregateCharSequence` 的 `from_two` 行为（Rust 侧辅助/私有路径）。
    pub fn from_two(
        component0: Option<AggregateComponent>,
        component1: Option<AggregateComponent>,
    ) -> Result<Self, AggregateCharSequenceError> {
        let values = require_arguments([component0, component1])?;
        let length0 = values[0].java_length()?;
        let length = length0.wrapping_add(values[1].java_length()?);
        Ok(Self::new_parts(values, vec![0, length0], length))
    }

    /// 创建三组件聚合。
    /// 对应 Java 语义：`AggregateCharSequence` 的 `from_three` 行为（Rust 侧辅助/私有路径）。
    pub fn from_three(
        component0: Option<AggregateComponent>,
        component1: Option<AggregateComponent>,
        component2: Option<AggregateComponent>,
    ) -> Result<Self, AggregateCharSequenceError> {
        let values = require_arguments([component0, component1, component2])?;
        let offset1 = values[0].java_length()?;
        let offset2 = values[0]
            .java_length()?
            .wrapping_add(values[1].java_length()?);
        let length = offset2.wrapping_add(values[2].java_length()?);
        Ok(Self::new_parts(values, vec![0, offset1, offset2], length))
    }

    /// 创建四组件聚合。
    /// 对应 Java 语义：`AggregateCharSequence` 的 `from_four` 行为（Rust 侧辅助/私有路径）。
    pub fn from_four(
        component0: Option<AggregateComponent>,
        component1: Option<AggregateComponent>,
        component2: Option<AggregateComponent>,
        component3: Option<AggregateComponent>,
    ) -> Result<Self, AggregateCharSequenceError> {
        let values = require_arguments([component0, component1, component2, component3])?;
        let offset1 = values[0].java_length()?;
        let offset2 = values[0]
            .java_length()?
            .wrapping_add(values[1].java_length()?);
        let offset3 = values[0]
            .java_length()?
            .wrapping_add(values[1].java_length()?)
            .wrapping_add(values[2].java_length()?);
        let length = offset3.wrapping_add(values[3].java_length()?);
        Ok(Self::new_parts(
            values,
            vec![0, offset1, offset2, offset3],
            length,
        ))
    }

    /// 创建五组件聚合。
    ///
    /// 保留上游 3.1.5 的可观察缺陷：总长度使用第四组件长度而非第五组件。
    /// 对应 Java 语义：`AggregateCharSequence` 的 `from_five` 行为（Rust 侧辅助/私有路径）。
    pub fn from_five(
        component0: Option<AggregateComponent>,
        component1: Option<AggregateComponent>,
        component2: Option<AggregateComponent>,
        component3: Option<AggregateComponent>,
        component4: Option<AggregateComponent>,
    ) -> Result<Self, AggregateCharSequenceError> {
        let values =
            require_arguments([component0, component1, component2, component3, component4])?;
        let offset1 = values[0].java_length()?;
        let offset2 = values[0]
            .java_length()?
            .wrapping_add(values[1].java_length()?);
        let offset3 = values[0]
            .java_length()?
            .wrapping_add(values[1].java_length()?)
            .wrapping_add(values[2].java_length()?);
        let offset4 = values[0]
            .java_length()?
            .wrapping_add(values[1].java_length()?)
            .wrapping_add(values[2].java_length()?)
            .wrapping_add(values[3].java_length()?);
        let length = offset4.wrapping_add(values[3].java_length()?);
        Ok(Self::new_parts(
            values,
            vec![0, offset1, offset2, offset3, offset4],
            length,
        ))
    }

    /// 从 Java 数组或 List 语义的组件集合创建聚合。
    /// 对应 Java 语义：`AggregateCharSequence` 的 `from_components` 行为（Rust 侧辅助/私有路径）。
    pub fn from_components(
        components: Option<Vec<Option<AggregateComponent>>>,
    ) -> Result<Self, AggregateCharSequenceError> {
        let components = components.ok_or(AggregateCharSequenceError::NullComponents)?;
        if components.is_empty() {
            let empty: AggregateComponent = Arc::new(Utf16String::from_utf16(Vec::new()));
            return Ok(Self::new_parts(vec![empty], vec![0], 0));
        }
        let mut values: Vec<AggregateComponent> = Vec::with_capacity(components.len());
        let mut offsets: Vec<i32> = Vec::with_capacity(components.len());
        let mut total_length = 0_i32;
        for (index, component) in components.into_iter().enumerate() {
            let component = component.ok_or(AggregateCharSequenceError::NullContainedComponent)?;
            let component_length = component.java_length()?;
            let offset = if index == 0 {
                0
            } else {
                offsets[index - 1].wrapping_add(values[index - 1].java_length()?)
            };
            values.push(component);
            offsets.push(offset);
            total_length = total_length.wrapping_add(component_length);
        }
        Ok(Self::new_parts(values, offsets, total_length))
    }

    fn new_parts(values: Vec<AggregateComponent>, offsets: Vec<i32>, length: i32) -> Self {
        Self {
            values,
            offsets,
            length,
            hash: Mutex::new(0),
        }
    }

    /// 返回构造时缓存的 Java `int` 总长度。
    #[must_use]
    pub const fn length(&self) -> i32 {
        self.length
    }

    /// 返回指定聚合 UTF-16 位置的代码单元。
    /// 对应 Java: `AggregateCharSequence#charAt()`。
    pub fn char_at(&self, index: i32) -> Result<u16, TextUtilsError> {
        if index < 0 || index >= self.length {
            return Err(range_error(index, self.length));
        }
        for component_index in (0..self.values.len()).rev() {
            if self.offsets[component_index] <= index {
                return self.values[component_index]
                    .java_char_at(index - self.offsets[component_index]);
            }
        }
        Err(TextUtilsError::SequenceAccess {
            class_name: "java.lang.IllegalStateException".into(),
            message: Some(Utf16String::from_rust_str(
                "Bad computing of charAt at AggregatedString",
            )),
        })
    }

    /// 返回指定范围的 Java String 子序列。
    /// 对应 Java: `AggregateCharSequence#subSequence()`。
    pub fn sub_sequence(
        &self,
        begin_index: i32,
        end_index: i32,
    ) -> Result<Utf16String, TextUtilsError> {
        if begin_index < 0 {
            return Err(range_error(begin_index, self.length));
        }
        if end_index > self.length {
            return Err(range_error(end_index, self.length));
        }
        let sub_length = end_index.wrapping_sub(begin_index);
        if sub_length < 0 {
            return Err(range_error(sub_length, self.length));
        }
        let mut result = Vec::with_capacity(sub_length as usize);
        for index in begin_index..end_index {
            result.push(self.char_at(index)?);
        }
        Ok(Utf16String::from_utf16(result))
    }

    /// 按聚合内容比较另一个同类对象。
    /// 对应 Java 语义：`AggregateCharSequence` 的 `equals_java` 行为（Rust 侧辅助/私有路径）。
    pub fn equals_java(&self, other: &Self) -> Result<bool, TextUtilsError> {
        if std::ptr::eq(self, other) {
            return Ok(true);
        }
        if self.length != other.length {
            return Ok(false);
        }
        if self.length == 0 {
            return Ok(true);
        }
        let own_hash = *lock(&self.hash);
        let other_hash = *lock(&other.hash);
        if own_hash != 0 && other_hash != 0 && own_hash != other_hash {
            return Ok(false);
        }
        for index in 0..self.length {
            if self.char_at(index)? != other.char_at(index)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// 返回并缓存 Java hashCode。
    /// 对应 Java: `AggregateCharSequence#hashCode()`。
    pub fn hash_code(&self) -> Result<i32, TextUtilsError> {
        let cached = *lock(&self.hash);
        if cached != 0 || self.length <= 0 {
            return Ok(cached);
        }
        let mut hash = if self.values.len() == 1 {
            self.values[0].java_sequence_hash_code()?
        } else {
            0
        };
        if self.values.len() != 1 {
            for component in &self.values {
                for index in 0..component.java_length()? {
                    hash = hash
                        .wrapping_mul(31)
                        .wrapping_add(i32::from(component.java_char_at(index)?));
                }
            }
        }
        *lock(&self.hash) = hash;
        Ok(hash)
    }

    /// 按字符内容比较任意 Java CharSequence。
    /// 对应 Java: `AggregateCharSequence#contentEquals()`。
    pub fn content_equals(&self, other: &dyn CharSequenceValue) -> Result<bool, TextUtilsError> {
        if self.length != other.java_length()? {
            return Ok(false);
        }
        if self.length == 0 || other.java_sequence_equals(self)? {
            return Ok(true);
        }
        for index in 0..self.length {
            if self.char_at(index)? != other.java_char_at(index)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// 将所有组件的 `toString()` 结果串联。
    /// 对应 Java 语义：`AggregateCharSequence` 的 `to_utf16_string` 行为（Rust 侧辅助/私有路径）。
    pub fn to_utf16_string(&self) -> Result<Utf16String, TextUtilsError> {
        if self.length == 0 {
            return Ok(Utf16String::from_utf16(Vec::new()));
        }
        if self.values.len() == 1 {
            return self.values[0].java_to_string();
        }
        let result_length =
            usize::try_from(self.length).map_err(|_| TextUtilsError::SequenceAccess {
                class_name: "java.lang.NegativeArraySizeException".into(),
                message: Some(Utf16String::from_rust_str(&self.length.to_string())),
            })?;
        let mut result = vec![0_u16; result_length];
        for (component_index, component) in self.values.iter().enumerate() {
            let component_length = component.java_length()?;
            for source_index in 0..component_length {
                let destination_index = self.offsets[component_index].wrapping_add(source_index);
                let destination_index = usize::try_from(destination_index).map_err(|_| {
                    TextUtilsError::ArrayIndexOutOfBounds {
                        index: destination_index,
                        length: result.len(),
                    }
                })?;
                let Some(slot) = result.get_mut(destination_index) else {
                    return Err(TextUtilsError::ArrayIndexOutOfBounds {
                        index: destination_index as i32,
                        length: result.len(),
                    });
                };
                *slot = component.java_char_at(source_index)?;
            }
        }
        Ok(Utf16String::from_utf16(result))
    }
}

impl CharSequenceValue for AggregateCharSequence {
    fn java_sequence_class_name(&self) -> &str {
        "org.thymeleaf.util.AggregateCharSequence"
    }

    fn java_length(&self) -> Result<i32, TextUtilsError> {
        Ok(self.length)
    }

    fn java_char_at(&self, index: i32) -> Result<u16, TextUtilsError> {
        self.char_at(index)
    }

    fn as_utf16_string(&self) -> Option<&Utf16String> {
        None
    }

    fn java_to_string(&self) -> Result<Utf16String, TextUtilsError> {
        self.to_utf16_string()
    }

    fn java_sub_sequence(&self, start: i32, end: i32) -> Result<Utf16String, TextUtilsError> {
        self.sub_sequence(start, end)
    }

    fn write_direct(&self, writer: &mut dyn TemplateWriter) -> Option<io::Result<()>> {
        Some(IWritableCharSequence::write(self, writer))
    }

    fn java_sequence_hash_code(&self) -> Result<i32, TextUtilsError> {
        self.hash_code()
    }

    fn java_sequence_equals(&self, other: &dyn CharSequenceValue) -> Result<bool, TextUtilsError> {
        if other.java_sequence_class_name() != "org.thymeleaf.util.AggregateCharSequence"
            || self.length != other.java_length()?
        {
            return Ok(false);
        }
        for index in 0..self.length {
            if self.char_at(index)? != other.java_char_at(index)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl IWritableCharSequence for AggregateCharSequence {
    fn write(&self, writer: &mut dyn TemplateWriter) -> io::Result<()> {
        for component in &self.values {
            let value = component
                .java_to_string()
                .map_err(|error| io::Error::other(error.to_string()))?;
            writer.write_utf16(value.as_utf16())?;
        }
        Ok(())
    }
}

fn require_arguments<const N: usize>(
    components: [Option<AggregateComponent>; N],
) -> Result<Vec<AggregateComponent>, AggregateCharSequenceError> {
    components
        .into_iter()
        .map(|component| component.ok_or(AggregateCharSequenceError::NullComponentArgument))
        .collect()
}

fn range_error(index: i32, length: i32) -> TextUtilsError {
    TextUtilsError::StringIndexOutOfBounds {
        index,
        length: usize::try_from(length).unwrap_or_default(),
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
