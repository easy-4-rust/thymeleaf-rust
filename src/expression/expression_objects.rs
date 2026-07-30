use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};

use indexmap::{IndexMap, IndexSet};
use thiserror::Error;

use crate::context::IExpressionContext;
use crate::util::JavaString;

use super::{
    IExpressionObjectFactory, IExpressionObjects, StandardExpressionResult, TemplateValue,
};

/// 表达式工具对象容器。
///
/// 对应 Java: `org.thymeleaf.expression.ExpressionObjects`。
///
/// 容器保存工厂在构造时声明的完整名称集合。对象仅在第一次读取时创建；可缓存对象
/// 连同 Java `null` 结果一起缓存，非缓存对象每次读取都重新调用工厂。
pub struct ExpressionObjects {
    context: Weak<dyn IExpressionContext>,
    expression_object_factory: Arc<dyn IExpressionObjectFactory>,
    expression_object_names: IndexSet<Option<JavaString>>,
    objects: RwLock<IndexMap<Option<JavaString>, Option<Arc<TemplateValue>>>>,
}

/// 创建或使用表达式对象容器时可能出现的结构错误。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ExpressionObjectsError {
    /// Java `Validate.notNull` 对应错误。
    #[error("{message}")]
    IllegalArgument {
        /// Java 异常消息。
        message: &'static str,
    },
}

impl ExpressionObjects {
    /// 创建表达式对象容器。
    ///
    /// 对应 Java: `ExpressionObjects#ExpressionObjects(IExpressionContext,
    /// IExpressionObjectFactory)`。
    ///
    /// # 参数
    /// - `context`：当前表达式上下文的弱引用，避免上下文和懒加载容器形成强引用环。
    /// - `expression_object_factory`：声明并创建表达式对象的工厂。
    ///
    /// # 错误
    /// 任一参数为 `None` 时返回与 Java 参数校验同类的错误。
    pub fn new(
        context: Option<Weak<dyn IExpressionContext>>,
        expression_object_factory: Option<Arc<dyn IExpressionObjectFactory>>,
    ) -> Result<Self, ExpressionObjectsError> {
        let context = context.ok_or(ExpressionObjectsError::IllegalArgument {
            message: "Context cannot be null",
        })?;
        let expression_object_factory =
            expression_object_factory.ok_or(ExpressionObjectsError::IllegalArgument {
                message: "Expression Object Factory cannot be null",
            })?;
        let expression_object_names = expression_object_factory
            .get_all_expression_object_names()
            .into_iter()
            .collect();

        Ok(Self {
            context,
            expression_object_factory,
            expression_object_names,
            objects: RwLock::new(IndexMap::with_capacity(3)),
        })
    }
}

impl IExpressionObjects for ExpressionObjects {
    fn size(&self) -> i32 {
        i32::try_from(self.expression_object_names.len()).unwrap_or(i32::MAX)
    }

    fn contains_object(&self, name: Option<&JavaString>) -> bool {
        self.expression_object_names.contains(&name.cloned())
    }

    fn get_object_names(&self) -> Vec<Option<JavaString>> {
        self.expression_object_names.iter().cloned().collect()
    }

    fn get_object(
        &self,
        name: Option<&JavaString>,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let key = name.cloned();

        // HashMap#containsKey 与 get 分开，确保缓存的 Java null 不会被误判成未缓存。
        if let Some(object) = read_recovering_poison(&self.objects).get(&key) {
            return Ok(object.clone());
        }
        if !self.expression_object_names.contains(&key) {
            return Ok(None);
        }

        let Some(context) = self.context.upgrade() else {
            return Ok(None);
        };
        let object = self.expression_object_factory.build_object(context, name)?;
        if !self.expression_object_factory.is_cacheable(name) {
            return Ok(object);
        }

        write_recovering_poison(&self.objects).insert(key, object.clone());
        Ok(object)
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
