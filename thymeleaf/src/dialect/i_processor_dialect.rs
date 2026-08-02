use crate::processor::ProcessorSet;

use super::IDialect;

/// 为模板引擎提供 Processor 的方言基础契约。
///
/// 对应 Java: `org.thymeleaf.dialect.IProcessorDialect`。
///
/// 此类方言可以声明默认前缀，但用户在把方言加入模板引擎时仍可覆盖它。前缀为
/// `None` 时，Processor 作用于没有命名空间的属性和元素。方言级 Processor
/// precedence 用于跨方言排序：它可以让某一方言的所有 Processor 在另一方言的
/// 所有 Processor 之前或之后执行，而不受各 Processor 自身 precedence 影响。
///
/// Java 接口本身允许默认前缀、调用参数、返回集合及集合元素为 `null`；配置阶段
/// 才拒绝后两种非法结果。因此本 trait 不提前收窄这些可观察边界。
///
/// 自 Thymeleaf 3.0.0 起提供。
pub trait IProcessorDialect: IDialect {
    /// 返回此方言建议的默认前缀。
    ///
    /// 对应 Java: `IProcessorDialect#getPrefix()`。
    ///
    /// 用户注册方言时可以覆盖此值。
    ///
    /// # 返回
    ///
    /// 默认前缀；`None` 对应 Java `null`，表示 Processor 作用于无命名空间名称。
    fn get_prefix(&self) -> Option<&str>;

    /// 返回跨方言 Processor 排序使用的方言级 precedence。
    ///
    /// 对应 Java: `IProcessorDialect#getDialectProcessorPrecedence()`。
    ///
    /// # 返回
    ///
    /// 完整 Java `int` 取值范围内的方言级优先级。
    fn get_dialect_processor_precedence(&self) -> i32;

    /// 为实际生效的方言前缀创建 Processor 集合。
    ///
    /// 对应 Java: `IProcessorDialect#getProcessors(String)`。
    ///
    /// # 参数
    ///
    /// - `dialect_prefix`：Java 参数 `dialectPrefix`；可能是默认前缀、用户覆盖值或
    ///   `None`（Java `null`），实现必须按收到的原值创建 Processor。
    ///
    /// # 返回
    ///
    /// Processor 集合；`None` 精确保留任意 Java 实现返回 `null` 的接口边界。
    /// 集合本身可以包含 `None` 元素，后续配置聚合阶段负责生成对应配置错误。
    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet>;
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use super::IProcessorDialect;
    use crate::dialect::IDialect;
    use crate::processor::{IProcessor, ProcessorSet};
    use crate::templatemode::TemplateMode;

    struct ProbeProcessor {
        template_mode: Option<TemplateMode>,
        precedence: i32,
    }

    impl IProcessor for ProbeProcessor {
        fn get_template_mode(&self) -> Option<TemplateMode> {
            self.template_mode
        }

        fn get_precedence(&self) -> i32 {
            self.precedence
        }
    }

    struct MutableProcessorDialect {
        name: Option<String>,
        prefix: Option<String>,
        precedence: AtomicI32,
        calls: AtomicUsize,
        last_prefix: Mutex<Option<Option<String>>>,
    }

    impl IDialect for MutableProcessorDialect {
        fn get_name(&self) -> Option<&str> {
            self.name.as_deref()
        }
    }

    impl IProcessorDialect for MutableProcessorDialect {
        fn get_prefix(&self) -> Option<&str> {
            self.prefix.as_deref()
        }

        fn get_dialect_processor_precedence(&self) -> i32 {
            self.precedence.load(Ordering::SeqCst)
        }

        fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            *self.last_prefix.lock().expect("last prefix lock") =
                Some(dialect_prefix.map(str::to_owned));

            if dialect_prefix == Some("return-null") {
                return None;
            }

            let mut processors = ProcessorSet::new();
            processors.insert(None);
            processors.insert(Some(Arc::new(ProbeProcessor {
                template_mode: Some(TemplateMode::HTML),
                precedence: i32::MIN,
            })));
            processors.insert(Some(Arc::new(ProbeProcessor {
                template_mode: None,
                precedence: i32::MAX,
            })));
            Some(processors)
        }
    }

    #[test]
    fn preserves_nullable_prefixes_sets_elements_boundaries_and_dynamic_dispatch() {
        let dialect = MutableProcessorDialect {
            name: Some("probe".to_owned()),
            prefix: None,
            precedence: AtomicI32::new(i32::MIN),
            calls: AtomicUsize::new(0),
            last_prefix: Mutex::new(None),
        };
        let contract: &dyn IProcessorDialect = &dialect;

        assert_eq!(contract.get_name(), Some("probe"));
        assert_eq!(contract.get_prefix(), None);
        assert_eq!(contract.get_dialect_processor_precedence(), i32::MIN);

        let processors = contract
            .get_processors(None)
            .expect("null prefix still returns a set");
        let observed = processors
            .iter()
            .map(|processor| {
                processor.map(|value| (value.get_template_mode(), value.get_precedence()))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            observed,
            [
                None,
                Some((Some(TemplateMode::HTML), i32::MIN)),
                Some((None, i32::MAX)),
            ]
        );
        assert_eq!(
            *dialect.last_prefix.lock().expect("last prefix lock"),
            Some(None)
        );

        dialect.precedence.store(i32::MAX, Ordering::SeqCst);
        assert_eq!(contract.get_dialect_processor_precedence(), i32::MAX);
        assert!(contract.get_processors(Some("return-null")).is_none());
        assert_eq!(
            *dialect.last_prefix.lock().expect("last prefix lock"),
            Some(Some("return-null".to_owned()))
        );
        assert_eq!(dialect.calls.load(Ordering::SeqCst), 2);
    }
}
