use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use indexmap::IndexMap;

use crate::expression::TemplateValue;
use crate::util::{JavaLocale, JavaString, ValidateError};

use super::{ContextVariableEntries, IContext, IContextVariableNames};

type Variables = IndexMap<Option<JavaString>, Arc<TemplateValue>>;

/// 非 Web Context 的共享可变基础实现。
///
/// 对应 Java: `org.thymeleaf.context.AbstractContext`。
///
/// 变量使用插入有序 Map，构造时复制输入；变量名视图实时连接到同一 Map。Locale
/// 为 null 时读取构造瞬间的进程默认 Locale，而不是每次访问动态查询。
///
/// Thymeleaf 1.0 起曾存在同名类，3.0 对其进行了完整重写。
pub struct AbstractContext {
    variables: Arc<RwLock<Variables>>,
    variable_names: Arc<VariableNamesView>,
    locale: RwLock<JavaLocale>,
}

impl AbstractContext {
    /// 创建基础 Context 状态。
    ///
    /// 对应 Java:
    /// `AbstractContext#AbstractContext(Locale, Map<String,Object>)`，同时承接另外两个
    /// protected 构造器的默认参数委托。
    ///
    /// # 参数
    ///
    /// - `locale`：可空 Locale；为空时读取当前进程默认值。
    /// - `variables`：可空变量 Map；非空时执行浅复制并保留迭代顺序、null 键和值。
    ///
    /// # 返回值
    ///
    /// 返回与输入 Map 独立、但保留各变量值共享身份的基础状态。
    pub(super) fn new(locale: Option<JavaLocale>, variables: ContextVariableEntries<'_>) -> Self {
        let variables = variables.map_or_else(
            || IndexMap::with_capacity(10),
            |entries| {
                entries
                    .iter()
                    .map(|(name, value)| {
                        (
                            name.clone(),
                            value
                                .clone()
                                .unwrap_or_else(|| Arc::new(TemplateValue::Null)),
                        )
                    })
                    .collect()
            },
        );
        let variables = Arc::new(RwLock::new(variables));
        let variable_names = Arc::new(VariableNamesView {
            variables: Arc::clone(&variables),
        });
        Self {
            variables,
            variable_names,
            locale: RwLock::new(locale.unwrap_or_else(JavaLocale::get_default)),
        }
    }

    /// 修改模板处理 Locale。
    ///
    /// 对应 Java: `AbstractContext#setLocale(Locale)`。
    ///
    /// # 参数
    ///
    /// - `locale`：新的非空 Locale。
    ///
    /// # 错误
    ///
    /// `locale=None` 时返回 Java `IllegalArgumentException` 对应校验错误。
    pub fn set_locale(&self, locale: Option<JavaLocale>) -> Result<(), ValidateError> {
        let locale = locale.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Locale cannot be null".to_owned()),
        })?;
        *write_recovering_poison(&self.locale) = locale;
        Ok(())
    }

    /// 新增或替换单个可空名称变量。
    ///
    /// 对应 Java: `AbstractContext#setVariable(String,Object)`。
    ///
    /// # 参数
    ///
    /// - `name`：可空变量名。
    /// - `value`：可空值；为空时保存显式 Java `null`。
    pub fn set_variable(&self, name: Option<JavaString>, value: Option<Arc<TemplateValue>>) {
        write_recovering_poison(&self.variables)
            .insert(name, value.unwrap_or_else(|| Arc::new(TemplateValue::Null)));
    }

    /// 按迭代顺序批量新增或替换变量；null Map 等价输入不执行任何操作。
    ///
    /// 对应 Java: `AbstractContext#setVariables(Map)`。
    ///
    /// # 参数
    ///
    /// - `variables`：可空变量条目；重复名称按后出现条目覆盖，原有位置遵循
    ///   `LinkedHashMap#putAll`。
    pub fn set_variables(&self, variables: ContextVariableEntries<'_>) {
        let Some(variables) = variables else {
            return;
        };
        let mut target = write_recovering_poison(&self.variables);
        for (name, value) in variables {
            target.insert(
                name.clone(),
                value
                    .clone()
                    .unwrap_or_else(|| Arc::new(TemplateValue::Null)),
            );
        }
    }

    /// 删除指定可空名称变量。
    ///
    /// 对应 Java: `AbstractContext#removeVariable(String)`。
    ///
    /// # 参数
    ///
    /// - `name`：待删除的可空变量名；不存在时无副作用。
    pub fn remove_variable(&self, name: Option<&JavaString>) {
        write_recovering_poison(&self.variables).shift_remove(&owned_key(name));
    }

    /// 删除全部变量。
    ///
    /// 对应 Java: `AbstractContext#clearVariables()`。已取得的实时名称视图立即变空。
    pub fn clear_variables(&self) {
        write_recovering_poison(&self.variables).clear();
    }
}

impl IContext for AbstractContext {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_locale(&self) -> JavaLocale {
        read_recovering_poison(&self.locale).clone()
    }

    fn contains_variable(&self, name: Option<&JavaString>) -> bool {
        read_recovering_poison(&self.variables).contains_key(&owned_key(name))
    }

    fn get_variable_names(&self) -> Arc<dyn IContextVariableNames + '_> {
        self.variable_names.clone()
    }

    fn get_variable(&self, name: Option<&JavaString>) -> Option<Arc<TemplateValue>> {
        read_recovering_poison(&self.variables)
            .get(&owned_key(name))
            .cloned()
    }
}

struct VariableNamesView {
    variables: Arc<RwLock<Variables>>,
}

impl IContextVariableNames for VariableNamesView {
    fn len(&self) -> usize {
        read_recovering_poison(&self.variables).len()
    }

    fn contains(&self, name: Option<&JavaString>) -> bool {
        read_recovering_poison(&self.variables).contains_key(&owned_key(name))
    }

    fn snapshot(&self) -> Vec<Option<JavaString>> {
        read_recovering_poison(&self.variables)
            .keys()
            .cloned()
            .collect()
    }

    fn remove(&self, name: Option<&JavaString>) -> bool {
        write_recovering_poison(&self.variables)
            .shift_remove(&owned_key(name))
            .is_some()
    }
}

fn read_recovering_poison<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_recovering_poison<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn owned_key(name: Option<&JavaString>) -> Option<JavaString> {
    name.cloned()
}
