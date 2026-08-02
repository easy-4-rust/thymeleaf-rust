import java.io.Writer;

import org.thymeleaf.util.FastStringWriter;

/**
 * 从固定 Thymeleaf Java 源码导出 FastStringWriter Golden。
 */
public final class FastStringWriterGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final long FNV_OFFSET = 0xcbf29ce484222325L;
    private static final long FNV_PRIME = 0x100000001b3L;

    private FastStringWriterGolden() {
    }

    public static void main(final String[] args) throws Exception {
        emit("baseline", BASELINE);
        constructorCases();
        writeIntCases();
        writeStringCases();
        writeStringRangeCases();
        writeCharArrayCases();
        lifecycleAndInheritedWriterCases();
        exhaustiveCases();
    }

    private static void constructorCases() {
        emit("constructor.default", new FastStringWriter().toString());
        emit("constructor.zero", new FastStringWriter(0).toString());
        emit("constructor.one", new FastStringWriter(1).toString());
        emitOutcome("constructor.negative", () -> new FastStringWriter(-1));
        emitOutcome(
                "constructor.minimum",
                () -> new FastStringWriter(Integer.MIN_VALUE));
    }

    private static void writeIntCases() {
        final FastStringWriter writer = new FastStringWriter();
        writer.write('A');
        writer.write(-1);
        writer.write(0x10000);
        writer.write(0x1F600);
        emit("writeInt.utf16", toUtf16Hex(writer.toString()));
    }

    private static void writeStringCases() {
        final FastStringWriter writer = new FastStringWriter();
        writer.write("ab");
        writer.write((String) null);
        writer.write("\uD800");
        emit("writeString.utf16", toUtf16Hex(writer.toString()));

        final FastStringWriter empty = new FastStringWriter();
        emit("toString.emptyIdentity", empty.toString() == empty.toString());
        writer.write("x");
        emit("toString.nonEmptyIdentity", writer.toString() == writer.toString());
        final String snapshot = writer.toString();
        writer.write("tail");
        emit("toString.snapshot", toUtf16Hex(snapshot));
        emit("toString.current", toUtf16Hex(writer.toString()));
    }

    private static void writeStringRangeCases() {
        emitStringRange("normal", "abcdef", 1, 3);
        emitStringRange("emptyAtEnd", "abcdef", 6, 0);
        emitStringRange("nullFull", null, 0, 4);
        emitStringRange("nullMiddle", null, 1, 2);
        emitStringRange("negativeOffset", "abc", -1, 1);
        emitStringRange("negativeLength", "abc", 0, -1);
        emitStringRange("offsetAfterEnd", "abc", 4, 0);
        emitStringRange("endAfterEnd", "abc", 2, 2);
        emitStringRange("overflow", "abc", Integer.MAX_VALUE, 1);
        emitStringRange("nullAfterEnd", null, 0, 5);
    }

    private static void writeCharArrayCases() {
        final char[] chars = {'A', '\uD800', 'B', 'C'};
        emitCharRange("full", chars, 0, chars.length, false);
        emitCharRange("middle", chars, 1, 2, false);
        emitCharRange("emptyAtEnd", chars, chars.length, 0, false);
        emitCharRange("negativeOffset", chars, -1, 1, false);
        emitCharRange("negativeLength", chars, 0, -1, false);
        emitCharRange("offsetAfterEnd", chars, chars.length + 1, 0, false);
        emitCharRange("endAfterEnd", chars, 3, 2, false);
        emitCharRange("overflow", chars, 1, Integer.MAX_VALUE, false);
        emitCharRange("nullFull", null, 0, 0, true);
        emitCharRange("nullRange", null, 0, 0, false);
        emitCharRange("nullNegativeOffset", null, -1, 0, false);
        emitCharRange("nullNegativeLength", null, 0, -1, false);
    }

    private static void lifecycleAndInheritedWriterCases() throws Exception {
        final FastStringWriter concrete = new FastStringWriter();
        final Writer writer = concrete;
        writer.write("A");
        writer.flush();
        writer.close();
        writer.write("B");
        writer.close();
        writer.flush();
        emit("lifecycle.afterClose", concrete.toString());

        final Writer appendWriter = new FastStringWriter();
        emit("append.nullIdentity", appendWriter.append(null) == appendWriter);
        emit("append.rangeIdentity", appendWriter.append("abcdef", 1, 4) == appendWriter);
        emit("append.charIdentity", appendWriter.append('\uD800') == appendWriter);
        emit("append.utf16", toUtf16Hex(appendWriter.toString()));

        emitAppendRange("nullMiddle", null, 1, 3);
        emitAppendRange("negativeStart", "abc", -1, 1);
        emitAppendRange("reversed", "abc", 2, 1);
        emitAppendRange("endAfterLength", "abc", 1, 4);
    }

    private static void exhaustiveCases() {
        final FastStringWriter intWriter = new FastStringWriter(196608);
        for (int value = -65536; value <= 131071; value++) {
            intWriter.write(value);
        }
        emit("exhaustive.writeIntHash", hex(hashString(intWriter.toString())));

        long stringRangeHash = FNV_OFFSET;
        final String string = "A\uD800BC";
        for (int off = -2; off <= 7; off++) {
            for (int len = -2; len <= 7; len++) {
                final FastStringWriter writer = new FastStringWriter();
                try {
                    writer.write(string, off, len);
                    stringRangeHash = mix(stringRangeHash, 1);
                    stringRangeHash = mixString(stringRangeHash, writer.toString());
                } catch (final RuntimeException exception) {
                    stringRangeHash = mix(stringRangeHash, 0);
                    stringRangeHash = mixString(
                            stringRangeHash,
                            exception.getClass().getName());
                    stringRangeHash = mixString(
                            stringRangeHash,
                            String.valueOf(exception.getMessage()));
                }
            }
        }
        emit("exhaustive.stringRangeHash", hex(stringRangeHash));

        long charRangeHash = FNV_OFFSET;
        final char[] chars = {'A', '\uD800', 'B', 'C'};
        for (int off = -2; off <= 7; off++) {
            for (int len = -2; len <= 7; len++) {
                final FastStringWriter writer = new FastStringWriter();
                try {
                    writer.write(chars, off, len);
                    charRangeHash = mix(charRangeHash, 1);
                    charRangeHash = mixString(charRangeHash, writer.toString());
                } catch (final RuntimeException exception) {
                    charRangeHash = mix(charRangeHash, 0);
                    charRangeHash = mixString(
                            charRangeHash,
                            exception.getClass().getName());
                    charRangeHash = mixString(
                            charRangeHash,
                            String.valueOf(exception.getMessage()));
                }
            }
        }
        emit("exhaustive.charRangeHash", hex(charRangeHash));
    }

    private static void emitStringRange(
            final String key, final String value, final int off, final int len) {
        final FastStringWriter writer = new FastStringWriter();
        emitOutcome(
                "writeStringRange." + key,
                () -> {
                    writer.write(value, off, len);
                    return toUtf16Hex(writer.toString());
                });
    }

    private static void emitCharRange(
            final String key,
            final char[] value,
            final int off,
            final int len,
            final boolean fullOverload) {
        final FastStringWriter writer = new FastStringWriter();
        emitOutcome(
                "writeChars." + key,
                () -> {
                    if (fullOverload) {
                        writer.write(value);
                    } else {
                        writer.write(value, off, len);
                    }
                    return toUtf16Hex(writer.toString());
                });
    }

    private static void emitAppendRange(
            final String key, final CharSequence value, final int start, final int end) {
        final Writer writer = new FastStringWriter();
        emitOutcome(
                "appendRange." + key,
                () -> {
                    writer.append(value, start, end);
                    return toUtf16Hex(writer.toString());
                });
    }

    private static void emitOutcome(final String key, final ThrowingSupplier action) {
        try {
            emit(key, "OK:" + String.valueOf(action.get()));
        } catch (final Throwable throwable) {
            emit(
                    key,
                    "ERR:" + throwable.getClass().getName() + ":"
                            + toUtf16Hex(String.valueOf(throwable.getMessage())));
        }
    }

    private static String toUtf16Hex(final String value) {
        final StringBuilder result = new StringBuilder(value.length() * 5);
        for (int i = 0; i < value.length(); i++) {
            if (i > 0) {
                result.append(',');
            }
            result.append(String.format("%04x", (int) value.charAt(i)));
        }
        return result.toString();
    }

    private static long hashString(final String value) {
        return mixString(FNV_OFFSET, value);
    }

    private static long mixString(long hash, final String value) {
        for (int i = 0; i < value.length(); i++) {
            final char unit = value.charAt(i);
            hash = mix(hash, unit & 0x00FF);
            hash = mix(hash, unit >>> 8);
        }
        return hash;
    }

    private static long mix(final long hash, final int value) {
        return (hash ^ value) * FNV_PRIME;
    }

    private static String hex(final long value) {
        return String.format("%016x", value);
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    @FunctionalInterface
    private interface ThrowingSupplier {
        Object get() throws Exception;
    }
}
