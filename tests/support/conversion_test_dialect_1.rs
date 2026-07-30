use std::sync::Arc;

use thymeleaf::StandardDialect;
use thymeleaf::dialect::{
    ExecutionAttributeMap, IDialect, IExecutionAttributeDialect, IExpressionObjectDialect,
    IProcessorDialect,
};
use thymeleaf::expression::{IExpressionObjectFactory, NativeVariableExpressionEvaluator};
use thymeleaf::processor::ProcessorSet;

use super::{CorpusOgnlRuntime, TestStandardConversionService1};

/// 使用 conversion1 自定义转换服务的 Standard Dialect。
///
/// 对应 Java:
/// `org.thymeleaf.templateengine.conversion.conversion1.ConversionTestDialect1`。
pub struct ConversionTestDialect1 {
    standard_dialect: StandardDialect,
}

impl ConversionTestDialect1 {
    /// 创建保留 Standard Dialect 全部能力、仅替换转换服务的方言。
    pub fn new() -> Self {
        let standard_dialect = StandardDialect::new();
        standard_dialect.set_variable_expression_evaluator(Arc::new(
            NativeVariableExpressionEvaluator::with_runtime(true, Arc::new(CorpusOgnlRuntime)),
        ));
        standard_dialect.set_conversion_service(Arc::new(TestStandardConversionService1));
        Self { standard_dialect }
    }
}

impl Default for ConversionTestDialect1 {
    fn default() -> Self {
        Self::new()
    }
}

impl IDialect for ConversionTestDialect1 {
    fn is_standard_dialect(&self) -> bool {
        true
    }

    fn as_processor_dialect(&self) -> Option<&dyn IProcessorDialect> {
        Some(self)
    }

    fn as_execution_attribute_dialect(&self) -> Option<&dyn IExecutionAttributeDialect> {
        Some(self)
    }

    fn as_expression_object_dialect(&self) -> Option<&dyn IExpressionObjectDialect> {
        Some(self)
    }

    fn get_name(&self) -> Option<&str> {
        self.standard_dialect.get_name()
    }
}

impl IProcessorDialect for ConversionTestDialect1 {
    fn get_prefix(&self) -> Option<&str> {
        self.standard_dialect.get_prefix()
    }

    fn get_dialect_processor_precedence(&self) -> i32 {
        self.standard_dialect.get_dialect_processor_precedence()
    }

    fn get_processors(&self, dialect_prefix: Option<&str>) -> Option<ProcessorSet> {
        self.standard_dialect.get_processors(dialect_prefix)
    }
}

impl IExecutionAttributeDialect for ConversionTestDialect1 {
    fn get_execution_attributes(&self) -> Option<ExecutionAttributeMap> {
        self.standard_dialect.get_execution_attributes()
    }
}

impl IExpressionObjectDialect for ConversionTestDialect1 {
    fn get_expression_object_factory(&self) -> Arc<dyn IExpressionObjectFactory> {
        self.standard_dialect.get_expression_object_factory()
    }
}
