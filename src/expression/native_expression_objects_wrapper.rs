use std::sync::Arc;

use indexmap::IndexMap;
use thiserror::Error;

use crate::util::JavaString;

use super::{IExpressionObjects, NativeContextPropertyAccessor, TemplateValue};

/// 把惰性 `IExpressionObjects` 暴露为 OGNL 可读取 Map 的适配器。
///
/// 对应 Java: `org.thymeleaf.standard.expression.OGNLExpressionObjectsWrapper`。
pub struct NativeExpressionObjectsWrapper<'a> {
    expression_objects: &'a dyn IExpressionObjects,
    local_values: IndexMap<JavaString, Option<Arc<TemplateValue>>>,
}

impl<'a> NativeExpressionObjectsWrapper<'a> {
    const RESTRICTED_NAMES: [&'static str; 5] = ["ctx", "vars", "root", "this", "execInfo"];

    /// 创建包装器；表达式对象仍由原容器惰性构建。
    pub fn new(expression_objects: &'a dyn IExpressionObjects) -> Self {
        Self {
            expression_objects,
            local_values: IndexMap::with_capacity(5),
        }
    }

    /// 判断名称在受限执行上下文中是否禁止访问。
    pub fn is_restricted(name: Option<&JavaString>) -> bool {
        name.is_some_and(|name| {
            Self::RESTRICTED_NAMES
                .iter()
                .any(|candidate| name == &JavaString::from_rust_str(candidate))
        })
    }

    /// 返回表达式对象与本地标记项总数。
    pub fn size(&self) -> i32 {
        self.expression_objects
            .size()
            .saturating_add(i32::try_from(self.local_values.len()).unwrap_or(i32::MAX))
    }

    /// 判断适配 Map 是否为空。
    pub fn is_empty(&self) -> bool {
        self.expression_objects.size() == 0 && self.local_values.is_empty()
    }

    /// 判断名称是否存在。
    pub fn contains_key(&self, key: Option<&JavaString>) -> bool {
        self.expression_objects.contains_object(key)
            || key.is_some_and(|key| self.local_values.contains_key(key))
    }

    /// 惰性读取表达式对象或本地项。
    pub fn get(
        &self,
        key: Option<&JavaString>,
    ) -> Result<Option<Arc<TemplateValue>>, NativeExpressionObjectsWrapperError> {
        if self.expression_objects.contains_object(key) {
            if self.restrict_expression_objects() && Self::is_restricted(key) {
                return Err(NativeExpressionObjectsWrapperError::Restricted {
                    name: key.map_or_else(|| "null".to_owned(), JavaString::to_string_lossy),
                });
            }
            return self.expression_objects.get_object(key).map_err(|error| {
                NativeExpressionObjectsWrapperError::Build {
                    message: error.to_string(),
                }
            });
        }
        Ok(key.and_then(|key| self.local_values.get(key).cloned().flatten()))
    }

    /// 写入仅供 OGNL PropertyAccessor 通信的本地标记项。
    pub fn put(
        &mut self,
        key: JavaString,
        value: Option<Arc<TemplateValue>>,
    ) -> Result<Option<Arc<TemplateValue>>, NativeExpressionObjectsWrapperError> {
        if self.expression_objects.contains_object(Some(&key)) {
            return Err(
                NativeExpressionObjectsWrapperError::ExpressionObjectMutation {
                    operation: "put",
                    name: key.to_string_lossy(),
                },
            );
        }
        Ok(self.local_values.insert(key, value).flatten())
    }

    /// 删除本地标记项；表达式对象名称不可删除。
    pub fn remove(
        &mut self,
        key: Option<&JavaString>,
    ) -> Result<Option<Arc<TemplateValue>>, NativeExpressionObjectsWrapperError> {
        if self.expression_objects.contains_object(key) {
            return Err(
                NativeExpressionObjectsWrapperError::ExpressionObjectMutation {
                    operation: "remove",
                    name: key.map_or_else(|| "null".to_owned(), JavaString::to_string_lossy),
                },
            );
        }
        Ok(key
            .and_then(|key| self.local_values.shift_remove(key))
            .flatten())
    }

    /// 返回表达式对象名在前、本地键在后的去重快照。
    pub fn key_set(&self) -> Vec<Option<JavaString>> {
        let mut keys = self.expression_objects.get_object_names();
        for key in self.local_values.keys() {
            if !keys.contains(&Some(key.clone())) {
                keys.push(Some(key.clone()));
            }
        }
        keys
    }

    /// 返回本地 Map 值快照；与 Java 一致，不会强制构建所有表达式对象。
    pub fn values(&self) -> Vec<Option<Arc<TemplateValue>>> {
        self.local_values.values().cloned().collect()
    }

    fn restrict_expression_objects(&self) -> bool {
        self.local_values.contains_key(&JavaString::from_rust_str(
            NativeContextPropertyAccessor::RESTRICT_EXPRESSION_OBJECTS,
        ))
    }
}

impl std::fmt::Display for NativeExpressionObjectsWrapper<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names = self
            .key_set()
            .into_iter()
            .map(|name| name.map_or_else(|| "null".to_owned(), |name| name.to_string_lossy()))
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            formatter,
            "{{EXPRESSION OBJECTS WRAPPER MAP FOR KEYS: [{names}]}}"
        )
    }
}

/// 表达式对象包装 Map 的访问错误。
#[derive(Debug, Error, Eq, PartialEq)]
pub enum NativeExpressionObjectsWrapperError {
    /// 受限执行上下文拒绝访问对象。
    #[error("Access to variable '{name}' is forbidden in this context.")]
    Restricted {
        /// 对象名。
        name: String,
    },
    /// 试图覆盖或删除惰性表达式对象。
    #[error(
        "Cannot {operation} entry with key \"{name}\" in Expression Objects wrapper map: key matches the name of one of the expression objects"
    )]
    ExpressionObjectMutation {
        /// 操作名。
        operation: &'static str,
        /// 对象名。
        name: String,
    },
    /// Java 明确禁止的完整 Map 操作。
    #[error("Operation is not supported on Expression Objects wrapper map")]
    UnsupportedOperation,
    /// 表达式对象工厂构建失败。
    #[error("{message}")]
    Build {
        /// 原错误消息。
        message: String,
    },
}
