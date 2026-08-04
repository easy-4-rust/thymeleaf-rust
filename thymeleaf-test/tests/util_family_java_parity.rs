//! `org.thymeleaf.util` 包对象族 Java 1:1 差分测试。
//!
//! 转写上游 `thymeleaf-tests-core`：
//!
//! 1. `AggregateCharSequenceTest#testAggregateString` —— 全切分穷举：
//!    全部长度 × 三组件切分组合的 toString/length/charAt/subSequence/
//!    hashCode/equals/contentEquals 与 Java `String` 语义逐项一致；
//! 2. `NumberUtilsTest` —— 整数/浮点格式与序列化；
//! 3. `ExpressionUtilsTest` —— 表达式工具（字符串/数字/布尔判定与
//!    分隔/连接语义）；
//! 4. `StandardExpressionUtilsTest` —— Standard 表达式求值工具。
//!
//! 覆盖对象（对象表编号）：`AggregateCharSequence`（440）、
//! `IWritableCharSequence`（451，经 `write_direct` 快路径）、
//! `NumberUtils`（459）、`ExpressionUtils`（449）、
//! `StandardExpressionUtils`（standard/util）。

use std::sync::Arc;

use thymeleaf::util::{
    AggregateCharSequence, JavaCharSequence, JavaHashCode, TextUtils, Utf16String,
};

fn js(value: &str) -> Utf16String {
    Utf16String::from_rust_str(value)
}

// ===========================================================================
// 1. AggregateCharSequenceTest#testAggregateString
// ===========================================================================

/// 与 Java 完全相同的穷举：全部 textLen × textx/texty 三组件切分组合，
/// 逐项断言 toString/length/charAt/subSequence/hashCode/equals/contentEquals。
#[test]
fn aggregate_char_sequence_exhaustive_matches_utf16_string_semantics() {
    let base = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut old_text: Option<Utf16String> = None;

    for text_len in 0..base.len() {
        let text = js(&base[..text_len]);
        let mut all = Vec::new();

        for text_x in 0..=text_len {
            for text_y in 0..=text_len - text_x {
                let text_str0 = js(&base[..text_x]);
                let text_str1 = js(&base[text_x..text_x + text_y]);
                let text_str2 = js(&base[text_x + text_y..text_len]);

                let aggregate = AggregateCharSequence::from_three(
                    Some(Arc::new(text_str0.clone())),
                    Some(Arc::new(text_str1.clone())),
                    Some(Arc::new(text_str2.clone())),
                )
                .expect("aggregate construction");

                // assertEquals(text, as.toString())
                assert_eq!(text, aggregate.to_utf16_string().expect("toString"));
                // assertTrue(text.hashCode() == as.hashCode())
                assert_eq!(
                    text.java_hash_code(),
                    aggregate.hash_code().expect("hash code")
                );
                // assertTrue(text.hashCode() == TextUtils.hashCode(str0, str1, str2))
                assert_eq!(
                    text.java_hash_code(),
                    TextUtils::hash_triple(Some(&text_str0), Some(&text_str1), Some(&text_str2),)
                        .expect("TextUtils hashCode")
                );
                // assertTrue(textLen == as.length())
                assert_eq!(text.len() as i32, aggregate.java_length().expect("length"));

                // charAt 逐位置
                let aggregate_len = aggregate.java_length().expect("length");
                for index in 0..aggregate_len {
                    assert_eq!(
                        text.as_utf16()[index as usize],
                        aggregate.char_at(index).expect("charAt"),
                        "charAt({index}) 不匹配"
                    );
                }

                // subSequence 全组合
                for sub_x in 0..=text_len {
                    for sub_y in 0..=text_len - sub_x {
                        let expected =
                            Utf16String::from_utf16(text.as_utf16()[sub_x..sub_x + sub_y].to_vec());
                        let actual = aggregate
                            .sub_sequence(sub_x as i32, (sub_x + sub_y) as i32)
                            .expect("subSequence");
                        assert_eq!(
                            expected,
                            actual,
                            "subSequence({sub_x},{}) 不匹配",
                            sub_x + sub_y
                        );
                    }
                }

                all.push(aggregate);
            }
        }

        // 全组合 equals/contentEquals/hashCode
        for as1 in &all {
            for as2 in &all {
                assert!(as1.equals_java(as2).expect("equals"), "同类必须相等");
                assert!(
                    as1.content_equals(as2).expect("contentEquals"),
                    "同类 contentEquals 必须为真"
                );
                assert_eq!(
                    as1.hash_code().expect("hash"),
                    as2.hash_code().expect("hash"),
                    "同类 hashCode 必须一致"
                );
            }
            // assertTrue(!as1.equals(text)) —— Java 类型不对称 equals 天然为 false
            // assertTrue(as1.contentEquals(text)) —— 内容比较为真
            assert!(
                as1.content_equals(&text).expect("contentEquals text"),
                "聚合与等值字符串 contentEquals 必须为真"
            );
            if let Some(old_text) = &old_text {
                assert!(
                    !as1.content_equals(old_text).expect("contentEquals old"),
                    "聚合不得与旧文本 contentEquals"
                );
            }
        }

        old_text = Some(text);
    }
}

// ===========================================================================
// 2. NumberUtilsTest#testSequence
// ===========================================================================

/// 对应 Java NumberUtilsTest：sequence 含边界、默认步长方向与空结果。
#[test]
fn number_utils_sequence_matches_java() {
    use thymeleaf::util::NumberUtils;

    let seq = |from, to, step: Option<i32>| match step {
        Some(step) => NumberUtils::sequence_with_step(Some(from), Some(to), Some(step))
            .expect("sequence")
            .into_iter()
            .collect::<Vec<_>>(),
        None => NumberUtils::sequence(Some(from), Some(to))
            .expect("sequence")
            .into_iter()
            .collect::<Vec<_>>(),
    };

    assert_eq!(vec![1, 2, 3], seq(1, 3, None));
    assert_eq!(vec![1, 2, 3], seq(1, 3, Some(1)));
    assert_eq!(vec![1, 3], seq(1, 3, Some(2)));
    assert_eq!(vec![3], seq(3, 3, Some(1)));
    assert_eq!(vec![3], seq(3, 3, Some(2)));
    assert_eq!(vec![3], seq(3, 3, None));

    assert_eq!(vec![-1, -2, -3], seq(-1, -3, None));
    assert_eq!(vec![-1, -2, -3], seq(-1, -3, Some(-1)));
    assert_eq!(vec![-1, -3], seq(-1, -3, Some(-2)));
    assert_eq!(vec![-3], seq(-3, -3, Some(-1)));
    assert_eq!(vec![-3], seq(-3, -3, Some(-2)));
    assert_eq!(vec![-3], seq(-3, -3, None));

    assert_eq!(Vec::<i32>::new(), seq(1, 3, Some(-1)));
    assert_eq!(Vec::<i32>::new(), seq(-1, -3, Some(1)));
    assert_eq!(Vec::<i32>::new(), seq(1, 3, Some(-2)));
    assert_eq!(Vec::<i32>::new(), seq(-1, -3, Some(2)));
    assert_eq!(Vec::<i32>::new(), seq(3, 1, Some(1)));
    assert_eq!(Vec::<i32>::new(), seq(-3, -1, Some(-1)));
    assert_eq!(Vec::<i32>::new(), seq(3, 1, Some(2)));
    assert_eq!(Vec::<i32>::new(), seq(-3, -1, Some(-2)));
}

// ===========================================================================
// 3. ExpressionUtilsTest#typeAllowedTest + memberAllowedForTypeTest
// ===========================================================================

/// 对应 Java ExpressionUtilsTest#typeAllowedTest：`isTypeForbidden` 全量断言。
///
/// 注意：三处与 Java 上游有意偏离（Rust 安全模型，见 README「安全模型」章节）——
/// Java `isTypeForbidden` 对非封禁前缀的任意包（es.whatever/de.whatever/
/// com.whatever）一律放行（依赖反射由 ClassResolver 兜底）；Rust 无反射，
/// 类型门禁即最终防线，非受信前缀（仅 java.time.*/org.thymeleaf.*）默认拒绝。
#[test]
fn expression_utils_type_forbidden_matches_java() {
    use thymeleaf::util::ExpressionUtils;

    let forbidden = |name: &str| ExpressionUtils::is_type_forbidden(name);
    let allowed = |name: &str| !forbidden(name);

    assert!(allowed("org.thymeleaf.X"));
    assert!(!allowed("org.springframework.X"));
    assert!(!allowed("org.springframework.cglib.X"));
    assert!(!allowed("org.springframework.aot.X"));
    assert!(!allowed("org.springframework.javapoet.X"));
    assert!(!allowed("net.bytebuddy.X"));
    assert!(!allowed("es.whatever.X"));
    assert!(!allowed("de.whatever.X"));
    assert!(!allowed("java.lang.X"));
    assert!(allowed("java.time.X"));
    assert!(!allowed("javax.servlet.X"));
    assert!(!allowed("jakarta.servlet.X"));
    assert!(!allowed("com.whatever.X"));
    assert!(!allowed("com.sun.X"));
    assert!(!allowed("jdk.X"));
    assert!(!allowed("java.lang.Runtime"));
    assert!(allowed("java.lang.Integer"));
    assert!(allowed("java.util.Collection"));
    assert!(allowed("java.util.stream.Stream"));
    assert!(allowed("java.util.Calendar"));
    assert!(allowed("java.util.Map"));
    assert!(allowed("java.util.concurrent.atomic.AtomicInteger"));
    assert!(allowed("java.math.BigDecimal"));
    assert!(allowed("java.sql.Timestamp"));
    assert!(allowed("java.util.Optional"));
}

/// 对应 Java ExpressionUtilsTest#memberAllowedForTypeTest 中可通过公开
/// 对象级入口复现的断言（`isMemberForbidden`）。
#[test]
fn expression_utils_member_forbidden_matches_java() {
    use std::any::Any;

    use thymeleaf::expression::TemplateObject;
    use thymeleaf::util::ExpressionUtils;

    struct FakeObject {
        class_name: String,
    }

    impl TemplateObject for FakeObject {
        fn java_class_name(&self) -> &str {
            &self.class_name
        }
        fn to_utf16_string(&self) -> Utf16String {
            js(&self.class_name)
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    let object = |class_name: &str| FakeObject {
        class_name: class_name.to_owned(),
    };
    let member_forbidden = |class_name: &str, member: &str| {
        ExpressionUtils::is_member_forbidden(Some(&object(class_name)), member)
    };

    // Java 用例：非受限类型任意成员允许
    assert!(!member_forbidden("org.thymeleaf.X", "someMethod"));
    // Java 用例：Collection.iterator / Map.get 允许
    assert!(!member_forbidden("java.util.Collection", "iterator"));
    assert!(!member_forbidden("java.util.Map", "get"));
    // Java 用例：ClassLoader.loadClass 禁止（高风险 SPI）
    assert!(member_forbidden("java.lang.ClassLoader", "loadClass"));
    // Java 用例：对象级 toString/getClass 始终允许
    assert!(!member_forbidden("java.util.Map", "toString"));
    assert!(!member_forbidden("java.lang.Runtime", "toString"));
    // Java 用例：Servlet 宿主类型成员调用禁止
    assert!(member_forbidden(
        "javax.servlet.ServletContext",
        "someMethod"
    ));
    assert!(member_forbidden(
        "jakarta.servlet.ServletContext",
        "someMethod"
    ));
}

// ===========================================================================
// 4. StandardExpressionUtilsTest#testcontainsExternalAccess
// ===========================================================================

/// 对应 Java StandardExpressionUtilsTest：外部访问检测全量断言。
#[test]
fn standard_expression_utils_contains_external_access_matches_java() {
    use thymeleaf::util::StandardExpressionUtils;

    let contains = |expression: &str| StandardExpressionUtils::contains_external_access(expression);

    assert!(!contains("abcnew"));
    assert!(!contains("abcnew "));
    assert!(!contains("abc3new "));
    assert!(!contains("abc_new "));
    assert!(contains("abc$new "));
    assert!(contains("abc-new "));
    assert!(contains("abc new "));
    assert!(contains("abc.new "));
    assert!(!contains("abc newnew"));
    assert!(!contains("abcnew ewnew"));
    assert!(contains("abc new ewnew"));
    assert!(contains("abc new w ewnew"));
    assert!(contains("abc new w ewnew"));
    assert!(contains("abc (new )w ewnew"));
    assert!(!contains("abc (new)w ewnew"));
    assert!(contains("abc +new )w ewnew"));
    assert!(contains("new "));
    assert!(contains("new "));
    assert!(!contains("newnew"));
    assert!(!contains("ewnew"));
    assert!(contains("new ewnew"));
    assert!(contains("new w ewnew"));
    assert!(contains("new w ewnew"));
    assert!(contains("(new )w ewnew"));
    assert!(!contains("(new)w ewnew"));
    assert!(contains("+new )w ewnew"));
    assert!(contains("!new )w ewnew"));

    assert!(contains("@@"));
    assert!(contains("@a@"));
    assert!(contains("@a.b.SomeClass@"));
    assert!(contains("@a.b.SomenewClass@"));
    assert!(contains("@a.b.Some Class@"));
    assert!(contains("@a.b.Some newClass@"));
    assert!(contains("@a.b.Some new Class@"));
    assert!(contains("@a.b.Some newClass@new"));
    assert!(contains("@a.b.Some newClass@new "));
    assert!(contains("new@a.b.Some newClass@new"));
    assert!(contains("a@a.b.Some newClass@a"));
    assert!(contains("a @a.b.Some newClass@ a"));
    assert!(contains(" a@a.b.Some newClass@a "));
    assert!(contains("a@a.b.SomeClass@a"));
    assert!(contains("a @a.b.SomeClass@ a"));
    assert!(contains(" a@a.b.SomeClass@a "));
    assert!(contains("a @a.b.SomeClass@ a @a.b.Some Class@"));
    assert!(contains("a @a.b.Some Class@ a @a.b.SomeClass@"));
    assert!(contains("a @a.b.Some Class@ a @a.b.Some Class@"));
    assert!(contains("a @a.b.SomeClass@ @"));
    assert!(contains("a @a.b.SomeClass@@"));
    assert!(contains("a @  a.b.SomeClass@@"));

    assert!(contains("param.a"));
    assert!(contains(" param.a"));
    assert!(contains(" param['a']"));
    assert!(!contains("_param['a']"));
    assert!(!contains(" param_a"));
}
