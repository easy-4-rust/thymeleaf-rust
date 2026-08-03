use std::sync::{Arc, Weak};

use indexmap::IndexSet;

use crate::context::IExpressionContext;
use crate::messageresolver::{MessageResolutionError, MessageResolutionResult};
use crate::util::{JavaString, ValidateError};

use super::TemplateValue;

/// 从模板上下文解析国际化消息的表达式工具对象。
///
/// 对应 Java: `org.thymeleaf.expression.Messages`。
pub struct Messages {
    /// Context 的弱引用避免被 ExpressionObjects 缓存后形成 Arc 引用环。
    context: Weak<dyn IExpressionContext>,
}

impl Messages {
    /// 创建绑定模板上下文的消息工具对象。
    /// 对应 Java 语义：`Messages` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(context: Option<Arc<dyn IExpressionContext>>) -> Result<Self, ValidateError> {
        let context = context.ok_or_else(|| ValidateError::IllegalArgument {
            message: Some("Context cannot be null".to_owned()),
        })?;
        if context.as_template_context().is_none() {
            return Err(ValidateError::IllegalArgument {
                message: Some("Context must implement ITemplateContext".to_owned()),
            });
        }
        Ok(Self {
            context: Arc::downgrade(&context),
        })
    }

    /// 解析消息；缺失时使用 absent-message 表示。
    /// 对应 Java: `Messages#msg()`。
    pub fn msg(&self, message_key: &JavaString) -> MessageResolutionResult<Option<JavaString>> {
        self.msg_with_params(message_key, &[])
    }

    /// 使用参数解析消息；缺失时使用 absent-message 表示。
    /// 对应 Java: `Messages#msgWithParams()`。
    pub fn msg_with_params(
        &self,
        message_key: &JavaString,
        message_parameters: &[Option<Arc<TemplateValue>>],
    ) -> MessageResolutionResult<Option<JavaString>> {
        self.get_message(message_key, message_parameters, true)
    }

    /// 解析消息；缺失时返回 Java null。
    /// 对应 Java: `Messages#msgOrNull()`。
    pub fn msg_or_null(
        &self,
        message_key: &JavaString,
    ) -> MessageResolutionResult<Option<JavaString>> {
        self.msg_or_null_with_params(message_key, &[])
    }

    /// 使用参数解析消息；缺失时返回 Java null。
    /// 对应 Java: `Messages#msgOrNullWithParams()`。
    pub fn msg_or_null_with_params(
        &self,
        message_key: &JavaString,
        message_parameters: &[Option<Arc<TemplateValue>>],
    ) -> MessageResolutionResult<Option<JavaString>> {
        self.get_message(message_key, message_parameters, false)
    }

    /// 批量解析数组消息。
    /// 对应 Java: `Messages#arrayMsg()`。
    pub fn array_msg(
        &self,
        message_keys: Option<&[JavaString]>,
    ) -> MessageResolutionResult<Vec<Option<JavaString>>> {
        self.array_msg_with_params(message_keys, &[])
    }

    /// 使用共同参数批量解析数组消息。
    /// 对应 Java: `Messages#arrayMsgWithParams()`。
    pub fn array_msg_with_params(
        &self,
        message_keys: Option<&[JavaString]>,
        message_parameters: &[Option<Arc<TemplateValue>>],
    ) -> MessageResolutionResult<Vec<Option<JavaString>>> {
        self.map_messages(message_keys, message_parameters, true)
    }

    /// 批量解析数组消息，缺失项保留 Java null。
    /// 对应 Java: `Messages#arrayMsgOrNull()`。
    pub fn array_msg_or_null(
        &self,
        message_keys: Option<&[JavaString]>,
    ) -> MessageResolutionResult<Vec<Option<JavaString>>> {
        self.array_msg_or_null_with_params(message_keys, &[])
    }

    /// 使用共同参数批量解析数组消息，缺失项保留 Java null。
    /// 对应 Java: `Messages#arrayMsgOrNullWithParams()`。
    pub fn array_msg_or_null_with_params(
        &self,
        message_keys: Option<&[JavaString]>,
        message_parameters: &[Option<Arc<TemplateValue>>],
    ) -> MessageResolutionResult<Vec<Option<JavaString>>> {
        self.map_messages(message_keys, message_parameters, false)
    }

    /// 按 List 顺序批量解析消息。
    /// 对应 Java: `Messages#listMsg()`。
    pub fn list_msg(
        &self,
        message_keys: Option<&[JavaString]>,
    ) -> MessageResolutionResult<Vec<Option<JavaString>>> {
        self.array_msg(message_keys)
    }

    /// 按 List 顺序并使用共同参数批量解析消息。
    /// 对应 Java: `Messages#listMsgWithParams()`。
    pub fn list_msg_with_params(
        &self,
        message_keys: Option<&[JavaString]>,
        message_parameters: &[Option<Arc<TemplateValue>>],
    ) -> MessageResolutionResult<Vec<Option<JavaString>>> {
        self.array_msg_with_params(message_keys, message_parameters)
    }

    /// 按 List 顺序批量解析消息，缺失项保留 Java null。
    /// 对应 Java: `Messages#listMsgOrNull()`。
    pub fn list_msg_or_null(
        &self,
        message_keys: Option<&[JavaString]>,
    ) -> MessageResolutionResult<Vec<Option<JavaString>>> {
        self.array_msg_or_null(message_keys)
    }

    /// 按 List 顺序使用共同参数解析，缺失项保留 Java null。
    /// 对应 Java: `Messages#listMsgOrNullWithParams()`。
    pub fn list_msg_or_null_with_params(
        &self,
        message_keys: Option<&[JavaString]>,
        message_parameters: &[Option<Arc<TemplateValue>>],
    ) -> MessageResolutionResult<Vec<Option<JavaString>>> {
        self.array_msg_or_null_with_params(message_keys, message_parameters)
    }

    /// 按 Set 迭代顺序解析并去重结果。
    /// 对应 Java: `Messages#setMsg()`。
    pub fn set_msg(
        &self,
        message_keys: Option<&IndexSet<JavaString>>,
    ) -> MessageResolutionResult<IndexSet<Option<JavaString>>> {
        self.set_msg_with_params(message_keys, &[])
    }

    /// 按 Set 迭代顺序使用共同参数解析并去重结果。
    /// 对应 Java: `Messages#setMsgWithParams()`。
    pub fn set_msg_with_params(
        &self,
        message_keys: Option<&IndexSet<JavaString>>,
        message_parameters: &[Option<Arc<TemplateValue>>],
    ) -> MessageResolutionResult<IndexSet<Option<JavaString>>> {
        self.map_set(message_keys, message_parameters, true)
    }

    /// 按 Set 迭代顺序解析，缺失消息保留 Java null。
    /// 对应 Java: `Messages#setMsgOrNull()`。
    pub fn set_msg_or_null(
        &self,
        message_keys: Option<&IndexSet<JavaString>>,
    ) -> MessageResolutionResult<IndexSet<Option<JavaString>>> {
        self.set_msg_or_null_with_params(message_keys, &[])
    }

    /// 按 Set 迭代顺序使用共同参数解析，缺失消息保留 Java null。
    /// 对应 Java: `Messages#setMsgOrNullWithParams()`。
    pub fn set_msg_or_null_with_params(
        &self,
        message_keys: Option<&IndexSet<JavaString>>,
        message_parameters: &[Option<Arc<TemplateValue>>],
    ) -> MessageResolutionResult<IndexSet<Option<JavaString>>> {
        self.map_set(message_keys, message_parameters, false)
    }

    fn map_messages(
        &self,
        message_keys: Option<&[JavaString]>,
        message_parameters: &[Option<Arc<TemplateValue>>],
        use_absent_message_representation: bool,
    ) -> MessageResolutionResult<Vec<Option<JavaString>>> {
        let message_keys = message_keys
            .ok_or_else(|| ValidateError::IllegalArgument {
                message: Some("Message keys cannot be null".to_owned()),
            })
            .map_err(|error| Box::new(error) as MessageResolutionError)?;
        message_keys
            .iter()
            .map(|key| self.get_message(key, message_parameters, use_absent_message_representation))
            .collect()
    }

    fn map_set(
        &self,
        message_keys: Option<&IndexSet<JavaString>>,
        message_parameters: &[Option<Arc<TemplateValue>>],
        use_absent_message_representation: bool,
    ) -> MessageResolutionResult<IndexSet<Option<JavaString>>> {
        let message_keys = message_keys
            .ok_or_else(|| ValidateError::IllegalArgument {
                message: Some("Message keys cannot be null".to_owned()),
            })
            .map_err(|error| Box::new(error) as MessageResolutionError)?;
        message_keys
            .iter()
            .map(|key| self.get_message(key, message_parameters, use_absent_message_representation))
            .collect()
    }

    fn get_message(
        &self,
        message_key: &JavaString,
        message_parameters: &[Option<Arc<TemplateValue>>],
        use_absent_message_representation: bool,
    ) -> MessageResolutionResult<Option<JavaString>> {
        let context = self.context.upgrade().ok_or_else(|| {
            Box::new(ValidateError::IllegalArgument {
                message: Some("Expression context is no longer available".to_owned()),
            }) as MessageResolutionError
        })?;
        context
            .as_template_context()
            .ok_or_else(|| {
                Box::new(ValidateError::IllegalArgument {
                    message: Some("Context must implement ITemplateContext".to_owned()),
                }) as MessageResolutionError
            })?
            .get_message(
                None,
                message_key,
                Some(message_parameters),
                use_absent_message_representation,
            )
    }
}
