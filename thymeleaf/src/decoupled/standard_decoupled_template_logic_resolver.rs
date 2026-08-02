use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::templateresource::{ITemplateResource, TemplateResourceError};
use crate::util::JavaString;
use crate::{IEngineConfiguration, TemplateMode};

use super::IDecoupledTemplateLogicResolver;

/// 通过主模板资源的相对位置解析解耦逻辑资源。
///
/// 默认位置为 `{base_name}.th.xml`；prefix 与 suffix 可独立设为 Java `null`。
/// 配置通过读写锁共享，复现 Java 对象可并发读取和重新配置的生命周期。
///
/// 对应 Java:
/// `org.thymeleaf.templateparser.markup.decoupled.StandardDecoupledTemplateLogicResolver`。
pub struct StandardDecoupledTemplateLogicResolver {
    prefix: RwLock<Option<JavaString>>,
    suffix: RwLock<Option<JavaString>>,
}

impl StandardDecoupledTemplateLogicResolver {
    /// 默认解耦逻辑文件后缀。
    pub const DECOUPLED_TEMPLATE_LOGIC_FILE_SUFFIX: &'static str = ".th.xml";

    /// 创建无 prefix、使用 `.th.xml` suffix 的 resolver。
    ///
    /// 对应 Java:
    /// `StandardDecoupledTemplateLogicResolver#StandardDecoupledTemplateLogicResolver()`。
    #[must_use]
    pub fn new() -> Self {
        Self {
            prefix: RwLock::new(None),
            suffix: RwLock::new(Some(JavaString::from_rust_str(
                Self::DECOUPLED_TEMPLATE_LOGIC_FILE_SUFFIX,
            ))),
        }
    }

    /// 返回当前 suffix；`None` 对应 Java `null`。
    ///
    /// 对应 Java: `StandardDecoupledTemplateLogicResolver#getSuffix()`。
    #[must_use]
    pub fn get_suffix(&self) -> Option<JavaString> {
        read_lock(&self.suffix).clone()
    }

    /// 替换 suffix；`None` 使相对位置不追加后缀。
    ///
    /// 对应 Java: `StandardDecoupledTemplateLogicResolver#setSuffix(String)`。
    pub fn set_suffix(&self, suffix: Option<JavaString>) {
        *write_lock(&self.suffix) = suffix;
    }

    /// 返回当前 prefix；`None` 对应 Java `null`。
    ///
    /// 对应 Java: `StandardDecoupledTemplateLogicResolver#getPrefix()`。
    #[must_use]
    pub fn get_prefix(&self) -> Option<JavaString> {
        read_lock(&self.prefix).clone()
    }

    /// 替换 prefix；`None` 使相对位置不添加前缀。
    ///
    /// 对应 Java: `StandardDecoupledTemplateLogicResolver#setPrefix(String)`。
    pub fn set_prefix(&self, prefix: Option<JavaString>) {
        *write_lock(&self.prefix) = prefix;
    }
}

impl Default for StandardDecoupledTemplateLogicResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl IDecoupledTemplateLogicResolver for StandardDecoupledTemplateLogicResolver {
    fn resolve_decoupled_template_logic(
        &self,
        _configuration: &dyn IEngineConfiguration,
        _owner_template: Option<&JavaString>,
        _template: &JavaString,
        _template_selectors: Option<&[JavaString]>,
        resource: &dyn ITemplateResource,
        _template_mode: TemplateMode,
    ) -> Result<Option<Arc<dyn ITemplateResource>>, TemplateResourceError> {
        let prefix = read_lock(&self.prefix);
        let suffix = read_lock(&self.suffix);
        let base_name = resource.get_base_name();

        // Java 字符串连接会把 null baseName 格式化为 "null"，但当 prefix/suffix
        // 也都为 null 时，原始 null 会直接传给 resource.relative。
        let relative_location = match (prefix.as_ref(), base_name, suffix.as_ref()) {
            (None, None, None) => None,
            (prefix, base_name, suffix) => {
                let mut value = String::new();
                if let Some(prefix) = prefix {
                    value.push_str(&prefix.to_string_lossy());
                }
                value.push_str(base_name.as_deref().unwrap_or("null"));
                if let Some(suffix) = suffix {
                    value.push_str(&suffix.to_string_lossy());
                }
                Some(value)
            }
        };
        resource
            .relative(relative_location.as_deref())
            .map(|resource| Some(Arc::from(resource)))
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
