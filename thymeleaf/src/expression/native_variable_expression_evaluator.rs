use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use num_bigint::BigInt;

use crate::context::IExpressionContext;
use crate::exceptions::TemplateProcessingException;
use crate::temporal::TemporalCreationUtils;
use crate::util::StandardExpressionUtils;
use crate::util::{ExpressionUtils, JavaBigDecimal, JavaNumber, JavaString};

use super::{
    AdditionExpression, AndExpression, ClassNotFoundException, ConditionalExpression,
    DefaultExpression, DivisionExpression, EqualsExpression, GreaterOrEqualToExpression,
    GreaterThanExpression, IStandardExpression, IStandardVariableExpression,
    IStandardVariableExpressionEvaluator, JavaConversionResult, JavaConversionValue,
    JavaTargetClass, LessOrEqualToExpression, LessThanExpression, LiteralValue, MinusExpression,
    MultiplicationExpression, NativeExpressionObjectsWrapper, NativeShortcutExpression,
    NegationExpression, NoOpOgnlRuntime, NoSuchMethodException, NotEqualsExpression, OgnlException,
    OgnlRuntime, OrExpression, RemainderExpression, StandardExpressionExecutionContext,
    StandardExpressionResult, StandardExpressions, SubtractionExpression, TemplateObject,
    TemplateValue,
    binary_operation_expression::{evaluate_as_boolean, evaluate_as_number},
    iterator_value::IteratorValue,
    map_entry_value::MapEntryValue,
    stream_value::StreamValue,
};

/// Thymeleaf Standard Dialect 的 OGNL 变量表达式求值器。
///
/// 对应 Java: `org.thymeleaf.standard.expression.OGNLVariableExpressionEvaluator`。
///
/// Rust 不具备 JVM 反射，因此 JavaBean 属性读取通过 `TemplateObject::java_get_property`
/// SPI 完成；Context、Map、List、数组以及表达式对象保留 OGNL 的动态访问语义。
pub struct NativeVariableExpressionEvaluator {
    apply_ognl_shortcuts: bool,
    runtime: Arc<dyn OgnlRuntime>,
}

impl NativeVariableExpressionEvaluator {
    /// 创建求值器并决定是否优先启用点分属性快速路径。
    #[must_use]
    /// 对应 Java 语义：`OGNLVariableExpressionEvaluator` 的 `new` 行为（Rust 侧辅助/私有路径）。
    pub fn new(apply_ognl_shortcuts: bool) -> Self {
        Self {
            apply_ognl_shortcuts,
            runtime: Arc::new(NoOpOgnlRuntime),
        }
    }

    /// 使用宿主提供的静态成员与构造器运行时创建求值器。
    #[must_use]
    /// 对应 Java 语义：`OGNLVariableExpressionEvaluator` 的 `with_runtime` 行为（Rust 侧辅助/私有路径）。
    pub fn with_runtime(apply_ognl_shortcuts: bool, runtime: Arc<dyn OgnlRuntime>) -> Self {
        Self {
            apply_ognl_shortcuts,
            runtime,
        }
    }

    fn evaluate_computed(
        &self,
        context: &dyn IExpressionContext,
        expression: &dyn IStandardVariableExpression,
        expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        let source = expression.get_expression().ok_or_else(|| {
            processing_error("Expression content is null, which is not allowed".to_owned())
        })?;
        if expression_context.get_restrict_external_access()
            && StandardExpressionUtils::contains_external_access(&source.to_string_lossy())
        {
            return Err(processing_error(
                "Instantiation of new objects and access to static classes or parameters is forbidden in this context"
                    .to_owned(),
            ));
        }

        let restrictions_apply = expression_context.get_restrict_variable_access()
            || expression_context.get_restrict_external_access();
        let cached = expression
            .get_cached_expression()
            .and_then(|value| value.downcast::<ComputedOGNLExpression>().ok());
        let computed = if cached
            .as_deref()
            .is_some_and(|value| !restrictions_apply || !value.is_shortcut())
        {
            cached.expect("cached expression was checked above")
        } else {
            // 对应 Java obtainComputedOGNLExpression：变量访问或外部访问受限时，
            // 必须交给完整 OGNL 路径执行 AST/成员 ACL，不能使用属性 shortcut。
            parse_and_cache_expression(
                expression,
                source,
                self.apply_ognl_shortcuts && !restrictions_apply,
            )
        };

        let result = with_ognl_runtime(Arc::clone(&self.runtime), || {
            with_ognl_locals(|| {
                let result = match &computed.expression {
                    ComputedExpression::Shortcut(shortcut) => match shortcut.evaluate(
                        context,
                        expression.get_use_selection_as_root(),
                        expression_context.get_restrict_variable_access(),
                    ) {
                        Ok(value) => value,
                        Err(
                            super::NativeShortcutError::NotApplicable(_)
                            | super::NativeShortcutError::PropertyGetter { .. },
                        ) => {
                            // 对应 Java evaluate 对
                            // OGNLShortcutExpressionNotApplicableException 的处理：shortcut
                            // 只是优化，不能改变表达式可执行性。失配后立即替换缓存并按
                            // 完整路径求值，后续调用不会再次进入 shortcut。
                            let fallback = parse_and_cache_expression(expression, source, false);
                            evaluate_computed_expression(
                                context,
                                &fallback.expression,
                                expression.get_use_selection_as_root(),
                                expression_context,
                            )?
                        }
                        Err(error) => {
                            let message = error.to_string();
                            return Err(ognl_processing_error(
                                format!(
                                    "Exception evaluating OGNL expression: \"{}\": {message}",
                                    source.to_string_lossy()
                                ),
                                message,
                            ));
                        }
                    },
                    ComputedExpression::Path(path) => evaluate_path(
                        context,
                        path,
                        expression.get_use_selection_as_root(),
                        expression_context,
                    )?,
                    ComputedExpression::Literal(value) => value.to_template_value(),
                    ComputedExpression::Operation(expression) => {
                        expression.execute_with_context(context, expression_context)?
                    }
                    ComputedExpression::StaticReference(reference) => evaluate_static_reference(
                        context,
                        reference,
                        expression.get_use_selection_as_root(),
                        expression_context,
                    )?,
                    ComputedExpression::Constructor(constructor) => evaluate_constructor(
                        context,
                        constructor,
                        expression.get_use_selection_as_root(),
                        expression_context,
                    )?,
                    ComputedExpression::ListLiteral(values) => evaluate_list_literal(
                        context,
                        values,
                        expression.get_use_selection_as_root(),
                        expression_context,
                    )?,
                    ComputedExpression::MapLiteral(entries) => evaluate_map_literal(
                        context,
                        entries,
                        expression.get_use_selection_as_root(),
                        expression_context,
                    )?,
                    ComputedExpression::Inclusion {
                        left,
                        right,
                        negated,
                    } => evaluate_inclusion(
                        context,
                        left,
                        right,
                        *negated,
                        expression.get_use_selection_as_root(),
                        expression_context,
                    )?,
                    ComputedExpression::Sequence(values) => evaluate_sequence(
                        context,
                        values,
                        expression.get_use_selection_as_root(),
                        expression_context,
                    )?,
                    ComputedExpression::Assignment { name, value } => evaluate_assignment(
                        context,
                        name,
                        value,
                        expression.get_use_selection_as_root(),
                        expression_context,
                    )?,
                    ComputedExpression::NativeBinary {
                        operator,
                        left,
                        right,
                    } => evaluate_native_binary(
                        context,
                        *operator,
                        left,
                        right,
                        expression.get_use_selection_as_root(),
                        expression_context,
                    )?,
                    ComputedExpression::BitNegate(value) => evaluate_bit_negate(
                        context,
                        value,
                        expression.get_use_selection_as_root(),
                        expression_context,
                    )?,
                    ComputedExpression::InstanceOf { value, type_name } => evaluate_instance_of(
                        context,
                        value,
                        type_name,
                        expression.get_use_selection_as_root(),
                        expression_context,
                    )?,
                    ComputedExpression::Navigation { root, steps } => evaluate_navigation(
                        context,
                        root,
                        steps,
                        expression.get_use_selection_as_root(),
                        expression_context,
                    )?,
                    ComputedExpression::Unsupported => {
                        return Err(processing_error(format!(
                            "Exception evaluating OGNL expression: \"{}\": unsupported OGNL syntax",
                            source.to_string_lossy()
                        )));
                    }
                };
                Ok(result)
            })
        })?;

        // Context 内部用 TemplateValue::Null 保存 Java null 哨兵，但表达式 API
        // 必须继续以 None 暴露 Java null，DefaultExpression 才会执行右操作数。
        let result = normalize_java_null(result);
        if !expression_context.get_perform_type_conversion() {
            return Ok(result);
        }
        convert_to_string(context, result)
    }
}

impl IStandardVariableExpressionEvaluator for NativeVariableExpressionEvaluator {
    fn evaluate(
        &self,
        context: &dyn IExpressionContext,
        expression: &dyn IStandardVariableExpression,
        expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        self.evaluate_computed(context, expression, expression_context)
    }
}

impl std::fmt::Display for NativeVariableExpressionEvaluator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("OGNL")
    }
}

/// 已解析 OGNL 表达式及其可执行表示。
///
/// 对应 Java: `OGNLVariableExpressionEvaluator.ComputedOGNLExpression`。
struct ComputedOGNLExpression {
    expression: ComputedExpression,
}

impl ComputedOGNLExpression {
    fn is_shortcut(&self) -> bool {
        matches!(self.expression, ComputedExpression::Shortcut(_))
    }
}

fn parse_and_cache_expression(
    expression: &dyn IStandardVariableExpression,
    source: &JavaString,
    apply_shortcuts: bool,
) -> Arc<ComputedOGNLExpression> {
    let value = Arc::new(parse_expression(
        source,
        apply_shortcuts,
        expression.get_use_selection_as_root(),
    ));
    let cached: Arc<dyn std::any::Any + Send + Sync> = value.clone();
    expression.set_cached_expression(Some(cached));
    value
}

/// 在解析类型前执行 Thymeleaf 禁止类型 ACL。
///
/// 对应 Java: `OGNLVariableExpressionEvaluator.ThymeleafACLClassResolver`。
struct ThymeleafACLClassResolver;

impl ThymeleafACLClassResolver {
    fn class_for_name(class_name: &str) -> StandardExpressionResult<&str> {
        if ExpressionUtils::is_type_forbidden(class_name) {
            return Err(processing_error(format!(
                "Access is forbidden for type '{class_name}' in this expression context."
            )));
        }
        ThymeleafDefaultClassResolver::class_for_name(class_name)
    }
}

/// 不会隐式补全 `java.lang.` 的严格类型名解析器。
///
/// 对应 Java: `OGNLVariableExpressionEvaluator.ThymeleafDefaultClassResolver`。
struct ThymeleafDefaultClassResolver;

impl ThymeleafDefaultClassResolver {
    fn class_for_name(class_name: &str) -> StandardExpressionResult<&str> {
        if class_name.trim().is_empty() {
            return Err(processing_error("Class name cannot be empty".to_owned()));
        }
        // Java Thymeleaf 的默认 OGNL ClassResolver 直接调用 Class.forName，不会把
        // `String` 隐式补全为 `java.lang.String`。
        if !class_name.contains('.') {
            return Err(processing_error_with_cause(
                format!("Class not found: {class_name}"),
                ClassNotFoundException::new(class_name.to_owned()),
            ));
        }
        Ok(class_name)
    }
}

/// OGNL 公共成员访问 ACL。
///
/// 对应 Java: `OGNLVariableExpressionEvaluator.ThymeleafACLMemberAccess`。
struct ThymeleafACLMemberAccess;

impl ThymeleafACLMemberAccess {
    fn is_accessible(
        target: Option<&dyn TemplateObject>,
        member_name: &str,
    ) -> StandardExpressionResult<()> {
        if ExpressionUtils::is_member_forbidden(target, member_name) {
            return Err(processing_error(format!(
                "Accessing member '{member_name}' is forbidden in this expression context."
            )));
        }
        Ok(())
    }
}

enum ComputedExpression {
    Shortcut(NativeShortcutExpression),
    Path(OgnlPath),
    Literal(OgnlLiteral),
    Operation(Arc<dyn IStandardExpression>),
    StaticReference(OgnlStaticReference),
    Constructor(OgnlConstructor),
    ListLiteral(Vec<ComputedExpression>),
    MapLiteral(Vec<(Box<ComputedExpression>, Box<ComputedExpression>)>),
    Inclusion {
        left: Box<ComputedExpression>,
        right: Box<ComputedExpression>,
        negated: bool,
    },
    Sequence(Vec<ComputedExpression>),
    Assignment {
        name: JavaString,
        value: Box<ComputedExpression>,
    },
    NativeBinary {
        operator: OgnlBinaryOperator,
        left: Box<ComputedExpression>,
        right: Box<ComputedExpression>,
    },
    BitNegate(Box<ComputedExpression>),
    InstanceOf {
        value: Box<ComputedExpression>,
        type_name: JavaString,
    },
    Navigation {
        root: Box<ComputedExpression>,
        steps: Vec<PathStep>,
    },
    Unsupported,
}

#[derive(Clone, Copy)]
enum OgnlBinaryOperator {
    Divide,
    BitOr,
    BitXor,
    BitAnd,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
}

struct OgnlStaticReference {
    type_name: JavaString,
    member_name: JavaString,
    arguments: Option<Vec<ComputedExpression>>,
    trailing_steps: Vec<PathStep>,
}

struct OgnlConstructor {
    type_name: JavaString,
    arguments: Vec<ComputedExpression>,
    trailing_steps: Vec<PathStep>,
}

struct OgnlLeafExpression {
    source: JavaString,
    expression: Box<ComputedExpression>,
    use_selection_as_root: bool,
}

impl IStandardExpression for OgnlLeafExpression {
    fn get_string_representation(&self) -> StandardExpressionResult<JavaString> {
        Ok(self.source.clone())
    }

    fn execute_with_context(
        &self,
        context: &dyn IExpressionContext,
        expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        evaluate_computed_expression(
            context,
            self.expression.as_ref(),
            self.use_selection_as_root,
            expression_context,
        )
    }

    fn execute_raw(
        &self,
        context: &dyn IExpressionContext,
        expression_context: &'static StandardExpressionExecutionContext,
    ) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
        // 对应 Java `Expression.execute` 的 LiteralValue 不解包语义：
        // 字面量叶子按原始字面量返回（Java OGNL/Thymeleaf 的加法对
        // String 字面量拼接而非数值相加），其他计算表达式与公开执行一致。
        match self.expression.as_ref() {
            ComputedExpression::Literal(OgnlLiteral::String(value)) => Ok(Some(Arc::new(
                TemplateValue::Literal(Arc::new(LiteralValue::new(Some(value.clone())))),
            ))),
            ComputedExpression::Literal(OgnlLiteral::Character(value)) => {
                Ok(Some(Arc::new(TemplateValue::Literal(Arc::new(
                    LiteralValue::new(Some(JavaString::from_utf16(vec![*value]))),
                )))))
            }
            _ => self.execute_with_context(context, expression_context),
        }
    }
}

enum OgnlLiteral {
    Null,
    Boolean(bool),
    Character(u16),
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    BigInteger(num_bigint::BigInt),
    BigDecimal(JavaBigDecimal),
    String(JavaString),
}

impl OgnlLiteral {
    fn to_template_value(&self) -> Option<Arc<TemplateValue>> {
        match self {
            Self::Null => None,
            Self::Boolean(value) => Some(Arc::new(TemplateValue::Boolean(*value))),
            Self::Character(value) => Some(Arc::new(TemplateValue::Character(*value))),
            Self::Integer(value) => {
                Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(*value))))
            }
            Self::Long(value) => Some(Arc::new(TemplateValue::Number(JavaNumber::Long(*value)))),
            Self::Float(value) => Some(Arc::new(TemplateValue::Number(JavaNumber::Float(*value)))),
            Self::Double(value) => {
                Some(Arc::new(TemplateValue::Number(JavaNumber::Double(*value))))
            }
            Self::BigInteger(value) => Some(Arc::new(TemplateValue::Number(
                JavaNumber::BigInteger(value.clone()),
            ))),
            Self::BigDecimal(value) => Some(Arc::new(TemplateValue::Number(
                JavaNumber::BigDecimal(value.clone()),
            ))),
            Self::String(value) => Some(Arc::new(TemplateValue::string(value.clone()))),
        }
    }
}

struct OgnlPath {
    root: PathRoot,
    steps: Vec<PathStep>,
}

enum PathRoot {
    Context(JavaString),
    ExpressionObject(JavaString),
}

enum PathStep {
    Property(JavaString),
    Method(JavaString, Vec<ComputedExpression>),
    Projection(Box<ComputedExpression>),
    Selection(SelectionKind, Box<ComputedExpression>),
    StringIndex(JavaString),
    NumericIndex(usize),
    DynamicSubscript(OgnlDynamicSubscript),
    DynamicIndex(Box<ComputedExpression>),
}

#[derive(Clone, Copy)]
enum OgnlDynamicSubscript {
    First,
    Mid,
    Last,
    All,
}

#[derive(Clone, Copy)]
enum SelectionKind {
    All,
    First,
    Last,
}

fn parse_expression(
    source: &JavaString,
    apply_shortcuts: bool,
    use_selection_as_root: bool,
) -> ComputedOGNLExpression {
    let trimmed = java_trim(source);
    let expression = parse_ognl_range(trimmed.as_utf16(), apply_shortcuts, use_selection_as_root)
        .unwrap_or(ComputedExpression::Unsupported);
    ComputedOGNLExpression { expression }
}

fn parse_ognl_range(
    input: &[u16],
    apply_shortcuts: bool,
    use_selection_as_root: bool,
) -> Option<ComputedExpression> {
    let input = java_trim_units(input);
    if input.is_empty() {
        return None;
    }
    if is_outer_parenthesized(input) {
        return parse_ognl_range(
            &input[1..input.len() - 1],
            apply_shortcuts,
            use_selection_as_root,
        );
    }
    let sequence = split_ognl_entries(input)?;
    if sequence.len() > 1 {
        return sequence
            .into_iter()
            .map(|entry| parse_ognl_range(entry, apply_shortcuts, use_selection_as_root))
            .collect::<Option<Vec<_>>>()
            .map(ComputedExpression::Sequence);
    }
    if let Some(position) = find_assignment_operator(input) {
        let target = java_trim_units(&input[..position]);
        if target.first() != Some(&(b'#' as u16))
            || target.len() < 2
            || !target[1..].iter().copied().all(is_ascii_identifier_part)
        {
            return None;
        }
        let value = parse_ognl_range(
            &input[position + 1..],
            apply_shortcuts,
            use_selection_as_root,
        )?;
        return Some(ComputedExpression::Assignment {
            name: JavaString::from_utf16(target[1..].to_vec()),
            value: Box::new(value),
        });
    }
    if let Some(navigation) =
        parse_primary_navigation(input, apply_shortcuts, use_selection_as_root)
    {
        return Some(navigation);
    }
    if let Some(collection) =
        parse_collection_literal(input, apply_shortcuts, use_selection_as_root)
    {
        return Some(collection);
    }
    if let Some(reference) = parse_static_reference(input, apply_shortcuts, use_selection_as_root) {
        return Some(ComputedExpression::StaticReference(reference));
    }
    if let Some(constructor) = parse_constructor(input, apply_shortcuts, use_selection_as_root) {
        return Some(ComputedExpression::Constructor(constructor));
    }
    if let Some((question, colon)) = find_conditional(input) {
        let colon = colon?;
        let condition =
            parse_ognl_operand(&input[..question], apply_shortcuts, use_selection_as_root)?;
        let then_expression = parse_ognl_operand(
            &input[question + 1..colon],
            apply_shortcuts,
            use_selection_as_root,
        )?;
        let else_expression =
            parse_ognl_operand(&input[colon + 1..], apply_shortcuts, use_selection_as_root)?;
        return ConditionalExpression::new(
            Some(condition),
            Some(then_expression),
            Some(else_expression),
        )
        .ok()
        .map(|value| ComputedExpression::Operation(Arc::new(value)));
    }
    if let Some((question, colon)) = find_default_operator(input) {
        let queried =
            parse_ognl_operand(&input[..question], apply_shortcuts, use_selection_as_root)?;
        let default =
            parse_ognl_operand(&input[colon + 1..], apply_shortcuts, use_selection_as_root)?;
        return DefaultExpression::new(Some(queried), Some(default))
            .ok()
            .map(|value| ComputedExpression::Operation(Arc::new(value)));
    }

    macro_rules! binary_group {
        ($operators:expr) => {
            if let Some((position, operator)) = find_binary_operator(input, $operators) {
                let left =
                    parse_ognl_operand(&input[..position], apply_shortcuts, use_selection_as_root)?;
                let right = parse_ognl_operand(
                    &input[position + operator.len()..],
                    apply_shortcuts,
                    use_selection_as_root,
                )?;
                return build_ognl_binary(operator, left, right);
            }
        };
    }
    macro_rules! native_binary_group {
        ($operators:expr, $operator:expr) => {
            if let Some((position, token)) = find_binary_operator(input, $operators) {
                let left =
                    parse_ognl_range(&input[..position], apply_shortcuts, use_selection_as_root)?;
                let right = parse_ognl_range(
                    &input[position + token.len()..],
                    apply_shortcuts,
                    use_selection_as_root,
                )?;
                return Some(ComputedExpression::NativeBinary {
                    operator: $operator,
                    left: Box::new(left),
                    right: Box::new(right),
                });
            }
        };
    }
    binary_group!(&[OP_OR, OP_DOUBLE_PIPE]);
    binary_group!(&[OP_AND, OP_DOUBLE_AMPERSAND]);
    native_binary_group!(&[OP_BOR, OP_PIPE], OgnlBinaryOperator::BitOr);
    native_binary_group!(&[OP_XOR, OP_CARET], OgnlBinaryOperator::BitXor);
    native_binary_group!(&[OP_BAND, OP_AMPERSAND], OgnlBinaryOperator::BitAnd);
    binary_group!(&[OP_NEQ, OP_NE, OP_NOT_EQUALS, OP_EQ, OP_EQUALS]);
    binary_group!(&[
        OP_GTE,
        OP_GE,
        OP_GREATER_EQUAL,
        OP_GT,
        OP_GREATER,
        OP_LTE,
        OP_LE,
        OP_LESS_EQUAL,
        OP_LT,
        OP_LESS,
    ]);
    if let Some((position, operator_length, negated)) = find_inclusion_operator(input) {
        let left = parse_ognl_range(&input[..position], apply_shortcuts, use_selection_as_root)?;
        let right = parse_ognl_range(
            &input[position + operator_length..],
            apply_shortcuts,
            use_selection_as_root,
        )?;
        return Some(ComputedExpression::Inclusion {
            left: Box::new(left),
            right: Box::new(right),
            negated,
        });
    }
    if let Some(position) = find_word_operator(input, OP_INSTANCEOF) {
        let value = parse_ognl_range(&input[..position], apply_shortcuts, use_selection_as_root)?;
        let type_name = java_trim_units(&input[position + OP_INSTANCEOF.len()..]);
        if type_name.is_empty()
            || !type_name
                .iter()
                .copied()
                .all(|unit| is_ascii_identifier_part(unit) || unit == b'.' as u16)
        {
            return None;
        }
        return Some(ComputedExpression::InstanceOf {
            value: Box::new(value),
            type_name: JavaString::from_utf16(type_name.to_vec()),
        });
    }
    native_binary_group!(
        &[OP_USHR, OP_UNSIGNED_SHIFT_RIGHT],
        OgnlBinaryOperator::UnsignedShiftRight
    );
    native_binary_group!(&[OP_SHR, OP_SHIFT_RIGHT], OgnlBinaryOperator::ShiftRight);
    native_binary_group!(&[OP_SHL, OP_SHIFT_LEFT], OgnlBinaryOperator::ShiftLeft);
    binary_group!(&[OP_PLUS, OP_MINUS]);
    binary_group!(&[OP_MULTIPLY, OP_MOD, OP_REMAINDER]);
    native_binary_group!(&[OP_DIV, OP_DIVIDE], OgnlBinaryOperator::Divide);

    if input[0] == b'-' as u16 {
        let operand = parse_ognl_operand(&input[1..], apply_shortcuts, use_selection_as_root)?;
        return MinusExpression::new(Some(operand))
            .ok()
            .map(|value| ComputedExpression::Operation(Arc::new(value)));
    }
    if input[0] == b'+' as u16 {
        return parse_ognl_range(&input[1..], apply_shortcuts, use_selection_as_root);
    }
    if input[0] == b'~' as u16 {
        let operand = parse_ognl_range(&input[1..], apply_shortcuts, use_selection_as_root)?;
        return Some(ComputedExpression::BitNegate(Box::new(operand)));
    }
    if input[0] == b'!' as u16 {
        let operand = parse_ognl_operand(&input[1..], apply_shortcuts, use_selection_as_root)?;
        return NegationExpression::new(Some(operand))
            .ok()
            .map(|value| ComputedExpression::Operation(Arc::new(value)));
    }
    if starts_with_word(input, "not") {
        let operand = parse_ognl_operand(&input[3..], apply_shortcuts, use_selection_as_root)?;
        return NegationExpression::new(Some(operand))
            .ok()
            .map(|value| ComputedExpression::Operation(Arc::new(value)));
    }

    let source = JavaString::from_utf16(input.to_vec());
    if let Some(value) = parse_literal(&source) {
        return Some(ComputedExpression::Literal(value));
    }
    if apply_shortcuts && let Some(levels) = NativeShortcutExpression::parse(Some(&source)) {
        return Some(ComputedExpression::Shortcut(NativeShortcutExpression::new(
            levels,
        )));
    }
    parse_path(&source, apply_shortcuts, use_selection_as_root).map(ComputedExpression::Path)
}

fn parse_primary_navigation(
    input: &[u16],
    apply_shortcuts: bool,
    use_selection_as_root: bool,
) -> Option<ComputedExpression> {
    let root_end = if matches!(input.first(), Some(0x27 | 0x22)) {
        let quote = input[0];
        (1..input.len())
            .find(|position| input[*position] == quote && !is_escaped(input, *position))
            .map(|position| position + 1)?
    } else if input.first() == Some(&(b'(' as u16)) {
        find_closing_parenthesis(input, 0)? + 1
    } else if input.starts_with(&[b'#' as u16, b'{' as u16]) {
        find_closing_delimiter(input, 1, b'{' as u16, b'}' as u16)? + 1
    } else if input.first() == Some(&(b'{' as u16)) {
        find_closing_delimiter(input, 0, b'{' as u16, b'}' as u16)? + 1
    } else {
        return None;
    };
    if root_end == input.len()
        || !matches!(input[root_end], value if value == b'.' as u16 || value == b'[' as u16)
    {
        return None;
    }
    let root = parse_ognl_range(&input[..root_end], apply_shortcuts, use_selection_as_root)?;
    let mut position = root_end;
    let steps = parse_suffix_steps(input, &mut position, apply_shortcuts, use_selection_as_root)?;
    Some(ComputedExpression::Navigation {
        root: Box::new(root),
        steps,
    })
}

fn parse_ognl_operand(
    input: &[u16],
    apply_shortcuts: bool,
    use_selection_as_root: bool,
) -> Option<Arc<dyn IStandardExpression>> {
    let expression = parse_ognl_range(input, apply_shortcuts, use_selection_as_root)?;
    match expression {
        ComputedExpression::Operation(expression) => Some(expression),
        expression => Some(Arc::new(OgnlLeafExpression {
            source: JavaString::from_utf16(java_trim_units(input).to_vec()),
            expression: Box::new(expression),
            use_selection_as_root,
        })),
    }
}

fn build_ognl_binary(
    operator: &[u16],
    left: Arc<dyn IStandardExpression>,
    right: Arc<dyn IStandardExpression>,
) -> Option<ComputedExpression> {
    let operator = String::from_utf16_lossy(operator).to_ascii_lowercase();
    macro_rules! create {
        ($type:ty) => {
            <$type>::new(Some(left), Some(right))
                .ok()
                .map(|value| ComputedExpression::Operation(Arc::new(value)))
        };
    }
    match operator.as_str() {
        "or" | "||" => create!(OrExpression),
        "and" | "&&" => create!(AndExpression),
        "eq" | "==" => create!(EqualsExpression),
        "neq" | "ne" | "!=" => create!(NotEqualsExpression),
        "gt" | ">" => create!(GreaterThanExpression),
        "gte" | "ge" | ">=" => create!(GreaterOrEqualToExpression),
        "lt" | "<" => create!(LessThanExpression),
        "lte" | "le" | "<=" => create!(LessOrEqualToExpression),
        "+" => create!(AdditionExpression),
        "-" => create!(SubtractionExpression),
        "*" => create!(MultiplicationExpression),
        "div" | "/" => create!(DivisionExpression),
        "mod" | "%" => create!(RemainderExpression),
        _ => None,
    }
}

fn parse_literal(source: &JavaString) -> Option<OgnlLiteral> {
    let text = source.to_string_lossy();
    if text == "null" {
        return Some(OgnlLiteral::Null);
    }
    if text == "true" {
        return Some(OgnlLiteral::Boolean(true));
    }
    if text == "false" {
        return Some(OgnlLiteral::Boolean(false));
    }
    if source.as_utf16().len() >= 2
        && matches!(source.as_utf16().first(), Some(0x27 | 0x22))
        && source.as_utf16().last() == source.as_utf16().first()
    {
        let contents = unescape_ognl_string(&source.as_utf16()[1..source.as_utf16().len() - 1])?;
        if source.as_utf16().first() == Some(&(b'\'' as u16)) && contents.len() == 1 {
            return Some(OgnlLiteral::Character(contents[0]));
        }
        return Some(OgnlLiteral::String(JavaString::from_utf16(contents)));
    }
    let unsigned_source = text.trim_start_matches(['-', '+']);
    let hexadecimal = unsigned_source.starts_with("0x") || unsigned_source.starts_with("0X");
    let (number, suffix) = text
        .char_indices()
        .last()
        .filter(|(_, value)| {
            value.is_ascii_alphabetic()
                && (!hexadecimal || matches!(value.to_ascii_lowercase(), 'h' | 'l'))
        })
        .map_or((text.as_str(), None), |(position, value)| {
            (&text[..position], Some(value.to_ascii_lowercase()))
        });
    let signed = number.starts_with('-') || number.starts_with('+');
    let unsigned = number.trim_start_matches(['-', '+']);
    let radix_value = if unsigned.starts_with("0x") || unsigned.starts_with("0X") {
        i64::from_str_radix(&unsigned[2..], 16).ok().map(|value| {
            if number.starts_with('-') {
                -value
            } else {
                value
            }
        })
    } else if unsigned.len() > 1
        && unsigned.starts_with('0')
        && unsigned.chars().all(|value| matches!(value, '0'..='7'))
    {
        i64::from_str_radix(&unsigned[1..], 8).ok().map(|value| {
            if number.starts_with('-') {
                -value
            } else {
                value
            }
        })
    } else {
        None
    };
    if matches!(suffix, Some('h')) {
        let value = if unsigned.starts_with("0x") || unsigned.starts_with("0X") {
            num_bigint::BigInt::parse_bytes(&unsigned.as_bytes()[2..], 16).map(|value| {
                if number.starts_with('-') {
                    -value
                } else {
                    value
                }
            })
        } else {
            number.parse().ok()
        }?;
        return Some(OgnlLiteral::BigInteger(value));
    }
    if matches!(suffix, Some('b')) {
        return JavaBigDecimal::parse(number)
            .ok()
            .map(OgnlLiteral::BigDecimal);
    }
    if matches!(suffix, Some('l')) {
        return radix_value
            .or_else(|| number.parse().ok())
            .map(OgnlLiteral::Long);
    }
    if matches!(suffix, Some('f')) {
        return number.parse().ok().map(OgnlLiteral::Float);
    }
    if matches!(suffix, Some('d')) {
        return number.parse().ok().map(OgnlLiteral::Double);
    }
    if suffix.is_some() || signed && number.len() == 1 {
        return None;
    }
    if let Some(value) = radix_value {
        return i32::try_from(value)
            .map(OgnlLiteral::Integer)
            .ok()
            .or(Some(OgnlLiteral::Long(value)));
    }
    if let Ok(value) = number.parse::<i32>() {
        return Some(OgnlLiteral::Integer(value));
    }
    if let Ok(value) = number.parse::<i64>() {
        return Some(OgnlLiteral::Long(value));
    }
    if let Ok(value) = number.parse::<f64>() {
        return Some(OgnlLiteral::Double(value));
    }
    None
}

fn parse_collection_literal(
    input: &[u16],
    apply_shortcuts: bool,
    use_selection_as_root: bool,
) -> Option<ComputedExpression> {
    let (map, body) =
        if input.starts_with(&[b'#' as u16, b'{' as u16]) && input.last() == Some(&(b'}' as u16)) {
            (true, &input[2..input.len() - 1])
        } else if input.first() == Some(&(b'{' as u16)) && input.last() == Some(&(b'}' as u16)) {
            (false, &input[1..input.len() - 1])
        } else {
            return None;
        };
    let entries = split_ognl_entries(body)?;
    if map {
        let mut values = Vec::with_capacity(entries.len());
        for entry in entries {
            let colon = find_top_level_sequence(entry, &[b':' as u16])?;
            let key = parse_ognl_range(&entry[..colon], apply_shortcuts, use_selection_as_root)?;
            let value =
                parse_ognl_range(&entry[colon + 1..], apply_shortcuts, use_selection_as_root)?;
            values.push((Box::new(key), Box::new(value)));
        }
        Some(ComputedExpression::MapLiteral(values))
    } else {
        entries
            .into_iter()
            .map(|entry| parse_ognl_range(entry, apply_shortcuts, use_selection_as_root))
            .collect::<Option<Vec<_>>>()
            .map(ComputedExpression::ListLiteral)
    }
}

fn split_ognl_entries(input: &[u16]) -> Option<Vec<&[u16]>> {
    if java_trim_units(input).is_empty() {
        return Some(Vec::new());
    }
    let mut entries = Vec::new();
    let mut start = 0;
    scan_top_level(input, |position, unit| {
        if unit == b',' as u16 {
            entries.push(java_trim_units(&input[start..position]));
            start = position + 1;
        }
    });
    entries.push(java_trim_units(&input[start..]));
    entries
        .iter()
        .all(|entry| !entry.is_empty())
        .then_some(entries)
}

fn parse_static_reference(
    input: &[u16],
    apply_shortcuts: bool,
    use_selection_as_root: bool,
) -> Option<OgnlStaticReference> {
    if input.first() != Some(&(b'@' as u16)) {
        return None;
    }
    let second_at = input[1..].iter().position(|unit| *unit == b'@' as u16)? + 1;
    let type_name = JavaString::from_utf16(java_trim_units(&input[1..second_at]).to_vec());
    if type_name.is_empty() {
        return None;
    }
    let mut position = second_at + 1;
    let member_start = position;
    while position < input.len() && is_ascii_identifier_part(input[position]) {
        position += 1;
    }
    if position == member_start {
        return None;
    }
    let member_name = JavaString::from_utf16(input[member_start..position].to_vec());
    let arguments = if input.get(position) == Some(&(b'(' as u16)) {
        let end = find_closing_parenthesis(input, position)?;
        let arguments = split_method_arguments(&input[position + 1..end])?
            .into_iter()
            .map(|argument| {
                parse_expression(
                    &JavaString::from_utf16(argument.to_vec()),
                    apply_shortcuts,
                    use_selection_as_root,
                )
                .expression
            })
            .collect();
        position = end + 1;
        Some(arguments)
    } else {
        None
    };
    let trailing_steps =
        parse_suffix_steps(input, &mut position, apply_shortcuts, use_selection_as_root)?;
    Some(OgnlStaticReference {
        type_name,
        member_name,
        arguments,
        trailing_steps,
    })
}

fn parse_constructor(
    input: &[u16],
    apply_shortcuts: bool,
    use_selection_as_root: bool,
) -> Option<OgnlConstructor> {
    if !starts_with_word(input, "new") {
        return None;
    }
    let mut position = 3;
    while input.get(position).is_some_and(|unit| *unit <= 0x20) {
        position += 1;
    }
    let type_start = position;
    while input
        .get(position)
        .is_some_and(|unit| is_ascii_identifier_part(*unit) || *unit == b'.' as u16)
    {
        position += 1;
    }
    if position == type_start {
        return None;
    }
    let mut type_name_units = input[type_start..position].to_vec();
    let (arguments, end) = if input.get(position) == Some(&(b'(' as u16)) {
        let end = find_closing_parenthesis(input, position)?;
        let arguments = split_method_arguments(&input[position + 1..end])?;
        (arguments, end)
    } else if input.get(position..position + 2) == Some(&[b'[' as u16, b']' as u16][..]) {
        type_name_units.extend_from_slice(&[b'[' as u16, b']' as u16]);
        position += 2;
        while input.get(position).is_some_and(|unit| *unit <= 0x20) {
            position += 1;
        }
        if input.get(position) != Some(&(b'{' as u16)) {
            return None;
        }
        let end = find_closing_delimiter(input, position, b'{' as u16, b'}' as u16)?;
        let arguments = split_method_arguments(&input[position + 1..end])?;
        (arguments, end)
    } else {
        return None;
    };
    let type_name = JavaString::from_utf16(type_name_units);
    let arguments = arguments
        .into_iter()
        .map(|argument| {
            parse_expression(
                &JavaString::from_utf16(argument.to_vec()),
                apply_shortcuts,
                use_selection_as_root,
            )
            .expression
        })
        .collect();
    position = end + 1;
    let trailing_steps =
        parse_suffix_steps(input, &mut position, apply_shortcuts, use_selection_as_root)?;
    Some(OgnlConstructor {
        type_name,
        arguments,
        trailing_steps,
    })
}

fn parse_suffix_steps(
    input: &[u16],
    position: &mut usize,
    apply_shortcuts: bool,
    use_selection_as_root: bool,
) -> Option<Vec<PathStep>> {
    let mut steps = Vec::new();
    while *position < input.len() {
        if input[*position] == b'.' as u16 {
            *position += 1;
            if input.get(*position) == Some(&(b'{' as u16)) {
                let end = find_closing_delimiter(input, *position, b'{' as u16, b'}' as u16)?;
                let body = java_trim_units(&input[*position + 1..end]);
                let (selection, body) = match body.first().copied() {
                    Some(value) if value == b'?' as u16 => {
                        (Some(SelectionKind::All), java_trim_units(&body[1..]))
                    }
                    Some(value) if value == b'^' as u16 => {
                        (Some(SelectionKind::First), java_trim_units(&body[1..]))
                    }
                    Some(value) if value == b'$' as u16 => {
                        (Some(SelectionKind::Last), java_trim_units(&body[1..]))
                    }
                    _ => (None, body),
                };
                let expression = parse_ognl_range(body, false, true)?;
                steps.push(match selection {
                    Some(kind) => PathStep::Selection(kind, Box::new(expression)),
                    None => PathStep::Projection(Box::new(expression)),
                });
                *position = end + 1;
                continue;
            }
            let start = *position;
            while *position < input.len() && is_ascii_identifier_part(input[*position]) {
                *position += 1;
            }
            if *position == start {
                return None;
            }
            let name = JavaString::from_utf16(input[start..*position].to_vec());
            if input.get(*position) == Some(&(b'(' as u16)) {
                let end = find_closing_parenthesis(input, *position)?;
                let arguments = split_method_arguments(&input[*position + 1..end])?
                    .into_iter()
                    .map(|argument| {
                        parse_expression(
                            &JavaString::from_utf16(argument.to_vec()),
                            apply_shortcuts,
                            use_selection_as_root,
                        )
                        .expression
                    })
                    .collect();
                steps.push(PathStep::Method(name, arguments));
                *position = end + 1;
            } else {
                steps.push(PathStep::Property(name));
            }
            continue;
        }
        if input[*position] != b'[' as u16 {
            return None;
        }
        let start = *position + 1;
        let end = find_closing_delimiter(input, *position, b'[' as u16, b']' as u16)?;
        let index = java_trim_units(&input[start..end]);
        *position = end + 1;
        if index.len() == 1
            && let Some(subscript) = match index[0] {
                value if value == b'^' as u16 => Some(OgnlDynamicSubscript::First),
                value if value == b'|' as u16 => Some(OgnlDynamicSubscript::Mid),
                value if value == b'$' as u16 => Some(OgnlDynamicSubscript::Last),
                value if value == b'*' as u16 => Some(OgnlDynamicSubscript::All),
                _ => None,
            }
        {
            steps.push(PathStep::DynamicSubscript(subscript));
        } else if let Some(OgnlLiteral::String(value)) =
            parse_literal(&JavaString::from_utf16(index.to_vec()))
        {
            steps.push(PathStep::StringIndex(value));
        } else if let Ok(value) = String::from_utf16_lossy(index).parse::<usize>() {
            steps.push(PathStep::NumericIndex(value));
        } else {
            steps.push(PathStep::DynamicIndex(Box::new(parse_ognl_range(
                index,
                apply_shortcuts,
                use_selection_as_root,
            )?)));
        }
    }
    Some(steps)
}

fn unescape_ognl_string(input: &[u16]) -> Option<Vec<u16>> {
    let mut output = Vec::with_capacity(input.len());
    let mut position = 0;
    while position < input.len() {
        if input[position] != b'\\' as u16 {
            output.push(input[position]);
            position += 1;
            continue;
        }
        position += 1;
        let escaped = *input.get(position)?;
        match escaped {
            value if value == b'n' as u16 => output.push(b'\n' as u16),
            value if value == b'r' as u16 => output.push(b'\r' as u16),
            value if value == b't' as u16 => output.push(b'\t' as u16),
            value if value == b'b' as u16 => output.push(0x08),
            value if value == b'f' as u16 => output.push(0x0c),
            value if matches!(value, 0x27 | 0x22 | 0x5c) => output.push(value),
            _ => {
                // OGNL 不把未知转义当作“删除反斜杠”。这对 `\uXXXX`
                // 这样的模板数据尤其重要：它是待输出的文本，不是 Rust/Java
                // 源代码层面的 Unicode 转义。
                output.push(b'\\' as u16);
                output.push(escaped);
            }
        }
        position += 1;
    }
    Some(output)
}

fn parse_path(
    source: &JavaString,
    apply_shortcuts: bool,
    use_selection_as_root: bool,
) -> Option<OgnlPath> {
    let input = source.as_utf16();
    let mut position = 0;
    let expression_object = input.first() == Some(&(b'#' as u16));
    if expression_object {
        position += 1;
    }
    let root_start = position;
    while position < input.len() && is_ascii_identifier_part(input[position]) {
        position += 1;
    }
    if position == root_start {
        return None;
    }
    let name = JavaString::from_utf16(input[root_start..position].to_vec());
    let root = if expression_object {
        PathRoot::ExpressionObject(name)
    } else {
        PathRoot::Context(name)
    };
    let mut steps = Vec::new();
    if input.get(position) == Some(&(b'(' as u16)) {
        let end = find_closing_parenthesis(input, position)?;
        let arguments = split_method_arguments(&input[position + 1..end])?
            .into_iter()
            .map(|argument| {
                parse_expression(
                    &JavaString::from_utf16(argument.to_vec()),
                    apply_shortcuts,
                    use_selection_as_root,
                )
                .expression
            })
            .collect();
        steps.push(PathStep::Method(
            JavaString::from_rust_str("__invoke_root__"),
            arguments,
        ));
        position = end + 1;
    }
    steps.extend(parse_suffix_steps(
        input,
        &mut position,
        apply_shortcuts,
        use_selection_as_root,
    )?);
    Some(OgnlPath { root, steps })
}

fn find_closing_parenthesis(input: &[u16], start: usize) -> Option<usize> {
    find_closing_delimiter(input, start, b'(' as u16, b')' as u16)
}

fn find_closing_delimiter(
    input: &[u16],
    start: usize,
    opening: u16,
    closing: u16,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote = None;
    let mut position = start + 1;
    while position < input.len() {
        let unit = input[position];
        if let Some(active_quote) = quote {
            if unit == active_quote {
                if input.get(position + 1) == Some(&active_quote) {
                    position += 2;
                    continue;
                }
                quote = None;
            }
        } else if matches!(unit, 0x27 | 0x22) {
            quote = Some(unit);
        } else if unit == opening {
            depth += 1;
        } else if unit == closing {
            if depth == 0 {
                return Some(position);
            }
            depth -= 1;
        }
        position += 1;
    }
    None
}

fn split_method_arguments(input: &[u16]) -> Option<Vec<&[u16]>> {
    if java_trim_units(input).is_empty() {
        return Some(Vec::new());
    }
    let mut arguments = Vec::new();
    let mut start = 0usize;
    let mut parentheses = 0usize;
    let mut brackets = 0usize;
    let mut braces = 0usize;
    let mut quote = None;
    let mut position = 0usize;
    while position < input.len() {
        let unit = input[position];
        if let Some(active_quote) = quote {
            if unit == active_quote {
                if input.get(position + 1) == Some(&active_quote) {
                    position += 2;
                    continue;
                }
                quote = None;
            }
        } else if matches!(unit, 0x27 | 0x22) {
            quote = Some(unit);
        } else if unit == b'(' as u16 {
            parentheses += 1;
        } else if unit == b')' as u16 {
            parentheses = parentheses.checked_sub(1)?;
        } else if unit == b'[' as u16 {
            brackets += 1;
        } else if unit == b']' as u16 {
            brackets = brackets.checked_sub(1)?;
        } else if unit == b'{' as u16 {
            braces += 1;
        } else if unit == b'}' as u16 {
            braces = braces.checked_sub(1)?;
        } else if unit == b',' as u16 && parentheses == 0 && brackets == 0 && braces == 0 {
            arguments.push(java_trim_units(&input[start..position]));
            start = position + 1;
        }
        position += 1;
    }
    if quote.is_some() || parentheses != 0 || brackets != 0 || braces != 0 {
        return None;
    }
    arguments.push(java_trim_units(&input[start..]));
    Some(arguments)
}

fn evaluate_path(
    context: &dyn IExpressionContext,
    path: &OgnlPath,
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let restrict_variable_access = expression_context.get_restrict_variable_access();
    let value = match &path.root {
        PathRoot::Context(name) => {
            if restrict_variable_access && name == &JavaString::from_rust_str("param") {
                return Err(processing_error(
                    "Access to variable \"param\" is forbidden in this context.".to_owned(),
                ));
            }
            if use_selection_as_root {
                let selection = current_projection_root().or_else(|| {
                    context
                        .as_template_context()
                        .filter(|template_context| template_context.has_selection_target())
                        .and_then(crate::context::ITemplateContext::get_selection_target)
                });
                match selection {
                    Some(selection) => read_dynamic_property(selection.as_ref(), name)?,
                    None => context.get_variable(Some(name)),
                }
            } else {
                context.get_variable(Some(name))
            }
        }
        PathRoot::ExpressionObject(name) => {
            if let Some(value) = current_ognl_local(name) {
                return evaluate_path_steps(
                    context,
                    path,
                    value,
                    use_selection_as_root,
                    expression_context,
                );
            }
            if name == &JavaString::from_rust_str("this") {
                let root = if let Some(root) = current_projection_root() {
                    Some(root)
                } else if use_selection_as_root {
                    let selection = context
                        .as_template_context()
                        .filter(|template_context| template_context.has_selection_target())
                        .and_then(crate::context::ITemplateContext::get_selection_target);
                    match selection {
                        Some(selection) => Some(selection),
                        None => context
                            .get_expression_objects()
                            .get_object(Some(&JavaString::from_rust_str("root")))?,
                    }
                } else {
                    context
                        .get_expression_objects()
                        .get_object(Some(&JavaString::from_rust_str("root")))?
                };
                return evaluate_path_steps(
                    context,
                    path,
                    root,
                    use_selection_as_root,
                    expression_context,
                );
            }
            if restrict_variable_access && NativeExpressionObjectsWrapper::is_restricted(Some(name))
            {
                return Err(processing_error(format!(
                    "Access to variable '#{}' is forbidden in this context.",
                    name.to_string_lossy()
                )));
            }
            context.get_expression_objects().get_object(Some(name))?
        }
    };
    evaluate_path_steps(
        context,
        path,
        value,
        use_selection_as_root,
        expression_context,
    )
}

fn evaluate_path_steps(
    context: &dyn IExpressionContext,
    path: &OgnlPath,
    value: Option<Arc<TemplateValue>>,
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let root_method_name = path_root_method_name(&path.root);
    evaluate_navigation_steps(
        context,
        &path.steps,
        value,
        use_selection_as_root,
        expression_context,
        Some(&root_method_name),
    )
}

fn evaluate_navigation_steps(
    context: &dyn IExpressionContext,
    steps: &[PathStep],
    value: Option<Arc<TemplateValue>>,
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
    root_method_name: Option<&JavaString>,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let mut value = normalize_java_null(value);
    for step in steps {
        let target = value.as_deref().ok_or_else(|| {
            ognl_processing_error(
                "source is null while evaluating OGNL property path".to_owned(),
                "source is null while evaluating OGNL property path".to_owned(),
            )
        })?;
        value = normalize_java_null(match step {
            PathStep::Property(name) | PathStep::StringIndex(name) => {
                read_dynamic_property(target, name)?
            }
            PathStep::Method(name, arguments) => {
                let arguments = arguments
                    .iter()
                    .map(|argument| {
                        evaluate_computed_expression(
                            context,
                            argument,
                            use_selection_as_root,
                            expression_context,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if name == &JavaString::from_rust_str("__invoke_root__") {
                    let root_method_name = root_method_name.ok_or_else(|| {
                        processing_error("OGNL root invocation has no root method name".to_owned())
                    })?;
                    invoke_dynamic_method(target, root_method_name, &arguments)?
                } else {
                    invoke_dynamic_method(target, name, &arguments)?
                }
            }
            PathStep::Projection(expression) => {
                let values = iterable_values(target).ok_or_else(|| {
                    processing_error(format!(
                        "projection cannot be applied to {}",
                        target.java_class_name()
                    ))
                })?;
                let projected = values
                    .iter()
                    .map(|item| {
                        with_projection_root(Arc::clone(item), || {
                            evaluate_computed_expression(
                                context,
                                expression,
                                true,
                                expression_context,
                            )
                            .map(|value| value.unwrap_or_else(|| Arc::new(TemplateValue::Null)))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Some(Arc::new(TemplateValue::List(Arc::new(projected))))
            }
            PathStep::Selection(kind, expression) => {
                let values = iterable_values(target).ok_or_else(|| {
                    processing_error(format!(
                        "selection cannot be applied to {}",
                        target.java_class_name()
                    ))
                })?;
                let mut selected = Vec::new();
                for item in values {
                    let result = with_projection_root(Arc::clone(&item), || {
                        evaluate_computed_expression(context, expression, true, expression_context)
                    })?;
                    if evaluate_as_boolean(result.as_ref())? {
                        selected.push(item);
                        if matches!(kind, SelectionKind::First) {
                            break;
                        }
                    }
                }
                if matches!(kind, SelectionKind::Last) {
                    selected = selected.into_iter().last().into_iter().collect();
                }
                Some(Arc::new(TemplateValue::List(Arc::new(selected))))
            }
            PathStep::DynamicIndex(expression) => {
                let index = evaluate_computed_expression(
                    context,
                    expression,
                    use_selection_as_root,
                    expression_context,
                )?
                .unwrap_or_else(|| Arc::new(TemplateValue::Null));
                match target {
                    TemplateValue::Map(entries) => entries
                        .iter()
                        .find(|(key, _)| key.java_equals(index.as_ref()))
                        .map(|(_, value)| Arc::clone(value)),
                    TemplateValue::List(values) => {
                        let index = ognl_list_index(&index).ok_or_else(|| {
                            processing_error("list index is not an integer".to_owned())
                        })?;
                        Some(values.get(index).cloned().ok_or_else(|| {
                            processing_error(format!("index {index} is out of bounds"))
                        })?)
                    }
                    _ => {
                        return Err(processing_error(format!(
                            "dynamic index cannot be applied to {}",
                            target.java_class_name()
                        )));
                    }
                }
            }
            PathStep::DynamicSubscript(subscript) => {
                evaluate_dynamic_subscript(target, *subscript)?
            }
            PathStep::NumericIndex(index) => match target {
                TemplateValue::List(values) => values
                    .get(*index)
                    .cloned()
                    .ok_or_else(|| processing_error(format!("index {index} is out of bounds")))?,
                TemplateValue::Bytes(values) => values
                    .get(*index)
                    .map(|value| Arc::new(TemplateValue::Number(JavaNumber::Byte(*value))))
                    .ok_or_else(|| processing_error(format!("index {index} is out of bounds")))?,
                TemplateValue::String(value) | TemplateValue::SafeHtml(value) => value
                    .as_utf16()
                    .get(*index)
                    .map(|value| Arc::new(TemplateValue::Character(*value)))
                    .ok_or_else(|| processing_error(format!("index {index} is out of bounds")))?,
                TemplateValue::Object(value) if value.java_iterable_values().is_some() => value
                    .java_iterable_values()
                    .and_then(|values| values.get(*index).cloned())
                    .ok_or_else(|| processing_error(format!("index {index} is out of bounds")))?,
                _ => {
                    return Err(processing_error(format!(
                        "numeric index cannot be applied to {}",
                        target.java_class_name()
                    )));
                }
            }
            .into(),
        });
    }
    Ok(value)
}

fn normalize_java_null(value: Option<Arc<TemplateValue>>) -> Option<Arc<TemplateValue>> {
    value.filter(|value| !matches!(value.as_ref(), TemplateValue::Null))
}

fn evaluate_dynamic_subscript(
    target: &TemplateValue,
    subscript: OgnlDynamicSubscript,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let values = match target {
        TemplateValue::List(values) => values.as_ref().clone(),
        TemplateValue::Bytes(values) => values
            .iter()
            .map(|value| Arc::new(TemplateValue::Number(JavaNumber::Byte(*value))))
            .collect(),
        TemplateValue::String(value) | TemplateValue::SafeHtml(value) => value
            .as_utf16()
            .iter()
            .map(|value| Arc::new(TemplateValue::Character(*value)))
            .collect(),
        _ => {
            return Err(processing_error(format!(
                "dynamic subscript cannot be applied to {}",
                target.java_class_name()
            )));
        }
    };
    if matches!(subscript, OgnlDynamicSubscript::All) {
        return Ok(Some(Arc::new(TemplateValue::List(Arc::new(values)))));
    }
    if values.is_empty() {
        return Ok(None);
    }
    let index = match subscript {
        OgnlDynamicSubscript::First => 0,
        OgnlDynamicSubscript::Mid => values.len() / 2,
        OgnlDynamicSubscript::Last => values.len() - 1,
        OgnlDynamicSubscript::All => unreachable!("all was handled above"),
    };
    Ok(values.get(index).cloned())
}

fn iterable_values(target: &TemplateValue) -> Option<Vec<Arc<TemplateValue>>> {
    match target {
        TemplateValue::List(values) => Some(values.as_ref().clone()),
        // OGNL 将 Map 作为其 values 集合参与 projection/selection。
        TemplateValue::Map(entries) => {
            Some(entries.iter().map(|(_, value)| Arc::clone(value)).collect())
        }
        TemplateValue::Bytes(values) => Some(
            values
                .iter()
                .map(|value| Arc::new(TemplateValue::Number(JavaNumber::Byte(*value))))
                .collect(),
        ),
        TemplateValue::Object(value) => value.java_iterable_values(),
        _ => None,
    }
}

thread_local! {
    static PROJECTION_ROOTS: RefCell<Vec<Arc<TemplateValue>>> = const { RefCell::new(Vec::new()) };
}

fn current_projection_root() -> Option<Arc<TemplateValue>> {
    PROJECTION_ROOTS.with(|roots| roots.borrow().last().cloned())
}

fn with_projection_root<T>(root: Arc<TemplateValue>, operation: impl FnOnce() -> T) -> T {
    PROJECTION_ROOTS.with(|roots| roots.borrow_mut().push(root));
    struct ProjectionRootGuard;
    impl Drop for ProjectionRootGuard {
        fn drop(&mut self) {
            PROJECTION_ROOTS.with(|roots| {
                roots.borrow_mut().pop();
            });
        }
    }
    let _guard = ProjectionRootGuard;
    operation()
}

fn evaluate_computed_expression(
    context: &dyn IExpressionContext,
    expression: &ComputedExpression,
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let restrict_variable_access = expression_context.get_restrict_variable_access();
    match expression {
        ComputedExpression::Shortcut(shortcut) => shortcut
            .evaluate(context, use_selection_as_root, restrict_variable_access)
            .map_err(|error| processing_error(error.to_string())),
        ComputedExpression::Path(path) => {
            evaluate_path(context, path, use_selection_as_root, expression_context)
        }
        ComputedExpression::Literal(value) => Ok(value.to_template_value()),
        ComputedExpression::Operation(expression) => {
            expression.execute_with_context(context, expression_context)
        }
        ComputedExpression::StaticReference(reference) => evaluate_static_reference(
            context,
            reference,
            use_selection_as_root,
            expression_context,
        ),
        ComputedExpression::Constructor(constructor) => evaluate_constructor(
            context,
            constructor,
            use_selection_as_root,
            expression_context,
        ),
        ComputedExpression::ListLiteral(values) => {
            evaluate_list_literal(context, values, use_selection_as_root, expression_context)
        }
        ComputedExpression::MapLiteral(entries) => {
            evaluate_map_literal(context, entries, use_selection_as_root, expression_context)
        }
        ComputedExpression::Inclusion {
            left,
            right,
            negated,
        } => evaluate_inclusion(
            context,
            left,
            right,
            *negated,
            use_selection_as_root,
            expression_context,
        ),
        ComputedExpression::Sequence(values) => {
            evaluate_sequence(context, values, use_selection_as_root, expression_context)
        }
        ComputedExpression::Assignment { name, value } => evaluate_assignment(
            context,
            name,
            value,
            use_selection_as_root,
            expression_context,
        ),
        ComputedExpression::NativeBinary {
            operator,
            left,
            right,
        } => evaluate_native_binary(
            context,
            *operator,
            left,
            right,
            use_selection_as_root,
            expression_context,
        ),
        ComputedExpression::BitNegate(value) => {
            evaluate_bit_negate(context, value, use_selection_as_root, expression_context)
        }
        ComputedExpression::InstanceOf { value, type_name } => evaluate_instance_of(
            context,
            value,
            type_name,
            use_selection_as_root,
            expression_context,
        ),
        ComputedExpression::Navigation { root, steps } => evaluate_navigation(
            context,
            root,
            steps,
            use_selection_as_root,
            expression_context,
        ),
        ComputedExpression::Unsupported => Err(processing_error(
            "unsupported OGNL method argument syntax".to_owned(),
        )),
    }
}

fn evaluate_navigation(
    context: &dyn IExpressionContext,
    root: &ComputedExpression,
    steps: &[PathStep],
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let value =
        evaluate_computed_expression(context, root, use_selection_as_root, expression_context)?;
    evaluate_navigation_steps(
        context,
        steps,
        value,
        use_selection_as_root,
        expression_context,
        None,
    )
}

fn evaluate_sequence(
    context: &dyn IExpressionContext,
    values: &[ComputedExpression],
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let mut result = None;
    for value in values {
        result = evaluate_computed_expression(
            context,
            value,
            use_selection_as_root,
            expression_context,
        )?;
    }
    Ok(result)
}

fn evaluate_assignment(
    context: &dyn IExpressionContext,
    name: &JavaString,
    value: &ComputedExpression,
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    if context.get_expression_objects().contains_object(Some(name)) {
        return Err(processing_error(format!(
            "Cannot put entry with key \"{}\" into Expression Objects wrapper map: key matches the name of one of the expression objects",
            name.to_string_lossy()
        )));
    }
    let value =
        evaluate_computed_expression(context, value, use_selection_as_root, expression_context)?;
    set_ognl_local(name.clone(), value.clone());
    Ok(value)
}

fn evaluate_native_binary(
    context: &dyn IExpressionContext,
    operator: OgnlBinaryOperator,
    left: &ComputedExpression,
    right: &ComputedExpression,
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let left =
        evaluate_computed_expression(context, left, use_selection_as_root, expression_context)?
            .unwrap_or_else(|| Arc::new(TemplateValue::Null));
    let right =
        evaluate_computed_expression(context, right, use_selection_as_root, expression_context)?
            .unwrap_or_else(|| Arc::new(TemplateValue::Null));
    if matches!(operator, OgnlBinaryOperator::Divide) {
        return evaluate_ognl_division(left, right);
    }
    let left = numeric_i64(left.as_ref())?;
    let right = numeric_i64(right.as_ref())?;
    let value = match operator {
        OgnlBinaryOperator::Divide => unreachable!("division is handled before integral coercion"),
        OgnlBinaryOperator::BitOr => left | right,
        OgnlBinaryOperator::BitXor => left ^ right,
        OgnlBinaryOperator::BitAnd => left & right,
        OgnlBinaryOperator::ShiftLeft => {
            left.wrapping_shl(u32::try_from(right & 0x3f).unwrap_or_default())
        }
        OgnlBinaryOperator::ShiftRight => {
            left.wrapping_shr(u32::try_from(right & 0x3f).unwrap_or_default())
        }
        OgnlBinaryOperator::UnsignedShiftRight => {
            ((left as u64) >> u32::try_from(right & 0x3f).unwrap_or_default()) as i64
        }
    };
    Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Long(
        value,
    )))))
}

fn evaluate_ognl_division(
    left: Arc<TemplateValue>,
    right: Arc<TemplateValue>,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let left_number = match left.as_ref() {
        TemplateValue::Number(value) => Some(value),
        _ => None,
    };
    let right_number = match right.as_ref() {
        TemplateValue::Number(value) => Some(value),
        _ => None,
    };
    if let (Some(left_number), Some(right_number)) = (left_number, right_number)
        && is_integral_number(left_number)
        && is_integral_number(right_number)
    {
        if matches!(left_number, JavaNumber::BigInteger(_))
            || matches!(right_number, JavaNumber::BigInteger(_))
        {
            let dividend = integral_bigint(left_number);
            let divisor = integral_bigint(right_number);
            if divisor == BigInt::from(0) {
                return Err(processing_error("Division by zero".to_owned()));
            }
            return Ok(Some(Arc::new(TemplateValue::Number(
                JavaNumber::BigInteger(dividend / divisor),
            ))));
        }
        let divisor = numeric_i64(right.as_ref())?;
        if divisor == 0 {
            return Err(processing_error("Division by zero".to_owned()));
        }
        let quotient = numeric_i64(left.as_ref())? / divisor;
        let result = if matches!(left_number, JavaNumber::Long(_))
            || matches!(right_number, JavaNumber::Long(_))
        {
            JavaNumber::Long(quotient)
        } else {
            JavaNumber::Integer(i32::try_from(quotient).map_err(|error| {
                processing_error(format!("Integer division result is out of range: {error}"))
            })?)
        };
        return Ok(Some(Arc::new(TemplateValue::Number(result))));
    }

    let left_number = evaluate_as_number(Some(&left))?
        .ok_or_else(|| processing_error("Left division operand is not numeric".to_owned()))?;
    let right_number = evaluate_as_number(Some(&right))?
        .ok_or_else(|| processing_error("Right division operand is not numeric".to_owned()))?;
    let result = match left_number.divide_java(&right_number) {
        Ok(result) => result,
        Err(_) => {
            let scale = left_number.scale().max(right_number.scale()).max(10);
            left_number.divide_java_half_up(&right_number, scale)?
        }
    };
    Ok(Some(Arc::new(TemplateValue::Number(
        JavaNumber::BigDecimal(result),
    ))))
}

fn is_integral_number(number: &JavaNumber) -> bool {
    matches!(
        number,
        JavaNumber::BigInteger(_)
            | JavaNumber::Byte(_)
            | JavaNumber::Short(_)
            | JavaNumber::Integer(_)
            | JavaNumber::Long(_)
    )
}

fn integral_bigint(number: &JavaNumber) -> BigInt {
    match number {
        JavaNumber::BigInteger(value) => value.clone(),
        JavaNumber::Byte(value) => BigInt::from(*value),
        JavaNumber::Short(value) => BigInt::from(*value),
        JavaNumber::Integer(value) => BigInt::from(*value),
        JavaNumber::Long(value) => BigInt::from(*value),
        _ => unreachable!("caller verifies the integral number family"),
    }
}

fn evaluate_bit_negate(
    context: &dyn IExpressionContext,
    value: &ComputedExpression,
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let value =
        evaluate_computed_expression(context, value, use_selection_as_root, expression_context)?
            .unwrap_or_else(|| Arc::new(TemplateValue::Null));
    Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Long(
        !numeric_i64(value.as_ref())?,
    )))))
}

fn evaluate_instance_of(
    context: &dyn IExpressionContext,
    value: &ComputedExpression,
    type_name: &JavaString,
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    ThymeleafACLClassResolver::class_for_name(&type_name.to_string_lossy())?;
    let value =
        evaluate_computed_expression(context, value, use_selection_as_root, expression_context)?;
    let result = match value.as_deref() {
        None | Some(TemplateValue::Null) => false,
        Some(value) => {
            if let Some(result) =
                current_ognl_runtime().and_then(|runtime| runtime.is_instance_of(value, type_name))
            {
                result.map_err(|error| processing_error(error.to_string()))?
            } else {
                builtin_instance_of(value, &type_name.to_string_lossy())
            }
        }
    };
    Ok(Some(Arc::new(TemplateValue::Boolean(result))))
}

fn builtin_instance_of(value: &TemplateValue, type_name: &str) -> bool {
    if value.java_class_name() == type_name || type_name == "java.lang.Object" {
        return true;
    }
    match value {
        TemplateValue::String(_) | TemplateValue::SafeHtml(_) => {
            matches!(
                type_name,
                "java.lang.String"
                    | "java.lang.CharSequence"
                    | "java.io.Serializable"
                    | "java.lang.Comparable"
            )
        }
        TemplateValue::Number(_) => matches!(
            type_name,
            "java.lang.Number" | "java.io.Serializable" | "java.lang.Comparable"
        ),
        TemplateValue::Boolean(_) | TemplateValue::Character(_) => {
            matches!(type_name, "java.io.Serializable" | "java.lang.Comparable")
        }
        TemplateValue::List(_) => matches!(
            type_name,
            "java.util.List"
                | "java.util.Collection"
                | "java.lang.Iterable"
                | "java.io.Serializable"
        ),
        TemplateValue::Map(_) => {
            matches!(type_name, "java.util.Map" | "java.io.Serializable")
        }
        TemplateValue::Bytes(_) => matches!(
            type_name,
            "byte[]" | "[B" | "java.lang.Cloneable" | "java.io.Serializable"
        ),
        TemplateValue::Object(value) => value.java_class_name() == type_name,
        TemplateValue::Literal(_) | TemplateValue::NoOp | TemplateValue::Null => false,
    }
}

/// 将动态索引转换为列表下标，对应 Java OGNL `OgnlOps.getIntValue`
/// （Double/BigDecimal 截断为 int；字符串按数字解析）。
fn ognl_list_index(value: &TemplateValue) -> Option<usize> {
    match value {
        TemplateValue::Number(number) => Some(truncated_i64(number)? as usize),
        other => other
            .to_java_string()
            .and_then(|value| value.to_string_lossy().parse::<usize>().ok()),
    }
}

fn truncated_i64(number: &JavaNumber) -> Option<i64> {
    match number {
        JavaNumber::Byte(value) => Some(i64::from(*value)),
        JavaNumber::Short(value) => Some(i64::from(*value)),
        JavaNumber::Integer(value) => Some(i64::from(*value)),
        JavaNumber::Long(value) => Some(*value),
        JavaNumber::Float(value) => Some(*value as i64),
        JavaNumber::Double(value) => Some(*value as i64),
        JavaNumber::BigDecimal(value) => {
            let divisor = BigInt::from(10_u32).pow(u32::try_from(value.scale()).unwrap_or(0));
            (value.unscaled_value() / divisor).to_string().parse().ok()
        }
        JavaNumber::BigInteger(value) => value.to_string().parse().ok(),
        JavaNumber::Other { double_value, .. } => Some(*double_value as i64),
    }
}

fn numeric_i64(value: &TemplateValue) -> StandardExpressionResult<i64> {
    match value {
        TemplateValue::Number(JavaNumber::Byte(value)) => Ok(i64::from(*value)),
        TemplateValue::Number(JavaNumber::Short(value)) => Ok(i64::from(*value)),
        TemplateValue::Number(JavaNumber::Integer(value)) => Ok(i64::from(*value)),
        TemplateValue::Number(JavaNumber::Long(value)) => Ok(*value),
        TemplateValue::Number(JavaNumber::Float(value)) => Ok(*value as i64),
        TemplateValue::Number(JavaNumber::Double(value))
        | TemplateValue::Number(JavaNumber::Other {
            double_value: value,
            ..
        }) => Ok(*value as i64),
        TemplateValue::Number(JavaNumber::BigInteger(value)) => {
            value.to_string().parse().map_err(|error| {
                processing_error(format!("Value cannot be represented as long: {error}"))
            })
        }
        TemplateValue::Number(JavaNumber::BigDecimal(value)) => value
            .to_string()
            .parse::<f64>()
            .map(|value| value as i64)
            .map_err(|error| {
                processing_error(format!("Value cannot be represented as long: {error}"))
            }),
        TemplateValue::Boolean(value) => Ok(i64::from(*value)),
        TemplateValue::Character(value) => Ok(i64::from(*value)),
        _ => Err(processing_error(format!(
            "{} cannot be converted to an integral number",
            value.java_class_name()
        ))),
    }
}

fn evaluate_list_literal(
    context: &dyn IExpressionContext,
    values: &[ComputedExpression],
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let values = values
        .iter()
        .map(|value| {
            evaluate_computed_expression(context, value, use_selection_as_root, expression_context)
                .map(|value| value.unwrap_or_else(|| Arc::new(TemplateValue::Null)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(Arc::new(TemplateValue::List(Arc::new(values)))))
}

fn evaluate_map_literal(
    context: &dyn IExpressionContext,
    entries: &[(Box<ComputedExpression>, Box<ComputedExpression>)],
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let entries = entries
        .iter()
        .map(|(key, value)| {
            let key = evaluate_computed_expression(
                context,
                key,
                use_selection_as_root,
                expression_context,
            )?
            .unwrap_or_else(|| Arc::new(TemplateValue::Null));
            let value = evaluate_computed_expression(
                context,
                value,
                use_selection_as_root,
                expression_context,
            )?
            .unwrap_or_else(|| Arc::new(TemplateValue::Null));
            Ok((key, value))
        })
        .collect::<StandardExpressionResult<Vec<_>>>()?;
    Ok(Some(Arc::new(TemplateValue::Map(Arc::new(entries)))))
}

fn evaluate_inclusion(
    context: &dyn IExpressionContext,
    left: &ComputedExpression,
    right: &ComputedExpression,
    negated: bool,
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let left =
        evaluate_computed_expression(context, left, use_selection_as_root, expression_context)?
            .unwrap_or_else(|| Arc::new(TemplateValue::Null));
    let right =
        evaluate_computed_expression(context, right, use_selection_as_root, expression_context)?
            .unwrap_or_else(|| Arc::new(TemplateValue::Null));
    let contains = match right.as_ref() {
        TemplateValue::List(values) => values.iter().any(|value| value.java_equals(&left)),
        TemplateValue::Map(entries) => entries.iter().any(|(_, value)| value.java_equals(&left)),
        TemplateValue::Bytes(values) => values
            .iter()
            .any(|value| TemplateValue::Number(JavaNumber::Byte(*value)).java_equals(&left)),
        TemplateValue::Object(value) => value
            .java_iterable_values()
            .is_some_and(|values| values.iter().any(|value| value.java_equals(&left))),
        value => value.java_equals(&left),
    };
    Ok(Some(Arc::new(TemplateValue::Boolean(if negated {
        !contains
    } else {
        contains
    }))))
}

fn evaluate_static_reference(
    context: &dyn IExpressionContext,
    reference: &OgnlStaticReference,
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let type_name = reference.type_name.to_string_lossy();
    if expression_context.get_restrict_external_access() {
        return Err(processing_error(format!(
            "Access to type \"{type_name}\" is forbidden"
        )));
    }
    let member = reference.member_name.to_string_lossy();
    let value = if let Some(arguments) = &reference.arguments {
        let arguments = arguments
            .iter()
            .map(|argument| {
                evaluate_computed_expression(
                    context,
                    argument,
                    use_selection_as_root,
                    expression_context,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        invoke_static_method(&type_name, &member, &arguments)?
    } else {
        read_static_field(&type_name, &member)?
    };
    evaluate_navigation_steps(
        context,
        &reference.trailing_steps,
        value,
        use_selection_as_root,
        expression_context,
        None,
    )
}

fn evaluate_constructor(
    context: &dyn IExpressionContext,
    constructor: &OgnlConstructor,
    use_selection_as_root: bool,
    expression_context: &'static StandardExpressionExecutionContext,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let type_name = constructor.type_name.to_string_lossy();
    if expression_context.get_restrict_external_access() {
        return Err(processing_error(format!(
            "Instantiation of type \"{type_name}\" is forbidden"
        )));
    }
    let arguments = constructor
        .arguments
        .iter()
        .map(|argument| {
            evaluate_computed_expression(
                context,
                argument,
                use_selection_as_root,
                expression_context,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let value = if let Some(result) = current_ognl_runtime()
        .and_then(|runtime| runtime.construct(&constructor.type_name, &arguments))
    {
        result.map_err(|error| processing_error(error.to_string()))?
    } else {
        ThymeleafACLClassResolver::class_for_name(&type_name)?;
        match (type_name.as_str(), arguments.as_slice()) {
            ("java.lang.String", []) => Ok(Some(Arc::new(TemplateValue::string(
                JavaString::from_rust_str(""),
            )))),
            ("java.lang.String", [value]) => Ok(Some(Arc::new(TemplateValue::string(
                value
                    .as_deref()
                    .and_then(TemplateValue::to_java_string)
                    .unwrap_or_else(|| JavaString::from_rust_str("null")),
            )))),
            ("java.math.BigDecimal", [Some(value)]) => {
                let text = value.to_java_string().ok_or_else(|| {
                    processing_error("BigDecimal constructor argument cannot be null".to_owned())
                })?;
                let value = JavaBigDecimal::parse(&text.to_string_lossy())
                    .map_err(|error| processing_error(format!("Invalid BigDecimal: {error}")))?;
                Ok(Some(Arc::new(TemplateValue::Number(
                    JavaNumber::BigDecimal(value),
                ))))
            }
            ("java.math.BigInteger", [Some(value)]) => {
                let text = value.to_java_string().ok_or_else(|| {
                    processing_error("BigInteger constructor argument cannot be null".to_owned())
                })?;
                let value = text
                    .to_string_lossy()
                    .parse()
                    .map_err(|error| processing_error(format!("Invalid BigInteger: {error}")))?;
                Ok(Some(Arc::new(TemplateValue::Number(
                    JavaNumber::BigInteger(value),
                ))))
            }
            ("java.util.ArrayList" | "java.util.LinkedList", []) => {
                Ok(Some(Arc::new(TemplateValue::List(Arc::new(Vec::new())))))
            }
            ("java.util.HashMap" | "java.util.LinkedHashMap", []) => {
                Ok(Some(Arc::new(TemplateValue::Map(Arc::new(Vec::new())))))
            }
            ("java.util.HashMap" | "java.util.LinkedHashMap", [Some(value)])
                if matches!(value.as_ref(), TemplateValue::Map(_)) =>
            {
                Ok(Some(Arc::clone(value)))
            }
            _ => Err(processing_error(format!(
                "Constructor for type \"{type_name}\" with {} arguments is not available",
                arguments.len()
            ))),
        }?
    };
    evaluate_navigation_steps(
        context,
        &constructor.trailing_steps,
        value,
        use_selection_as_root,
        expression_context,
        None,
    )
}

fn read_static_field(
    type_name: &str,
    member: &str,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let runtime_type_name = JavaString::from_rust_str(type_name);
    let runtime_member_name = JavaString::from_rust_str(member);
    if let Some(result) = current_ognl_runtime()
        .and_then(|runtime| runtime.read_static_field(&runtime_type_name, &runtime_member_name))
    {
        return result.map_err(|error| processing_error(error.to_string()));
    }
    ThymeleafACLClassResolver::class_for_name(type_name)?;
    if member == "class" {
        return Ok(Some(java_class_value(type_name)));
    }
    let value = match (type_name, member) {
        ("java.lang.Math", "PI") => TemplateValue::Number(JavaNumber::Double(std::f64::consts::PI)),
        ("java.lang.Math", "E") => TemplateValue::Number(JavaNumber::Double(std::f64::consts::E)),
        ("java.lang.Boolean", "TRUE") => TemplateValue::Boolean(true),
        ("java.lang.Boolean", "FALSE") => TemplateValue::Boolean(false),
        ("java.lang.Integer", "MAX_VALUE") => TemplateValue::Number(JavaNumber::Integer(i32::MAX)),
        ("java.lang.Integer", "MIN_VALUE") => TemplateValue::Number(JavaNumber::Integer(i32::MIN)),
        ("java.lang.Long", "MAX_VALUE") => TemplateValue::Number(JavaNumber::Long(i64::MAX)),
        ("java.lang.Long", "MIN_VALUE") => TemplateValue::Number(JavaNumber::Long(i64::MIN)),
        ("java.math.BigInteger", "ZERO") | ("java.math.BigDecimal", "ZERO") => {
            TemplateValue::Number(JavaNumber::Integer(0))
        }
        ("java.math.BigInteger", "ONE") | ("java.math.BigDecimal", "ONE") => {
            TemplateValue::Number(JavaNumber::Integer(1))
        }
        ("java.math.BigInteger", "TEN") | ("java.math.BigDecimal", "TEN") => {
            TemplateValue::Number(JavaNumber::Integer(10))
        }
        ("java.util.Calendar", "HOUR_OF_DAY") => TemplateValue::Number(JavaNumber::Integer(11)),
        ("java.util.Calendar", "MINUTE") => TemplateValue::Number(JavaNumber::Integer(12)),
        ("java.util.Calendar", "SECOND") => TemplateValue::Number(JavaNumber::Integer(13)),
        ("java.util.Calendar", "MILLISECOND") => TemplateValue::Number(JavaNumber::Integer(14)),
        ("java.util.Calendar", "DAY_OF_MONTH" | "DATE") => {
            TemplateValue::Number(JavaNumber::Integer(5))
        }
        ("java.util.Calendar", "MONTH") => TemplateValue::Number(JavaNumber::Integer(2)),
        ("java.util.Calendar", "YEAR") => TemplateValue::Number(JavaNumber::Integer(1)),
        ("org.thymeleaf.TemplateEngine", "TIMER_LOGGER_NAME") => TemplateValue::string(
            JavaString::from_rust_str("org.thymeleaf.TemplateEngine.TIMER"),
        ),
        _ => {
            return Err(processing_error(format!(
                "Static field \"{member}\" is not available on {type_name}"
            )));
        }
    };
    Ok(Some(Arc::new(value)))
}

fn invoke_static_method(
    type_name: &str,
    member: &str,
    arguments: &[Option<Arc<TemplateValue>>],
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let runtime_type_name = JavaString::from_rust_str(type_name);
    let runtime_member_name = JavaString::from_rust_str(member);
    if let Some(result) = current_ognl_runtime().and_then(|runtime| {
        runtime.invoke_static_method(&runtime_type_name, &runtime_member_name, arguments)
    }) {
        return result.map_err(|error| processing_error(error.to_string()));
    }
    ThymeleafACLClassResolver::class_for_name(type_name)?;
    match (type_name, member, arguments) {
        ("java.lang.Math", "abs", [Some(value)]) => {
            let value = numeric_f64(value)?;
            number_result(value.abs())
        }
        ("java.lang.Math", "ceil", [Some(value)]) => number_result(numeric_f64(value)?.ceil()),
        ("java.lang.Math", "floor", [Some(value)]) => number_result(numeric_f64(value)?.floor()),
        ("java.lang.Math", "sqrt", [Some(value)]) => number_result(numeric_f64(value)?.sqrt()),
        ("java.lang.Math", "cbrt", [Some(value)]) => number_result(numeric_f64(value)?.cbrt()),
        ("java.lang.Math", "sin", [Some(value)]) => number_result(numeric_f64(value)?.sin()),
        ("java.lang.Math", "cos", [Some(value)]) => number_result(numeric_f64(value)?.cos()),
        ("java.lang.Math", "tan", [Some(value)]) => number_result(numeric_f64(value)?.tan()),
        ("java.lang.Math", "log", [Some(value)]) => number_result(numeric_f64(value)?.ln()),
        ("java.lang.Math", "log10", [Some(value)]) => number_result(numeric_f64(value)?.log10()),
        ("java.lang.Math", "exp", [Some(value)]) => number_result(numeric_f64(value)?.exp()),
        ("java.lang.Math", "pow", [Some(left), Some(right)]) => {
            number_result(numeric_f64(left)?.powf(numeric_f64(right)?))
        }
        ("java.lang.Math", "min", [Some(left), Some(right)]) => {
            number_result(numeric_f64(left)?.min(numeric_f64(right)?))
        }
        ("java.lang.Math", "max", [Some(left), Some(right)]) => {
            number_result(numeric_f64(left)?.max(numeric_f64(right)?))
        }
        ("java.lang.Math", "round", [Some(value)]) => Ok(Some(Arc::new(TemplateValue::Number(
            JavaNumber::Long(numeric_f64(value)?.round() as i64),
        )))),
        ("java.lang.Integer", "parseInt" | "valueOf", [Some(value)]) => {
            let value = required_java_string(value, "Integer text cannot be null")?;
            let parsed = value
                .to_string_lossy()
                .parse::<i32>()
                .map_err(|error| processing_error(error.to_string()))?;
            Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(
                parsed,
            )))))
        }
        ("java.lang.Byte", "parseByte" | "valueOf", [Some(value)]) => {
            let value = required_java_string(value, "Byte text cannot be null")?;
            let parsed = value
                .to_string_lossy()
                .parse::<i8>()
                .map_err(|error| processing_error(error.to_string()))?;
            Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Byte(
                parsed,
            )))))
        }
        ("java.lang.Short", "parseShort" | "valueOf", [Some(value)]) => {
            let value = required_java_string(value, "Short text cannot be null")?;
            let parsed = value
                .to_string_lossy()
                .parse::<i16>()
                .map_err(|error| processing_error(error.to_string()))?;
            Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Short(
                parsed,
            )))))
        }
        ("java.lang.Long", "parseLong" | "valueOf", [Some(value)]) => {
            let value = required_java_string(value, "Long text cannot be null")?;
            let parsed = value
                .to_string_lossy()
                .parse::<i64>()
                .map_err(|error| processing_error(error.to_string()))?;
            Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Long(
                parsed,
            )))))
        }
        ("java.lang.Double", "parseDouble" | "valueOf", [Some(value)]) => {
            let value = required_java_string(value, "Double text cannot be null")?;
            let parsed = value
                .to_string_lossy()
                .parse::<f64>()
                .map_err(|error| processing_error(error.to_string()))?;
            number_result(parsed)
        }
        ("java.lang.Boolean", "parseBoolean" | "valueOf", [Some(value)]) => {
            let value = required_java_string(value, "Boolean text cannot be null")?;
            Ok(Some(Arc::new(TemplateValue::Boolean(
                value.to_string_lossy().eq_ignore_ascii_case("true"),
            ))))
        }
        ("java.time.LocalDateTime", "of", values @ [_, _, _, _, ..]) => {
            let fields = values
                .iter()
                .map(|value| {
                    value
                        .as_deref()
                        .ok_or_else(|| {
                            processing_error("LocalDateTime field cannot be null".to_owned())
                        })
                        .and_then(|value| {
                            integer_argument(value, "LocalDateTime field is not an integer")
                        })
                        .and_then(|value| {
                            i32::try_from(value)
                                .map_err(|error| processing_error(error.to_string()))
                        })
                })
                .collect::<StandardExpressionResult<Vec<_>>>()?;
            let temporal = TemporalCreationUtils::new()
                .create(&fields)
                .map_err(|error| processing_error(error.to_string()))?;
            Ok(Some(Arc::new(TemplateValue::Object(Arc::new(temporal)))))
        }
        ("java.lang.String", "format", [Some(format), values @ ..]) => {
            let mut output = format
                .to_java_string()
                .ok_or_else(|| processing_error("Format cannot be null".to_owned()))?
                .to_string_lossy();
            for value in values {
                let replacement = value
                    .as_deref()
                    .and_then(TemplateValue::to_java_string)
                    .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy());
                output = output.replacen("%s", &replacement, 1);
            }
            Ok(Some(Arc::new(TemplateValue::string(
                JavaString::from_rust_str(&output),
            ))))
        }
        _ => Err(processing_error(format!(
            "Static method \"{member}\" with {} arguments is not available on {type_name}",
            arguments.len()
        ))),
    }
}

fn numeric_f64(value: &TemplateValue) -> StandardExpressionResult<f64> {
    match value {
        TemplateValue::Number(JavaNumber::Byte(value)) => Ok(f64::from(*value)),
        TemplateValue::Number(JavaNumber::Short(value)) => Ok(f64::from(*value)),
        TemplateValue::Number(JavaNumber::Integer(value)) => Ok(f64::from(*value)),
        TemplateValue::Number(JavaNumber::Long(value)) => Ok(*value as f64),
        TemplateValue::Number(JavaNumber::Float(value)) => Ok(f64::from(*value)),
        TemplateValue::Number(JavaNumber::Double(value))
        | TemplateValue::Number(JavaNumber::Other {
            double_value: value,
            ..
        }) => Ok(*value),
        TemplateValue::Number(JavaNumber::BigInteger(value)) => value
            .to_string()
            .parse()
            .map_err(|error: std::num::ParseFloatError| processing_error(error.to_string())),
        TemplateValue::Number(JavaNumber::BigDecimal(value)) => value
            .to_string()
            .parse()
            .map_err(|error: std::num::ParseFloatError| processing_error(error.to_string())),
        _ => Err(processing_error(format!(
            "{} cannot be converted to a number",
            value.java_class_name()
        ))),
    }
}

fn number_result(value: f64) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Double(
        value,
    )))))
}

struct ClassObjectValue {
    type_name: JavaString,
}

impl super::TemplateObject for ClassObjectValue {
    fn java_class_name(&self) -> &str {
        "java.lang.Class"
    }

    fn to_java_string(&self) -> JavaString {
        JavaString::from_rust_str(&format!("class {}", self.type_name.to_string_lossy()))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn java_get_property(
        &self,
        property_name: &JavaString,
    ) -> Option<Result<Option<Arc<TemplateValue>>, super::TemplateObjectPropertyError>> {
        let value = match property_name.to_string_lossy().as_str() {
            "name" => self.type_name.clone(),
            "simpleName" => JavaString::from_rust_str(
                self.type_name
                    .to_string_lossy()
                    .rsplit('.')
                    .next()
                    .unwrap_or(""),
            ),
            _ => return None,
        };
        Some(Ok(Some(Arc::new(TemplateValue::string(value)))))
    }

    fn java_invoke_method(
        &self,
        method_name: &JavaString,
        arguments: &[Option<Arc<TemplateValue>>],
    ) -> Option<Result<Option<Arc<TemplateValue>>, super::TemplateObjectMethodError>> {
        let value = match (method_name.to_string_lossy().as_str(), arguments) {
            ("getName", []) => self.type_name.clone(),
            ("getSimpleName", []) => JavaString::from_rust_str(
                self.type_name
                    .to_string_lossy()
                    .rsplit('.')
                    .next()
                    .unwrap_or(""),
            ),
            _ => return None,
        };
        Some(Ok(Some(Arc::new(TemplateValue::string(value)))))
    }
}

fn java_class_value(type_name: &str) -> Arc<TemplateValue> {
    Arc::new(TemplateValue::Object(Arc::new(ClassObjectValue {
        type_name: JavaString::from_rust_str(type_name),
    })))
}

fn path_root_method_name(root: &PathRoot) -> JavaString {
    match root {
        PathRoot::Context(name) | PathRoot::ExpressionObject(name) => name.clone(),
    }
}

fn invoke_dynamic_method(
    target: &TemplateValue,
    name: &JavaString,
    arguments: &[Option<Arc<TemplateValue>>],
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    match (name.to_string_lossy().as_str(), arguments) {
        ("toString", []) => {
            return Ok(target
                .to_java_string()
                .map(|value| Arc::new(TemplateValue::string(value))));
        }
        ("getClass", []) => return Ok(Some(java_class_value(target.java_class_name()))),
        ("equals", [other]) => {
            return Ok(Some(Arc::new(TemplateValue::Boolean(
                other
                    .as_deref()
                    .is_some_and(|other| target.java_equals(other)),
            ))));
        }
        _ => {}
    }
    match target {
        TemplateValue::Object(value) => {
            if value.java_class_name() == "java.util.stream.Stream"
                && !matches!(name.to_string_lossy().as_str(), "count" | "iterator")
            {
                return Err(processing_error_with_cause(
                    format!(
                        "method \"{}\" is not callable on {}",
                        name.to_string_lossy(),
                        value.java_class_name()
                    ),
                    NoSuchMethodException::new(format!(
                        "{}.{}",
                        value.java_class_name(),
                        name.to_string_lossy()
                    )),
                ));
            }
            ThymeleafACLMemberAccess::is_accessible(Some(value.as_ref()), &name.to_string_lossy())?;
            value.java_invoke_method(name, arguments).map_or_else(
                || {
                    Err(processing_error(format!(
                        "method \"{}\" is not callable on {}",
                        name.to_string_lossy(),
                        value.java_class_name()
                    )))
                },
                |result| result.map_err(|error| processing_error(error.to_string())),
            )
        }
        TemplateValue::String(value) | TemplateValue::SafeHtml(value) => {
            invoke_java_string_method(value, name, arguments)
        }
        TemplateValue::List(values) => invoke_java_list_method(values, name, arguments),
        TemplateValue::Map(entries) => invoke_java_map_method(entries, name, arguments),
        _ => Err(processing_error(format!(
            "method \"{}\" is not callable on {}",
            name.to_string_lossy(),
            target.java_class_name()
        ))),
    }
}

fn invoke_java_map_method(
    entries: &[(Arc<TemplateValue>, Arc<TemplateValue>)],
    name: &JavaString,
    arguments: &[Option<Arc<TemplateValue>>],
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let name = name.to_string_lossy();
    match (name.as_str(), arguments) {
        ("size", []) => Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(
            i32::try_from(entries.len()).unwrap_or(i32::MAX),
        ))))),
        ("isEmpty", []) => Ok(Some(Arc::new(TemplateValue::Boolean(entries.is_empty())))),
        ("get", [key]) => Ok(entries
            .iter()
            .find(|(candidate, _)| dynamic_values_equal(Some(candidate), key.as_ref()))
            .map(|(_, value)| Arc::clone(value))),
        ("containsKey", [key]) => {
            Ok(Some(Arc::new(TemplateValue::Boolean(entries.iter().any(
                |(candidate, _)| dynamic_values_equal(Some(candidate), key.as_ref()),
            )))))
        }
        ("containsValue", [value]) => {
            Ok(Some(Arc::new(TemplateValue::Boolean(entries.iter().any(
                |(_, candidate)| dynamic_values_equal(Some(candidate), value.as_ref()),
            )))))
        }
        ("keySet", []) => Ok(Some(Arc::new(TemplateValue::List(Arc::new(
            entries.iter().map(|(key, _)| Arc::clone(key)).collect(),
        ))))),
        ("values", []) => Ok(Some(Arc::new(TemplateValue::List(Arc::new(
            entries.iter().map(|(_, value)| Arc::clone(value)).collect(),
        ))))),
        ("entrySet", []) => Ok(Some(Arc::new(TemplateValue::List(Arc::new(
            entries
                .iter()
                .map(|(key, value)| {
                    Arc::new(TemplateValue::Object(Arc::new(MapEntryValue::new(
                        Arc::clone(key),
                        Arc::clone(value),
                    ))))
                })
                .collect(),
        ))))),
        _ => Err(processing_error(format!(
            "method \"{name}\" with {} arguments is not callable on java.util.Map",
            arguments.len()
        ))),
    }
}

fn dynamic_values_equal(
    left: Option<&Arc<TemplateValue>>,
    right: Option<&Arc<TemplateValue>>,
) -> bool {
    match (left.map(Arc::as_ref), right.map(Arc::as_ref)) {
        (None | Some(TemplateValue::Null), None | Some(TemplateValue::Null)) => true,
        (Some(left), Some(right)) => left.java_equals(right),
        _ => false,
    }
}

fn invoke_java_string_method(
    value: &JavaString,
    name: &JavaString,
    arguments: &[Option<Arc<TemplateValue>>],
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let name = name.to_string_lossy();
    match (name.as_str(), arguments) {
        ("length", []) => Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(
            i32::try_from(value.len()).unwrap_or(i32::MAX),
        ))))),
        ("isEmpty", []) => Ok(Some(Arc::new(TemplateValue::Boolean(value.is_empty())))),
        ("toString", []) => Ok(Some(Arc::new(TemplateValue::string(value.clone())))),
        ("contains", [Some(argument)]) => {
            let argument = argument.to_java_string().ok_or_else(|| {
                processing_error("String.contains argument cannot be null".to_owned())
            })?;
            Ok(Some(Arc::new(TemplateValue::Boolean(
                value
                    .to_string_lossy()
                    .contains(&argument.to_string_lossy()),
            ))))
        }
        ("equalsIgnoreCase", [Some(argument)]) => {
            let argument = required_java_string(argument, "String argument cannot be null")?;
            Ok(Some(Arc::new(TemplateValue::Boolean(
                value.to_string_lossy().to_lowercase() == argument.to_string_lossy().to_lowercase(),
            ))))
        }
        ("startsWith", [Some(argument)]) | ("endsWith", [Some(argument)]) => {
            let argument = required_java_string(argument, "String argument cannot be null")?;
            let result = if name == "startsWith" {
                value.as_utf16().starts_with(argument.as_utf16())
            } else {
                value.as_utf16().ends_with(argument.as_utf16())
            };
            Ok(Some(Arc::new(TemplateValue::Boolean(result))))
        }
        ("startsWith", [Some(argument), Some(offset)]) => {
            let argument = required_java_string(argument, "String argument cannot be null")?;
            let offset = integer_argument(offset, "String offset is not an integer")?;
            let result = usize::try_from(offset)
                .ok()
                .and_then(|offset| value.as_utf16().get(offset..))
                .is_some_and(|remaining| remaining.starts_with(argument.as_utf16()));
            Ok(Some(Arc::new(TemplateValue::Boolean(result))))
        }
        ("substring" | "subSequence", [Some(begin)]) => {
            let begin = string_index(begin, value.len())?;
            Ok(Some(Arc::new(TemplateValue::string(
                JavaString::from_utf16(value.as_utf16()[begin..].to_vec()),
            ))))
        }
        ("substring" | "subSequence", [Some(begin), Some(end)]) => {
            let begin = string_index(begin, value.len())?;
            let end = string_index(end, value.len())?;
            if begin > end {
                return Err(processing_error(format!(
                    "begin {begin} is greater than end {end}"
                )));
            }
            Ok(Some(Arc::new(TemplateValue::string(
                JavaString::from_utf16(value.as_utf16()[begin..end].to_vec()),
            ))))
        }
        ("charAt", [Some(index)]) => {
            let index = string_index(index, value.len().saturating_sub(1))?;
            let character = value
                .as_utf16()
                .get(index)
                .copied()
                .ok_or_else(|| processing_error(format!("index {index} is out of bounds")))?;
            Ok(Some(Arc::new(TemplateValue::Character(character))))
        }
        ("indexOf" | "lastIndexOf", [Some(argument)]) => {
            let argument = required_java_string(argument, "String argument cannot be null")?;
            if argument.is_empty() {
                let index = if name == "indexOf" {
                    0
                } else {
                    i32::try_from(value.len()).unwrap_or(i32::MAX)
                };
                return Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(
                    index,
                )))));
            }
            let positions = value
                .as_utf16()
                .windows(argument.len())
                .enumerate()
                .filter(|(_, candidate)| *candidate == argument.as_utf16())
                .map(|(index, _)| index);
            let index = if name == "indexOf" {
                positions.into_iter().next()
            } else {
                positions.into_iter().next_back()
            }
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1);
            Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(
                index,
            )))))
        }
        ("concat", [Some(argument)]) => {
            let argument = required_java_string(argument, "String argument cannot be null")?;
            let mut output = value.as_utf16().to_vec();
            output.extend_from_slice(argument.as_utf16());
            Ok(Some(Arc::new(TemplateValue::string(
                JavaString::from_utf16(output),
            ))))
        }
        ("trim" | "strip", []) => Ok(Some(Arc::new(TemplateValue::string(java_trim(value))))),
        ("toUpperCase", []) => Ok(Some(Arc::new(TemplateValue::string(
            JavaString::from_rust_str(&value.to_string_lossy().to_uppercase()),
        )))),
        ("toLowerCase", []) => Ok(Some(Arc::new(TemplateValue::string(
            JavaString::from_rust_str(&value.to_string_lossy().to_lowercase()),
        )))),
        ("repeat", [Some(count)]) => {
            let count = integer_argument(count, "String repeat count is not an integer")?;
            let count = usize::try_from(count)
                .map_err(|_| processing_error("String repeat count is negative".to_owned()))?;
            Ok(Some(Arc::new(TemplateValue::string(
                JavaString::from_utf16(value.as_utf16().repeat(count)),
            ))))
        }
        ("getBytes", []) => Ok(Some(Arc::new(TemplateValue::Bytes(Arc::new(
            value
                .to_string_lossy()
                .into_bytes()
                .into_iter()
                .map(|byte| byte as i8)
                .collect(),
        ))))),
        _ => Err(processing_error(format!(
            "method \"{name}\" with {} arguments is not callable on java.lang.String",
            arguments.len()
        ))),
    }
}

fn invoke_java_list_method(
    values: &[Arc<TemplateValue>],
    name: &JavaString,
    arguments: &[Option<Arc<TemplateValue>>],
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let name = name.to_string_lossy();
    match (name.as_str(), arguments) {
        ("size", []) => Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(
            i32::try_from(values.len()).unwrap_or(i32::MAX),
        ))))),
        ("isEmpty", []) => Ok(Some(Arc::new(TemplateValue::Boolean(values.is_empty())))),
        ("iterator", []) => Ok(Some(Arc::new(TemplateValue::Object(Arc::new(
            IteratorValue::new(Arc::new(values.to_vec())),
        ))))),
        ("stream", []) => Ok(Some(Arc::new(TemplateValue::Object(Arc::new(
            StreamValue::new(Arc::new(values.to_vec())),
        ))))),
        ("get", [Some(index)]) => {
            let index = match index.as_ref() {
                TemplateValue::Number(JavaNumber::Integer(index)) => usize::try_from(*index).ok(),
                TemplateValue::Number(JavaNumber::Long(index)) => usize::try_from(*index).ok(),
                _ => None,
            }
            .ok_or_else(|| processing_error("List.get index is not an integer".to_owned()))?;
            values
                .get(index)
                .cloned()
                .map(Some)
                .ok_or_else(|| processing_error(format!("index {index} is out of bounds")))
        }
        ("contains", [value]) => {
            Ok(Some(Arc::new(TemplateValue::Boolean(values.iter().any(
                |candidate| dynamic_values_equal(Some(candidate), value.as_ref()),
            )))))
        }
        ("indexOf" | "lastIndexOf", [value]) => {
            let indexes = values
                .iter()
                .enumerate()
                .filter(|(_, candidate)| dynamic_values_equal(Some(candidate), value.as_ref()))
                .map(|(index, _)| index);
            let index = if name == "indexOf" {
                indexes.into_iter().next()
            } else {
                indexes.into_iter().next_back()
            }
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1);
            Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(
                index,
            )))))
        }
        ("subList", [Some(begin), Some(end)]) => {
            let begin = list_index(begin, values.len())?;
            let end = list_index(end, values.len())?;
            if begin > end {
                return Err(processing_error(format!(
                    "fromIndex({begin}) > toIndex({end})"
                )));
            }
            Ok(Some(Arc::new(TemplateValue::List(Arc::new(
                values[begin..end].to_vec(),
            )))))
        }
        _ => Err(processing_error(format!(
            "method \"{name}\" with {} arguments is not callable on java.util.List",
            arguments.len()
        ))),
    }
}

fn required_java_string(
    value: &TemplateValue,
    message: &str,
) -> StandardExpressionResult<JavaString> {
    value
        .to_java_string()
        .ok_or_else(|| processing_error(message.to_owned()))
}

fn integer_argument(value: &TemplateValue, message: &str) -> StandardExpressionResult<i64> {
    match value {
        TemplateValue::Number(JavaNumber::Byte(value)) => Ok(i64::from(*value)),
        TemplateValue::Number(JavaNumber::Short(value)) => Ok(i64::from(*value)),
        TemplateValue::Number(JavaNumber::Integer(value)) => Ok(i64::from(*value)),
        TemplateValue::Number(JavaNumber::Long(value)) => Ok(*value),
        _ => Err(processing_error(message.to_owned())),
    }
}

fn string_index(value: &TemplateValue, maximum: usize) -> StandardExpressionResult<usize> {
    let index = integer_argument(value, "String index is not an integer")?;
    let index = usize::try_from(index)
        .map_err(|_| processing_error(format!("index {index} is out of bounds")))?;
    if index > maximum {
        return Err(processing_error(format!("index {index} is out of bounds")));
    }
    Ok(index)
}

fn list_index(value: &TemplateValue, maximum: usize) -> StandardExpressionResult<usize> {
    let index = integer_argument(value, "List index is not an integer")?;
    let index = usize::try_from(index)
        .map_err(|_| processing_error(format!("index {index} is out of bounds")))?;
    if index > maximum {
        return Err(processing_error(format!("index {index} is out of bounds")));
    }
    Ok(index)
}

fn read_dynamic_property(
    target: &TemplateValue,
    name: &JavaString,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    if name == &JavaString::from_rust_str("class") {
        if let TemplateValue::Map(entries) = target
            && let Some(value) = entries.iter().find_map(|(key, value)| {
                matches!(
                    key.as_ref(),
                    TemplateValue::String(key_value) | TemplateValue::SafeHtml(key_value)
                        if key_value.as_ref() == name
                )
                .then(|| Arc::clone(value))
            })
        {
            return Ok(Some(value));
        }
        if let TemplateValue::Object(value) = target
            && let Some(result) = value.java_get_property(name)
        {
            // OGNL 的专用 PropertyAccessor（例如 ContextMap）先于 Object#getClass；
            // 因而名为 class 的上下文变量必须遮蔽反射类属性。
            return result.map_err(|error| processing_error(error.to_string()));
        }
        return Ok(Some(java_class_value(target.java_class_name())));
    }
    match target {
        TemplateValue::Map(entries) => match name.to_string_lossy().as_str() {
            "size" => Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(
                i32::try_from(entries.len()).unwrap_or(i32::MAX),
            ))))),
            "isEmpty" | "empty" => Ok(Some(Arc::new(TemplateValue::Boolean(entries.is_empty())))),
            "keys" | "keySet" => Ok(Some(Arc::new(TemplateValue::List(Arc::new(
                entries.iter().map(|(key, _)| Arc::clone(key)).collect(),
            ))))),
            "values" => Ok(Some(Arc::new(TemplateValue::List(Arc::new(
                entries.iter().map(|(_, value)| Arc::clone(value)).collect(),
            ))))),
            _ => Ok(entries.iter().find_map(|(key, value)| {
                matches!(
                    key.as_ref(),
                    TemplateValue::String(key_value) | TemplateValue::SafeHtml(key_value)
                        if key_value.as_ref() == name
                )
                .then(|| Arc::clone(value))
            })),
        },
        TemplateValue::List(values) => match name.to_string_lossy().as_str() {
            "size" | "length" => Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(
                i32::try_from(values.len()).unwrap_or(i32::MAX),
            ))))),
            "isEmpty" | "empty" => Ok(Some(Arc::new(TemplateValue::Boolean(values.is_empty())))),
            _ => Err(processing_error(format!(
                "property \"{}\" is not readable on {}",
                name.to_string_lossy(),
                target.java_class_name()
            ))),
        },
        TemplateValue::Bytes(values) if name == &JavaString::from_rust_str("length") => {
            Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(
                i32::try_from(values.len()).unwrap_or(i32::MAX),
            )))))
        }
        TemplateValue::String(value) | TemplateValue::SafeHtml(value) => {
            match name.to_string_lossy().as_str() {
                "length" => Ok(Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(
                    i32::try_from(value.len()).unwrap_or(i32::MAX),
                ))))),
                "isEmpty" | "empty" => Ok(Some(Arc::new(TemplateValue::Boolean(value.is_empty())))),
                _ => Err(processing_error(format!(
                    "property \"{}\" is not readable on {}",
                    name.to_string_lossy(),
                    target.java_class_name()
                ))),
            }
        }
        TemplateValue::Object(value) => {
            let property_name = name.to_string_lossy();
            let acl_member_name = match (value.java_class_name(), property_name.as_str()) {
                ("java.lang.Class", "name") => "getName",
                ("java.lang.Class", "simpleName") => "getSimpleName",
                ("java.lang.Class", "package") => "getPackage",
                ("java.util.Map$Entry", "key") => "getKey",
                ("java.util.Map$Entry", "value") => "getValue",
                _ => property_name.as_str(),
            };
            ThymeleafACLMemberAccess::is_accessible(Some(value.as_ref()), acl_member_name)?;
            value.java_get_property(name).map_or_else(
                || {
                    Err(processing_error(format!(
                        "property \"{}\" is not readable on {}",
                        name.to_string_lossy(),
                        value.java_class_name()
                    )))
                },
                |result| result.map_err(|error| processing_error(error.to_string())),
            )
        }
        TemplateValue::Null => Err(ognl_processing_error(
            format!(
                "source is null for getProperty(null, \"{}\")",
                name.to_string_lossy()
            ),
            format!(
                "source is null for getProperty(null, \"{}\")",
                name.to_string_lossy()
            ),
        )),
        _ => Err(processing_error(format!(
            "property \"{}\" is not readable on {}",
            name.to_string_lossy(),
            target.java_class_name()
        ))),
    }
}

fn convert_to_string(
    context: &dyn IExpressionContext,
    value: Option<Arc<TemplateValue>>,
) -> StandardExpressionResult<Option<Arc<TemplateValue>>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let service = StandardExpressions::get_conversion_service(context.get_configuration())?;
    let conversion_value = match value.as_ref() {
        TemplateValue::Null => JavaConversionValue::Null,
        TemplateValue::String(string) | TemplateValue::SafeHtml(string) => {
            JavaConversionValue::String(string)
        }
        object => JavaConversionValue::Object(object),
    };
    let converted = service
        .convert(
            Some(context.as_any()),
            conversion_value,
            Some(&JavaTargetClass::String),
        )
        .map_err(|error| Box::new(error) as super::StandardExpressionError)?;
    Ok(match converted {
        JavaConversionResult::Null => None,
        JavaConversionResult::BorrowedString(value) => {
            Some(Arc::new(TemplateValue::string(value.clone())))
        }
        JavaConversionResult::OwnedString(value) => Some(Arc::new(TemplateValue::string(value))),
        JavaConversionResult::BorrowedObject(_) | JavaConversionResult::OwnedObject(_) => {
            return Err(processing_error(
                "Conversion service returned a non-String value for String.class".to_owned(),
            ));
        }
    })
}

fn java_trim(input: &JavaString) -> JavaString {
    JavaString::from_utf16(java_trim_units(input.as_utf16()).to_vec())
}

fn java_trim_units(input: &[u16]) -> &[u16] {
    let start = input
        .iter()
        .position(|unit| *unit > 0x20)
        .unwrap_or(input.len());
    let end = input
        .iter()
        .rposition(|unit| *unit > 0x20)
        .map_or(start, |position| position + 1);
    &input[start..end]
}

fn is_ascii_identifier_part(value: u16) -> bool {
    value == b'_' as u16
        || value == b'$' as u16
        || (b'a' as u16..=b'z' as u16).contains(&value)
        || (b'A' as u16..=b'Z' as u16).contains(&value)
        || (b'0' as u16..=b'9' as u16).contains(&value)
}

fn find_conditional(input: &[u16]) -> Option<(usize, Option<usize>)> {
    let mut question = None;
    let mut colon = None;
    let mut nested_conditionals = 0usize;
    scan_top_level(input, |position, unit| {
        if unit == b'?' as u16
            && next_non_whitespace(input, position + 1)
                .is_none_or(|next| input[next] != b':' as u16)
        {
            if question.is_none() {
                question = Some(position);
            } else {
                nested_conditionals += 1;
            }
        } else if unit == b':' as u16 && question.is_some() && colon.is_none() {
            if nested_conditionals == 0 {
                colon = Some(position);
            } else {
                nested_conditionals -= 1;
            }
        }
    });
    question.map(|position| (position, colon))
}

fn find_default_operator(input: &[u16]) -> Option<(usize, usize)> {
    let mut found = None;
    scan_top_level(input, |position, unit| {
        if found.is_none()
            && unit == b'?' as u16
            && let Some(colon) = next_non_whitespace(input, position + 1)
            && input[colon] == b':' as u16
        {
            found = Some((position, colon));
        }
    });
    found
}

fn next_non_whitespace(input: &[u16], mut position: usize) -> Option<usize> {
    while input.get(position).is_some_and(|unit| *unit <= 0x20) {
        position += 1;
    }
    (position < input.len()).then_some(position)
}

fn find_assignment_operator(input: &[u16]) -> Option<usize> {
    let mut found = None;
    scan_top_level(input, |position, unit| {
        if found.is_some() || unit != b'=' as u16 {
            return;
        }
        let before = position
            .checked_sub(1)
            .and_then(|index| input.get(index))
            .copied();
        let after = input.get(position + 1).copied();
        if !before.is_some_and(|value| {
            [b'=' as u16, b'!' as u16, b'<' as u16, b'>' as u16].contains(&value)
        }) && after != Some(b'=' as u16)
        {
            found = Some(position);
        }
    });
    found
}

fn find_word_operator(input: &[u16], operator: &[u16]) -> Option<usize> {
    find_binary_operator(input, &[operator]).map(|(position, _)| position)
}

fn find_binary_operator<'a>(
    input: &[u16],
    operators: &'a [&'a [u16]],
) -> Option<(usize, &'a [u16])> {
    let mut found = None;
    scan_top_level(input, |position, _| {
        for operator in operators {
            if position + operator.len() <= input.len()
                && eq_ignore_ascii_case(&input[position..position + operator.len()], operator)
                && operator_boundary(input, position, operator)
                && !((operator == &OP_MINUS || operator == &OP_PLUS)
                    && is_unary_sign_position(input, position))
            {
                let replace =
                    found
                        .as_ref()
                        .is_none_or(|(old_position, old_operator): &(usize, &[u16])| {
                            position > *old_position
                                || (position == *old_position
                                    && operator.len() > old_operator.len())
                        });
                if replace {
                    found = Some((position, *operator));
                }
            }
        }
    });
    found.filter(|(position, operator)| {
        !java_trim_units(&input[..*position]).is_empty()
            && !java_trim_units(&input[*position + operator.len()..]).is_empty()
    })
}

fn is_unary_sign_position(input: &[u16], position: usize) -> bool {
    let prefix = java_trim_units(&input[..position]);
    let Some(last) = prefix.last().copied() else {
        return true;
    };
    if [
        b'+' as u16,
        b'-' as u16,
        b'*' as u16,
        b'/' as u16,
        b'%' as u16,
        b'<' as u16,
        b'>' as u16,
        b'=' as u16,
        b'!' as u16,
        b'&' as u16,
        b'|' as u16,
        b'^' as u16,
        b'~' as u16,
        b'?' as u16,
        b':' as u16,
        b',' as u16,
        b'(' as u16,
        b'[' as u16,
        b'{' as u16,
    ]
    .contains(&last)
    {
        return true;
    }
    [OP_DIV, OP_MOD, OP_AND, OP_OR].iter().any(|operator| {
        prefix.len() >= operator.len()
            && eq_ignore_ascii_case(&prefix[prefix.len() - operator.len()..], operator)
            && prefix
                .get(prefix.len().saturating_sub(operator.len() + 1))
                .is_none_or(|unit| !is_word_unit(*unit))
    })
}

fn find_inclusion_operator(input: &[u16]) -> Option<(usize, usize, bool)> {
    let mut found = None;
    scan_top_level(input, |position, _| {
        if position + OP_IN.len() > input.len()
            || !eq_ignore_ascii_case(&input[position..position + OP_IN.len()], OP_IN)
            || !operator_boundary(input, position, OP_IN)
        {
            return;
        }

        let mut before_in = position;
        while before_in > 0 && input[before_in - 1] <= 0x20 {
            before_in -= 1;
        }
        let not_start = before_in.checked_sub(3);
        let negated = not_start.is_some_and(|start| {
            eq_ignore_ascii_case(&input[start..before_in], OP_NOT)
                && start
                    .checked_sub(1)
                    .and_then(|index| input.get(index))
                    .is_none_or(|unit| !is_word_unit(*unit))
        });
        let operator_start = if negated {
            not_start.expect("negated operator has a start")
        } else {
            position
        };
        let operator_length = position + OP_IN.len() - operator_start;
        if !java_trim_units(&input[..operator_start]).is_empty()
            && !java_trim_units(&input[position + OP_IN.len()..]).is_empty()
            && found
                .as_ref()
                .is_none_or(|(old_position, _, _)| operator_start > *old_position)
        {
            found = Some((operator_start, operator_length, negated));
        }
    });
    found
}

fn operator_boundary(input: &[u16], position: usize, operator: &[u16]) -> bool {
    if operator
        .iter()
        .all(|unit| is_ascii_alphabetic(*unit) || *unit == b' ' as u16)
    {
        let before = position.checked_sub(1).and_then(|index| input.get(index));
        let after = input.get(position + operator.len());
        return before.is_none_or(|unit| !is_word_unit(*unit))
            && after.is_none_or(|unit| !is_word_unit(*unit));
    }
    if operator == OP_MINUS {
        let before = position.checked_sub(1).and_then(|index| input.get(index));
        let after = input.get(position + 1);
        return before.is_none_or(|unit| !is_word_unit(*unit))
            || after.is_none_or(|unit| !is_word_unit(*unit))
            || before.is_some_and(|unit| (b'0' as u16..=b'9' as u16).contains(unit))
            || after.is_some_and(|unit| (b'0' as u16..=b'9' as u16).contains(unit));
    }
    true
}

fn find_top_level_sequence(input: &[u16], sequence: &[u16]) -> Option<usize> {
    let mut found = None;
    scan_top_level(input, |position, _| {
        if found.is_none()
            && position + sequence.len() <= input.len()
            && input[position..position + sequence.len()] == *sequence
        {
            found = Some(position);
        }
    });
    found
}

fn scan_top_level(input: &[u16], mut visitor: impl FnMut(usize, u16)) {
    let mut parentheses = 0_i32;
    let mut brackets = 0_i32;
    let mut braces = 0_i32;
    let mut quote = None;
    for (position, unit) in input.iter().copied().enumerate() {
        if let Some(active_quote) = quote {
            if unit == active_quote && !is_escaped(input, position) {
                quote = None;
            }
            continue;
        }
        if matches!(unit, 0x27 | 0x22) {
            quote = Some(unit);
            continue;
        }
        match unit {
            value if value == b'(' as u16 => parentheses += 1,
            value if value == b')' as u16 => parentheses -= 1,
            value if value == b'[' as u16 => brackets += 1,
            value if value == b']' as u16 => brackets -= 1,
            value if value == b'{' as u16 => braces += 1,
            value if value == b'}' as u16 => braces -= 1,
            _ if parentheses == 0 && brackets == 0 && braces == 0 => visitor(position, unit),
            _ => {}
        }
    }
}

fn is_outer_parenthesized(input: &[u16]) -> bool {
    if input.first() != Some(&(b'(' as u16)) || input.last() != Some(&(b')' as u16)) {
        return false;
    }
    let mut level = 0_i32;
    let mut quote = None;
    for (position, unit) in input.iter().copied().enumerate() {
        if let Some(active_quote) = quote {
            if unit == active_quote && !is_escaped(input, position) {
                quote = None;
            }
        } else if matches!(unit, 0x27 | 0x22) {
            quote = Some(unit);
        } else if unit == b'(' as u16 {
            level += 1;
        } else if unit == b')' as u16 {
            level -= 1;
            if level == 0 && position + 1 != input.len() {
                return false;
            }
        }
    }
    level == 0 && quote.is_none()
}

fn starts_with_word(input: &[u16], word: &str) -> bool {
    let word = word.as_bytes();
    input.len() > word.len()
        && input[..word.len()]
            .iter()
            .zip(word)
            .all(|(left, right)| ascii_lower(*left) == u16::from(right.to_ascii_lowercase()))
        && !is_word_unit(input[word.len()])
}

fn eq_ignore_ascii_case(left: &[u16], right: &[u16]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| ascii_lower(*left) == ascii_lower(*right))
}

fn is_word_unit(unit: u16) -> bool {
    is_ascii_alphabetic(unit)
        || (b'0' as u16..=b'9' as u16).contains(&unit)
        || matches!(unit, value if value == b'_' as u16 || value == b'$' as u16)
}

fn is_ascii_alphabetic(unit: u16) -> bool {
    (b'a' as u16..=b'z' as u16).contains(&unit) || (b'A' as u16..=b'Z' as u16).contains(&unit)
}

fn ascii_lower(unit: u16) -> u16 {
    if (b'A' as u16..=b'Z' as u16).contains(&unit) {
        unit + u16::from(b'a' - b'A')
    } else {
        unit
    }
}

fn is_escaped(input: &[u16], position: usize) -> bool {
    let mut slash_count = 0;
    let mut current = position;
    while current > 0 && input[current - 1] == b'\\' as u16 {
        slash_count += 1;
        current -= 1;
    }
    slash_count % 2 == 1
}

const OP_OR: &[u16] = &[b'o' as u16, b'r' as u16];
const OP_DOUBLE_PIPE: &[u16] = &[b'|' as u16, b'|' as u16];
const OP_AND: &[u16] = &[b'a' as u16, b'n' as u16, b'd' as u16];
const OP_DOUBLE_AMPERSAND: &[u16] = &[b'&' as u16, b'&' as u16];
const OP_BOR: &[u16] = &[b'b' as u16, b'o' as u16, b'r' as u16];
const OP_PIPE: &[u16] = &[b'|' as u16];
const OP_XOR: &[u16] = &[b'x' as u16, b'o' as u16, b'r' as u16];
const OP_CARET: &[u16] = &[b'^' as u16];
const OP_BAND: &[u16] = &[b'b' as u16, b'a' as u16, b'n' as u16, b'd' as u16];
const OP_AMPERSAND: &[u16] = &[b'&' as u16];
const OP_NEQ: &[u16] = &[b'n' as u16, b'e' as u16, b'q' as u16];
const OP_NE: &[u16] = &[b'n' as u16, b'e' as u16];
const OP_NOT_EQUALS: &[u16] = &[b'!' as u16, b'=' as u16];
const OP_EQ: &[u16] = &[b'e' as u16, b'q' as u16];
const OP_EQUALS: &[u16] = &[b'=' as u16, b'=' as u16];
const OP_GTE: &[u16] = &[b'g' as u16, b't' as u16, b'e' as u16];
const OP_GE: &[u16] = &[b'g' as u16, b'e' as u16];
const OP_GREATER_EQUAL: &[u16] = &[b'>' as u16, b'=' as u16];
const OP_GT: &[u16] = &[b'g' as u16, b't' as u16];
const OP_GREATER: &[u16] = &[b'>' as u16];
const OP_LTE: &[u16] = &[b'l' as u16, b't' as u16, b'e' as u16];
const OP_LE: &[u16] = &[b'l' as u16, b'e' as u16];
const OP_LESS_EQUAL: &[u16] = &[b'<' as u16, b'=' as u16];
const OP_LT: &[u16] = &[b'l' as u16, b't' as u16];
const OP_LESS: &[u16] = &[b'<' as u16];
const OP_NOT: &[u16] = &[b'n' as u16, b'o' as u16, b't' as u16];
const OP_IN: &[u16] = &[b'i' as u16, b'n' as u16];
const OP_INSTANCEOF: &[u16] = &[
    b'i' as u16,
    b'n' as u16,
    b's' as u16,
    b't' as u16,
    b'a' as u16,
    b'n' as u16,
    b'c' as u16,
    b'e' as u16,
    b'o' as u16,
    b'f' as u16,
];
const OP_SHIFT_LEFT: &[u16] = &[b'<' as u16, b'<' as u16];
const OP_SHL: &[u16] = &[b's' as u16, b'h' as u16, b'l' as u16];
const OP_SHIFT_RIGHT: &[u16] = &[b'>' as u16, b'>' as u16];
const OP_SHR: &[u16] = &[b's' as u16, b'h' as u16, b'r' as u16];
const OP_UNSIGNED_SHIFT_RIGHT: &[u16] = &[b'>' as u16, b'>' as u16, b'>' as u16];
const OP_USHR: &[u16] = &[b'u' as u16, b's' as u16, b'h' as u16, b'r' as u16];
const OP_PLUS: &[u16] = &[b'+' as u16];
const OP_MINUS: &[u16] = &[b'-' as u16];
const OP_MULTIPLY: &[u16] = &[b'*' as u16];
const OP_DIV: &[u16] = &[b'd' as u16, b'i' as u16, b'v' as u16];
const OP_DIVIDE: &[u16] = &[b'/' as u16];
const OP_MOD: &[u16] = &[b'm' as u16, b'o' as u16, b'd' as u16];
const OP_REMAINDER: &[u16] = &[b'%' as u16];

thread_local! {
    static OGNL_RUNTIMES: RefCell<Vec<Arc<dyn OgnlRuntime>>> = const { RefCell::new(Vec::new()) };
    static OGNL_LOCALS: RefCell<Vec<HashMap<JavaString, Option<Arc<TemplateValue>>>>> =
        const { RefCell::new(Vec::new()) };
}

fn current_ognl_local(name: &JavaString) -> Option<Option<Arc<TemplateValue>>> {
    OGNL_LOCALS.with(|scopes| {
        scopes
            .borrow()
            .last()
            .and_then(|scope| scope.get(name).cloned())
    })
}

fn set_ognl_local(name: JavaString, value: Option<Arc<TemplateValue>>) {
    OGNL_LOCALS.with(|scopes| {
        if let Some(scope) = scopes.borrow_mut().last_mut() {
            scope.insert(name, value);
        }
    });
}

fn with_ognl_locals<T>(operation: impl FnOnce() -> T) -> T {
    OGNL_LOCALS.with(|scopes| scopes.borrow_mut().push(HashMap::new()));
    struct OgnlLocalsGuard;
    impl Drop for OgnlLocalsGuard {
        fn drop(&mut self) {
            OGNL_LOCALS.with(|scopes| {
                scopes.borrow_mut().pop();
            });
        }
    }
    let _guard = OgnlLocalsGuard;
    operation()
}

fn current_ognl_runtime() -> Option<Arc<dyn OgnlRuntime>> {
    OGNL_RUNTIMES.with(|runtimes| runtimes.borrow().last().cloned())
}

fn with_ognl_runtime<T>(runtime: Arc<dyn OgnlRuntime>, operation: impl FnOnce() -> T) -> T {
    OGNL_RUNTIMES.with(|runtimes| runtimes.borrow_mut().push(runtime));
    struct OgnlRuntimeGuard;
    impl Drop for OgnlRuntimeGuard {
        fn drop(&mut self) {
            OGNL_RUNTIMES.with(|runtimes| {
                runtimes.borrow_mut().pop();
            });
        }
    }
    let _guard = OgnlRuntimeGuard;
    operation()
}

fn processing_error(message: String) -> super::StandardExpressionError {
    Box::new(TemplateProcessingException::new(Some(message)))
}

fn processing_error_with_cause<E>(message: String, cause: E) -> super::StandardExpressionError
where
    E: std::error::Error + Send + Sync + 'static,
{
    Box::new(TemplateProcessingException::with_cause(
        Some(message),
        cause,
    ))
}

fn ognl_processing_error(message: String, ognl_message: String) -> super::StandardExpressionError {
    Box::new(TemplateProcessingException::with_cause(
        Some(message),
        OgnlException::new(ognl_message),
    ))
}

// ===========================================================================
// 表达式动态分派器直接单测（不经过渲染管线）
// ===========================================================================

#[cfg(test)]
#[cfg(test)]
mod dispatcher_direct_tests {
    use super::{
        ComputedExpression, invoke_java_string_method, invoke_static_method, parse_ognl_range,
    };
    use crate::expression::TemplateValue;
    use crate::util::{JavaNumber, JavaString};
    use std::sync::Arc;

    fn js(value: &str) -> JavaString {
        JavaString::from_rust_str(value)
    }

    fn text(value: &Arc<TemplateValue>) -> String {
        value
            .to_java_string()
            .map_or_else(|| "null".to_owned(), |value| value.to_string_lossy())
    }

    #[test]
    fn invoke_java_string_method_dispatch_matches_java() {
        let target = js("Hello World");
        let result = invoke_java_string_method(&target, &js("length"), &[])
            .expect("length ok")
            .expect("non-null");
        assert_eq!(text(&result), "11");
        let result = invoke_java_string_method(&js(""), &js("isEmpty"), &[])
            .expect("isEmpty ok")
            .expect("non-null");
        assert_eq!(text(&result), "true");
        let result = invoke_java_string_method(&target, &js("toString"), &[])
            .expect("toString ok")
            .expect("non-null");
        assert_eq!(text(&result), "Hello World");
        let result = invoke_java_string_method(
            &target,
            &js("contains"),
            &[Some(Arc::new(TemplateValue::string(js("World"))))],
        )
        .expect("contains ok")
        .expect("non-null");
        assert_eq!(text(&result), "true");
        let result = invoke_java_string_method(
            &target,
            &js("contains"),
            &[Some(Arc::new(TemplateValue::string(js("xyz"))))],
        )
        .expect("contains ok")
        .expect("non-null");
        assert_eq!(text(&result), "false");
        let result = invoke_java_string_method(
            &target,
            &js("charAt"),
            &[Some(Arc::new(TemplateValue::Number(JavaNumber::Integer(
                1,
            ))))],
        )
        .expect("charAt ok")
        .expect("non-null");
        assert_eq!(text(&result), "e");
        // 未知方法 -> 错误（Java String 方法分派拒绝）
        assert!(invoke_java_string_method(&target, &js("noSuchMethod"), &[]).is_err());
        // 参数个数不匹配 -> 错误
        assert!(invoke_java_string_method(&target, &js("charAt"), &[]).is_err());
    }

    #[test]
    fn invoke_static_method_dispatch_matches_java() {
        let number = |value: i64| Some(Arc::new(TemplateValue::Number(JavaNumber::Long(value))));
        // Java Math 静态方法返回 double：sqrt(16.0)=4.0、ceil(7.0)=7.0
        let result = invoke_static_method("java.lang.Math", "sqrt", &[number(16)])
            .expect("sqrt ok")
            .expect("non-null");
        assert_eq!(text(&result), "4.0");
        let result = invoke_static_method("java.lang.Math", "ceil", &[number(7)])
            .expect("ceil ok")
            .expect("non-null");
        assert_eq!(text(&result), "7.0");
        // abs 用 double 输入（Java Math.abs(double) 返回 double）
        let double = Some(Arc::new(TemplateValue::Number(JavaNumber::Double(-5.5))));
        let result = invoke_static_method("java.lang.Math", "abs", &[double])
            .expect("abs ok")
            .expect("non-null");
        assert_eq!(text(&result), "5.5");
        // 未知静态成员 -> 错误
        assert!(invoke_static_method("java.lang.Math", "noSuchMember", &[]).is_err());
        // 空类名 -> 错误
        assert!(invoke_static_method("", "abs", &[number(1)]).is_err());
    }

    #[test]
    fn parse_ognl_range_sequence_and_assignment() {
        // 集合字面量 {1,2,3} -> ListLiteral 3 项
        let input: Vec<u16> = "{1,2,3}".encode_utf16().collect();
        let parsed = parse_ognl_range(&input, true, false).expect("list literal parses");
        match parsed {
            ComputedExpression::ListLiteral(items) => {
                assert_eq!(items.len(), 3, "list literal 为 3 项");
            }
            _ => panic!("expected ListLiteral for {{1,2,3}}"),
        }

        // 顶层逗号序列 1,2,3 -> Sequence 3 项
        let input: Vec<u16> = "1,2,3".encode_utf16().collect();
        let parsed = parse_ognl_range(&input, true, false).expect("sequence parses");
        match parsed {
            ComputedExpression::Sequence(items) => {
                assert_eq!(items.len(), 3, "顶层序列为 3 项");
            }
            _ => panic!("expected Sequence for 1,2,3"),
        }

        // 赋值：`#x = 'y'`
        let input: Vec<u16> = "#x = 'y'".encode_utf16().collect();
        let parsed = parse_ognl_range(&input, true, false).expect("assignment parses");
        match parsed {
            ComputedExpression::Assignment { name, .. } => {
                assert_eq!(name.to_string_lossy(), "x");
            }
            _ => panic!("expected Assignment"),
        }

        // 无效输入 -> None
        assert!(parse_ognl_range(&[], true, false).is_none());
        let bad: Vec<u16> = "..".encode_utf16().collect();
        assert!(parse_ognl_range(&bad, true, false).is_none());
        // 非 # 前缀的赋值目标 -> None
        let bad: Vec<u16> = "x = 1".encode_utf16().collect();
        assert!(parse_ognl_range(&bad, true, false).is_none());
    }
}
