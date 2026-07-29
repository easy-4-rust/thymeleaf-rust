import java.math.BigDecimal;
import java.math.BigInteger;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

import org.thymeleaf.expression.Bools;
import org.thymeleaf.standard.expression.LiteralValue;
import org.thymeleaf.util.EvaluationUtils;

/**
 * 从固定 Thymeleaf Java 源码导出 EvaluationUtils 与 Bools Golden。
 */
public final class EvaluationUtilsGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private EvaluationUtilsGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emitBooleanCases();
        emitNumberCases();
        emitCollectionCases();
        emitBoolsCases();
    }

    private static void emitBooleanCases() {
        emitBoolean("bool.null", null);
        emitBoolean("bool.false", Boolean.FALSE);
        emitBoolean("bool.true", Boolean.TRUE);
        emitBoolean("bool.big_decimal.zero_scale", new BigDecimal("0.000"));
        emitBoolean("bool.big_decimal.nonzero", new BigDecimal("-0.01"));
        emitBoolean("bool.big_integer.zero", BigInteger.ZERO);
        emitBoolean("bool.integer.zero", Integer.valueOf(0));
        emitBoolean("bool.double.negative_zero", Double.valueOf(-0.0d));
        emitBoolean("bool.double.nan", Double.valueOf(Double.NaN));
        emitBoolean("bool.character.zero", Character.valueOf('\0'));
        emitBoolean("bool.character.value", Character.valueOf('x'));
        emitBoolean("bool.string.false", " \tFALSE\r\n");
        emitBoolean("bool.string.off", "OfF");
        emitBoolean("bool.string.no", "NO");
        emitBoolean("bool.string.empty", "");
        emitBoolean("bool.string.nbsp", "\u00a0false\u00a0");
        emitBoolean("bool.literal.false", new LiteralValue(" false "));
        emitOutcome("bool.literal.null", () -> EvaluationUtils.evaluateAsBoolean(new LiteralValue(null)));
        emitBoolean("bool.empty_list", new ArrayList<Object>());
        emitBoolean("bool.empty_array", new Object[0]);
        emitBoolean("bool.other", EvaluationUtilsGolden.class);
    }

    private static void emitNumberCases() {
        final BigDecimal decimal = new BigDecimal("1.20");
        emitNumber("number.null", null, null);
        emitNumber("number.decimal", decimal, decimal);
        emitNumber("number.big_integer", new BigInteger("123"), null);
        emitNumber("number.byte", Byte.valueOf((byte)-2), null);
        emitNumber("number.short", Short.valueOf((short)-3), null);
        emitNumber("number.integer", Integer.valueOf(-4), null);
        emitNumber("number.long", Long.valueOf(Long.MIN_VALUE), null);
        emitNumber("number.float", Float.valueOf(0.1f), null);
        emitNumber("number.double", Double.valueOf(0.1d), null);
        emitNumber("number.negative_zero", Double.valueOf(-0.0d), null);
        emitOutcome("number.nan", () -> EvaluationUtils.evaluateAsNumber(Double.valueOf(Double.NaN)));
        emitOutcome("number.infinity",
                () -> EvaluationUtils.evaluateAsNumber(Double.valueOf(Double.POSITIVE_INFINITY)));
        final Number custom = new Number() {
            private static final long serialVersionUID = 1L;
            @Override public int intValue() { return 7; }
            @Override public long longValue() { return 7L; }
            @Override public float floatValue() { return 7.0f; }
            @Override public double doubleValue() { return 7.0d; }
        };
        emitNumber("number.custom", custom, null);
        emitNumber("number.string.integer", "123", null);
        emitNumber("number.string.scale", "-1.20E+3", null);
        emitNumber("number.string.unicode_digits", "1\u0662.\u0663", null);
        emitNumber("number.string.leading_space", " 123 ", null);
        emitNumber("number.string.trailing_space", "123 ", null);
        emitNumber("number.string.dot_prefix", ".5", null);
        emitNumber("number.string.invalid", "+ 1", null);
        emitNumber("number.literal", new LiteralValue("12"), null);
        emitNumber("number.other", EvaluationUtilsGolden.class, null);
        emitNumberMatrix();
    }

    private static void emitCollectionCases() {
        final List<Object> nullList = EvaluationUtils.evaluateAsList(null);
        emit("list.null", describeList(nullList));
        emit("list.empty_iterable", describeList(EvaluationUtils.evaluateAsList(new ArrayList<Object>())));
        emit("list.iterable", describeList(EvaluationUtils.evaluateAsList(Arrays.asList("a", null, "b"))));

        final Map<String,String> map = new LinkedHashMap<String,String>();
        map.put("a", "1");
        map.put("b", null);
        final Map.Entry<String,String> rawEntry = map.entrySet().iterator().next();
        final List<Object> mapList = EvaluationUtils.evaluateAsList(map);
        emit("list.map", describeList(mapList));
        emit("list.map.fresh_entry", Boolean.toString(mapList.get(0) != rawEntry));
        emit("list.map.entry_hash", Integer.toString(mapList.get(0).hashCode()));

        emit("list.bytes", describeList(EvaluationUtils.evaluateAsList(new byte[] {-1, 2})));
        emit("list.shorts", describeList(EvaluationUtils.evaluateAsList(new short[] {-2, 3})));
        emit("list.ints", describeList(EvaluationUtils.evaluateAsList(new int[] {-3, 4})));
        emit("list.longs", describeList(EvaluationUtils.evaluateAsList(new long[] {-4L, 5L})));
        emit("list.floats", describeList(EvaluationUtils.evaluateAsList(new float[] {-0.0f, 0.5f})));
        emit("list.doubles", describeList(EvaluationUtils.evaluateAsList(new double[] {-0.0d, 0.5d})));
        emit("list.booleans", describeList(EvaluationUtils.evaluateAsList(new boolean[] {false, true})));
        emit("list.characters", describeList(EvaluationUtils.evaluateAsList(new char[] {'\0', 'x'})));
        final String[] reference = new String[] {"a", null};
        emit("list.reference", describeList(EvaluationUtils.evaluateAsList(reference)));
        emit("list.scalar", describeList(EvaluationUtils.evaluateAsList("a")));

        emit("array.null", describeArray(EvaluationUtils.evaluateAsArray(null), null));
        emit("array.iterable",
                describeArray(EvaluationUtils.evaluateAsArray(Arrays.asList("a", null)), null));
        final Object[] mapArray = EvaluationUtils.evaluateAsArray(map);
        emit("array.map", describeArray(mapArray, null));
        emit("array.map.raw_entry", Boolean.toString(mapArray[0] == rawEntry));
        emit("array.reference", describeArray(EvaluationUtils.evaluateAsArray(reference), reference));
        emit("array.scalar", describeArray(EvaluationUtils.evaluateAsArray("a"), null));
        emitOutcome("array.primitive.bytes", () -> EvaluationUtils.evaluateAsArray(new byte[] {1}));
        emitOutcome("array.primitive.ints", () -> EvaluationUtils.evaluateAsArray(new int[] {1}));
        emitOutcome("array.primitive.booleans",
                () -> EvaluationUtils.evaluateAsArray(new boolean[] {true}));
    }

    private static void emitBoolsCases() {
        final Bools bools = new Bools();
        final Object[] values = new Object[] {null, "false", 1, "no"};
        final List<Object> list = Arrays.asList(values);
        final Set<Object> set = new LinkedHashSet<Object>(list);
        emitOutcome("bools.is_true", () -> bools.isTrue("yes"));
        emitOutcome("bools.is_false", () -> bools.isFalse("off"));
        emitOutcome("bools.array_is_true", () -> Arrays.toString(bools.arrayIsTrue(values)));
        emitOutcome("bools.list_is_true", () -> bools.listIsTrue(list).toString());
        emitOutcome("bools.set_is_true", () -> bools.setIsTrue(set).toString());
        emitOutcome("bools.array_is_false", () -> Arrays.toString(bools.arrayIsFalse(values)));
        emitOutcome("bools.list_is_false", () -> bools.listIsFalse(list).toString());
        emitOutcome("bools.set_is_false", () -> bools.setIsFalse(set).toString());
        emitOutcome("bools.array_and", () -> bools.arrayAnd(values));
        emitOutcome("bools.list_and", () -> bools.listAnd(list));
        emitOutcome("bools.set_and", () -> bools.setAnd(set));
        emitOutcome("bools.array_or", () -> bools.arrayOr(values));
        emitOutcome("bools.list_or", () -> bools.listOr(list));
        emitOutcome("bools.set_or", () -> bools.setOr(set));
        emitOutcome("bools.empty_and", () -> bools.arrayAnd(new Object[0]));
        emitOutcome("bools.empty_or", () -> bools.arrayOr(new Object[0]));
        emitOutcome("bools.null_array", () -> bools.arrayAnd(null));
        emitOutcome("bools.short_circuit_and",
                () -> bools.arrayAnd(new Object[] {false, new LiteralValue(null)}));
        emitOutcome("bools.short_circuit_or",
                () -> bools.arrayOr(new Object[] {true, new LiteralValue(null)}));
    }

    private static void emitBoolean(final String key, final Object value) {
        emitOutcome(key, () -> EvaluationUtils.evaluateAsBoolean(value));
    }

    private static void emitNumber(final String key, final Object value, final BigDecimal identity) {
        emitOutcome(key, () -> {
            final BigDecimal result = EvaluationUtils.evaluateAsNumber(value);
            if (result == null) {
                return "null";
            }
            return result.toString() + "|scale=" + result.scale()
                    + "|unscaled=" + result.unscaledValue()
                    + "|same=" + (result == identity);
        });
    }

    private static void emitNumberMatrix() {
        final long[] edges = new long[] {
                0L, Long.MIN_VALUE, 1L, Long.MIN_VALUE | 1L,
                Double.doubleToRawLongBits(Double.MIN_NORMAL),
                Double.doubleToRawLongBits(Double.MAX_VALUE),
                Double.doubleToRawLongBits(0.1d),
                Double.doubleToRawLongBits(-0.1d),
                Double.doubleToRawLongBits(Double.NaN),
                Double.doubleToRawLongBits(Double.POSITIVE_INFINITY)
        };
        long hash = 0xcbf29ce484222325L;
        int count = 0;
        for (final long bits : edges) {
            hash = hashNumberOutcome(hash, bits);
            count++;
        }
        long bits = 0x6a09e667f3bcc909L;
        for (int i = 0; i < 20000; i++) {
            bits = bits * 6364136223846793005L + 1442695040888963407L;
            hash = hashNumberOutcome(hash, bits);
            count++;
        }
        emit("number.double_matrix.count", Integer.toString(count));
        emit("number.double_matrix.fnv64", Long.toUnsignedString(hash, 16));
    }

    private static long hashNumberOutcome(long hash, final long bits) {
        final double value = Double.longBitsToDouble(bits);
        final String text = Long.toUnsignedString(bits, 16) + ":"
                + describeOutcome(() -> {
                    final BigDecimal result = EvaluationUtils.evaluateAsNumber(Double.valueOf(value));
                    if (result == null) {
                        return "null";
                    }
                    return result.toString() + "|scale=" + result.scale()
                            + "|unscaled=" + result.unscaledValue() + "|same=false";
                });
        for (int i = 0; i < text.length(); i++) {
            hash ^= text.charAt(i);
            hash *= 0x100000001b3L;
        }
        return hash;
    }

    private static String describeList(final List<Object> values) {
        final StringBuilder result = new StringBuilder(values.getClass().getName());
        result.append('|').append(values.size()).append('|');
        for (int i = 0; i < values.size(); i++) {
            if (i > 0) {
                result.append(',');
            }
            result.append(describeValue(values.get(i)));
        }
        try {
            values.add("x");
            result.append("|mutable");
        } catch (final RuntimeException exception) {
            result.append("|").append(exception.getClass().getName());
        }
        return result.toString();
    }

    private static String describeArray(final Object[] values, final Object[] identity) {
        final StringBuilder result = new StringBuilder(values.getClass().getName());
        result.append('|').append(values.length).append("|same=").append(values == identity).append('|');
        for (int i = 0; i < values.length; i++) {
            if (i > 0) {
                result.append(',');
            }
            result.append(describeValue(values[i]));
        }
        return result.toString();
    }

    private static String describeValue(final Object value) {
        if (value == null) {
            return "null";
        }
        if (value instanceof Character) {
            return value.getClass().getName() + ":"
                    + Integer.toHexString(((Character)value).charValue());
        }
        return value.getClass().getName() + ":" + value;
    }

    private static void emitOutcome(final String key, final ThrowingSupplier action) {
        emit(key, describeOutcome(action));
    }

    private static String describeOutcome(final ThrowingSupplier action) {
        try {
            return "OK:" + String.valueOf(action.get());
        } catch (final RuntimeException exception) {
            final String message = exception instanceof IllegalArgumentException
                    && !(exception instanceof NumberFormatException)
                    ? ":" + String.valueOf(exception.getMessage()) : "";
            return "ERR:" + exception.getClass().getName() + message;
        }
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private interface ThrowingSupplier {
        Object get();
    }
}
