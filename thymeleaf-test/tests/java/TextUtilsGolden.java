import java.util.Arrays;

import org.thymeleaf.util.TextUtils;

/**
 * 从固定 Thymeleaf Java 源码导出 TextUtils 全重载与 UTF-16 行为 Golden。
 */
public final class TextUtilsGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emitAllOverloads();
        emitErrors();
        emitDynamicSequenceTraces();
        emitCaseFoldDigest();
        emitContainsCorpus();
    }

    private static void emitAllOverloads() {
        final String text = "xxAb\u0131\u03C2zzyy";
        final String fragment = "aB\u0049\u03C3";
        final char[] textChars = text.toCharArray();
        final char[] fragmentChars = fragment.toCharArray();

        emitCall("equals.sequence_sequence",
                () -> TextUtils.equals(false, "Ab\u0131\u03C2", fragment));
        emitCall("equals.sequence_chars",
                () -> TextUtils.equals(false, "Ab\u0131\u03C2", fragmentChars));
        emitCall("equals.chars_chars",
                () -> TextUtils.equals(false, "Ab\u0131\u03C2".toCharArray(), fragmentChars));
        emitCall("equals.chars_range",
                () -> TextUtils.equals(false, textChars, 2, 4, fragmentChars, 0, 4));
        emitCall("equals.sequence_chars_range",
                () -> TextUtils.equals(false, text, 2, 4, fragmentChars, 0, 4));
        emitCall("equals.sequence_sequence_range",
                () -> TextUtils.equals(false, text, 2, 4, fragment, 0, 4));

        emitCall("starts.sequence_sequence",
                () -> TextUtils.startsWith(false, "Ab\u0131\u03C2-tail", fragment));
        emitCall("starts.sequence_chars",
                () -> TextUtils.startsWith(false, "Ab\u0131\u03C2-tail", fragmentChars));
        emitCall("starts.chars_chars",
                () -> TextUtils.startsWith(false, "Ab\u0131\u03C2-tail".toCharArray(), fragmentChars));
        emitCall("starts.chars_range",
                () -> TextUtils.startsWith(false, textChars, 2, 6, fragmentChars, 0, 4));
        emitCall("starts.sequence_chars_range",
                () -> TextUtils.startsWith(false, text, 2, 6, fragmentChars, 0, 4));
        emitCall("starts.chars_sequence_range",
                () -> TextUtils.startsWith(false, textChars, 2, 6, fragment, 0, 4));
        emitCall("starts.sequence_sequence_range",
                () -> TextUtils.startsWith(false, text, 2, 6, fragment, 0, 4));

        emitCall("ends.sequence_sequence",
                () -> TextUtils.endsWith(false, "head-Ab\u0131\u03C2", fragment));
        emitCall("ends.sequence_chars",
                () -> TextUtils.endsWith(false, "head-Ab\u0131\u03C2", fragmentChars));
        emitCall("ends.chars_chars",
                () -> TextUtils.endsWith(false, "head-Ab\u0131\u03C2".toCharArray(), fragmentChars));
        emitCall("ends.chars_range",
                () -> TextUtils.endsWith(false, textChars, 0, 6, fragmentChars, 0, 4));
        emitCall("ends.sequence_chars_range",
                () -> TextUtils.endsWith(false, text, 0, 6, fragmentChars, 0, 4));
        emitCall("ends.chars_sequence_range",
                () -> TextUtils.endsWith(false, textChars, 0, 6, fragment, 0, 4));
        emitCall("ends.sequence_sequence_range",
                () -> TextUtils.endsWith(false, text, 0, 6, fragment, 0, 4));

        emitCall("contains.sequence_sequence",
                () -> TextUtils.contains(false, text, fragment));
        emitCall("contains.sequence_chars",
                () -> TextUtils.contains(false, text, fragmentChars));
        emitCall("contains.chars_chars",
                () -> TextUtils.contains(false, textChars, fragmentChars));
        emitCall("contains.chars_range",
                () -> TextUtils.contains(false, textChars, 0, textChars.length, fragmentChars, 0, 4));
        emitCall("contains.sequence_chars_range",
                () -> TextUtils.contains(false, text, 0, text.length(), fragmentChars, 0, 4));
        emitCall("contains.chars_sequence_range",
                () -> TextUtils.contains(false, textChars, 0, textChars.length, fragment, 0, 4));
        emitCall("contains.sequence_sequence_range",
                () -> TextUtils.contains(false, text, 0, text.length(), fragment, 0, 4));

        emitCall("compare.sequence_sequence",
                () -> TextUtils.compareTo(false, "Ab\u0131\u03C2", fragment));
        emitCall("compare.sequence_chars",
                () -> TextUtils.compareTo(false, "Ab\u0131\u03C2", fragmentChars));
        emitCall("compare.chars_chars",
                () -> TextUtils.compareTo(false, "Ab\u0131\u03C2".toCharArray(), fragmentChars));
        emitCall("compare.chars_range",
                () -> TextUtils.compareTo(false, textChars, 2, 4, fragmentChars, 0, 4));
        emitCall("compare.sequence_chars_range",
                () -> TextUtils.compareTo(false, text, 2, 4, fragmentChars, 0, 4));
        emitCall("compare.sequence_sequence_range",
                () -> TextUtils.compareTo(false, text, 2, 4, fragment, 0, 4));

        final char[][] charValues = {
                "A".toCharArray(), "ab".toCharArray(), "b".toCharArray(), "z".toCharArray()
        };
        final CharSequence[] sequenceValues = {"A", new StringBuilder("ab"), "b", "z"};
        final char[] searchChars = "--AB--".toCharArray();
        final CharSequence searchSequence = new StringBuilder("--AB--");
        emitCall("binary.char_values_chars",
                () -> TextUtils.binarySearch(false, charValues, searchChars, 2, 2));
        emitCall("binary.char_values_sequence",
                () -> TextUtils.binarySearch(false, charValues, searchSequence, 2, 2));
        emitCall("binary.sequence_values_chars",
                () -> TextUtils.binarySearch(false, sequenceValues, searchChars, 2, 2));
        emitCall("binary.sequence_values_sequence",
                () -> TextUtils.binarySearch(false, sequenceValues, searchSequence, 2, 2));
        emitCall("binary.char_values_chars_range",
                () -> TextUtils.binarySearch(false, charValues, 1, 2, searchChars, 2, 2));
        emitCall("binary.char_values_sequence_range",
                () -> TextUtils.binarySearch(false, charValues, 1, 2, searchSequence, 2, 2));
        emitCall("binary.sequence_values_chars_range",
                () -> TextUtils.binarySearch(false, sequenceValues, 1, 2, searchChars, 2, 2));
        emitCall("binary.sequence_values_sequence_range",
                () -> TextUtils.binarySearch(false, sequenceValues, 1, 2, searchSequence, 2, 2));

        emitCall("hash.chars_range",
                () -> TextUtils.hashCode(textChars, 2, 4));
        emitCall("hash.sequence", () -> TextUtils.hashCode((CharSequence) "Ab\u0131\u03C2"));
        emitCall("hash.sequence_range", () -> TextUtils.hashCode((CharSequence) text, 2, 6));
        emitCall("hash.pair", () -> TextUtils.hashCode("Ab", "\u0131\u03C2"));
        emitCall("hash.triple", () -> TextUtils.hashCode("A", "b", "\u0131\u03C2"));
        emitCall("hash.quadruple", () -> TextUtils.hashCode("A", "b", "\u0131", "\u03C2"));
        emitCall("hash.quintuple", () -> TextUtils.hashCode("", "A", "b", "\u0131", "\u03C2"));
    }

    private static void emitErrors() {
        emitCall("error.equals.short_null_first",
                () -> TextUtils.equals(true, (CharSequence) null, new char[0]));
        emitCall("error.equals.range_null_first",
                () -> TextUtils.equals(true, (char[]) null, 0, 0, new char[0], 0, 0));
        emitCall("error.starts.short_null_prefix",
                () -> TextUtils.startsWith(true, "a", (char[]) null));
        emitCall("error.ends.range_null_suffix",
                () -> TextUtils.endsWith(true, "a", 0, 1, (char[]) null, 0, 0));
        emitCall("error.contains.range_invalid_text",
                () -> TextUtils.contains(true, new char[0], 1, 1, new char[]{'a'}, 0, 1));
        emitCall("error.compare.range_invalid_second",
                () -> TextUtils.compareTo(true, new char[]{'a'}, 0, 1, new char[0], 0, 1));
        emitCall("error.binary.null_values",
                () -> TextUtils.binarySearch(true, (char[][]) null, new char[0], 0, 0));
        emitCall("error.binary.null_text",
                () -> TextUtils.binarySearch(true, new char[0][], 0, 0, (char[]) null, 0, 0));
        emitCall("error.binary.null_mid",
                () -> TextUtils.binarySearch(true, new char[][]{null}, new char[0], 0, 0));
        emitCall("error.binary.outer_index",
                () -> TextUtils.binarySearch(true, new char[0][], 1, 1, new char[0], 0, 0));
        emitCall("error.hash.null_chars_empty",
                () -> TextUtils.hashCode((char[]) null, 0, 0));
        emitCall("error.hash.null_chars_one",
                () -> TextUtils.hashCode((char[]) null, 0, 1));
        emitCall("error.hash.null_sequence_empty_range",
                () -> TextUtils.hashCode((CharSequence) null, 1, 1));
        emitCall("error.hash.null_sequence_zero_range",
                () -> TextUtils.hashCode((CharSequence) null, 0, 0));
    }

    private static void emitDynamicSequenceTraces() {
        final ProbeSequence left = new ProbeSequence("Ab");
        final ProbeSequence right = new ProbeSequence("aB");
        emitCall("trace.equals.result", () -> TextUtils.equals(false, left, right));
        emit("trace.equals.left", left.trace.toString());
        emit("trace.equals.right", right.trace.toString());

        final ProbeSequence hash = new ProbeSequence("Ab");
        emitCall("trace.hash.result", () -> TextUtils.hashCode((CharSequence) hash));
        emit("trace.hash.calls", hash.trace.toString());

        final ProbeSequence suffixText = new ProbeSequence("xxAb");
        final ProbeSequence suffix = new ProbeSequence("ab");
        emitCall("trace.ends.result", () -> TextUtils.endsWith(false, suffixText, suffix));
        emit("trace.ends.text", suffixText.trace.toString());
        emit("trace.ends.suffix", suffix.trace.toString());
    }

    private static void emitCaseFoldDigest() {
        long digest = 0xcbf29ce484222325L;
        for (int value = Character.MIN_VALUE; value <= Character.MAX_VALUE; value++) {
            final char source = (char) value;
            final char upper = Character.toUpperCase(source);
            final char lower = Character.toLowerCase(source);
            digest = mix(digest, TextUtils.equals(false, new char[]{source}, new char[]{upper}) ? 1 : 0);
            digest = mix(digest, TextUtils.equals(false, new char[]{source}, new char[]{lower}) ? 1 : 0);
            digest = mix(digest, TextUtils.compareTo(false, new char[]{source}, new char[]{upper}));
            digest = mix(digest, TextUtils.compareTo(false, new char[]{source}, new char[]{lower}));
        }
        emit("digest.case_fold", Long.toUnsignedString(digest, 16));
    }

    private static void emitContainsCorpus() {
        final String[] texts = {
                "", "a", "aa", "aab", "ababab", "mississippi",
                "\u0131I\u03C2\u03A3", "\uD800x\uDC00", "0123456789"
        };
        final String[] fragments = {
                "", "a", "ab", "aba", "issi", "ssip", "I\u03A3",
                "\uD800x", "\uDC00", "xyz"
        };
        long digest = 0xcbf29ce484222325L;
        int cases = 0;
        for (final boolean caseSensitive : new boolean[]{false, true}) {
            for (final String text : texts) {
                for (final String fragment : fragments) {
                    final boolean expected = TextUtils.contains(caseSensitive, text, fragment);
                    final char[] paddedText = ("#" + text + "!").toCharArray();
                    final char[] paddedFragment = ("#" + fragment + "!").toCharArray();
                    final boolean ranged = TextUtils.contains(
                            caseSensitive,
                            paddedText, 1, text.length(),
                            paddedFragment, 1, fragment.length());
                    digest = mix(digest, expected ? 1 : 0);
                    digest = mix(digest, ranged ? 1 : 0);
                    cases += 2;
                }
            }
        }
        emit("digest.contains", Long.toUnsignedString(digest, 16));
        emit("digest.contains_cases", Integer.toString(cases));
    }

    private static long mix(final long hash, final int value) {
        return (hash ^ Integer.toUnsignedLong(value)) * 0x100000001b3L;
    }

    private static void emitCall(final String key, final ThrowingSupplier supplier) {
        try {
            emit(key, String.valueOf(supplier.get()));
        } catch (final Throwable throwable) {
            final String message = throwable instanceof NullPointerException
                    ? "<ignored>"
                    : encode(throwable.getMessage());
            emit(key, throwable.getClass().getName() + "|" + message);
        }
    }

    private static String encode(final String value) {
        if (value == null) {
            return "<null>";
        }
        final StringBuilder encoded = new StringBuilder();
        for (int index = 0; index < value.length(); index++) {
            if (index > 0) {
                encoded.append(',');
            }
            encoded.append(String.format("%04X", Integer.valueOf(value.charAt(index))));
        }
        return encoded.toString();
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private interface ThrowingSupplier {
        Object get();
    }

    private static final class ProbeSequence implements CharSequence {
        private final char[] value;
        private final StringBuilder trace = new StringBuilder();

        private ProbeSequence(final String value) {
            this.value = value.toCharArray();
        }

        @Override
        public int length() {
            trace.append('L').append(';');
            return value.length;
        }

        @Override
        public char charAt(final int index) {
            trace.append('C').append(index).append(';');
            return value[index];
        }

        @Override
        public CharSequence subSequence(final int start, final int end) {
            trace.append('S').append(start).append(',').append(end).append(';');
            return new String(Arrays.copyOfRange(value, start, end));
        }
    }

    private TextUtilsGolden() {
    }
}
