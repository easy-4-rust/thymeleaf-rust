use std::any::Any;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard};

use super::Utf16String;

/// Java `char[]` 的线程安全共享适配。
///
/// 对应 Java: `char[]`，由
/// `org.thymeleaf.util.CharArrayWrapperSequence` 保留原数组引用。调用方持有同一个
/// `Arc<RwLock<Vec<u16>>>` 即可在构造后修改数组，并让所有视图与浅克隆立即观察到
/// 变化；读写锁承接上游 JavaDoc 声明的线程安全合同。
pub type SharedCharArray = Arc<RwLock<Vec<u16>>>;

/// `CharArrayWrapperSequence` 操作失败。
///
/// 对应 Java: `org.thymeleaf.util.CharArrayWrapperSequence` 显式校验、数组访问及
/// `String(char[],int,int)` 构造路径抛出的运行时异常。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CharArrayWrapperSequenceError {
    /// 构造器收到 null 缓冲区。
    NullBuffer,
    /// 构造器收到小于零或不小于数组长度的 offset。
    InvalidOffset {
        /// 非法 offset。
        offset: i32,
        /// Java 数组长度。
        size: usize,
    },
    /// offset 与 length 的 Java 回绕和大于数组长度。
    InvalidLength {
        /// 非法 length。
        length: i32,
        /// 构造器 offset。
        offset: i32,
        /// Java 数组长度。
        size: usize,
    },
    /// `charAt`/`subSequence` 显式抛出的数组下标异常。
    ExplicitArrayIndex {
        /// 被拒绝的调用参数。
        index: i32,
    },
    /// 通过 Java 数组取值指令触发的下标异常。
    BufferIndex {
        /// offset 与 index 按 Java `int` 回绕后的数组下标。
        index: i32,
        /// Java 数组长度。
        size: usize,
    },
    /// 从非法视图构造 Java `String` 时的范围异常。
    StringRange {
        /// 视图 offset。
        offset: i32,
        /// 视图 length。
        length: i32,
        /// Java 数组长度。
        size: usize,
    },
}

impl CharArrayWrapperSequenceError {
    /// 返回对应的 Java 异常全限定名。
    ///
    /// # 返回
    /// Java `Throwable#getClass().getName()` 的精确结果。
    #[must_use]
    pub const fn java_class_name(&self) -> &'static str {
        match self {
            Self::NullBuffer | Self::InvalidOffset { .. } | Self::InvalidLength { .. } => {
                "java.lang.IllegalArgumentException"
            }
            Self::ExplicitArrayIndex { .. } | Self::BufferIndex { .. } => {
                "java.lang.ArrayIndexOutOfBoundsException"
            }
            Self::StringRange { .. } => "java.lang.StringIndexOutOfBoundsException",
        }
    }

    /// 返回对应 Java 异常消息。
    ///
    /// # 返回
    /// 保存精确消息 UTF-16 代码单元的新 Java 字符串。
    #[must_use]
    pub fn message(&self) -> Utf16String {
        let message = match self {
            Self::NullBuffer => "Buffer cannot be null".to_owned(),
            Self::InvalidOffset { offset, size } => {
                format!("{offset} is not a valid offset for buffer (size: {size})")
            }
            Self::InvalidLength {
                length,
                offset,
                size,
            } => format!(
                "{length} is not a valid length for buffer using offset {offset} (size: {size})"
            ),
            Self::ExplicitArrayIndex { index } => {
                format!("Array index out of range: {index}")
            }
            Self::BufferIndex { index, size } => {
                format!("Index {index} out of bounds for length {size}")
            }
            Self::StringRange {
                offset,
                length,
                size,
            } => format!("Range [{offset}, {offset} + {length}) out of bounds for length {size}"),
        };
        Utf16String::from_rust_str(&message)
    }
}

impl Display for CharArrayWrapperSequenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message().to_string_lossy())
    }
}

impl Error for CharArrayWrapperSequenceError {}

/// 不复制底层 Java `char[]` 的字符序列视图。
///
/// 对应 Java: `org.thymeleaf.util.CharArrayWrapperSequence`。
///
/// 对象持有共享 UTF-16 数组、offset 和 Java `int` length。底层数组的外部修改会
/// 立即改变本对象及其子序列/浅克隆。构造器故意保留上游边界：空数组没有合法
/// offset；负 length 或加法溢出的 length 可能成功构造，但随后 `charAt`、
/// `subSequence` 或字符串物化会按 Java 的后续检查抛出异常。
pub struct CharArrayWrapperSequence {
    buffer: SharedCharArray,
    offset: i32,
    len: i32,
}

impl CharArrayWrapperSequence {
    /// 包装完整 Java `char[]`，不复制内容。
    ///
    /// 对应 Java: `CharArrayWrapperSequence#CharArrayWrapperSequence(char[])`。
    ///
    /// # 参数
    /// - `array`：共享 UTF-16 数组；`None` 对应 Java null。
    ///
    /// # 错误
    /// null 返回 `IllegalArgumentException("Buffer cannot be null")`；空数组继续
    /// 调用三参数构造器并因 offset 0 非法而失败。
    pub fn new(array: Option<SharedCharArray>) -> Result<Self, CharArrayWrapperSequenceError> {
        let length = array
            .as_ref()
            .map(|buffer| read_buffer(buffer).len() as i32)
            .unwrap_or(-1);
        Self::with_range(array, 0, length)
    }

    /// 包装 Java `char[]` 的指定视图，不复制内容。
    ///
    /// 对应 Java:
    /// `CharArrayWrapperSequence#CharArrayWrapperSequence(char[],int,int)`。
    ///
    /// # 参数
    /// - `buffer`：共享 UTF-16 数组；`None` 对应 Java null。
    /// - `offset`：视图起点。
    /// - `length`：视图声明长度。
    ///
    /// # 错误
    /// 严格按上游顺序先校验 null，再校验 offset，最后只检查 Java 回绕的
    /// `offset + length > buffer.length`；不上移额外的负 length 校验。
    pub fn with_range(
        buffer: Option<SharedCharArray>,
        offset: i32,
        length: i32,
    ) -> Result<Self, CharArrayWrapperSequenceError> {
        let buffer = buffer.ok_or(CharArrayWrapperSequenceError::NullBuffer)?;
        let size = read_buffer(&buffer).len();
        if offset < 0 || usize::try_from(offset).map_or(true, |offset| offset >= size) {
            return Err(CharArrayWrapperSequenceError::InvalidOffset { offset, size });
        }
        if offset.wrapping_add(length) > size as i32 {
            return Err(CharArrayWrapperSequenceError::InvalidLength {
                length,
                offset,
                size,
            });
        }
        Ok(Self {
            buffer,
            offset,
            len: length,
        })
    }

    /// 返回指定相对下标的 Java UTF-16 `char`。
    ///
    /// 对应 Java: `CharArrayWrapperSequence#charAt(int)`。
    ///
    /// # 参数
    /// - `index`：相对本视图的下标。
    ///
    /// # 返回
    /// 原数组中的单个 UTF-16 代码单元，可为孤立代理项。
    ///
    /// # 错误
    /// 先保留显式 index/length 检查的异常消息；畸形溢出视图通过该检查后，再按
    /// Java 数组访问指令报告回绕后的真实数组下标。
    pub fn char_at(&self, index: i32) -> Result<u16, CharArrayWrapperSequenceError> {
        if index < 0 || index >= self.len {
            return Err(CharArrayWrapperSequenceError::ExplicitArrayIndex { index });
        }
        let buffer = read_buffer(&self.buffer);
        let absolute_index = self.offset.wrapping_add(index);
        let absolute_index_usize = usize::try_from(absolute_index).map_err(|_| {
            CharArrayWrapperSequenceError::BufferIndex {
                index: absolute_index,
                size: buffer.len(),
            }
        })?;
        buffer.get(absolute_index_usize).copied().ok_or(
            CharArrayWrapperSequenceError::BufferIndex {
                index: absolute_index,
                size: buffer.len(),
            },
        )
    }

    /// 返回 Java `CharSequence#length()` 声明值。
    ///
    /// 对应 Java: `CharArrayWrapperSequence#length()`。
    ///
    /// # 返回
    /// 原样返回构造器保存的 Java `int`，包括上游允许保存的负数。
    #[must_use]
    pub const fn length(&self) -> i32 {
        self.len
    }

    /// 创建共享同一底层数组的子序列。
    ///
    /// 对应 Java: `CharArrayWrapperSequence#subSequence(int,int)`。
    ///
    /// # 参数
    /// - `start`：相对起点。
    /// - `end`：相对终点。
    ///
    /// # 返回
    /// 与原对象共享数组的新视图。
    ///
    /// # 错误
    /// 先按上游显式检查 start 和 end；随后使用 Java 回绕运算调用同一构造器，
    /// 因而 reversed 范围可能先形成负 length，再在物化时失败。
    pub fn sub_sequence(
        &self,
        start: i32,
        end: i32,
    ) -> Result<Self, CharArrayWrapperSequenceError> {
        if start < 0 || start >= self.len {
            return Err(CharArrayWrapperSequenceError::ExplicitArrayIndex { index: start });
        }
        if end > self.len {
            return Err(CharArrayWrapperSequenceError::ExplicitArrayIndex { index: end });
        }
        Self::with_range(
            Some(Arc::clone(&self.buffer)),
            self.offset.wrapping_add(start),
            end.wrapping_sub(start),
        )
    }

    /// 计算与 Java `String#hashCode()` 兼容的 32 位哈希值。
    ///
    /// 对应 Java: `CharArrayWrapperSequence#hashCode()`。
    ///
    /// # 返回
    /// 按 31 倍累乘及 Java `int` 回绕得到的值；零或上游畸形负/溢出视图的空循环
    /// 返回 0。
    #[must_use]
    pub fn hash_code(&self) -> i32 {
        if self.len == 0 {
            return 0;
        }
        let buffer = read_buffer(&self.buffer);
        let maximum = self.offset.wrapping_add(self.len);
        let mut result = 0_i32;
        let mut index = self.offset;
        while index < maximum {
            let unit = buffer[index as usize];
            result = result.wrapping_mul(31).wrapping_add(i32::from(unit));
            index = index.wrapping_add(1);
        }
        result
    }

    /// 执行 Java `equals(Object)` 的类型、身份和内容比较。
    ///
    /// 对应 Java: `CharArrayWrapperSequence#equals(Object)`。
    ///
    /// # 参数
    /// - `object`：任意 Rust `'static` 对象；`None` 对应 Java null。
    ///
    /// # 返回
    /// 仅另一个 `CharArrayWrapperSequence` 可以相等；Utf16String/String 等其他
    /// 类型即使内容相同也返回 false。相同引用立即返回 true。
    #[must_use]
    pub fn equals_object(&self, object: Option<&dyn Any>) -> bool {
        let Some(other) = object.and_then(|value| value.downcast_ref::<Self>()) else {
            return false;
        };
        if std::ptr::eq(self, other) {
            return true;
        }
        self.content_equals(other)
    }

    /// 将当前视图物化为新的 Java UTF-16 字符串。
    ///
    /// 对应 Java: `CharArrayWrapperSequence#toString()`。
    ///
    /// # 返回
    /// 当前范围的独立快照。
    ///
    /// # 错误
    /// 构造阶段被上游接受的负 length 或溢出 length 在此返回精确
    /// `StringIndexOutOfBoundsException` 类别与范围消息。
    pub fn to_utf16_string(&self) -> Result<Utf16String, CharArrayWrapperSequenceError> {
        let buffer = read_buffer(&self.buffer);
        let end = i64::from(self.offset) + i64::from(self.len);
        // 所有构造路径都已拒绝负 offset；这里仅复现仍可能延迟到 String 构造阶段的
        // 负 length 和回绕后超长视图。
        if self.len < 0 || end > buffer.len() as i64 {
            return Err(CharArrayWrapperSequenceError::StringRange {
                offset: self.offset,
                length: self.len,
                size: buffer.len(),
            });
        }
        Ok(Utf16String::from_utf16(
            buffer[self.offset as usize..end as usize].to_vec(),
        ))
    }

    fn content_equals(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        let left = read_buffer(&self.buffer);
        let right = read_buffer(&other.buffer);
        for index in 0..self.len {
            if left[self.offset.wrapping_add(index) as usize]
                != right[other.offset.wrapping_add(index) as usize]
            {
                return false;
            }
        }
        true
    }
}

impl Clone for CharArrayWrapperSequence {
    /// 创建与 Java protected `clone()` 相同的浅克隆。
    ///
    /// Rust 以标准 `Clone` trait 映射 Java `Cloneable`；新对象保留相同 offset 和
    /// length，并共享同一底层数组。
    fn clone(&self) -> Self {
        Self {
            buffer: Arc::clone(&self.buffer),
            offset: self.offset,
            len: self.len,
        }
    }
}

impl PartialEq for CharArrayWrapperSequence {
    fn eq(&self, other: &Self) -> bool {
        self.content_equals(other)
    }
}

impl Eq for CharArrayWrapperSequence {}

fn read_buffer(buffer: &SharedCharArray) -> RwLockReadGuard<'_, Vec<u16>> {
    buffer.read().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use super::{CharArrayWrapperSequence, CharArrayWrapperSequenceError};

    fn chars() -> Arc<RwLock<Vec<u16>>> {
        Arc::new(RwLock::new(vec![0x0041, 0xD800, 0x0042, 0x0043]))
    }

    #[test]
    fn clone_and_subsequence_share_external_mutation() {
        let buffer = chars();
        let sequence =
            CharArrayWrapperSequence::with_range(Some(Arc::clone(&buffer)), 1, 2).expect("view");
        let clone = sequence.clone();
        let sub = sequence.sub_sequence(0, 2).expect("subsequence");
        buffer.write().expect("write lock")[1] = 0x005A;

        assert_eq!(
            sequence.to_utf16_string().expect("sequence").as_utf16(),
            &[0x005A, 0x0042]
        );
        assert!(clone == sequence);
        assert!(sub == sequence);
    }

    #[test]
    fn preserves_empty_negative_and_overflow_view_boundaries() {
        assert_eq!(
            CharArrayWrapperSequence::new(Some(Arc::new(RwLock::new(Vec::new()))))
                .err()
                .expect("empty error"),
            CharArrayWrapperSequenceError::InvalidOffset { offset: 0, size: 0 }
        );
        let negative =
            CharArrayWrapperSequence::with_range(Some(chars()), 1, -2).expect("negative view");
        assert_eq!(negative.length(), -2);
        assert_eq!(negative.hash_code(), 0);
        assert_eq!(
            negative.to_utf16_string(),
            Err(CharArrayWrapperSequenceError::StringRange {
                offset: 1,
                length: -2,
                size: 4
            })
        );

        let overflow =
            CharArrayWrapperSequence::with_range(Some(chars()), 1, i32::MAX).expect("overflow");
        assert_eq!(overflow.hash_code(), 0);
        assert_eq!(
            overflow.char_at(i32::MAX - 1),
            Err(CharArrayWrapperSequenceError::BufferIndex {
                index: i32::MAX,
                size: 4
            })
        );

        let negative_absolute =
            CharArrayWrapperSequence::with_range(Some(chars()), 2, i32::MAX).expect("overflow");
        assert_eq!(
            negative_absolute.char_at(i32::MAX - 1),
            Err(CharArrayWrapperSequenceError::BufferIndex {
                index: i32::MIN,
                size: 4
            })
        );
        assert_eq!(
            CharArrayWrapperSequenceError::NullBuffer.to_string(),
            "Buffer cannot be null"
        );
    }
}
