import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;

import org.thymeleaf.util.ArrayUtils;

/**
 * 从固定 Thymeleaf Java 源码导出 ArrayUtils 与 Arrays 的行为 Golden。
 */
public final class ArrayUtilsGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private ArrayUtilsGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        final String[] strings = new String[] {"one", null, "two"};
        emit("to_array.reference.identity",
                Boolean.toString(ArrayUtils.toArray(strings) == strings));
        emit("to_array.reference.class", ArrayUtils.toArray(strings).getClass().getName());
        emitOutcome("to_array.primitive", () -> ArrayUtils.toArray(new int[] {1, 2}));
        emitOutcome("to_array.null", () -> ArrayUtils.toArray(null));
        emitOutcome("to_array.other", () -> ArrayUtils.toArray(Integer.valueOf(1)));

        final ArrayList<Object> homogeneous =
                new ArrayList<Object>(Arrays.<Object>asList("one", null, "two"));
        final ArrayList<Object> mixed =
                new ArrayList<Object>(Arrays.<Object>asList("one", Integer.valueOf(2)));
        final ArrayList<Object> allNull =
                new ArrayList<Object>(Arrays.<Object>asList(null, null));
        emitArray("to_array.iterable.homogeneous", ArrayUtils.toArray(homogeneous));
        emitArray("to_array.iterable.mixed", ArrayUtils.toArray(mixed));
        emitArray("to_array.iterable.all_null", ArrayUtils.toArray(allNull));
        emitArray("to_array.iterable.empty", ArrayUtils.toArray(new ArrayList<Object>()));

        emit("to_string.reference.identity",
                Boolean.toString(ArrayUtils.toStringArray(strings) == strings));
        emitOutcome("to_string.reference.incompatible",
                () -> ArrayUtils.toStringArray(new Integer[] {1}));
        emitOutcome("to_string.primitive",
                () -> ArrayUtils.toStringArray(new int[] {1}));
        emitArray("to_string.iterable",
                ArrayUtils.toStringArray(Arrays.<Object>asList("one", null)));
        emitOutcome("to_string.iterable.incompatible",
                () -> ArrayUtils.toStringArray(Arrays.<Object>asList("one", Integer.valueOf(2))));
        emitOutcome("to_string.other", () -> ArrayUtils.toStringArray(Integer.valueOf(1)));

        emitArray("typed.integer", ArrayUtils.toIntegerArray(Arrays.asList(1, null, 2)));
        emitArray("typed.long", ArrayUtils.toLongArray(Arrays.asList(1L, null, 2L)));
        emitArray("typed.double", ArrayUtils.toDoubleArray(Arrays.asList(1.0, null, 2.0)));
        emitArray("typed.float", ArrayUtils.toFloatArray(Arrays.asList(1.0f, null, 2.0f)));
        emitArray("typed.boolean", ArrayUtils.toBooleanArray(Arrays.asList(true, null, false)));

        emitOutcome("length.value", () -> ArrayUtils.length(strings));
        emitOutcome("length.null", () -> ArrayUtils.length(null));
        emit("empty.null", Boolean.toString(ArrayUtils.isEmpty(null)));
        emit("empty.zero", Boolean.toString(ArrayUtils.isEmpty(new Object[0])));
        emit("empty.value", Boolean.toString(ArrayUtils.isEmpty(strings)));
        emitOutcome("contains.null", () -> ArrayUtils.contains(strings, null));
        emitOutcome("contains.value", () -> ArrayUtils.contains(strings, "two"));
        emitOutcome("contains.missing", () -> ArrayUtils.contains(strings, "missing"));
        emitOutcome("contains.target_null", () -> ArrayUtils.contains(null, "one"));

        emitOutcome("contains_all.array",
                () -> ArrayUtils.containsAll(strings, new Object[] {"one", null, "one"}));
        emitOutcome("contains_all.array.missing",
                () -> ArrayUtils.containsAll(strings, new Object[] {"missing"}));
        emitOutcome("contains_all.array.target_null",
                () -> ArrayUtils.containsAll(null, new Object[] {"one"}));
        emitOutcome("contains_all.array.elements_null",
                () -> ArrayUtils.containsAll(strings, (Object[])null));
        final Collection<Object> requested =
                Arrays.<Object>asList("one", null, "one");
        emitOutcome("contains_all.collection",
                () -> ArrayUtils.containsAll(strings, requested));
        emitOutcome("contains_all.collection.target_null",
                () -> ArrayUtils.containsAll(null, requested));
        emitOutcome("contains_all.collection.elements_null",
                () -> ArrayUtils.containsAll(strings, (Collection<?>)null));

        final String[] copied = ArrayUtils.copyOf(strings, 5);
        emitArray("copy.reference.extend", copied);
        emit("copy.reference.class", copied.getClass().getName());
        emit("copy.reference.distinct", Boolean.toString(copied != strings));
        emitArray("copy.reference.truncate", ArrayUtils.copyOf(strings, 1));
        emitArray("copy.reference.object_type",
                ArrayUtils.copyOf(strings, 4, Object[].class));
        emitOutcome("copy.reference.negative", () -> ArrayUtils.copyOf(strings, -1));
        emitOutcome("copy.reference.null", () -> ArrayUtils.copyOf((String[])null, 1));
        emitOutcome("copy.reference.null_negative", () -> ArrayUtils.copyOf((String[])null, -1));
        emitOutcome("copy.reference.type_null",
                () -> ArrayUtils.copyOf(strings, 1, null));
        emitOutcome("copy.reference.store",
                () -> ArrayUtils.copyOf(new Object[] {"one"}, 1, Integer[].class));

        emitChars("copy.char.extend", ArrayUtils.copyOf(new char[] {'a', '\0', 'z'}, 5));
        emitChars("copy.char.truncate", ArrayUtils.copyOf(new char[] {'a', 'b'}, 1));
        emitOutcome("copy.char.negative", () -> ArrayUtils.copyOf(new char[] {'a'}, -1));
        emitOutcome("copy.char.null", () -> ArrayUtils.copyOf((char[])null, 1));
        emitOutcome("copy.char.null_negative", () -> ArrayUtils.copyOf((char[])null, -1));

        final char[] rangeSource = new char[] {'a', 'b', 'c', 'd'};
        emitChars("range.middle", ArrayUtils.copyOfRange(rangeSource, 1, 3));
        emitChars("range.extend", ArrayUtils.copyOfRange(rangeSource, 2, 6));
        emitChars("range.empty_end", ArrayUtils.copyOfRange(rangeSource, 4, 4));
        emitOutcome("range.reverse", () -> ArrayUtils.copyOfRange(rangeSource, 3, 1));
        emitOutcome("range.negative_from", () -> ArrayUtils.copyOfRange(rangeSource, -1, 2));
        emitOutcome("range.from_beyond", () -> ArrayUtils.copyOfRange(rangeSource, 5, 6));
        emitOutcome("range.null", () -> ArrayUtils.copyOfRange(null, 0, 1));
        emitOutcome("range.overflow",
                () -> ArrayUtils.copyOfRange(rangeSource, Integer.MIN_VALUE, Integer.MAX_VALUE));

        final org.thymeleaf.expression.Arrays arrays =
                new org.thymeleaf.expression.Arrays();
        emitArray("facade.to_array", arrays.toArray(homogeneous));
        emitArray("facade.to_string", arrays.toStringArray(Arrays.asList("one", null)));
        emitArray("facade.to_integer", arrays.toIntegerArray(Arrays.asList(1, null)));
        emitArray("facade.to_long", arrays.toLongArray(Arrays.asList(1L, null)));
        emitArray("facade.to_double", arrays.toDoubleArray(Arrays.asList(1.0, null)));
        emitArray("facade.to_float", arrays.toFloatArray(Arrays.asList(1.0f, null)));
        emitArray("facade.to_boolean", arrays.toBooleanArray(Arrays.asList(true, null)));
        emitOutcome("facade.length", () -> arrays.length(strings));
        emit("facade.empty", Boolean.toString(arrays.isEmpty(strings)));
        emitOutcome("facade.contains", () -> arrays.contains(strings, "one"));
        emitOutcome("facade.contains_all.array",
                () -> arrays.containsAll(strings, new Object[] {"one", null}));
        emitOutcome("facade.contains_all.collection",
                () -> arrays.containsAll(strings, requested));
    }

    private static void emitArray(final String key, final Object[] value) {
        emit(key, value.getClass().getName() + ":" + java.util.Arrays.toString(value));
    }

    private static void emitChars(final String key, final char[] value) {
        final StringBuilder result = new StringBuilder();
        for (int i = 0; i < value.length; i++) {
            if (i > 0) {
                result.append(',');
            }
            result.append((int)value[i]);
        }
        emit(key, result.toString());
    }

    private static void emitOutcome(final String key, final ThrowingSupplier action) {
        try {
            emit(key, String.valueOf(action.get()));
        } catch (final RuntimeException exception) {
            if (exception instanceof ClassCastException ||
                    exception instanceof ArrayStoreException ||
                    exception instanceof NullPointerException ||
                    exception instanceof ArrayIndexOutOfBoundsException) {
                emit(key, exception.getClass().getName());
                return;
            }
            emit(key, exception.getClass().getName() + ":" +
                    String.valueOf(exception.getMessage()));
        }
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private interface ThrowingSupplier {
        Object get();
    }
}
