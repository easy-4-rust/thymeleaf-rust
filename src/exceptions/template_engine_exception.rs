use std::error::Error;

/// 模板引擎异常的公共标记契约。
///
/// 对应 Java: `org.thymeleaf.exceptions.TemplateEngineException`。
///
/// Java 对象是所有模板引擎运行时异常的抽象基类。Rust 没有异常继承，
/// 因此使用本 trait 保留统一的类型边界，并由各具体异常通过
/// `std::error::Error` 暴露消息和原因链。
pub trait TemplateEngineException: Error + Send + Sync + 'static {}

#[cfg(test)]
mod tests {
    use super::TemplateEngineException;
    use crate::exceptions::{
        AlreadyInitializedException, CacheConfigurationException, ConfigurationException,
        ParserInitializationException, TemplateInputException, TemplateOutputException,
        TemplateProcessingException,
    };

    #[test]
    fn all_java_subtypes_implement_the_common_contract() {
        fn assert_contract<T: TemplateEngineException>() {}

        assert_contract::<AlreadyInitializedException>();
        assert_contract::<CacheConfigurationException>();
        assert_contract::<ConfigurationException>();
        assert_contract::<ParserInitializationException>();
        assert_contract::<TemplateInputException>();
        assert_contract::<TemplateOutputException>();
        assert_contract::<TemplateProcessingException>();
    }
}
