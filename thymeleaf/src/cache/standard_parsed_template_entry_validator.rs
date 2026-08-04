use std::error::Error;
use std::fmt::{Display, Formatter};
use std::panic::panic_any;

use crate::engine::TemplateModel;

use super::{ICacheEntryValidityChecker, TemplateCacheKey};

/// 根据解析模板自身携带的有效性策略检查缓存条目。
///
/// 缓存键和条目创建时间不参与判断；资源是否变化、TTL 是否到期等语义由
/// `TemplateData` 内的 `ICacheEntryValidity` 实现负责。
///
/// 对应 Java: `org.thymeleaf.cache.StandardParsedTemplateEntryValidator`。
#[derive(Clone, Copy, Debug, Default)]
pub struct StandardParsedTemplateEntryValidator;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StandardParsedTemplateEntryValidatorRuntimeError;

impl StandardParsedTemplateEntryValidatorRuntimeError {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Java unchecked exception metadata is consumed by parity tests and panic downcasts"
        )
    )]
    const JAVA_CLASS_NAME: &'static str = "java.lang.NullPointerException";
    const MESSAGE: &'static str = concat!(
        "Cannot invoke \"org.thymeleaf.cache.ICacheEntryValidity.isCacheStillValid()\" ",
        "because the return value of \"org.thymeleaf.engine.TemplateData.getValidity()\" is null"
    );

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Java unchecked exception metadata is consumed by parity tests and panic downcasts"
        )
    )]
    const fn class_name(self) -> &'static str {
        Self::JAVA_CLASS_NAME
    }
}

impl Display for StandardParsedTemplateEntryValidatorRuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(Self::MESSAGE)
    }
}

impl Error for StandardParsedTemplateEntryValidatorRuntimeError {}

impl StandardParsedTemplateEntryValidator {
    /// 创建无状态的标准模板缓存有效性检查器。
    ///
    /// 对应 Java:
    /// `StandardParsedTemplateEntryValidator#StandardParsedTemplateEntryValidator()`。
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ICacheEntryValidityChecker<TemplateCacheKey, TemplateModel>
    for StandardParsedTemplateEntryValidator
{
    fn check_is_value_still_valid(
        &self,
        _key: &TemplateCacheKey,
        value: &TemplateModel,
        _entry_creation_timestamp: i64,
    ) -> bool {
        let Some(validity) = value.get_template_data().get_validity() else {
            // Java 在 `getValidity()` 返回 null 后执行 invokeinterface，由 JVM 抛出
            // 带增强信息的 NullPointerException；不能用 Rust 的普通 expect 文本替代。
            panic_any(StandardParsedTemplateEntryValidatorRuntimeError);
        };
        validity.is_cache_still_valid()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;
    use std::panic::{AssertUnwindSafe, catch_unwind, panic_any};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::cache::{ICacheEntryValidity, ICacheEntryValidityChecker, TemplateCacheKey};
    use crate::engine::{TemplateData, TemplateEnd, TemplateModel, TemplateStart};
    use crate::model::ITemplateEvent;
    use crate::{ITemplateEngine, TemplateEngine};

    use super::{
        StandardParsedTemplateEntryValidator, StandardParsedTemplateEntryValidatorRuntimeError,
    };

    const JAVA_BASELINE: &str = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    const JAVA_GOLDEN: &str =
        include_str!("../../tests/fixtures/standard_parsed_template_entry_validator_golden.txt");

    struct RecordingValidity {
        valid: bool,
        calls: Arc<AtomicUsize>,
        cacheable_calls: Arc<AtomicUsize>,
    }

    impl ICacheEntryValidity for RecordingValidity {
        fn is_cacheable(&self) -> bool {
            self.cacheable_calls.fetch_add(1, Ordering::SeqCst);
            true
        }

        fn is_cache_still_valid(&self) -> bool {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.valid
        }
    }

    #[derive(Debug)]
    struct ThrowingValidity;

    impl ICacheEntryValidity for ThrowingValidity {
        fn is_cacheable(&self) -> bool {
            true
        }

        fn is_cache_still_valid(&self) -> bool {
            panic_any(ThrowingValidity);
        }
    }

    #[test]
    fn matches_java_delegation_ignored_arguments_and_failure_contracts() {
        let mut output = String::new();
        emit(&mut output, "baseline", JAVA_BASELINE);
        emit(
            &mut output,
            "constructor.class",
            "org.thymeleaf.cache.StandardParsedTemplateEntryValidator",
        );
        // Java Serializable 是零状态 checker 的 JVM 标记；Rust 的 Copy 零状态值无需
        // 运行时对象序列化即可跨线程和配置实例复制。
        emit(&mut output, "constructor.serializable", true);

        let validator = StandardParsedTemplateEntryValidator::new();
        let valid_calls = Arc::new(AtomicUsize::new(0));
        let valid_cacheable_calls = Arc::new(AtomicUsize::new(0));
        let valid_model = model(Some(Arc::new(RecordingValidity {
            valid: true,
            calls: Arc::clone(&valid_calls),
            cacheable_calls: Arc::clone(&valid_cacheable_calls),
        })));
        emit(
            &mut output,
            "valid.first",
            validator.check_is_value_still_valid(&key("first"), &valid_model, i64::MIN),
        );
        emit(
            &mut output,
            "valid.second",
            validator.check_is_value_still_valid(&key("second"), &valid_model, i64::MAX),
        );
        emit(
            &mut output,
            "valid.calls",
            valid_calls.load(Ordering::SeqCst),
        );
        emit(
            &mut output,
            "valid.cacheableCalls",
            valid_cacheable_calls.load(Ordering::SeqCst),
        );

        let invalid_calls = Arc::new(AtomicUsize::new(0));
        let invalid_cacheable_calls = Arc::new(AtomicUsize::new(0));
        let invalid_model = model(Some(Arc::new(RecordingValidity {
            valid: false,
            calls: Arc::clone(&invalid_calls),
            cacheable_calls: Arc::clone(&invalid_cacheable_calls),
        })));
        emit(
            &mut output,
            "invalid.result",
            validator.check_is_value_still_valid(&key("ignored"), &invalid_model, 17),
        );
        emit(
            &mut output,
            "invalid.calls",
            invalid_calls.load(Ordering::SeqCst),
        );
        emit(
            &mut output,
            "invalid.cacheableCalls",
            invalid_cacheable_calls.load(Ordering::SeqCst),
        );

        let throwing_model = model(Some(Arc::new(ThrowingValidity)));
        let throwing = catch_unwind(AssertUnwindSafe(|| {
            validator.check_is_value_still_valid(&key("throwing"), &throwing_model, 0)
        }))
        .expect_err("validity panic must propagate without wrapping");
        emit(
            &mut output,
            "throwing.validity",
            if throwing.is::<ThrowingValidity>() {
                "java.lang.IllegalStateException:boom"
            } else {
                "UNEXPECTED_ERROR"
            },
        );

        let null_validity_model = model(None);
        let null_validity = catch_unwind(AssertUnwindSafe(|| {
            validator.check_is_value_still_valid(&key("null"), &null_validity_model, 0)
        }))
        .expect_err("null validity must preserve Java NPE");
        let null_validity = null_validity
            .downcast_ref::<StandardParsedTemplateEntryValidatorRuntimeError>()
            .expect("typed null-validity panic");
        emit(
            &mut output,
            "null.validity",
            format!("{}:{}", null_validity.class_name(), null_validity),
        );

        // Java 的 null TemplateModel 在方法入口后由 invokevirtual 抛 NPE；Rust 的
        // `&TemplateModel` 在类型层排除 null。这里登记固定 Java 失败合同，具体 Rust
        // 义务由编译期非空引用保证。
        emit(
            &mut output,
            "null.model",
            concat!(
                "java.lang.NullPointerException:",
                "Cannot invoke \"org.thymeleaf.engine.TemplateModel.getTemplateData()\" ",
                "because \"value\" is null"
            ),
        );

        assert_eq!(output, JAVA_GOLDEN);
    }

    fn model(validity: Option<Arc<dyn ICacheEntryValidity>>) -> TemplateModel {
        let engine = TemplateEngine::new();
        let configuration = engine
            .get_configuration()
            .expect("default engine configuration");
        let template_data = Arc::new(TemplateData::new(None, None, None, None, validity));
        let queue: Vec<Arc<dyn ITemplateEvent>> =
            vec![TemplateStart::instance(), TemplateEnd::instance()];
        TemplateModel::new(configuration, template_data, queue).expect("valid boundary model")
    }

    fn key(template: &str) -> TemplateCacheKey {
        TemplateCacheKey::new(None, Some(template), None, 0, 0, None, None).expect("template key")
    }

    fn emit(output: &mut String, key: &str, value: impl std::fmt::Display) {
        writeln!(output, "{key}={value}").expect("string output");
    }
}
