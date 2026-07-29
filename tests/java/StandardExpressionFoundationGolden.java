import org.thymeleaf.standard.expression.LiteralValue;
import org.thymeleaf.standard.expression.StandardExpressionExecutionContext;

/**
 * 从固定 Thymeleaf Java 源码导出标准表达式基础值对象 Golden。
 */
public final class StandardExpressionFoundationGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private StandardExpressionFoundationGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        literalValueCases();
        executionContextCases();
    }

    private static void literalValueCases() {
        final LiteralValue literal = new LiteralValue("4");
        final LiteralValue sameText = new LiteralValue("4");
        final Object other = new Object();

        emit("literal.value", literal.getValue());
        emit("literal.nullValue", new LiteralValue(null).getValue());
        emit("literal.identityEquals", literal.equals(literal));
        emit("literal.distinctEquals", literal.equals(sameText));
        emit("unwrap.null", LiteralValue.unwrap(null));
        emit("unwrap.literal", LiteralValue.unwrap(literal));
        emit("unwrap.literalNull", LiteralValue.unwrap(new LiteralValue(null)));
        emit("unwrap.otherIdentity", LiteralValue.unwrap(other) == other);
    }

    private static void executionContextCases() {
        describe("normal", StandardExpressionExecutionContext.NORMAL);
        describe("restricted", StandardExpressionExecutionContext.RESTRICTED);
        describe(
                "forbid",
                StandardExpressionExecutionContext.RESTRICTED_FORBID_UNSAFE_EXP_RESULTS);

        conversionIdentity("normal", StandardExpressionExecutionContext.NORMAL);
        conversionIdentity("restricted", StandardExpressionExecutionContext.RESTRICTED);
        conversionIdentity(
                "forbid",
                StandardExpressionExecutionContext.RESTRICTED_FORBID_UNSAFE_EXP_RESULTS);
    }

    private static void describe(
            final String name,
            final StandardExpressionExecutionContext context) {
        emit(
                "context." + name + ".flags",
                Boolean.toString(context.getRestrictVariableAccess())
                        + ","
                        + context.getRestrictExternalAccess()
                        + ","
                        + context.getForbidUnsafeExpressionResults()
                        + ","
                        + context.getPerformTypeConversion());
    }

    private static void conversionIdentity(
            final String name,
            final StandardExpressionExecutionContext context) {
        final StandardExpressionExecutionContext converted = context.withTypeConversion();
        emit("context." + name + ".withoutSame", context.withoutTypeConversion() == context);
        emit("context." + name + ".converted", converted.getPerformTypeConversion());
        emit("context." + name + ".withSame", converted.withTypeConversion() == converted);
        emit("context." + name + ".roundTrip", converted.withoutTypeConversion() == context);
        emit(
                "context." + name + ".canonical",
                context.withTypeConversion() == converted);
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }
}
