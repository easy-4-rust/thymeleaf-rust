use std::sync::Arc;

use indexmap::IndexMap;
use thiserror::Error;

use crate::util::Utf16String;

use super::{
    ExpressionObjectNames, IExpressionObjects, NativeContextPropertyAccessor, TemplateValue,
};

/// 把惰性 [`IExpressionObjects`] 暴露为原生求值器可读取的 Map 适配器。
///
/// 表达式对象优先于本地 Map 项并保持惰性构建；PropertyAccessor 使用本地项传递受限
/// 执行标记。表达式对象名称不能被覆盖或删除，Java 明确禁止的全量 Map 操作继续返回
/// 对应错误。本类型承接 OGNL 包装器的可观察 Map 合同，但不依赖 OGNL 运行时。
///
/// 对应 Java: `org.thymeleaf.standard.expression.OGNLExpressionObjectsWrapper`。
pub struct NativeExpressionObjectsWrapper<'a> {
    expression_objects: &'a dyn IExpressionObjects,
    local_values: IndexMap<Option<Utf16String>, Option<Arc<TemplateValue>>>,
}

impl<'a> NativeExpressionObjectsWrapper<'a> {
    const RESTRICTED_NAMES: [&'static str; 5] = ["ctx", "vars", "root", "this", "execInfo"];

    /// 创建包装器；表达式对象仍由原容器惰性构建。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#OGNLExpressionObjectsWrapper`。
    ///
    /// # 参数
    ///
    /// - `expression_objects`：当前模板执行的表达式对象容器。
    ///
    /// # 返回值
    ///
    /// 返回预留五个本地标记槽位的空包装 Map。
    #[must_use]
    pub fn new(expression_objects: &'a dyn IExpressionObjects) -> Self {
        Self {
            expression_objects,
            local_values: IndexMap::with_capacity(5),
        }
    }

    /// 判断名称在受限执行上下文中是否禁止访问。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#isRestricted(String)`。
    ///
    /// # 参数
    ///
    /// - `name`：待检查的可空名称。
    ///
    /// # 返回值
    ///
    /// `ctx`、`vars`、`root`、`this` 或 `execInfo` 返回 `true`。
    #[must_use]
    pub fn is_restricted(name: Option<&Utf16String>) -> bool {
        name.is_some_and(|name| {
            Self::RESTRICTED_NAMES
                .iter()
                .any(|candidate| name == &Utf16String::from_rust_str(candidate))
        })
    }

    /// 返回表达式对象与本地 Map 项总数。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#size()`。
    #[must_use]
    pub fn size(&self) -> i32 {
        self.expression_objects
            .size()
            .wrapping_add(i32::try_from(self.local_values.len()).unwrap_or(i32::MAX))
    }

    /// 判断适配 Map 是否为空。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#isEmpty()`。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.expression_objects.size() == 0 && self.local_values.is_empty()
    }

    /// 判断名称是否属于表达式对象或本地 Map。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#containsKey(Object)`。
    ///
    /// # 参数
    ///
    /// - `key`：待检查键；`None` 对应 Java `null`。
    ///
    /// # 返回值
    ///
    /// 任一命名空间包含该键时返回 `true`。
    ///
    /// # 错误
    ///
    /// Java 会先调用 `key.toString()`；键为 `None` 时返回对应空指针错误。
    pub fn contains_key(
        &self,
        key: Option<&Utf16String>,
    ) -> Result<bool, NativeExpressionObjectsWrapperError> {
        let key = require_key(key)?;
        Ok(self.expression_objects.contains_object(Some(key))
            || self.local_values.contains_key(&Some(key.clone())))
    }

    /// 惰性读取表达式对象或本地项。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#get(Object)`。
    ///
    /// # 参数
    ///
    /// - `key`：待读取键；`None` 对应 Java `null`。
    ///
    /// # 返回值
    ///
    /// 表达式对象名称优先委托容器，否则读取本地 Map。
    ///
    /// # 错误
    ///
    /// 键为 `None`、受限名称被禁止或工厂构建失败时返回对应错误。
    pub fn get(
        &self,
        key: Option<&Utf16String>,
    ) -> Result<Option<Arc<TemplateValue>>, NativeExpressionObjectsWrapperError> {
        let key = require_key(key)?;
        if self.expression_objects.contains_object(Some(key)) {
            if self.restrict_expression_objects() && Self::is_restricted(Some(key)) {
                return Err(NativeExpressionObjectsWrapperError::Restricted {
                    name: key.to_string_lossy(),
                });
            }
            return self
                .expression_objects
                .get_object(Some(key))
                .map_err(|error| NativeExpressionObjectsWrapperError::Build {
                    message: error.to_string(),
                });
        }
        Ok(self.local_values.get(&Some(key.clone())).cloned().flatten())
    }

    /// 写入仅供 OGNL PropertyAccessor 通信的本地标记项。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#put(String,Object)`。
    ///
    /// # 参数
    ///
    /// - `key`：待写入键；`None` 对应 Java `null`。
    /// - `value`：可空 Map 值。
    ///
    /// # 返回值
    ///
    /// 返回被替换的旧值。
    ///
    /// # 错误
    ///
    /// 键为 `None` 或与表达式对象名称冲突时拒绝写入。
    pub fn put(
        &mut self,
        key: Option<Utf16String>,
        value: Option<Arc<TemplateValue>>,
    ) -> Result<Option<Arc<TemplateValue>>, NativeExpressionObjectsWrapperError> {
        let key = key.ok_or(NativeExpressionObjectsWrapperError::NullStringKey)?;
        if self.expression_objects.contains_object(Some(&key)) {
            return Err(
                NativeExpressionObjectsWrapperError::ExpressionObjectMutation {
                    message: format!(
                        "Cannot put entry with key \"{}\" into Expression Objects wrapper map: key matches the name of one of the expression objects",
                        key.to_string_lossy()
                    ),
                },
            );
        }
        Ok(self.local_values.insert(Some(key), value).flatten())
    }

    /// 批量写入本地 Map。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#putAll(Map)`。
    ///
    /// # 参数
    ///
    /// - `entries`：按迭代顺序写入的键值项。
    ///
    /// # 错误
    ///
    /// Java `HashMap#putAll` 绕过覆盖后的 `put`，所以表达式对象同名键和空键都会直接
    /// 写入本地 Map。
    pub fn put_all<I>(&mut self, entries: I) -> Result<(), NativeExpressionObjectsWrapperError>
    where
        I: IntoIterator<Item = (Option<Utf16String>, Option<Arc<TemplateValue>>)>,
    {
        // HashMap#putAll 的内部 putMapEntries/putVal 不会动态派发到覆盖后的 put。
        // 因此这里必须绕过表达式对象名称冲突检查，保留上游真实运行时行为。
        for (key, value) in entries {
            self.local_values.insert(key, value);
        }
        Ok(())
    }

    /// 删除本地标记项；表达式对象名称不可删除。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#remove(Object)`。
    ///
    /// # 参数
    ///
    /// - `key`：待删除键；`None` 对应 Java `null`。
    ///
    /// # 返回值
    ///
    /// 返回被删除的本地值。
    ///
    /// # 错误
    ///
    /// 键为空或属于表达式对象时拒绝删除。
    pub fn remove(
        &mut self,
        key: Option<&Utf16String>,
    ) -> Result<Option<Arc<TemplateValue>>, NativeExpressionObjectsWrapperError> {
        let key = require_key(key)?;
        if self.expression_objects.contains_object(Some(key)) {
            return Err(
                NativeExpressionObjectsWrapperError::ExpressionObjectMutation {
                    message: format!(
                        "Cannot remove entry with key \"{}\" from Expression Objects wrapper map: key matches the name of one of the expression objects",
                        key.to_string_lossy()
                    ),
                },
            );
        }
        Ok(self.local_values.shift_remove(&Some(key.clone())).flatten())
    }

    /// 拒绝清空包装 Map。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#clear()`。
    pub fn clear(&mut self) -> Result<(), NativeExpressionObjectsWrapperError> {
        Err(NativeExpressionObjectsWrapperError::UnsupportedOperation {
            message: "Cannot clear Expression Objects wrapper map",
        })
    }

    /// 拒绝按值搜索包装 Map。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#containsValue(Object)`。
    pub fn contains_value(
        &self,
        _value: Option<&Arc<TemplateValue>>,
    ) -> Result<bool, NativeExpressionObjectsWrapperError> {
        Err(NativeExpressionObjectsWrapperError::UnsupportedOperation {
            message: "Cannot perform by-value search on Expression Objects wrapper map",
        })
    }

    /// 拒绝克隆完整包装 Map。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#clone()`。
    pub fn clone_map(&self) -> Result<(), NativeExpressionObjectsWrapperError> {
        Err(NativeExpressionObjectsWrapperError::UnsupportedOperation {
            message: "Cannot clone Expression Objects wrapper map",
        })
    }

    /// 返回表达式对象名在前、本地键在后的去重集合。
    ///
    /// 本地 Map 为空时返回表达式对象容器持有的同一共享名称集合；存在本地键时创建新的
    /// `LinkedHashSet` 等价快照。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#keySet()`。
    #[must_use]
    pub fn key_set(&self) -> ExpressionObjectNames {
        let expression_names = self.expression_objects.get_object_names();
        if self.local_values.is_empty() {
            return expression_names;
        }
        let mut keys = expression_names.to_vec();
        for key in self.local_values.keys() {
            if !keys.contains(key) {
                keys.push(key.clone());
            }
        }
        keys.into()
    }

    /// 返回本地 Map 值快照；与 Java 一致，不会强制构建所有表达式对象。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#values()`。
    #[must_use]
    pub fn values(&self) -> Vec<Option<Arc<TemplateValue>>> {
        self.local_values.values().cloned().collect()
    }

    /// 拒绝取得包含惰性表达式对象的完整 entry set。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#entrySet()`。
    pub fn entry_set(&self) -> Result<(), NativeExpressionObjectsWrapperError> {
        Err(NativeExpressionObjectsWrapperError::UnsupportedOperation {
            message: "Cannot retrieve a complete entry set for Expression Objects wrapper map. Get a key set instead",
        })
    }

    /// 拒绝对完整包装 Map 执行相等比较。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#equals(Object)`。
    pub fn equals_map(&self, _other: &Self) -> Result<bool, NativeExpressionObjectsWrapperError> {
        Err(NativeExpressionObjectsWrapperError::UnsupportedOperation {
            message: "Cannot execute equals operation on Expression Objects wrapper map",
        })
    }

    /// 拒绝计算完整包装 Map 的 Java hashCode。
    ///
    /// 对应 Java: `OGNLExpressionObjectsWrapper#hashCode()`。
    pub fn hash_code(&self) -> Result<i32, NativeExpressionObjectsWrapperError> {
        Err(NativeExpressionObjectsWrapperError::UnsupportedOperation {
            message: "Cannot execute hashCode operation on Expression Objects wrapper map",
        })
    }

    fn restrict_expression_objects(&self) -> bool {
        self.local_values
            .contains_key(&Some(Utf16String::from_rust_str(
                NativeContextPropertyAccessor::RESTRICT_EXPRESSION_OBJECTS,
            )))
    }
}

impl std::fmt::Display for NativeExpressionObjectsWrapper<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names = self
            .key_set()
            .iter()
            .map(|name| {
                name.as_ref()
                    .map_or_else(|| "null".to_owned(), Utf16String::to_string_lossy)
            })
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            formatter,
            "{{EXPRESSION OBJECTS WRAPPER MAP FOR KEYS: [{names}]}}"
        )
    }
}

impl NativeExpressionObjectsWrapperError {
    /// 返回对应 Java 异常类型。
    #[must_use]
    pub const fn class_name(&self) -> &'static str {
        match self {
            Self::NullKey | Self::NullStringKey => "java.lang.NullPointerException",
            Self::Restricted { .. } | Self::Build { .. } => {
                "org.thymeleaf.exceptions.TemplateProcessingException"
            }
            Self::ExpressionObjectMutation { .. } => "java.lang.IllegalArgumentException",
            Self::UnsupportedOperation { .. } => "java.lang.UnsupportedOperationException",
        }
    }
}

/// 表达式对象包装 Map 的访问错误。
#[derive(Debug, Error, Eq, PartialEq)]
/// 对应 Java 语义：`OGNLExpressionObjectsWrapper` 的 Rust 侧类型 `NativeExpressionObjectsWrapperError`。
pub enum NativeExpressionObjectsWrapperError {
    /// Java Map 操作在调用 `key.toString()` 时遇到空键。
    #[error("Cannot invoke \"Object.toString()\" because \"key\" is null")]
    NullKey,
    /// Java `put(String,Object)` 调用 `String#toString` 时遇到空键。
    #[error("Cannot invoke \"String.toString()\" because \"key\" is null")]
    NullStringKey,
    /// 受限执行上下文拒绝访问对象。
    #[error("Access to variable '{name}' is forbidden in this context.")]
    Restricted {
        /// 对象名。
        name: String,
    },
    /// 试图覆盖或删除惰性表达式对象。
    #[error("{message}")]
    ExpressionObjectMutation {
        /// 与 Java `IllegalArgumentException` 一致的完整消息。
        message: String,
    },
    /// Java 明确禁止的完整 Map 操作。
    #[error("{message}")]
    UnsupportedOperation {
        /// 与 Java `UnsupportedOperationException` 一致的消息。
        message: &'static str,
    },
    /// 表达式对象工厂构建失败。
    #[error("{message}")]
    Build {
        /// 原错误消息。
        message: String,
    },
}

fn require_key(
    key: Option<&Utf16String>,
) -> Result<&Utf16String, NativeExpressionObjectsWrapperError> {
    key.ok_or(NativeExpressionObjectsWrapperError::NullKey)
}
