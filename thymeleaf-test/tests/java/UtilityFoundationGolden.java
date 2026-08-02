import org.thymeleaf.util.IdentityCounter;
import org.thymeleaf.util.NumberPointType;

/**
 * 从固定 Thymeleaf Java 源码导出基础工具对象 Golden。
 */
public final class UtilityFoundationGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private UtilityFoundationGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        for (final NumberPointType value : NumberPointType.values()) {
            emit("point." + value.ordinal() + ".name", value.getName());
            emit("point." + value.ordinal() + ".display", value.toString());
            emit("point." + value.ordinal() + ".identity", NumberPointType.match(value.getName()) == value);
        }
        emitMatch("null", null);
        emitMatch("empty", "");
        emitMatch("lower", "point");
        emitMatch("leading_space", " POINT");
        emitMatch("unknown", "UNKNOWN");

        emitFailure("identity.negative", () -> new IdentityCounter<Object>(-1));
        emitFailure("identity.too_large", () -> new IdentityCounter<Object>(Integer.MAX_VALUE));
        emit("identity.zero", new IdentityCounter<Object>(0) != null);

        final IdentityCounter<Object> counter = new IdentityCounter<Object>(2);
        final String first = new String("same");
        final String firstAlias = first;
        final String equalButDistinct = new String("same");
        emit("identity.first.before", counter.isAlreadyCounted(first));
        counter.count(first);
        emit("identity.first.after", counter.isAlreadyCounted(first));
        emit("identity.alias", counter.isAlreadyCounted(firstAlias));
        emit("identity.equal_distinct.before", counter.isAlreadyCounted(equalButDistinct));
        counter.count(equalButDistinct);
        emit("identity.equal_distinct.after", counter.isAlreadyCounted(equalButDistinct));
        counter.count(first);

        emit("identity.null.before", counter.isAlreadyCounted(null));
        counter.count(null);
        counter.count(null);
        emit("identity.null.after", counter.isAlreadyCounted(null));
        emit("identity.unseen", counter.isAlreadyCounted(new String("same")));
    }

    private static void emitMatch(final String key, final String input) {
        final NumberPointType value = NumberPointType.match(input);
        emit("match." + key, value == null ? "null" : value.getName());
    }

    private static void emitFailure(final String key, final ThrowingRunnable action) {
        try {
            action.run();
            emit(key, "NO_ERROR");
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getName() + ":" + exception.getMessage());
        }
    }

    private static void emit(final String key, final boolean value) {
        emit(key, Boolean.toString(value));
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private interface ThrowingRunnable {
        void run();
    }
}
