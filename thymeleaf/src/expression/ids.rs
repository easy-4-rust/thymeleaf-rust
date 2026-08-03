use std::sync::{Arc, Weak};

use crate::context::IExpressionContext;
use crate::util::{JavaString, ValidateError};

use super::{StandardExpressionError, StandardExpressionResult, TemplateValue};

/// 在模板处理期间生成稳定递增的 HTML `id`。
///
/// 对应 Java: `org.thymeleaf.expression.Ids`。
pub struct Ids {
    /// Context 的弱引用避免被 ExpressionObjects 缓存后形成 Arc 引用环。
    context: Weak<dyn IExpressionContext>,
}

impl Ids {
    /// 创建绑定模板上下文的 ID 工具对象。
    /// 对应 Java 语义：`Ids` 的 `new` 行为（Rust 侧辅助/私有路径）。
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

    /// 返回当前序号并递增。对应 Java: `Ids#seq(Object)`。
    pub fn seq(&self, id: Option<&TemplateValue>) -> StandardExpressionResult<JavaString> {
        let id = id_to_string(id)?;
        let context = self.context()?;
        let value = context
            .as_template_context()
            .expect("constructor verifies template context")
            .get_identifier_sequences()
            .get_and_increment_id_seq(Some(&id))
            .map_err(|error| Box::new(error) as StandardExpressionError)?;
        Ok(append_number(&id, value))
    }

    /// 返回下一序号但不递增。对应 Java: `Ids#next(Object)`。
    pub fn next(&self, id: Option<&TemplateValue>) -> StandardExpressionResult<JavaString> {
        let id = id_to_string(id)?;
        let context = self.context()?;
        let value = context
            .as_template_context()
            .expect("constructor verifies template context")
            .get_identifier_sequences()
            .get_next_id_seq(Some(&id))
            .map_err(|error| Box::new(error) as StandardExpressionError)?;
        Ok(append_number(&id, value))
    }

    /// 返回最近一次已分配序号。对应 Java: `Ids#prev(Object)`。
    pub fn prev(&self, id: Option<&TemplateValue>) -> StandardExpressionResult<JavaString> {
        let id = id_to_string(id)?;
        let context = self.context()?;
        let value = context
            .as_template_context()
            .expect("constructor verifies template context")
            .get_identifier_sequences()
            .get_previous_id_seq(Some(&id))
            .map_err(|error| Box::new(error) as StandardExpressionError)?;
        Ok(append_number(&id, value))
    }

    /// 升级 Context 的弱引用；脱离模板执行后拒绝继续使用请求级工具对象。
    fn context(&self) -> StandardExpressionResult<Arc<dyn IExpressionContext>> {
        self.context.upgrade().ok_or_else(|| {
            Box::new(ValidateError::IllegalArgument {
                message: Some("Expression context is no longer available".to_owned()),
            }) as StandardExpressionError
        })
    }
}

fn id_to_string(id: Option<&TemplateValue>) -> StandardExpressionResult<JavaString> {
    id.ok_or_else(|| {
        Box::new(ValidateError::IllegalArgument {
            message: Some("ID cannot be null".to_owned()),
        }) as StandardExpressionError
    })?
    .to_java_string()
    .ok_or_else(|| Box::new(crate::expression::TokenError::NullPointer) as StandardExpressionError)
}

fn append_number(id: &JavaString, value: i32) -> JavaString {
    let mut units = id.as_utf16().to_vec();
    units.extend(value.to_string().encode_utf16());
    JavaString::from_utf16(units)
}
