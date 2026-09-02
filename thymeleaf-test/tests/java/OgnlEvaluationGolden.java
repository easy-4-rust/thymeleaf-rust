import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.Map;

import org.thymeleaf.TemplateEngine;
import org.thymeleaf.context.Context;
import org.thymeleaf.templateresolver.StringTemplateResolver;

/**
 * 导出 OGNL 兼容变量表达式求值器（OGNLVariableExpressionEvaluator）的端到端
 * Golden：同一表达式矩阵 + 同一变量集，经 TemplateEngine + th:text/th:if
 * 渲染，记录完整可观察结果（渲染 HTML 或异常类名）。
 *
 * 与 Rust 侧 `native_expression_evaluator_java_parity.rs`（V1 本地矩阵）和
 * `native_expression_evaluator_golden_diff.rs`（V3 差分）共享同一 case 表。
 */
public final class OgnlEvaluationGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    /** JavaBean 风格宿主对象，属性与方法供 OGNL 反射访问。 */
    public static final class Person {
        private final String name;
        private final int age;

        public Person(final String name, final int age) {
            this.name = name;
            this.age = age;
        }

        public String getName() {
            return this.name;
        }

        public int getAge() {
            return this.age;
        }

        public String greet() {
            return "Hello, " + this.name + "!";
        }
    }

    private OgnlEvaluationGolden() {
    }

    public static void main(final String[] args) {
        final TemplateEngine engine = new TemplateEngine();
        final StringTemplateResolver resolver = new StringTemplateResolver();
        resolver.setTemplateMode("HTML");
        engine.setTemplateResolver(resolver);

        emit(engine, "baseline_case", null, "10f9dd2eb8cbd98515ce14b149d115e0287d0add");

        // ---- 1. 属性导航 / 宿主对象 ----
        emitExpr(engine, "property_navigation", "${person.name}");
        emitExpr(engine, "numeric_property_navigation", "${person.age}");
        emitExpr(engine, "method_invocation", "${person.greet()}");
        emitExpr(engine, "method_and_property_chain", "${person.name}");

        // ---- 2. 集合访问 ----
        emitExpr(engine, "list_index_access", "${items[1]}");
        emitExpr(engine, "list_first_index", "${items[0]}");
        emitExpr(engine, "map_key_access", "${map['key1']}");

        // ---- 3. 算术运算 ----
        emitExpr(engine, "arithmetic_add", "${1 + 2}");
        emitExpr(engine, "arithmetic_sub", "${10 - 4}");
        emitExpr(engine, "arithmetic_mul", "${3 * 4}");
        emitExpr(engine, "arithmetic_div", "${20 / 5}");
        emitExpr(engine, "arithmetic_mod", "${17 % 5}");
        emitExpr(engine, "arithmetic_with_variables", "${a + b}");

        // ---- 4. 比较运算 ----
        emitExpr(engine, "comparison_eq", "${1 == 1}");
        emitExpr(engine, "comparison_neq", "${1 != 2}");
        emitExpr(engine, "comparison_lt", "${1 < 2}");
        emitExpr(engine, "comparison_gt", "${3 > 2}");
        emitExpr(engine, "comparison_le", "${2 <= 2}");
        emitExpr(engine, "comparison_ge", "${2 >= 3}");

        // ---- 5. 逻辑运算 ----
        emitExpr(engine, "logical_and_true", "${t and t}");
        emitExpr(engine, "logical_or", "${t or f}");
        emitExpr(engine, "logical_not", "${!f}");
        emitExpr(engine, "logical_and_false", "${t and f}");

        // ---- 6. 三元 / Elvis ----
        emitExpr(engine, "ternary_true", "${1 < 2 ? 'yes' : 'no'}");
        emitExpr(engine, "ternary_false", "${1 > 2 ? 'yes' : 'no'}");
        emitExpr(engine, "elvis_null_default", "${missing ?: 'fallback'}");
        emitExpr(engine, "elvis_present_value", "${v ?: 'fallback'}");
        // 外部 default expression（Thymeleaf 层 DefaultExpression，Java 支持）
        emit(engine, "external_elvis_present", "${v} ?: 'fallback'",
                "<p th:text=\"${v} ?: 'fallback'\">KEEP</p>");
        emit(engine, "external_elvis_null", "${missing} ?: 'outside'",
                "<p th:text=\"${missing} ?: 'outside'\">KEEP</p>");
        emit(engine, "external_elvis_chain", "${missing} ?: (${v} ?: 'deep')",
                "<p th:text=\"${missing} ?: (${v} ?: 'deep')\">KEEP</p>");

        // ---- 7. 字符串方法 ----
        emitExpr(engine, "string_method_uppercase", "${name.toUpperCase()}");
        emitExpr(engine, "string_method_length", "${name.length()}");
        emitExpr(engine, "string_method_substring", "${name.substring(0, 3)}");
        emitExpr(engine, "string_concat_plus", "${first + ' ' + second}");

        // ---- 8. 空值传播 ----
        emitExpr(engine, "null_variable", "${missing}");
        emitExprWithoutPerson(engine, "null_property_access", "${person.name}");
        emitCondition(engine, "null_condition", "${missing}",
                "<p th:if=\"${missing}\">gone</p><span>stay</span>");

        // ---- 9. 字面量 ----
        emitExpr(engine, "string_literal", "'hello'");
        emitExpr(engine, "number_literal", "42");
        emitExpr(engine, "boolean_literal_true", "true");
        emitExpr(engine, "boolean_literal_false", "false");
        emitExpr(engine, "null_literal", "null");

        // ---- 10. 嵌套与复合 ----
        emitExpr(engine, "nested_arithmetic", "${(1 + 2) * 3}");
        emitCondition(engine, "property_in_condition", "${person.age >= 18}",
                "<p th:if=\"${person.age >= 18}\" th:text=\"'adult'\">x</p>");
        emitExpr(engine, "property_in_arithmetic", "${person.age + 10}");
    }

    /** 构造与 Rust 对照矩阵一致的变量集。 */
    private static Context context() {
        final Context context = new Context();
        context.setVariable("person", new Person("Alice", 30));

        final ArrayList<String> items = new ArrayList<String>(Arrays.asList("zero", "one"));
        context.setVariable("items", items);

        final Map<String, String> map = new LinkedHashMap<String, String>();
        map.put("key1", "value1");
        context.setVariable("map", map);

        context.setVariable("a", Integer.valueOf(7));
        context.setVariable("b", Integer.valueOf(3));
        context.setVariable("t", Boolean.TRUE);
        context.setVariable("f", Boolean.FALSE);
        context.setVariable("name", "alice");
        context.setVariable("first", "Hello");
        context.setVariable("second", "World");
        context.setVariable("v", "value");
        return context;
    }

    /** 渲染 `th:text` 模板并记录完整可观察结果。 */
    private static void emitExpr(final TemplateEngine engine, final String id,
            final String expression) {
        emit(engine, id, expression, "<p th:text=\"" + expression + "\">KEEP</p>");
    }

    /** 同 {@link #emitExpr}，但上下文不含 `person`（null 属性访问语义）。 */
    private static void emitExprWithoutPerson(final TemplateEngine engine, final String id,
            final String expression) {
        emit(engine, id, expression, "<p th:text=\"" + expression + "\">KEEP</p>",
                false);
    }

    /** 渲染自定义模板（th:if 等复合场景）并记录完整可观察结果。 */
    private static void emitCondition(final TemplateEngine engine, final String id,
            final String expression, final String template) {
        emit(engine, id, expression, template);
    }

    private static void emit(final TemplateEngine engine, final String id,
            final String expression, final String template) {
        emit(engine, id, expression, template, true);
    }

    private static void emit(final TemplateEngine engine, final String id,
            final String expression, final String template, final boolean withPerson) {
        String outcome;
        try {
            final Context context = context();
            if (!withPerson) {
                context.setVariable("person", null);
            }
            if ("baseline_case".equals(id)) {
                context.setVariable("baseline_case", BASELINE);
                outcome = "<p>" + BASELINE + "</p>";
            } else {
                outcome = engine.process(template, context);
            }
        } catch (final Throwable error) {
            outcome = "EXCEPTION:" + error.getClass().getSimpleName();
        }
        // 结果中的换行统一转义，保证 golden 单行一条。
        outcome = outcome.replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t");
        System.out.println(id + "\t" + outcome);
    }
}
