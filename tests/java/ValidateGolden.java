import java.util.Arrays;
import java.util.Collection;
import java.util.Collections;
import java.util.List;

import org.thymeleaf.util.Validate;

/**
 * 从固定 Thymeleaf Java 源码导出 Validate Golden。
 */
public final class ValidateGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private ValidateGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        emitOutcome("not_null.value", () -> Validate.notNull("value", "failure"));
        emitOutcome("not_null.null", () -> Validate.notNull(null, "failure"));
        emitOutcome("not_null.null_message", () -> Validate.notNull(null, null));

        emitOutcome("not_empty_str.value", () -> Validate.notEmpty("value", "empty"));
        emitOutcome("not_empty_str.null", () -> Validate.notEmpty((String)null, "empty"));
        emitOutcome("not_empty_str.empty", () -> Validate.notEmpty("", "empty"));
        emitOutcome("not_empty_str.space", () -> Validate.notEmpty(" ", "empty"));
        emitOutcome("not_empty_str.control", () -> Validate.notEmpty("\u001c", "empty"));
        emitOutcome("not_empty_str.ogham", () -> Validate.notEmpty("\u1680", "empty"));
        emitOutcome("not_empty_str.punctuation_space", () -> Validate.notEmpty("\u2008", "empty"));
        emitOutcome("not_empty_str.line_separator", () -> Validate.notEmpty("\u2028", "empty"));
        emitOutcome("not_empty_str.medium_space", () -> Validate.notEmpty("\u205f", "empty"));
        emitOutcome("not_empty_str.ideographic_space", () -> Validate.notEmpty("\u3000", "empty"));
        emitOutcome("not_empty_str.nbsp", () -> Validate.notEmpty("\u00a0", "empty"));

        final Collection<String> emptyCollection = Collections.emptyList();
        final Collection<String> valueCollection = Collections.singletonList("value");
        emitOutcome("not_empty_collection.null",
                () -> Validate.notEmpty((Collection<?>)null, "empty"));
        emitOutcome("not_empty_collection.empty",
                () -> Validate.notEmpty(emptyCollection, "empty"));
        emitOutcome("not_empty_collection.value",
                () -> Validate.notEmpty(valueCollection, "empty"));

        final Object[] emptyArray = new Object[0];
        final Object[] valueArray = new Object[] {"value"};
        emitOutcome("not_empty_array.null", () -> Validate.notEmpty((Object[])null, "empty"));
        emitOutcome("not_empty_array.empty", () -> Validate.notEmpty(emptyArray, "empty"));
        emitOutcome("not_empty_array.value", () -> Validate.notEmpty(valueArray, "empty"));

        final List<Object> noNulls = Arrays.<Object>asList("one", Integer.valueOf(2));
        final List<Object> withNull = Arrays.<Object>asList("one", null, "three");
        emitOutcome("no_nulls_iterable.value",
                () -> Validate.containsNoNulls((Iterable<?>)noNulls, "null"));
        emitOutcome("no_nulls_iterable.element",
                () -> Validate.containsNoNulls((Iterable<?>)withNull, "null"));
        emitOutcome("no_nulls_iterable.null_message",
                () -> Validate.containsNoNulls((Iterable<?>)withNull, null));
        emitImplicit("no_nulls_iterable.null_container",
                () -> Validate.containsNoNulls((Iterable<?>)null, "ignored"));

        final List<String> noEmpties = Arrays.asList("value", "\u00a0");
        final List<String> withEmpty = Arrays.asList("value", "");
        final List<String> withWhitespace = Arrays.asList("value", "\u2008");
        final List<String> withNullString = Arrays.asList("value", null);
        emitOutcome("no_empties.value", () -> Validate.containsNoEmpties(noEmpties, "empty"));
        emitOutcome("no_empties.empty", () -> Validate.containsNoEmpties(withEmpty, "empty"));
        emitOutcome("no_empties.whitespace",
                () -> Validate.containsNoEmpties(withWhitespace, "empty"));
        emitOutcome("no_empties.null_element",
                () -> Validate.containsNoEmpties(withNullString, "empty"));
        emitImplicit("no_empties.null_container",
                () -> Validate.containsNoEmpties(null, "ignored"));

        final Object[] noNullArray = new Object[] {"one", Integer.valueOf(2)};
        final Object[] withNullArray = new Object[] {"one", null};
        emitOutcome("no_nulls_array.value",
                () -> Validate.containsNoNulls(noNullArray, "null"));
        emitOutcome("no_nulls_array.element",
                () -> Validate.containsNoNulls(withNullArray, "null"));
        emitImplicit("no_nulls_array.null_container",
                () -> Validate.containsNoNulls((Object[])null, "ignored"));

        emitOutcome("is_true.true", () -> Validate.isTrue(true, "failure"));
        emitOutcome("is_true.false", () -> Validate.isTrue(false, "failure"));
        emitOutcome("is_true.null_message", () -> Validate.isTrue(false, null));
    }

    private static void emitOutcome(final String key, final ThrowingRunnable action) {
        try {
            action.run();
            emit(key, "OK");
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getName() + ":" + String.valueOf(exception.getMessage()));
        }
    }

    private static void emitImplicit(final String key, final ThrowingRunnable action) {
        try {
            action.run();
            emit(key, "OK");
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getName());
        }
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private interface ThrowingRunnable {
        void run();
    }
}
