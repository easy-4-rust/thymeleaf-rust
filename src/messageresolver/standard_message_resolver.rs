use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::context::ITemplateContext;
use crate::expression::TemplateValue;
use crate::util::{JavaLocale, JavaString};

use super::{
    IMessageResolver, MessageResolutionError, MessageResolutionResult,
    StandardMessageResolutionUtils,
};

type Messages = HashMap<JavaString, JavaString>;
type LocalizedMessages = HashMap<JavaLocale, Arc<Messages>>;

/// 按模板栈、origin 和默认值依次解析外部化消息。
///
/// 对应 Java: `org.thymeleaf.messageresolver.StandardMessageResolver`。
pub struct StandardMessageResolver {
    name: Option<JavaString>,
    order: Option<i32>,
    messages_by_locale_by_template: RwLock<HashMap<JavaString, LocalizedMessages>>,
    messages_by_locale_by_origin: RwLock<HashMap<TypeId, LocalizedMessages>>,
    default_messages: RwLock<Messages>,
}

impl StandardMessageResolver {
    /// 创建名称为 Java 具体类名、顺序为空且无默认消息的解析器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: Some(JavaString::from_rust_str(
                "org.thymeleaf.messageresolver.StandardMessageResolver",
            )),
            order: None,
            messages_by_locale_by_template: RwLock::new(HashMap::new()),
            messages_by_locale_by_origin: RwLock::new(HashMap::new()),
            default_messages: RwLock::new(HashMap::new()),
        }
    }

    /// 设置可空解析器名称。
    pub fn set_name(&mut self, name: Option<JavaString>) {
        self.name = name;
    }

    /// 设置可空解析器执行顺序。
    pub fn set_order(&mut self, order: Option<i32>) {
        self.order = order;
    }

    /// 返回默认消息快照。
    #[must_use]
    pub fn get_default_messages(&self) -> Messages {
        read_lock(&self.default_messages).clone()
    }

    /// 把给定消息合并进默认消息，保留未被覆盖的旧条目。
    pub fn set_default_messages(&self, default_messages: Option<&Messages>) {
        if let Some(default_messages) = default_messages {
            write_lock(&self.default_messages).extend(default_messages.clone());
        }
    }

    /// 增加或覆盖一个默认消息。
    pub fn add_default_message(&self, key: JavaString, value: JavaString) {
        write_lock(&self.default_messages).insert(key, value);
    }

    /// 清空所有默认消息。
    pub fn clear_default_messages(&self) {
        write_lock(&self.default_messages).clear();
    }

    /// 为 Rust origin 类型登记由宿主加载的 classpath 等价消息。
    pub fn register_origin_messages(
        origin: TypeId,
        locale: JavaLocale,
        messages: HashMap<JavaString, JavaString>,
    ) {
        StandardMessageResolutionUtils::register_origin_messages(origin, locale, messages);
    }

    /// 可选择三个解析阶段并按 Java 的先后次序查找消息。
    #[expect(
        clippy::too_many_arguments,
        reason = "三个阶段开关与 Java 消息解析方法参数保持一一对应"
    )]
    pub fn resolve_message_with_phases(
        &self,
        context: &dyn ITemplateContext,
        origin: Option<TypeId>,
        key: &JavaString,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
        perform_template_based_resolution: bool,
        perform_origin_based_resolution: bool,
        perform_default_based_resolution: bool,
    ) -> MessageResolutionResult<Option<JavaString>> {
        let locale = context.get_locale();

        if perform_template_based_resolution {
            for template_data in context.get_template_stack() {
                let Some(template) = template_data.get_template().cloned() else {
                    continue;
                };
                let cacheable = template_data
                    .get_validity()
                    .is_some_and(crate::cache::ICacheEntryValidity::is_cacheable);

                let messages = if cacheable {
                    self.cached_template_messages(&template, &locale, || {
                        template_data.get_template_resource().map_or_else(
                            || Ok(Messages::new()),
                            |resource| {
                                StandardMessageResolutionUtils::resolve_messages_for_template(
                                    resource, &locale,
                                )
                                .map_err(|error| Box::new(error) as MessageResolutionError)
                            },
                        )
                    })?
                } else {
                    Arc::new(template_data.get_template_resource().map_or_else(
                        || Ok(Messages::new()),
                        |resource| {
                            StandardMessageResolutionUtils::resolve_messages_for_template(
                                resource, &locale,
                            )
                            .map_err(|error| Box::new(error) as MessageResolutionError)
                        },
                    )?)
                };

                if let Some(message) = messages.get(key) {
                    return Ok(Some(StandardMessageResolutionUtils::format_message(
                        &locale,
                        message,
                        message_parameters,
                    )));
                }
            }
        }

        if perform_origin_based_resolution && let Some(origin) = origin {
            let messages = self.cached_origin_messages(origin, &locale);
            if let Some(message) = messages.get(key) {
                return Ok(Some(StandardMessageResolutionUtils::format_message(
                    &locale,
                    message,
                    message_parameters,
                )));
            }
        }

        if perform_default_based_resolution
            && let Some(message) = read_lock(&self.default_messages).get(key)
        {
            return Ok(Some(StandardMessageResolutionUtils::format_message(
                &locale,
                message,
                message_parameters,
            )));
        }
        Ok(None)
    }

    fn cached_template_messages(
        &self,
        template: &JavaString,
        locale: &JavaLocale,
        load: impl FnOnce() -> MessageResolutionResult<Messages>,
    ) -> MessageResolutionResult<Arc<Messages>> {
        if let Some(messages) = read_lock(&self.messages_by_locale_by_template)
            .get(template)
            .and_then(|localized| localized.get(locale))
        {
            return Ok(Arc::clone(messages));
        }
        let loaded = Arc::new(load()?);
        let mut cache = write_lock(&self.messages_by_locale_by_template);
        Ok(Arc::clone(
            cache
                .entry(template.clone())
                .or_default()
                .entry(locale.clone())
                .or_insert(loaded),
        ))
    }

    fn cached_origin_messages(&self, origin: TypeId, locale: &JavaLocale) -> Arc<Messages> {
        if let Some(messages) = read_lock(&self.messages_by_locale_by_origin)
            .get(&origin)
            .and_then(|localized| localized.get(locale))
        {
            return Arc::clone(messages);
        }
        let loaded = Arc::new(StandardMessageResolutionUtils::resolve_messages_for_origin(
            origin, locale,
        ));
        let mut cache = write_lock(&self.messages_by_locale_by_origin);
        Arc::clone(
            cache
                .entry(origin)
                .or_default()
                .entry(locale.clone())
                .or_insert(loaded),
        )
    }
}

impl Default for StandardMessageResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl IMessageResolver for StandardMessageResolver {
    fn get_name(&self) -> Option<&JavaString> {
        self.name.as_ref()
    }

    fn get_order(&self) -> Option<i32> {
        self.order
    }

    fn resolve_message(
        &self,
        context: &dyn ITemplateContext,
        origin: Option<TypeId>,
        key: &JavaString,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>> {
        self.resolve_message_with_phases(context, origin, key, message_parameters, true, true, true)
    }

    fn create_absent_message_representation(
        &self,
        context: &dyn ITemplateContext,
        _origin: Option<TypeId>,
        key: &JavaString,
        _message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<JavaString>> {
        Ok(Some(JavaString::from_rust_str(&format!(
            "??{}_{}??",
            key.to_string_lossy(),
            context.get_locale()
        ))))
    }
}

fn read_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
