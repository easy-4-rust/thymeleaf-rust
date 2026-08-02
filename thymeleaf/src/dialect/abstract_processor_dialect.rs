use super::{AbstractDialect, AbstractDialectError, IDialect};

/// 可贡献 Processor 的 Thymeleaf 方言基础状态实现。
///
/// 对应 Java: `org.thymeleaf.dialect.AbstractProcessorDialect`。
///
/// Java 抽象类在 [`AbstractDialect`] 的非空、不可变名称之外，只保存可空默认前缀
/// 和不可变的方言级 Processor precedence。Rust 不模拟类继承；具体 Processor
/// 方言组合本对象，把名称、前缀和 precedence 的 trait 调用委托给它，并自行实现
/// `IProcessorDialect#getProcessors(String)` 扩展点。
///
/// Java 的三个 getter 都不允许子类改写已构造状态。本类型因此不提供 setter，
/// 也不在基础对象中伪造默认 Processor 集合。
///
/// 自 Thymeleaf 3.0.0 起提供。
pub struct AbstractProcessorDialect {
    dialect: AbstractDialect,
    prefix: Option<String>,
    processor_precedence: i32,
}

impl AbstractProcessorDialect {
    /// 创建 Processor 方言的基础状态。
    ///
    /// 对应 Java:
    /// `AbstractProcessorDialect#AbstractProcessorDialect(String, String, int)`。
    ///
    /// Java 构造器是 `protected`；Rust 将构造入口公开，使 crate 外具体方言也能获得
    /// 与 Java 外部子类相同的扩展能力。
    ///
    /// # 参数
    ///
    /// - `name`：Java 参数 `name`；允许空字符串，但 `None` 对应 Java `null`；
    /// - `prefix`：Java 参数 `prefix`；`None` 对应 Java `null`，表示默认不使用
    ///   namespace，空字符串仍是一个不同的合法值；
    /// - `processor_precedence`：Java 参数 `processorPrecedence`，保留完整有符号
    ///   32 位取值范围。
    ///
    /// # 返回
    ///
    /// 名称、前缀和 precedence 均固定且不可从外部修改的基础状态。
    ///
    /// # 错误
    ///
    /// `name` 为 `None` 时返回
    /// [`AbstractDialectError::DialectNameCannotBeNull`]，对应父构造器首先抛出的
    /// `IllegalArgumentException("Dialect name cannot be null")`。
    pub fn new(
        name: Option<&str>,
        prefix: Option<&str>,
        processor_precedence: i32,
    ) -> Result<Self, AbstractDialectError> {
        // Java 必须先成功执行 super(name)，之后才写入本类的两个 final 字段。
        let dialect = AbstractDialect::new(name)?;

        Ok(Self {
            dialect,
            prefix: prefix.map(str::to_owned),
            processor_precedence,
        })
    }

    /// 返回构造时指定的非空方言名称。
    ///
    /// 对应 Java: 继承的 `AbstractDialect#getName()`。
    ///
    /// # 返回
    ///
    /// 方言名称；包括 Java 允许的空字符串。
    #[must_use]
    pub fn get_name(&self) -> &str {
        self.dialect.get_name()
    }

    /// 返回构造时指定的可空默认前缀。
    ///
    /// 对应 Java: `AbstractProcessorDialect#getPrefix()`。
    ///
    /// # 返回
    ///
    /// 默认前缀；`None` 对应 Java `null`，空字符串不会与其合并。
    #[must_use]
    pub fn get_prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    /// 返回构造时指定的方言级 Processor precedence。
    ///
    /// 对应 Java: `AbstractProcessorDialect#getDialectProcessorPrecedence()`。
    ///
    /// # 返回
    ///
    /// 不可变的 Java `int` 对应值。
    #[must_use]
    pub const fn get_dialect_processor_precedence(&self) -> i32 {
        self.processor_precedence
    }
}

impl IDialect for AbstractProcessorDialect {
    fn java_class_name(&self) -> &'static str {
        "org.thymeleaf.dialect.AbstractProcessorDialect"
    }

    fn get_name(&self) -> Option<&str> {
        Some(AbstractProcessorDialect::get_name(self))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{AbstractDialectError, AbstractProcessorDialect};
    use crate::dialect::{IDialect, IProcessorDialect};
    use crate::processor::ProcessorSet;

    struct ProbeDialect {
        base: AbstractProcessorDialect,
        calls: AtomicUsize,
        last_prefix: Mutex<Option<Option<String>>>,
    }

    impl IDialect for ProbeDialect {
        fn get_name(&self) -> Option<&str> {
            Some(self.base.get_name())
        }
    }

    impl IProcessorDialect for ProbeDialect {
        fn get_prefix(&self) -> Option<&str> {
            self.base.get_prefix()
        }

        fn get_dialect_processor_precedence(&self) -> i32 {
            self.base.get_dialect_processor_precedence()
        }

        fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            *self.last_prefix.lock().expect("last prefix lock") =
                Some(dialect_prefix.map(str::to_owned));
            Some(ProcessorSet::new())
        }
    }

    #[test]
    fn rejects_null_name_through_parent_constructor_with_exact_error() {
        let error = AbstractProcessorDialect::new(None, Some("ignored"), i32::MAX)
            .err()
            .expect("null name must fail");

        assert_eq!(error, AbstractDialectError::DialectNameCannotBeNull);
        assert_eq!(error.to_string(), "Dialect name cannot be null");
    }

    #[test]
    fn preserves_composed_final_state_and_concrete_processor_extension_dispatch() {
        for (name, prefix, precedence) in [
            ("", None, i32::MIN),
            ("empty-prefix", Some(""), 0),
            ("方言", Some("前缀"), i32::MAX),
        ] {
            let base = AbstractProcessorDialect::new(Some(name), prefix, precedence)
                .expect("non-null name is valid");
            let implementation = ProbeDialect {
                base,
                calls: AtomicUsize::new(0),
                last_prefix: Mutex::new(None),
            };
            let contract: &dyn IProcessorDialect = &implementation;

            assert_eq!(implementation.base.get_name(), name);
            assert_eq!(implementation.base.get_prefix(), prefix);
            assert_eq!(
                implementation.base.get_dialect_processor_precedence(),
                precedence
            );
            assert_eq!(IDialect::get_name(&implementation.base), Some(name));
            assert_eq!(contract.get_name(), Some(name));
            assert_eq!(contract.get_prefix(), prefix);
            assert_eq!(contract.get_dialect_processor_precedence(), precedence);

            let actual_prefix = Some("用户覆盖");
            assert!(
                contract
                    .get_processors(actual_prefix)
                    .expect("probe returns an empty set")
                    .is_empty()
            );
            assert_eq!(implementation.calls.load(Ordering::SeqCst), 1);
            assert_eq!(
                *implementation.last_prefix.lock().expect("last prefix lock"),
                Some(actual_prefix.map(str::to_owned))
            );

            // Java 字段及 getter 都是 final；重复读取必须保持构造值。
            assert_eq!(implementation.base.get_name(), name);
            assert_eq!(implementation.base.get_prefix(), prefix);
            assert_eq!(
                implementation.base.get_dialect_processor_precedence(),
                precedence
            );
        }
    }
}
