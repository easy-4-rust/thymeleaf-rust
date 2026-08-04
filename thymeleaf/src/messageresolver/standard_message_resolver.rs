use std::any::TypeId;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};

use crate::context::ITemplateContext;
use crate::expression::TemplateValue;
use crate::templateresource::ITemplateResource;
use crate::util::{Locale, Utf16String};

use super::{
    AbstractMessageResolver, IMessageResolver, MessageResolutionError, MessageResolutionResult,
    StandardMessageResolutionUtils,
};

type Messages = HashMap<Utf16String, Utf16String>;
type LocalizedMessages = HashMap<Locale, Arc<Messages>>;
type TemplateMessagesHook = dyn Fn(&Utf16String, &dyn ITemplateResource, &Locale) -> MessageResolutionResult<Messages>
    + Send
    + Sync;
type OriginMessagesHook = dyn Fn(TypeId, &Locale) -> Messages + Send + Sync;
type MessageFormatterHook = dyn Fn(
        &Locale,
        &Utf16String,
        Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<Utf16String>>
    + Send
    + Sync;
type AbsentMessageHook = dyn Fn(
        Option<&dyn ITemplateContext>,
        Option<TypeId>,
        Option<&Utf16String>,
        Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<Utf16String>>
    + Send
    + Sync;

/// 标准外部化消息解析器，依次执行模板、origin 和默认消息三个阶段。
///
/// 对应 Java: `org.thymeleaf.messageresolver.StandardMessageResolver`。
///
/// 模板阶段按当前模板栈顺序检查与模板资源并列的 `.properties` 文件，并按
/// language、country、variant 从通用到具体合并；可缓存模板以“模板名 + Locale”为键
/// 缓存结果。origin 阶段读取触发对象及其父类型的同名资源，并以“具体类型覆盖父类型”
/// 的顺序合并，以“TypeId + Locale”为键缓存。最后才查找调用者配置的默认消息。
///
/// Java 通过 protected 方法允许子类改写模板加载、origin 加载和格式化过程。Rust
/// 不模拟继承，而是用 `with_*_hook` 组合钩子保持这些动态扩展点；解析主链会真实调用
/// 钩子，而不是只暴露同名辅助方法。内部锁只保护缓存和默认消息，使解析器可以跨线程共享。
pub struct StandardMessageResolver {
    base: AbstractMessageResolver,
    messages_by_locale_by_template: RwLock<HashMap<Utf16String, LocalizedMessages>>,
    messages_by_locale_by_origin: RwLock<HashMap<TypeId, LocalizedMessages>>,
    default_messages: RwLock<Messages>,
    template_messages_hook: Option<Arc<TemplateMessagesHook>>,
    origin_messages_hook: Option<Arc<OriginMessagesHook>>,
    message_formatter_hook: Option<Arc<MessageFormatterHook>>,
    absent_message_hook: Option<Arc<AbsentMessageHook>>,
}

impl StandardMessageResolver {
    /// 创建名称为 Java 具体类名、顺序为空且无默认消息的解析器。
    ///
    /// # 返回值
    ///
    /// 未配置扩展钩子、缓存为空的标准消息解析器。
    #[must_use]
    /// 对应 Java 语义：`StandardMessageResolver` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new() -> Self {
        Self {
            base: AbstractMessageResolver::new(
                "org.thymeleaf.messageresolver.StandardMessageResolver",
            ),
            messages_by_locale_by_template: RwLock::new(HashMap::new()),
            messages_by_locale_by_origin: RwLock::new(HashMap::new()),
            default_messages: RwLock::new(HashMap::new()),
            template_messages_hook: None,
            origin_messages_hook: None,
            message_formatter_hook: None,
            absent_message_hook: None,
        }
    }

    /// 组合替换 Java protected `resolveMessagesForTemplate` 扩展点。
    ///
    /// 回调在模板消息尚未缓存时执行；可缓存模板仍按模板名和 Locale 缓存回调结果。
    ///
    /// # 参数
    ///
    /// - `hook`：接收模板名、模板资源和 Locale，返回该模板的全部消息。
    ///
    /// # 返回值
    ///
    /// 安装钩子后的解析器，便于继续链式配置。
    #[must_use]
    /// 对应 Java 语义：`StandardMessageResolver` 的 `with_template_messages_hook` 行为（Rust 侧辅助/私有路径）。
    pub fn with_template_messages_hook<F>(mut self, hook: F) -> Self
    where
        F: Fn(&Utf16String, &dyn ITemplateResource, &Locale) -> MessageResolutionResult<Messages>
            + Send
            + Sync
            + 'static,
    {
        self.template_messages_hook = Some(Arc::new(hook));
        self
    }

    /// 组合替换 Java protected `resolveMessagesForOrigin` 扩展点。
    ///
    /// 回调结果与 Java 实现一样按 origin 和 Locale 永久缓存于解析器实例。
    ///
    /// # 参数
    ///
    /// - `hook`：接收 origin 和 Locale，返回该来源的全部消息。
    ///
    /// # 返回值
    ///
    /// 安装钩子后的解析器。
    #[must_use]
    /// 对应 Java 语义：`StandardMessageResolver` 的 `with_origin_messages_hook` 行为（Rust 侧辅助/私有路径）。
    pub fn with_origin_messages_hook<F>(mut self, hook: F) -> Self
    where
        F: Fn(TypeId, &Locale) -> Messages + Send + Sync + 'static,
    {
        self.origin_messages_hook = Some(Arc::new(hook));
        self
    }

    /// 组合替换 Java protected `formatMessage` 扩展点。
    ///
    /// 返回 `None` 对应 Java 子类返回 `null`，会使当前解析阶段返回未解析结果。
    ///
    /// # 参数
    ///
    /// - `hook`：接收 Locale、消息 pattern 与可空参数数组的格式化回调。
    ///
    /// # 返回值
    ///
    /// 安装钩子后的解析器。
    #[must_use]
    pub fn with_message_formatter_hook<F>(mut self, hook: F) -> Self
    where
        F: Fn(
                &Locale,
                &Utf16String,
                Option<&[Option<Arc<TemplateValue>>]>,
            ) -> MessageResolutionResult<Option<Utf16String>>
            + Send
            + Sync
            + 'static,
    {
        self.message_formatter_hook = Some(Arc::new(hook));
        self
    }

    /// 组合替换 Java 可覆写的 absent-message 表示扩展点。
    ///
    /// # 参数
    ///
    /// - `hook`：在解析器链全部未命中后创建缺失消息表示的回调。
    ///
    /// # 返回值
    ///
    /// 安装钩子后的解析器。
    #[must_use]
    /// 对应 Java 语义：`StandardMessageResolver` 的 `with_absent_message_hook` 行为（Rust 侧辅助/私有路径）。
    pub fn with_absent_message_hook<F>(mut self, hook: F) -> Self
    where
        F: Fn(
                Option<&dyn ITemplateContext>,
                Option<TypeId>,
                Option<&Utf16String>,
                Option<&[Option<Arc<TemplateValue>>]>,
            ) -> MessageResolutionResult<Option<Utf16String>>
            + Send
            + Sync
            + 'static,
    {
        self.absent_message_hook = Some(Arc::new(hook));
        self
    }

    /// 设置可空解析器名称。
    /// 对应 Java 语义：Java 接口/超类方法 `setName()` 的 Rust 移植（`StandardMessageResolver` 继承路径）。
    pub fn set_name(&mut self, name: Option<Utf16String>) {
        self.base.set_name(name);
    }

    /// 设置可空解析器执行顺序。
    /// 对应 Java 语义：Java 接口/超类方法 `setOrder()` 的 Rust 移植（`StandardMessageResolver` 继承路径）。
    pub fn set_order(&mut self, order: Option<i32>) {
        self.base.set_order(order);
    }

    /// 返回默认消息的同一可变容器。
    ///
    /// 对应 Java: `StandardMessageResolver#getDefaultMessages()`。Java 每次返回同一个
    /// `Properties` 对象并允许调用者直接修改；Rust 返回同一 `RwLock`，调用者可取得
    /// read/write guard，同时保持并发安全。
    #[must_use]
    pub const fn get_default_messages(&self) -> &RwLock<HashMap<Utf16String, Utf16String>> {
        &self.default_messages
    }

    /// 把给定消息合并进默认消息，保留未被覆盖的旧条目。
    /// 对应 Java: `StandardMessageResolver#setDefaultMessages()`。
    pub fn set_default_messages(&self, default_messages: Option<&Messages>) {
        if let Some(default_messages) = default_messages {
            write_lock(&self.default_messages).extend(default_messages.clone());
        }
    }

    /// 增加或覆盖一个默认消息。
    /// 对应 Java: `StandardMessageResolver#addDefaultMessage()`。
    pub fn add_default_message(
        &self,
        key: Utf16String,
        value: Utf16String,
    ) -> MessageResolutionResult<()> {
        self.add_default_message_nullable(Some(key), Some(value))
    }

    /// 增加或覆盖可空边界传入的默认消息，并保留 Java 校验顺序和消息。
    ///
    /// 对应 Java: `StandardMessageResolver#addDefaultMessage(String, String)`。
    pub fn add_default_message_nullable(
        &self,
        key: Option<Utf16String>,
        value: Option<Utf16String>,
    ) -> MessageResolutionResult<()> {
        let key = key.ok_or_else(|| {
            Box::new(MessageResolverArgumentError(
                "Key for default message cannot be null",
            )) as MessageResolutionError
        })?;
        let value = value.ok_or_else(|| {
            Box::new(MessageResolverArgumentError(
                "Value for default message cannot be null",
            )) as MessageResolutionError
        })?;
        write_lock(&self.default_messages).insert(key, value);
        Ok(())
    }

    /// 清空所有默认消息。
    /// 对应 Java: `StandardMessageResolver#clearDefaultMessages()`。
    pub fn clear_default_messages(&self) {
        write_lock(&self.default_messages).clear();
    }

    /// 为 Rust origin 类型登记由宿主加载的 classpath 等价消息。
    /// 对应 Java 语义：`StandardMessageResolver` 的 `register_origin_messages` 行为（Rust 侧辅助/私有路径）。
    pub fn register_origin_messages(
        origin: TypeId,
        locale: Locale,
        messages: HashMap<Utf16String, Utf16String>,
    ) {
        StandardMessageResolutionUtils::register_origin_messages(origin, locale, messages);
    }

    /// 登记 origin 类型的直接父类型，以复现 Java superclass 消息回退。
    ///
    /// Rust `TypeId` 不携带继承元数据，因此宿主对象适配层必须为存在继承关系的
    /// origin 显式登记该关系。具体类型消息始终覆盖父类型消息。
    /// 对应 Java 语义：`StandardMessageResolver` 的 `register_origin_parent` 行为（Rust 侧辅助/私有路径）。
    pub fn register_origin_parent(origin: TypeId, parent: TypeId) -> MessageResolutionResult<()> {
        StandardMessageResolutionUtils::register_origin_parent(origin, parent)
    }

    /// 使用 Java `MessageFormat` 语义合并消息文本和参数。
    ///
    /// 对应 Java: `StandardMessageResolver#formatMessage(Locale,String,Object[])`。
    /// Java 通过 protected 方法向子类开放该扩展点；Rust 没有类继承，因此公开同名
    /// 能力供组合式解析器调用。
    ///
    /// # 参数
    ///
    /// - `locale`：数字、日期和选择格式使用的 Locale。
    /// - `message`：Java `MessageFormat` pattern。
    /// - `message_parameters`：可空参数数组；元素也允许为空。
    ///
    /// # 返回值
    ///
    /// 格式化文本、钩子显式返回的 `None`，或非法 pattern 等格式化错误。
    pub fn format_message(
        &self,
        locale: &Locale,
        message: &Utf16String,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<Utf16String>> {
        if let Some(hook) = &self.message_formatter_hook {
            return hook(locale, message, message_parameters);
        }
        StandardMessageResolutionUtils::format_message(locale, message, message_parameters)
            .map(Some)
    }

    /// 解析指定模板资源和 Locale 的全部伴随 properties 消息。
    ///
    /// 对应 Java:
    /// `StandardMessageResolver#resolveMessagesForTemplate(String,ITemplateResource,Locale)`。
    /// `template` 在 Java 默认实现中不参与文件名计算，但作为子类扩展参数保留。
    ///
    /// # 返回值
    ///
    /// 按 Locale 层级合并后的消息映射，或资源读取/Properties 解析错误。
    pub fn resolve_messages_for_template(
        &self,
        template: &Utf16String,
        template_resource: &dyn ITemplateResource,
        locale: &Locale,
    ) -> MessageResolutionResult<HashMap<Utf16String, Utf16String>> {
        if let Some(hook) = &self.template_messages_hook {
            return hook(template, template_resource, locale);
        }
        StandardMessageResolutionUtils::resolve_messages_for_template(template_resource, locale)
    }

    /// 返回指定 Rust origin 及其已登记父类型的全部本地化消息。
    ///
    /// 对应 Java: `StandardMessageResolver#resolveMessagesForOrigin(Class,Locale)`。
    ///
    /// # 返回值
    ///
    /// 具体 origin 覆盖父 origin 后的消息映射。
    pub fn resolve_messages_for_origin(
        &self,
        origin: TypeId,
        locale: &Locale,
    ) -> HashMap<Utf16String, Utf16String> {
        if let Some(hook) = &self.origin_messages_hook {
            return hook(origin, locale);
        }
        StandardMessageResolutionUtils::resolve_messages_for_origin(origin, locale)
    }

    /// 可选择三个解析阶段并按 Java 的先后次序查找消息。
    ///
    /// # 返回值
    ///
    /// 首个命中的格式化消息、三个阶段均未命中的 `None`，或加载/格式化错误。
    #[expect(
        clippy::too_many_arguments,
        reason = "三个阶段开关与 Java 消息解析方法参数保持一一对应"
    )]
    /// 对应 Java 语义：`StandardMessageResolver` 的 `resolve_message_with_phases` 行为（Rust 侧辅助/私有路径）。
    pub fn resolve_message_with_phases(
        &self,
        context: &dyn ITemplateContext,
        origin: Option<TypeId>,
        key: &Utf16String,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
        perform_template_based_resolution: bool,
        perform_origin_based_resolution: bool,
        perform_default_based_resolution: bool,
    ) -> MessageResolutionResult<Option<Utf16String>> {
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
                                self.resolve_messages_for_template(&template, resource, &locale)
                            },
                        )
                    })?
                } else {
                    Arc::new(template_data.get_template_resource().map_or_else(
                        || Ok(Messages::new()),
                        |resource| self.resolve_messages_for_template(&template, resource, &locale),
                    )?)
                };

                if let Some(message) = messages.get(key) {
                    return self.format_message(&locale, message, message_parameters);
                }
            }
        }

        if perform_origin_based_resolution && let Some(origin) = origin {
            let messages = self.cached_origin_messages(origin, &locale);
            if let Some(message) = messages.get(key) {
                return self.format_message(&locale, message, message_parameters);
            }
        }

        if perform_default_based_resolution
            && let Some(message) = read_lock(&self.default_messages).get(key)
        {
            return self.format_message(&locale, message, message_parameters);
        }
        Ok(None)
    }

    fn cached_template_messages(
        &self,
        template: &Utf16String,
        locale: &Locale,
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

    fn cached_origin_messages(&self, origin: TypeId, locale: &Locale) -> Arc<Messages> {
        if let Some(messages) = read_lock(&self.messages_by_locale_by_origin)
            .get(&origin)
            .and_then(|localized| localized.get(locale))
        {
            return Arc::clone(messages);
        }
        let loaded = Arc::new(self.resolve_messages_for_origin(origin, locale));
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
    fn get_name(&self) -> Option<&Utf16String> {
        self.base.get_name()
    }

    fn get_order(&self) -> Option<i32> {
        self.base.get_order()
    }

    fn resolve_message_nullable(
        &self,
        context: Option<&dyn ITemplateContext>,
        origin: Option<TypeId>,
        key: Option<&Utf16String>,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<Utf16String>> {
        let context = context.ok_or_else(|| {
            Box::new(MessageResolverArgumentError("Context cannot be null"))
                as MessageResolutionError
        })?;
        let key = key.ok_or_else(|| {
            Box::new(MessageResolverArgumentError("Message key cannot be null"))
                as MessageResolutionError
        })?;
        self.resolve_message_with_phases(context, origin, key, message_parameters, true, true, true)
    }

    fn create_absent_message_representation_nullable(
        &self,
        context: Option<&dyn ITemplateContext>,
        origin: Option<TypeId>,
        key: Option<&Utf16String>,
        message_parameters: Option<&[Option<Arc<TemplateValue>>]>,
    ) -> MessageResolutionResult<Option<Utf16String>> {
        if let Some(hook) = &self.absent_message_hook {
            return hook(context, origin, key, message_parameters);
        }
        let key = key.ok_or_else(|| {
            Box::new(MessageResolverArgumentError("Message key cannot be null"))
                as MessageResolutionError
        })?;
        let context = context
            .ok_or_else(|| Box::new(MessageResolverNullContextError) as MessageResolutionError)?;
        Ok(Some(Utf16String::from_rust_str(&format!(
            "??{}_{}??",
            key.to_string_lossy(),
            context.get_locale()
        ))))
    }
}

#[derive(Debug)]
struct MessageResolverArgumentError(&'static str);

impl Display for MessageResolverArgumentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}

impl Error for MessageResolverArgumentError {}

#[derive(Debug)]
struct MessageResolverNullContextError;

impl Display for MessageResolverNullContextError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(
            "Cannot invoke \"org.thymeleaf.context.ITemplateContext.getLocale()\" because \
             \"context\" is null",
        )
    }
}

impl Error for MessageResolverNullContextError {}

fn read_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::sync::RwLock;

    use crate::util::Utf16String;

    use super::StandardMessageResolver;

    #[test]
    fn default_messages_are_live_merged_validated_and_cleared() {
        let resolver = StandardMessageResolver::new();
        let identity = resolver.get_default_messages() as *const RwLock<_>;
        assert_eq!(
            identity,
            resolver.get_default_messages() as *const RwLock<_>
        );

        resolver
            .add_default_message(
                Utf16String::from_rust_str("first"),
                Utf16String::from_rust_str("one"),
            )
            .expect("valid default");
        resolver.set_default_messages(Some(&std::collections::HashMap::from([
            (
                Utf16String::from_rust_str("second"),
                Utf16String::from_rust_str("two"),
            ),
            (
                Utf16String::from_rust_str("first"),
                Utf16String::from_rust_str("override"),
            ),
        ])));
        {
            let messages = resolver
                .get_default_messages()
                .read()
                .expect("default messages read");
            assert_eq!(messages.len(), 2);
            assert_eq!(
                messages.get(&Utf16String::from_rust_str("first")),
                Some(&Utf16String::from_rust_str("override"))
            );
        }

        let error = resolver
            .add_default_message_nullable(None, Some(Utf16String::from_rust_str("value")))
            .expect_err("null key");
        assert_eq!(error.to_string(), "Key for default message cannot be null");
        let error = resolver
            .add_default_message_nullable(Some(Utf16String::from_rust_str("key")), None)
            .expect_err("null value");
        assert_eq!(
            error.to_string(),
            "Value for default message cannot be null"
        );

        resolver.clear_default_messages();
        assert!(
            resolver
                .get_default_messages()
                .read()
                .expect("default messages read")
                .is_empty()
        );
    }
}
