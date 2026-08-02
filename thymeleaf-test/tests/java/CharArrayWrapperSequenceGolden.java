package org.thymeleaf.util;

/**
 * 从固定 Thymeleaf Java 源码导出 CharArrayWrapperSequence Golden。
 */
public final class CharArrayWrapperSequenceGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final long FNV_OFFSET = 0xcbf29ce484222325L;
    private static final long FNV_PRIME = 0x100000001b3L;

    private CharArrayWrapperSequenceGolden() {
    }

    public static void main(final String[] args) throws Exception {
        emit("baseline", BASELINE);
        constructorCases();
        mutationAndCloneCases();
        accessCases();
        subsequenceCases();
        equalityHashAndStringCases();
        exhaustiveCases();
    }

    private static void constructorCases() {
        emitSequence("constructor.full", () -> new CharArrayWrapperSequence(chars()));
        emitSequence("constructor.null", () -> new CharArrayWrapperSequence((char[]) null));
        emitSequence("constructor.empty", () -> new CharArrayWrapperSequence(new char[0]));
        emitSequence("constructor.range", () -> new CharArrayWrapperSequence(chars(), 1, 2));
        emitSequence("constructor.zero", () -> new CharArrayWrapperSequence(chars(), 3, 0));
        emitSequence("constructor.negativeLength", () -> new CharArrayWrapperSequence(chars(), 1, -2));
        emitOutcome(
                "constructor.negativeLengthStored",
                () -> new CharArrayWrapperSequence(chars(), 1, -2).length());
        emitSequence("constructor.negativeOffset", () -> new CharArrayWrapperSequence(chars(), -1, 1));
        emitSequence("constructor.offsetAtEnd", () -> new CharArrayWrapperSequence(chars(), 4, 0));
        emitSequence("constructor.long", () -> new CharArrayWrapperSequence(chars(), 2, 3));
        emitSequence(
                "constructor.overflow",
                () -> new CharArrayWrapperSequence(chars(), 1, Integer.MAX_VALUE));
        emitOutcome(
                "constructor.overflowStored",
                () -> new CharArrayWrapperSequence(chars(), 1, Integer.MAX_VALUE).length());
        emitSequence(
                "constructor.minimumLength",
                () -> new CharArrayWrapperSequence(chars(), 1, Integer.MIN_VALUE));
        emitOutcome(
                "constructor.minimumLengthStored",
                () -> new CharArrayWrapperSequence(chars(), 1, Integer.MIN_VALUE).length());
        emitSequence(
                "constructor.nullRange",
                () -> new CharArrayWrapperSequence(null, -1, Integer.MAX_VALUE));
    }

    private static void mutationAndCloneCases() throws Exception {
        final char[] buffer = chars();
        final CharArrayWrapperSequence sequence = new CharArrayWrapperSequence(buffer, 1, 2);
        final CharArrayWrapperSequence clone = sequence.clone();
        emit("clone.distinct", clone != sequence);
        emit("clone.equals", clone.equals(sequence));
        emit("clone.hash", clone.hashCode() == sequence.hashCode());
        buffer[1] = 'Z';
        emit("mutation.original", describe(sequence));
        emit("mutation.clone", describe(clone));
        emit("mutation.equals", clone.equals(sequence));
    }

    private static void accessCases() {
        final CharArrayWrapperSequence sequence = new CharArrayWrapperSequence(chars(), 1, 2);
        emitOutcome("charAt.zero", () -> hexUnit(sequence.charAt(0)));
        emitOutcome("charAt.one", () -> hexUnit(sequence.charAt(1)));
        emitOutcome("charAt.negative", () -> hexUnit(sequence.charAt(-1)));
        emitOutcome("charAt.atLength", () -> hexUnit(sequence.charAt(2)));

        final CharArrayWrapperSequence overflow =
                new CharArrayWrapperSequence(chars(), 1, Integer.MAX_VALUE);
        emitOutcome("charAt.overflowViewZero", () -> hexUnit(overflow.charAt(0)));
        emitOutcome(
                "charAt.overflowViewLast",
                () -> hexUnit(overflow.charAt(Integer.MAX_VALUE - 1)));
        final CharArrayWrapperSequence negativeOverflow =
                new CharArrayWrapperSequence(chars(), 2, Integer.MAX_VALUE);
        emitOutcome(
                "charAt.overflowViewNegativeIndex",
                () -> hexUnit(negativeOverflow.charAt(Integer.MAX_VALUE - 1)));

        final CharArrayWrapperSequence negative =
                new CharArrayWrapperSequence(chars(), 1, -2);
        emitOutcome("charAt.negativeView", () -> hexUnit(negative.charAt(0)));
    }

    private static void subsequenceCases() {
        final CharArrayWrapperSequence sequence = new CharArrayWrapperSequence(chars());
        emitSequence("sub.full", () -> sequence.subSequence(0, 4));
        emitSequence("sub.middle", () -> sequence.subSequence(1, 3));
        emitSequence("sub.zeroAtStart", () -> sequence.subSequence(0, 0));
        emitSequence("sub.zeroAtLast", () -> sequence.subSequence(3, 3));
        emitSequence("sub.zeroAtEnd", () -> sequence.subSequence(4, 4));
        emitSequence("sub.negativeStart", () -> sequence.subSequence(-1, 1));
        emitSequence("sub.endAfter", () -> sequence.subSequence(1, 5));
        emitSequence("sub.reversed", () -> sequence.subSequence(2, 1));
        emitOutcome(
                "sub.reversedLength",
                () -> ((CharArrayWrapperSequence) sequence.subSequence(2, 1)).length());
        emitSequence("sub.negativeEnd", () -> sequence.subSequence(1, -1));
        emitOutcome(
                "sub.negativeEndLength",
                () -> ((CharArrayWrapperSequence) sequence.subSequence(1, -1)).length());
    }

    private static void equalityHashAndStringCases() {
        final char[] firstBuffer = chars();
        final CharArrayWrapperSequence first = new CharArrayWrapperSequence(firstBuffer, 1, 2);
        final CharArrayWrapperSequence sameContent =
                new CharArrayWrapperSequence(new char[]{'X', '\uD800', 'B', 'Y'}, 1, 2);
        final CharArrayWrapperSequence differentContent =
                new CharArrayWrapperSequence(new char[]{'X', '\uD800', 'C', 'Y'}, 1, 2);
        final CharArrayWrapperSequence differentLength =
                new CharArrayWrapperSequence(firstBuffer, 1, 1);
        emit("equals.identity", first.equals(first));
        emit("equals.null", first.equals(null));
        emit("equals.string", first.equals(first.toString()));
        emit("equals.sameContent", first.equals(sameContent));
        emit("equals.differentContent", first.equals(differentContent));
        emit("equals.differentLength", first.equals(differentLength));
        emit("hash.first", first.hashCode());
        emit("hash.sameContent", sameContent.hashCode());
        emit("hash.stringCompatible", first.hashCode() == first.toString().hashCode());
        emit("toString.first", toUtf16Hex(first.toString()));

        final CharArrayWrapperSequence negative =
                new CharArrayWrapperSequence(chars(), 1, -2);
        emitOutcome("negative.hash", negative::hashCode);
        emitOutcome("negative.toString", negative::toString);

        final CharArrayWrapperSequence overflow =
                new CharArrayWrapperSequence(chars(), 1, Integer.MAX_VALUE);
        emitOutcome("overflow.hash", overflow::hashCode);
        emitOutcome("overflow.toString", overflow::toString);
    }

    private static void exhaustiveCases() {
        long constructorHash = FNV_OFFSET;
        final int[] specialLengths = {
                -2, -1, 0, 1, 2, 3, 4, 5, Integer.MAX_VALUE, Integer.MIN_VALUE
        };
        for (int size = 0; size <= 6; size++) {
            final char[] buffer = new char[size];
            for (int index = 0; index < size; index++) {
                buffer[index] = (char) (0xD7FE + index);
            }
            for (int offset = -2; offset <= 8; offset++) {
                for (final int len : specialLengths) {
                    try {
                        final CharArrayWrapperSequence value =
                                new CharArrayWrapperSequence(buffer, offset, len);
                        constructorHash = mix(constructorHash, 1);
                        constructorHash = mix(constructorHash, value.length());
                        constructorHash = mix(constructorHash, value.hashCode());
                        try {
                            constructorHash = mixString(constructorHash, value.toString());
                        } catch (final RuntimeException exception) {
                            constructorHash = mixThrowable(constructorHash, exception);
                        }
                    } catch (final RuntimeException exception) {
                        constructorHash = mix(constructorHash, 0);
                        constructorHash = mixThrowable(constructorHash, exception);
                    }
                }
            }
        }
        emit("exhaustive.constructorHash", hex(constructorHash));

        long subsequenceHash = FNV_OFFSET;
        final CharArrayWrapperSequence sequence = new CharArrayWrapperSequence(chars());
        for (int start = -2; start <= 7; start++) {
            for (int end = -2; end <= 7; end++) {
                try {
                    final CharArrayWrapperSequence sub =
                            (CharArrayWrapperSequence) sequence.subSequence(start, end);
                    subsequenceHash = mix(subsequenceHash, 1);
                    subsequenceHash = mix(subsequenceHash, sub.length());
                    subsequenceHash = mix(subsequenceHash, sub.hashCode());
                    try {
                        subsequenceHash = mixString(subsequenceHash, sub.toString());
                    } catch (final RuntimeException exception) {
                        subsequenceHash = mixThrowable(subsequenceHash, exception);
                    }
                } catch (final RuntimeException exception) {
                    subsequenceHash = mix(subsequenceHash, 0);
                    subsequenceHash = mixThrowable(subsequenceHash, exception);
                }
            }
        }
        emit("exhaustive.subsequenceHash", hex(subsequenceHash));
    }

    private static char[] chars() {
        return new char[]{'A', '\uD800', 'B', 'C'};
    }

    private static String describe(final CharArrayWrapperSequence sequence) {
        return sequence.length() + ":" + sequence.hashCode() + ":" + toUtf16Hex(sequence.toString());
    }

    private static void emitSequence(final String key, final ThrowingSupplier action) {
        emitOutcome(
                key,
                () -> {
                    final CharArrayWrapperSequence sequence =
                            (CharArrayWrapperSequence) action.get();
                    return describe(sequence);
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

    private static long mixThrowable(long hash, final Throwable throwable) {
        hash = mixString(hash, throwable.getClass().getName());
        return mixString(hash, String.valueOf(throwable.getMessage()));
    }

    private static String hexUnit(final char value) {
        return String.format("%04x", (int) value);
    }

    private static String toUtf16Hex(final String value) {
        final StringBuilder result = new StringBuilder(value.length() * 5);
        for (int index = 0; index < value.length(); index++) {
            if (index > 0) {
                result.append(',');
            }
            result.append(hexUnit(value.charAt(index)));
        }
        return result.toString();
    }

    private static long mixString(long hash, final String value) {
        for (int index = 0; index < value.length(); index++) {
            final char unit = value.charAt(index);
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
