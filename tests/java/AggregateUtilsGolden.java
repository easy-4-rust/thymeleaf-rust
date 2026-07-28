import java.math.BigDecimal;
import java.math.BigInteger;
import java.util.AbstractCollection;
import java.util.Arrays;
import java.util.Collections;
import java.util.Iterator;

import org.thymeleaf.expression.Aggregates;
import org.thymeleaf.util.AggregateUtils;

/**
 * 从固定 Thymeleaf Java 源码导出 AggregateUtils 与 Aggregates Golden。
 */
public final class AggregateUtilsGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private AggregateUtilsGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emitUtilityCases();
        emitFacadeCases();
    }

    private static void emitUtilityCases() {
        emitOutcome("util.sum.iterable.null", () -> AggregateUtils.sum((Iterable<Number>)null));
        emitOutcome("util.sum.iterable.empty",
                () -> AggregateUtils.sum(Collections.<Number>emptyList()));
        emitOutcome("util.sum.iterable.null_element",
                () -> AggregateUtils.sum(Arrays.<Number>asList(1, null)));
        emitOutcome("util.avg.iterable.null_element",
                () -> AggregateUtils.avg(Arrays.<Number>asList(1, null)));

        final CountingNumbers counting = new CountingNumbers(1, 2, 3);
        emitOutcome("util.sum.iterable.counting", () -> AggregateUtils.sum(counting));
        emit("util.sum.iterable.iterator_calls", Integer.toString(counting.iteratorCalls));

        emitOutcome("util.sum.objects.null", () -> AggregateUtils.sum((Object[])null));
        emitOutcome("util.sum.objects.empty", () -> AggregateUtils.sum(new Object[0]));
        emitOutcome("util.sum.objects.null_element",
                () -> AggregateUtils.sum(new Object[] {1, null}));
        emitOutcome("util.sum.objects.null_priority",
                () -> AggregateUtils.sum(new Object[] {"bad", null}));
        emitOutcome("util.sum.objects.class_cast",
                () -> AggregateUtils.sum(new Object[] {"bad"}));

        final Number custom = new Number() {
            private static final long serialVersionUID = 1L;
            @Override public int intValue() { return 0; }
            @Override public long longValue() { return 0L; }
            @Override public float floatValue() { return 0.05f; }
            @Override public double doubleValue() { return 0.05d; }
        };
        final Object[] mixed = new Object[] {
                new BigDecimal("1.20"), new BigInteger("2"), Byte.valueOf((byte)3),
                Short.valueOf((short)4), Integer.valueOf(5), Long.valueOf(6L),
                Float.valueOf(0.5f), Double.valueOf(0.25d), custom
        };
        emitOutcome("util.sum.objects.mixed", () -> AggregateUtils.sum(mixed));
        emitOutcome("util.avg.objects.mixed", () -> AggregateUtils.avg(mixed));

        emitPrimitiveCases("bytes", new byte[] {-128, 1, 127});
        emitPrimitiveCases("shorts", new short[] {-32768, 1, 32767});
        emitPrimitiveCases("ints", new int[] {Integer.MIN_VALUE, 1, Integer.MAX_VALUE});
        emitPrimitiveCases("longs", new long[] {Long.MIN_VALUE, 1L, Long.MAX_VALUE});
        emitPrimitiveCases("floats", new float[] {0.1f, -0.0f, 1.25f});
        emitPrimitiveCases("doubles", new double[] {0.1d, -0.0d, 1.25d});

        emitOutcome("util.sum.floats.min", () -> AggregateUtils.sum(new float[] {Float.MIN_VALUE}));
        emitOutcome("util.sum.floats.max", () -> AggregateUtils.sum(new float[] {Float.MAX_VALUE}));
        emitOutcome("util.sum.floats.nan", () -> AggregateUtils.sum(new float[] {Float.NaN}));
        emitOutcome("util.sum.floats.infinity",
                () -> AggregateUtils.sum(new float[] {Float.POSITIVE_INFINITY}));
        emitOutcome("util.sum.doubles.min",
                () -> AggregateUtils.sum(new double[] {Double.MIN_VALUE}));
        emitOutcome("util.sum.doubles.max",
                () -> AggregateUtils.sum(new double[] {Double.MAX_VALUE}));
        emitOutcome("util.sum.doubles.threshold_plain",
                () -> AggregateUtils.sum(new double[] {9999999.0d}));
        emitOutcome("util.sum.doubles.threshold_scientific",
                () -> AggregateUtils.sum(new double[] {10000000.0d}));
        emitOutcome("util.sum.doubles.small_plain",
                () -> AggregateUtils.sum(new double[] {0.001d}));
        emitOutcome("util.sum.doubles.small_scientific",
                () -> AggregateUtils.sum(new double[] {0.0001d}));
        emitOutcome("util.sum.doubles.negative_zero",
                () -> AggregateUtils.sum(new double[] {-0.0d}));
        emitOutcome("util.sum.doubles.nan", () -> AggregateUtils.sum(new double[] {Double.NaN}));
        emitOutcome("util.sum.doubles.infinity",
                () -> AggregateUtils.sum(new double[] {Double.NEGATIVE_INFINITY}));

        emitOutcome("util.avg.exact", () -> AggregateUtils.avg(new int[] {1, 2}));
        emitOutcome("util.avg.repeating", () -> AggregateUtils.avg(new int[] {1, 1, 2}));
        emitOutcome("util.avg.repeating_negative",
                () -> AggregateUtils.avg(new int[] {-1, -1, -2}));
        emitOutcome("util.avg.scale_12",
                () -> AggregateUtils.avg(new Object[] {
                        new BigDecimal("1.000000000000"), 2, 2
                }));
        emitOutcome("util.sum.long_no_overflow",
                () -> AggregateUtils.sum(new long[] {Long.MAX_VALUE, Long.MAX_VALUE}));
        emitDoubleMatrix();
    }

    private static void emitPrimitiveCases(final String key, final byte[] values) {
        emitOutcome("util.sum." + key, () -> AggregateUtils.sum(values));
        emitOutcome("util.avg." + key, () -> AggregateUtils.avg(values));
        emitOutcome("util.sum." + key + ".empty", () -> AggregateUtils.sum(new byte[0]));
    }

    private static void emitPrimitiveCases(final String key, final short[] values) {
        emitOutcome("util.sum." + key, () -> AggregateUtils.sum(values));
        emitOutcome("util.avg." + key, () -> AggregateUtils.avg(values));
        emitOutcome("util.sum." + key + ".empty", () -> AggregateUtils.sum(new short[0]));
    }

    private static void emitPrimitiveCases(final String key, final int[] values) {
        emitOutcome("util.sum." + key, () -> AggregateUtils.sum(values));
        emitOutcome("util.avg." + key, () -> AggregateUtils.avg(values));
        emitOutcome("util.sum." + key + ".empty", () -> AggregateUtils.sum(new int[0]));
    }

    private static void emitPrimitiveCases(final String key, final long[] values) {
        emitOutcome("util.sum." + key, () -> AggregateUtils.sum(values));
        emitOutcome("util.avg." + key, () -> AggregateUtils.avg(values));
        emitOutcome("util.sum." + key + ".empty", () -> AggregateUtils.sum(new long[0]));
    }

    private static void emitPrimitiveCases(final String key, final float[] values) {
        emitOutcome("util.sum." + key, () -> AggregateUtils.sum(values));
        emitOutcome("util.avg." + key, () -> AggregateUtils.avg(values));
        emitOutcome("util.sum." + key + ".empty", () -> AggregateUtils.sum(new float[0]));
    }

    private static void emitPrimitiveCases(final String key, final double[] values) {
        emitOutcome("util.sum." + key, () -> AggregateUtils.sum(values));
        emitOutcome("util.avg." + key, () -> AggregateUtils.avg(values));
        emitOutcome("util.sum." + key + ".empty", () -> AggregateUtils.sum(new double[0]));
    }

    private static void emitFacadeCases() {
        final Aggregates aggregates = new Aggregates();
        final Number[] numbers = new Number[] {1, 2};
        emitOutcome("facade.sum.iterable", () -> aggregates.sum(Arrays.<Number>asList(numbers)));
        emitOutcome("facade.sum.numbers", () -> aggregates.sum(numbers));
        emitOutcome("facade.sum.bytes", () -> aggregates.sum(new byte[] {1, 2}));
        emitOutcome("facade.sum.shorts", () -> aggregates.sum(new short[] {1, 2}));
        emitOutcome("facade.sum.ints", () -> aggregates.sum(new int[] {1, 2}));
        emitOutcome("facade.sum.longs", () -> aggregates.sum(new long[] {1L, 2L}));
        emitOutcome("facade.sum.floats", () -> aggregates.sum(new float[] {0.5f, 0.25f}));
        emitOutcome("facade.sum.doubles", () -> aggregates.sum(new double[] {0.5d, 0.25d}));
        emitOutcome("facade.avg.iterable", () -> aggregates.avg(Arrays.<Number>asList(numbers)));
        emitOutcome("facade.avg.numbers", () -> aggregates.avg(numbers));
        emitOutcome("facade.avg.bytes", () -> aggregates.avg(new byte[] {1, 2}));
        emitOutcome("facade.avg.shorts", () -> aggregates.avg(new short[] {1, 2}));
        emitOutcome("facade.avg.ints", () -> aggregates.avg(new int[] {1, 2}));
        emitOutcome("facade.avg.longs", () -> aggregates.avg(new long[] {1L, 2L}));
        emitOutcome("facade.avg.floats", () -> aggregates.avg(new float[] {0.5f, 0.25f}));
        emitOutcome("facade.avg.doubles", () -> aggregates.avg(new double[] {0.5d, 0.25d}));
    }

    private static String describe(final Object value) {
        if (value == null) {
            return "null";
        }
        final BigDecimal decimal = (BigDecimal)value;
        return decimal.toString() + "|scale=" + decimal.scale()
                + "|unscaled=" + decimal.unscaledValue()
                + "|plain=" + decimal.toPlainString();
    }

    private static void emitOutcome(final String key, final ThrowingSupplier action) {
        emit(key, describeOutcome(action));
    }

    private static String describeOutcome(final ThrowingSupplier action) {
        try {
            return "OK:" + describe(action.get());
        } catch (final RuntimeException exception) {
            final String message;
            if (exception instanceof IllegalArgumentException
                    && !(exception instanceof NumberFormatException)
                    && !(exception instanceof ClassCastException)) {
                message = ":" + String.valueOf(exception.getMessage());
            } else {
                message = "";
            }
            return "ERR:" + exception.getClass().getName() + message;
        }
    }

    private static void emitDoubleMatrix() {
        final long[] edges = new long[] {
                0L, Long.MIN_VALUE, 1L, Long.MIN_VALUE | 1L,
                Double.doubleToRawLongBits(Double.MIN_NORMAL),
                Double.doubleToRawLongBits(Double.MAX_VALUE),
                Double.doubleToRawLongBits(0.001d),
                Double.doubleToRawLongBits(10000000.0d),
                Double.doubleToRawLongBits(Double.NaN),
                Double.doubleToRawLongBits(Double.POSITIVE_INFINITY)
        };
        long hash = 0xcbf29ce484222325L;
        int count = 0;
        for (final long bits : edges) {
            hash = hashDoubleOutcome(hash, bits);
            count++;
        }
        long bits = 0x6a09e667f3bcc909L;
        for (int i = 0; i < 20000; i++) {
            bits = bits * 6364136223846793005L + 1442695040888963407L;
            hash = hashDoubleOutcome(hash, bits);
            count++;
        }
        emit("util.double_matrix.count", Integer.toString(count));
        emit("util.double_matrix.fnv64", Long.toUnsignedString(hash, 16));
    }

    private static long hashDoubleOutcome(long hash, final long bits) {
        final double value = Double.longBitsToDouble(bits);
        final String text = Long.toUnsignedString(bits, 16) + ":"
                + describeOutcome(() -> AggregateUtils.sum(new double[] {value}));
        for (int i = 0; i < text.length(); i++) {
            hash ^= text.charAt(i);
            hash *= 0x100000001b3L;
        }
        return hash;
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private interface ThrowingSupplier {
        Object get();
    }

    private static final class CountingNumbers extends AbstractCollection<Number> {
        private final Number[] values;
        private int iteratorCalls;

        private CountingNumbers(final Number... values) {
            this.values = values;
        }

        @Override
        public Iterator<Number> iterator() {
            iteratorCalls++;
            return Arrays.asList(values).iterator();
        }

        @Override
        public int size() {
            return values.length;
        }
    }
}
