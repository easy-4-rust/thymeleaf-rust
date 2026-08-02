package org.thymeleaf.templateparser.text;

/**
 * 从固定 Thymeleaf Java 源码导出 TextParsingUtil Golden。
 */
public final class TextParsingUtilGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final long FNV_OFFSET = 0xcbf29ce484222325L;
    private static final long FNV_PRIME = 0x100000001b3L;

    private TextParsingUtilGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        structureEndCases();
        commentCases();
        literalCases();
        structureStartCases();
        wildcardCases();
        runtimeCases();
        exhaustiveCases();
    }

    private static void structureEndCases() {
        emitScan("structure.basic", "abc]z", 0, 5, new int[]{1, 2}, 0, false, '\'');
        emitScan("structure.quotes", "\"a]b\"]", 0, 6, new int[]{1, 1}, 0, false, '\'');
        emitScan("structure.apos", "'a]b']", 0, 6, new int[]{1, 1}, 0, false, '\'');
        emitScan("structure.crossQuotes", "\"a']b\"]", 0, 7, new int[]{1, 1}, 0, false, '\'');
        emitScan("structure.newline", "a\nb]c", 0, 5, new int[]{4, 9}, 0, false, '\'');
        emitScan("structure.missing", "abc", 0, 3, new int[]{1, 2}, 0, false, '\'');
        emitScan("structure.range", "xx]yy", 2, 4, new int[]{2, 7}, 0, false, '\'');
    }

    private static void commentCases() {
        emitScan("block.basic", "ab*/c", 0, 5, new int[]{1, 1}, 1, false, '\'');
        emitScan("block.firstSlash", "/abc", 0, 4, new int[]{1, 1}, 1, false, '\'');
        emitScan("block.newline", "a\n*/", 0, 4, new int[]{3, 8}, 1, false, '\'');
        emitScan("block.missing", "a*b", 0, 3, new int[]{1, 2}, 1, false, '\'');

        emitScan("line.basic", "ab\nc", 0, 4, new int[]{1, 2}, 2, false, '\'');
        emitScan("line.firstLf", "\nabc", 0, 4, new int[]{5, 7}, 2, false, '\'');
        emitScan("line.missing", "abc", 0, 3, new int[]{1, 2}, 2, false, '\'');
    }

    private static void literalCases() {
        emitScan("literal.basic", "a'b", 0, 3, new int[]{1, 1}, 3, false, '\'');
        emitScan("literal.escaped", "a\\'b'c", 0, 6, new int[]{1, 1}, 3, false, '\'');
        emitScan("literal.evenEscapes", "a\\\\'b", 0, 5, new int[]{1, 1}, 3, false, '\'');
        emitScan("literal.firstMarker", "'a'", 0, 3, new int[]{1, 1}, 3, false, '\'');
        emitScan("literal.newline", "a\nb`c", 0, 5, new int[]{2, 8}, 3, false, '`');
        emitScan("literal.missing", "abc", 0, 3, new int[]{1, 2}, 3, false, '"');
    }

    private static void structureStartCases() {
        emitScan("start.element", "ab[c", 0, 4, new int[]{1, 1}, 4, false, '\'');
        emitScan("start.slashEnabled", "ab/c", 0, 4, new int[]{1, 1}, 4, true, '\'');
        emitScan("start.slashDisabled", "ab/c", 0, 4, new int[]{1, 1}, 4, false, '\'');
        emitScan("start.quoteEnabled", "ab'c", 0, 4, new int[]{1, 1}, 4, true, '\'');
        emitScan("start.quoteEscaped", "a\\'c", 0, 4, new int[]{1, 1}, 4, true, '\'');
        emitScan("start.quoteEvenEscapes", "a\\\\'c", 0, 5, new int[]{1, 1}, 4, true, '\'');
        emitScan("start.backtick", "ab`c", 0, 4, new int[]{1, 1}, 4, true, '\'');
        emitScan("start.newline", "a\n/b", 0, 4, new int[]{2, 8}, 4, true, '\'');
        emitScan("start.missing", "abc", 0, 3, new int[]{1, 2}, 4, true, '\'');
    }

    private static void wildcardCases() {
        emitScan("whitespace.basic", "ab cd", 0, 5, new int[]{1, 1}, 5, false, '\'');
        emitScan("whitespace.quoted", "\"a b\" c", 0, 7, new int[]{1, 1}, 5, true, '\'');
        emitScan("whitespace.apos", "'a b' c", 0, 7, new int[]{1, 1}, 5, true, '\'');
        emitScan("whitespace.noAvoid", "\"a b\"", 0, 5, new int[]{1, 1}, 5, false, '\'');
        emitScan("whitespace.unclosed", "\"a b", 0, 4, new int[]{1, 1}, 5, true, '\'');
        emitScan("whitespace.unicode", "a\u3000b", 0, 3, new int[]{1, 1}, 5, false, '\'');
        emitScan("whitespace.nbsp", "a\u00a0b", 0, 3, new int[]{1, 1}, 5, false, '\'');
        emitScan("whitespace.missing", "abc", 0, 3, new int[]{1, 2}, 5, false, '\'');

        emitScan("nonWhitespace.basic", " \t\nx", 0, 4, new int[]{3, 7}, 6, false, '\'');
        emitScan("nonWhitespace.first", "x ", 0, 2, new int[]{1, 1}, 6, false, '\'');
        emitScan("nonWhitespace.missing", " \t", 0, 2, new int[]{1, 1}, 6, false, '\'');

        emitScan("operator.equal", "ab=c", 0, 4, new int[]{1, 1}, 7, false, '\'');
        emitScan("operator.space", "ab c", 0, 4, new int[]{1, 1}, 7, false, '\'');
        emitScan("operator.missing", "abc", 0, 3, new int[]{1, 2}, 7, false, '\'');

        emitScan("nonOperator.basic", "= \tx", 0, 4, new int[]{1, 1}, 8, false, '\'');
        emitScan("nonOperator.first", "x=", 0, 2, new int[]{1, 1}, 8, false, '\'');
        emitScan("nonOperator.missing", "= \t", 0, 3, new int[]{1, 1}, 8, false, '\'');

        emitScan("any.basic", "abc", 0, 3, new int[]{1, 1}, 9, false, '\'');
        emitScan("any.quotes", "\"ab\"c", 0, 5, new int[]{1, 1}, 9, false, '\'');
        emitScan("any.apos", "'ab'c", 0, 5, new int[]{1, 1}, 9, false, '\'');
        emitScan("any.quoteAtEnd", "\"ab\"", 0, 4, new int[]{1, 1}, 9, false, '\'');
        emitScan("any.unclosed", "\"ab", 0, 3, new int[]{1, 1}, 9, false, '\'');
        emitScan("any.newline", "\"a\nb\"c", 0, 6, new int[]{2, 7}, 9, false, '\'');
    }

    private static void runtimeCases() {
        for (int operation = 0; operation <= 9; operation++) {
            final int current = operation;
            final int[] nullTextLocator = {1, 1};
            emitOutcome(
                    "runtime.nullText." + operation,
                    nullTextLocator,
                    () -> invoke(current, null, 0, 1, nullTextLocator, false, '\''));
            final int[] negativeOffsetLocator = {1, 1};
            emitOutcome(
                    "runtime.negativeOffset." + operation,
                    negativeOffsetLocator,
                    () -> invoke(
                            current,
                            "x".toCharArray(),
                            -1,
                            0,
                            negativeOffsetLocator,
                            false,
                            '\''));
        }

        emitOutcome(
                "runtime.structureNullLocator",
                null,
                () -> invoke(0, "]".toCharArray(), 0, 1, null, false, '\''));
        emitOutcome(
                "runtime.blockNullLocator",
                null,
                () -> invoke(1, "*/".toCharArray(), 0, 2, null, false, '\''));
        emitOutcome(
                "runtime.lineNullLocator",
                null,
                () -> invoke(2, "\n".toCharArray(), 0, 1, null, false, '\''));
        emitOutcome(
                "runtime.literalNullLocator",
                null,
                () -> invoke(3, "a'".toCharArray(), 0, 2, null, false, '\''));
        emitOutcome(
                "runtime.startNullLocator",
                null,
                () -> invoke(4, "[".toCharArray(), 0, 1, null, true, '\''));
        emitOutcome(
                "runtime.whitespaceNullLocator",
                null,
                () -> invoke(5, "x".toCharArray(), 0, 1, null, false, '\''));
        emitOutcome(
                "runtime.nonWhitespaceNullLocator",
                null,
                () -> invoke(6, " ".toCharArray(), 0, 1, null, false, '\''));
        emitOutcome(
                "runtime.operatorNullLocator",
                null,
                () -> invoke(7, "x".toCharArray(), 0, 1, null, false, '\''));
        emitOutcome(
                "runtime.nonOperatorNullLocator",
                null,
                () -> invoke(8, "=".toCharArray(), 0, 1, null, false, '\''));
        emitOutcome(
                "runtime.anyNullLocator",
                null,
                () -> invoke(9, "\"".toCharArray(), 0, 1, null, false, '\''));

        final int[] one = {Integer.MAX_VALUE};
        emitOutcome(
                "runtime.structureOneLocator",
                one,
                () -> invoke(0, "\n]".toCharArray(), 0, 2, one, false, '\''));
        final int[] empty = {};
        emitOutcome(
                "runtime.whitespaceEmptyLocator",
                empty,
                () -> invoke(5, "x".toCharArray(), 0, 1, empty, false, '\''));
    }

    private static void exhaustiveCases() {
        long whitespaceHash = FNV_OFFSET;
        for (int unit = Character.MIN_VALUE; unit <= Character.MAX_VALUE; unit++) {
            final char[] text = {(char) unit};
            for (int operation = 5; operation <= 8; operation++) {
                final int[] locator = {1, 1};
                try {
                    final int result = invoke(operation, text, 0, 1, locator, false, '\'');
                    whitespaceHash = mix(whitespaceHash, result);
                    whitespaceHash = mix(whitespaceHash, locator[0]);
                    whitespaceHash = mix(whitespaceHash, locator[1]);
                } catch (final Throwable throwable) {
                    whitespaceHash = mixString(whitespaceHash, throwable.getClass().getName());
                    whitespaceHash = mixString(whitespaceHash, String.valueOf(throwable.getMessage()));
                }
            }
        }
        emit("exhaustive.whitespaceHash", hex(whitespaceHash));

        long delimiterHash = FNV_OFFSET;
        for (int slashes = 0; slashes <= 12; slashes++) {
            final StringBuilder text = new StringBuilder("a");
            for (int index = 0; index < slashes; index++) {
                text.append('\\');
            }
            text.append('\'').append('z').append('\'');
            for (int operation : new int[]{3, 4}) {
                final int[] locator = {1, 1};
                final int result = invoke(
                        operation,
                        text.toString().toCharArray(),
                        0,
                        text.length(),
                        locator,
                        true,
                        '\'');
                delimiterHash = mix(delimiterHash, result);
                delimiterHash = mix(delimiterHash, locator[1]);
            }
        }
        emit("exhaustive.delimiterHash", hex(delimiterHash));
    }

    private static void emitScan(
            final String key,
            final String text,
            final int offset,
            final int maxi,
            final int[] locator,
            final int operation,
            final boolean flag,
            final char marker) {
        emitOutcome(
                key,
                locator,
                () -> invoke(operation, text.toCharArray(), offset, maxi, locator, flag, marker));
    }

    private static int invoke(
            final int operation,
            final char[] text,
            final int offset,
            final int maxi,
            final int[] locator,
            final boolean flag,
            final char marker) {
        switch (operation) {
            case 0:
                return TextParsingUtil.findNextStructureEndAvoidQuotes(text, offset, maxi, locator);
            case 1:
                return TextParsingUtil.findNextCommentBlockEnd(text, offset, maxi, locator);
            case 2:
                return TextParsingUtil.findNextCommentLineEnd(text, offset, maxi, locator);
            case 3:
                return TextParsingUtil.findNextLiteralEnd(text, offset, maxi, locator, marker);
            case 4:
                return TextParsingUtil.findNextStructureStartOrLiteralMarker(
                        text, offset, maxi, locator, flag);
            case 5:
                return TextParsingUtil.findNextWhitespaceCharWildcard(
                        text, offset, maxi, flag, locator);
            case 6:
                return TextParsingUtil.findNextNonWhitespaceCharWildcard(
                        text, offset, maxi, locator);
            case 7:
                return TextParsingUtil.findNextOperatorCharWildcard(
                        text, offset, maxi, locator);
            case 8:
                return TextParsingUtil.findNextNonOperatorCharWildcard(
                        text, offset, maxi, locator);
            case 9:
                return TextParsingUtil.findNextAnyCharAvoidQuotesWildcard(
                        text, offset, maxi, locator);
            default:
                throw new AssertionError(operation);
        }
    }

    private static void emitOutcome(
            final String key, final int[] locator, final ThrowingIntSupplier action) {
        try {
            emit(key, "OK:" + action.getAsInt() + ":" + describe(locator));
        } catch (final Throwable throwable) {
            emit(
                    key,
                    "ERR:" + throwable.getClass().getName() + ":"
                            + toUtf16Hex(String.valueOf(throwable.getMessage())) + ":"
                            + describe(locator));
        }
    }

    private static String describe(final int[] locator) {
        if (locator == null) {
            return "null";
        }
        final StringBuilder result = new StringBuilder();
        for (int index = 0; index < locator.length; index++) {
            if (index > 0) {
                result.append(',');
            }
            result.append(locator[index]);
        }
        return result.toString();
    }

    private static String toUtf16Hex(final String value) {
        final StringBuilder result = new StringBuilder(value.length() * 5);
        for (int index = 0; index < value.length(); index++) {
            if (index > 0) {
                result.append(',');
            }
            result.append(String.format("%04x", (int) value.charAt(index)));
        }
        return result.toString();
    }

    private static long mixString(long hash, final String value) {
        for (int index = 0; index < value.length(); index++) {
            final char unit = value.charAt(index);
            hash = mix(hash, unit & 0x00ff);
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
    private interface ThrowingIntSupplier {
        int getAsInt() throws Throwable;
    }
}
