use std::ptr;

static RESTRICTED_VALUE: StandardExpressionExecutionContext = StandardExpressionExecutionContext {
    restrict_variable_access: true,
    restrict_external_access: true,
    forbid_unsafe_expression_results: false,
    perform_type_conversion: false,
};
static RESTRICTED_FORBID_UNSAFE_EXP_RESULTS_VALUE: StandardExpressionExecutionContext =
    StandardExpressionExecutionContext {
        restrict_variable_access: true,
        restrict_external_access: true,
        forbid_unsafe_expression_results: true,
        perform_type_conversion: false,
    };
static NORMAL_VALUE: StandardExpressionExecutionContext = StandardExpressionExecutionContext {
    restrict_variable_access: false,
    restrict_external_access: false,
    forbid_unsafe_expression_results: false,
    perform_type_conversion: false,
};
static RESTRICTED_WITH_TYPE_CONVERSION_VALUE: StandardExpressionExecutionContext =
    StandardExpressionExecutionContext {
        restrict_variable_access: true,
        restrict_external_access: true,
        forbid_unsafe_expression_results: false,
        perform_type_conversion: true,
    };
static RESTRICTED_FORBID_UNSAFE_EXP_RESULTS_WITH_TYPE_CONVERSION_VALUE:
    StandardExpressionExecutionContext = StandardExpressionExecutionContext {
    restrict_variable_access: true,
    restrict_external_access: true,
    forbid_unsafe_expression_results: true,
    perform_type_conversion: true,
};
static NORMAL_WITH_TYPE_CONVERSION_VALUE: StandardExpressionExecutionContext =
    StandardExpressionExecutionContext {
        restrict_variable_access: false,
        restrict_external_access: false,
        forbid_unsafe_expression_results: false,
        perform_type_conversion: true,
    };

/// 标准表达式执行时使用的限制与类型转换上下文。
///
/// 本对象向表达式求值器传递变量访问、外部访问、不安全结果及类型转换条件。
/// 与 Java 一致，构造器不公开，只能从三个公开规范单例开始切换；切换操作会
/// 返回原实例或六个规范单例之一，而不会创建可观察的新对象。
///
/// 对应 Java:
/// `org.thymeleaf.standard.expression.StandardExpressionExecutionContext`。
#[derive(Debug)]
pub struct StandardExpressionExecutionContext {
    restrict_variable_access: bool,
    restrict_external_access: bool,
    forbid_unsafe_expression_results: bool,
    perform_type_conversion: bool,
}

impl StandardExpressionExecutionContext {
    /// 同时限制变量访问与外部访问的规范执行上下文。
    pub const RESTRICTED: &'static Self = &RESTRICTED_VALUE;

    /// 限制变量及外部访问，并禁止不安全表达式结果的规范执行上下文。
    pub const RESTRICTED_FORBID_UNSAFE_EXP_RESULTS: &'static Self =
        &RESTRICTED_FORBID_UNSAFE_EXP_RESULTS_VALUE;

    /// 不施加访问或结果限制的规范执行上下文。
    pub const NORMAL: &'static Self = &NORMAL_VALUE;

    /// 返回是否限制变量访问。
    ///
    /// # 返回
    /// 禁止受限变量访问时返回 `true`。
    #[must_use]
    pub const fn get_restrict_variable_access(&self) -> bool {
        self.restrict_variable_access
    }

    /// 返回是否限制新对象实例化及静态类访问。
    ///
    /// 对应 Java: `StandardExpressionExecutionContext#getRestrictExternalAccess()`。
    ///
    /// # 返回
    /// 应施加外部访问限制时返回 `true`。
    #[must_use]
    pub const fn get_restrict_external_access(&self) -> bool {
        self.restrict_external_access
    }

    /// 返回是否禁止不安全表达式结果。
    ///
    /// # 返回
    /// 不安全结果不得用于当前执行位置时返回 `true`。
    #[must_use]
    pub const fn get_forbid_unsafe_expression_results(&self) -> bool {
        self.forbid_unsafe_expression_results
    }

    /// 返回是否执行类型转换。
    ///
    /// # 返回
    /// 表达式结果需要经转换服务处理时返回 `true`。
    #[must_use]
    pub const fn get_perform_type_conversion(&self) -> bool {
        self.perform_type_conversion
    }

    /// 返回关闭类型转换后的规范上下文。
    ///
    /// 对应 Java:
    /// `StandardExpressionExecutionContext#withoutTypeConversion()`。
    ///
    /// # 返回
    /// 当前已关闭时返回同一实例；否则返回对应的无转换规范单例。
    #[must_use]
    pub fn without_type_conversion(&'static self) -> &'static Self {
        if !self.get_perform_type_conversion() {
            return self;
        }
        if ptr::eq(self, &NORMAL_WITH_TYPE_CONVERSION_VALUE) {
            return Self::NORMAL;
        }
        if ptr::eq(self, &RESTRICTED_WITH_TYPE_CONVERSION_VALUE) {
            return Self::RESTRICTED;
        }
        debug_assert!(ptr::eq(
            self,
            &RESTRICTED_FORBID_UNSAFE_EXP_RESULTS_WITH_TYPE_CONVERSION_VALUE
        ));
        Self::RESTRICTED_FORBID_UNSAFE_EXP_RESULTS
    }

    /// 返回启用类型转换后的规范上下文。
    ///
    /// 对应 Java: `StandardExpressionExecutionContext#withTypeConversion()`。
    ///
    /// # 返回
    /// 当前已启用时返回同一实例；否则返回对应的有转换规范单例。
    #[must_use]
    pub fn with_type_conversion(&'static self) -> &'static Self {
        if self.get_perform_type_conversion() {
            return self;
        }
        if ptr::eq(self, Self::NORMAL) {
            return &NORMAL_WITH_TYPE_CONVERSION_VALUE;
        }
        if ptr::eq(self, Self::RESTRICTED) {
            return &RESTRICTED_WITH_TYPE_CONVERSION_VALUE;
        }
        debug_assert!(ptr::eq(self, Self::RESTRICTED_FORBID_UNSAFE_EXP_RESULTS));
        &RESTRICTED_FORBID_UNSAFE_EXP_RESULTS_WITH_TYPE_CONVERSION_VALUE
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::StandardExpressionExecutionContext;

    #[test]
    fn preserves_flags_and_singleton_identity_across_conversion_switches() {
        let contexts = [
            StandardExpressionExecutionContext::NORMAL,
            StandardExpressionExecutionContext::RESTRICTED,
            StandardExpressionExecutionContext::RESTRICTED_FORBID_UNSAFE_EXP_RESULTS,
        ];
        let expected = [
            (false, false, false),
            (true, true, false),
            (true, true, true),
        ];

        for (context, flags) in contexts.into_iter().zip(expected) {
            assert_eq!(context.get_restrict_variable_access(), flags.0);
            assert_eq!(context.get_restrict_external_access(), flags.1);
            assert_eq!(context.get_forbid_unsafe_expression_results(), flags.2);
            assert!(!context.get_perform_type_conversion());
            assert!(ptr::eq(context.without_type_conversion(), context));

            let converted = context.with_type_conversion();
            assert!(converted.get_perform_type_conversion());
            assert!(ptr::eq(converted.with_type_conversion(), converted));
            assert!(ptr::eq(converted.without_type_conversion(), context));
        }
    }
}
