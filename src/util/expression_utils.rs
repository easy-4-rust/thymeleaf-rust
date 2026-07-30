use crate::expression::TemplateObject;

/// 变量表达式类型与成员访问安全规则。
///
/// 对应 Java: `org.thymeleaf.util.ExpressionUtils`。
pub struct ExpressionUtils;

impl ExpressionUtils {
    /// 统一控制字符及 Unicode 空白；可选执行与 Java `toLowerCase` 等价的大小写归一化。
    #[must_use]
    pub fn normalize(expression: Option<&str>, normalize_case: bool) -> Option<String> {
        expression.map(|expression| {
            let expression = if normalize_case {
                expression.to_lowercase()
            } else {
                expression.to_owned()
            };
            let mut normalized = String::with_capacity(expression.len());
            for character in expression.chars() {
                if character != '\n'
                    && (character < '\u{20}'
                        || ('\u{7f}'..='\u{9f}').contains(&character)
                        || character.is_whitespace())
                {
                    if character.is_whitespace() {
                        normalized.push(' ');
                    }
                } else {
                    normalized.push(character);
                }
            }
            normalized
        })
    }

    /// 判断静态类型引用是否被 Thymeleaf 安全策略禁止。
    #[must_use]
    pub fn is_type_forbidden(type_name: &str) -> bool {
        let normalized = Self::normalize(Some(type_name), false).expect("non-null");
        if !is_type_blocked_for_type_reference(&normalized) {
            return false;
        }
        !ALLOWED_JAVA_CLASS_NAMES.contains(&normalized.as_str())
            && !ALLOWED_JAVA_SUPERS_NAMES.contains(&normalized.as_str())
    }

    /// 判断类型名是否位于精确的 `java.` 根包。
    ///
    /// 对应 Java: `ExpressionUtils#isJavaPackage(String)`。
    #[must_use]
    pub(crate) fn is_java_package(type_name: &str) -> bool {
        type_name.starts_with("java.")
    }

    /// 判断类型是否实现禁止成员调用的高风险 SPI。
    ///
    /// 对应 Java: `ExpressionUtils#isTypeBlockedForMemberCalls(Class)`；Rust 无
    /// `Class#isAssignableFrom`，由 `TemplateObject::java_class_name` 的稳定类型名承接。
    #[must_use]
    pub(crate) fn is_type_blocked_for_member_calls(type_name: &str) -> bool {
        BLOCKED_MEMBER_CALL_JAVA_SUPERS_NAMES.contains(&type_name)
    }

    /// 按运行时类型名判断成员调用是否被禁止。
    ///
    /// 对应 Java: `ExpressionUtils#isMemberForbiddenForInstanceOfType(Class,String)`。
    #[must_use]
    pub(crate) fn is_member_forbidden_for_instance_of_type(
        type_name: &str,
        member_name: &str,
    ) -> bool {
        if !is_type_blocked_for_all_purposes(type_name)
            && !Self::is_type_blocked_for_member_calls(type_name)
        {
            return false;
        }
        if ALLOWED_JAVA_CLASS_NAMES.contains(&type_name) {
            return false;
        }
        !allowed_super_member(type_name, member_name)
    }

    /// 判断对运行时对象成员的访问是否被禁止。
    ///
    /// Rust 迁移以对象声明的 Java 类名执行同一白名单策略；`getClass` 与 `toString`
    /// 始终允许。
    #[must_use]
    pub fn is_member_forbidden(target: Option<&dyn TemplateObject>, member_name: &str) -> bool {
        let Some(target) = target else {
            return false;
        };
        let normalized = Self::normalize(Some(member_name), false).expect("non-null");
        if matches!(normalized.as_str(), "getClass" | "toString") {
            return false;
        }
        let class_name = target.java_class_name();
        if class_name == "java.lang.Class" {
            return !ALLOWED_CLASS_METHODS.contains(&normalized.as_str());
        }
        Self::is_member_forbidden_for_instance_of_type(class_name, &normalized)
    }
}

const BLOCKED_ALL_PURPOSES_PACKAGE_NAME_PREFIXES: &[&str] = &[
    "java.",
    "javax.",
    "jakarta.",
    "jdk.",
    "org.ietf.jgss.",
    "org.omg.",
    "org.w3c.dom.",
    "org.xml.sax.",
    "com.sun.",
    "sun.",
];
const ALLOWED_ALL_PURPOSES_PACKAGE_NAME_PREFIXES: &[&str] = &["java.time."];
const BLOCKED_TYPE_REFERENCE_PACKAGE_NAME_PREFIXES: &[&str] = &[
    "com.squareup.javapoet.",
    "com.zaxxer.hikari.",
    "com.fasterxml.jackson.",
    "tools.jackson.",
    "groovy.",
    "io.netty.",
    "javassist.",
    "javax0.geci.",
    "kotlin.",
    "net.bytebuddy.",
    "net.sf.cglib.",
    "org.apache.tomcat.jdbc.",
    "org.apache.commons.dbcp2.",
    "org.apache.commons.lang.reflect.",
    "org.apache.commons.lang3.reflect.",
    "org.apache.bcel.",
    "org.apache.logging.",
    "org.aspectj.",
    "org.codehaus.groovy.",
    "org.eclipse.jetty.",
    "org.glassfish.",
    "org.javassist.",
    "org.jboss.",
    "org.jetbrains.kotlin.",
    "org.jruby.",
    "org.junit.",
    "org.mockito.",
    "org.mortbay.jetty.",
    "org.objectweb.asm.",
    "org.objenesis.",
    "org.python.",
    "org.springframework.",
    "scala.",
];
const ALLOWED_JAVA_CLASS_NAMES: &[&str] = &[
    "java.lang.Boolean",
    "java.lang.Byte",
    "java.lang.Character",
    "java.lang.Double",
    "java.lang.Enum",
    "java.lang.Float",
    "java.lang.Integer",
    "java.lang.Long",
    "java.lang.Math",
    "java.lang.Number",
    "java.lang.Short",
    "java.lang.String",
    "java.math.BigDecimal",
    "java.math.BigInteger",
    "java.math.RoundingMode",
    "java.util.ArrayList",
    "java.util.LinkedList",
    "java.util.HashMap",
    "java.util.LinkedHashMap",
    "java.util.HashSet",
    "java.util.LinkedHashSet",
    "java.util.Iterator",
    "java.util.Enumeration",
    "java.util.Deque",
    "java.util.Locale",
    "java.util.Properties",
    "java.util.Date",
    "java.util.Calendar",
    "java.util.Optional",
    "java.util.OptionalDouble",
    "java.util.OptionalInt",
    "java.util.OptionalLong",
    "java.util.UUID",
    "java.util.Currency",
    "java.util.concurrent.atomic.AtomicBoolean",
    "java.util.concurrent.atomic.AtomicInteger",
    "java.util.concurrent.atomic.AtomicIntegerArray",
    "java.util.concurrent.atomic.AtomicIntegerFieldUpdater",
    "java.util.concurrent.atomic.AtomicLong",
    "java.util.concurrent.atomic.AtomicLongArray",
    "java.util.concurrent.atomic.AtomicLongFieldUpdater",
    "java.util.concurrent.atomic.AtomicMarkableReference",
    "java.util.concurrent.atomic.AtomicReference",
    "java.util.concurrent.atomic.AtomicReferenceArray",
    "java.util.concurrent.atomic.AtomicReferenceFieldUpdater",
    "java.util.concurrent.atomic.AtomicStampedReference",
    "java.util.concurrent.atomic.DoubleAccumulator",
    "java.util.concurrent.atomic.DoubleAdder",
    "java.util.concurrent.atomic.LongAccumulator",
    "java.util.concurrent.atomic.LongAdder",
    "java.sql.Date",
    "java.sql.Time",
    "java.sql.Timestamp",
];
const ALLOWED_JAVA_SUPERS_NAMES: &[&str] = &[
    "java.lang.CharSequence",
    "java.util.Collection",
    "java.lang.Iterable",
    "java.util.Iterator",
    "java.util.List",
    "java.util.Map",
    "java.util.Map$Entry",
    "java.util.Set",
    "java.util.Calendar",
    "java.util.TimeZone",
    "java.util.stream.Stream",
];
const BLOCKED_MEMBER_CALL_JAVA_SUPERS_NAMES: &[&str] = &[
    "java.lang.ClassLoader",
    "org.thymeleaf.standard.expression.IStandardVariableExpressionEvaluator",
    "org.thymeleaf.standard.expression.IStandardExpressionParser",
    "org.thymeleaf.standard.expression.IStandardConversionService",
    "org.springframework.web.servlet.support.RequestContext",
    "org.springframework.web.reactive.result.view.RequestContext",
    "org.springframework.core.io.ResourceLoader",
];
const ALLOWED_CLASS_METHODS: &[&str] = &[
    "getName",
    "getSimpleName",
    "isAssignableFrom",
    "isInstance",
    "isInterface",
    "isPrimitive",
    "isRecord",
    "isAnnotation",
    "isArray",
    "isEnum",
];

fn is_type_blocked_for_all_purposes(type_name: &str) -> bool {
    if ExpressionUtils::is_java_package(type_name)
        && ALLOWED_ALL_PURPOSES_PACKAGE_NAME_PREFIXES
            .iter()
            .any(|prefix| type_name.starts_with(prefix))
    {
        return false;
    }
    BLOCKED_ALL_PURPOSES_PACKAGE_NAME_PREFIXES
        .iter()
        .any(|prefix| type_name.starts_with(prefix))
}

fn is_type_blocked_for_type_reference(type_name: &str) -> bool {
    is_type_blocked_for_all_purposes(type_name)
        || BLOCKED_TYPE_REFERENCE_PACKAGE_NAME_PREFIXES
            .iter()
            .any(|prefix| type_name.starts_with(prefix))
}

fn allowed_super_member(class_name: &str, member_name: &str) -> bool {
    if class_name == "java.util.stream.Stream" {
        return matches!(member_name, "count" | "iterator");
    }
    if class_name == "java.util.Map$Entry" {
        return matches!(member_name, "getKey" | "getValue" | "key" | "value");
    }
    // Java 版通过 Class#isAssignableFrom 允许 Calendar 父类声明的方法。
    // Rust 侧以稳定 Java 类型名显式表达 GregorianCalendar → Calendar 的关系。
    if class_name == "java.util.GregorianCalendar" {
        return matches!(
            member_name,
            "get"
                | "getTime"
                | "getTimeInMillis"
                | "getTimeZone"
                | "isLenient"
                | "getFirstDayOfWeek"
                | "getMinimalDaysInFirstWeek"
        );
    }
    let collection_like = matches!(
        class_name,
        "java.util.ArrayList"
            | "java.util.LinkedList"
            | "java.util.HashMap"
            | "java.util.LinkedHashMap"
            | "java.util.HashSet"
            | "java.util.LinkedHashSet"
    );
    collection_like
        && matches!(
            member_name,
            "size"
                | "isEmpty"
                | "contains"
                | "containsKey"
                | "containsValue"
                | "get"
                | "iterator"
                | "stream"
                | "entrySet"
                | "keySet"
                | "values"
        )
}
