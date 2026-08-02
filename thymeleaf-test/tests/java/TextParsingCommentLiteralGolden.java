package org.thymeleaf.templateparser.text;

/**
 * 从固定 Thymeleaf Java 源码导出文本处理器、注释和正则字面量判定 Golden。
 */
public final class TextParsingCommentLiteralGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final long FNV_OFFSET = 0xcbf29ce484222325L;
    private static final long FNV_PRIME = 0x100000001b3L;

    private TextParsingCommentLiteralGolden() {
    }

    public static void main(final String[] args) throws Exception {
        emit("baseline", BASELINE);
        interfaceCases();
        commentCases();
        predicateCases();
        literalCases();
        exhaustiveCases();
    }

    private static void interfaceCases() throws Exception {
        final RecordingHandler handler = new RecordingHandler();
        final ITextHandler dynamic = handler;
        final char[] buffer = "n=o".toCharArray();
        dynamic.handleDocumentStart(11L, 1, 2);
        dynamic.handleDocumentEnd(13L, 2L, 3, 4);
        dynamic.handleText(buffer, 0, 3, 5, 6);
        dynamic.handleComment(buffer, 1, 1, 0, 3, 7, 8);
        dynamic.handleStandaloneElementStart(buffer, 0, 1, true, 9, 10);
        dynamic.handleStandaloneElementEnd(buffer, 0, 1, false, 11, 12);
        dynamic.handleOpenElementStart(buffer, 0, 1, 13, 14);
        dynamic.handleOpenElementEnd(buffer, 0, 1, 15, 16);
        dynamic.handleCloseElementStart(buffer, 0, 1, 17, 18);
        dynamic.handleCloseElementEnd(buffer, 0, 1, 19, 20);
        dynamic.handleAttribute(
                buffer, 0, 1, 21, 22,
                1, 1, 23, 24,
                2, 1, 1, 2, 25, 26);
        emit("interface.calls", handler.calls.toString());
    }

    private static void commentCases() {
        emitParse("comment.normal", "/*abc*/".toCharArray(), 0, 7, 3, 5, false, false);
        emitParse("comment.empty", "/**/".toCharArray(), 0, 4, -1, -2, false, false);
        emitParse("comment.embedded", "x/*a*/y".toCharArray(), 1, 5, 7, 9, false, false);
        emitParse("comment.mutation", "/*a*/".toCharArray(), 0, 5, 1, 1, true, false);
        emitParse("comment.handlerError", "/*a*/".toCharArray(), 0, 5, 2, 4, false, true);
        emitParse("comment.short", "/**".toCharArray(), 0, 3, 1, 2, false, false);
        emitParse("comment.badStart", "//a*/".toCharArray(), 0, 5, 1, 2, false, false);
        emitParse("comment.badEnd", "/*a**".toCharArray(), 0, 5, 1, 2, false, false);
        emitParse("comment.negativeOffset", "/*a*/".toCharArray(), -1, 5, 1, 2, false, false);
        emitParse("comment.longLength", "/*a*/".toCharArray(), 0, 6, 1, 2, false, false);
        emitParse("comment.overflowLength", "/*a*/".toCharArray(), 1, Integer.MAX_VALUE, 1, 2, false, false);
        emitParse("comment.nullShort", null, 0, 3, 1, 2, false, false);
        emitParse("comment.nullLong", null, 0, 4, 1, 2, false, false);
    }

    private static void predicateCases() {
        final char[] buffer = "x/*///**/".toCharArray();
        emitOutcome("blockStart.true", () -> TextParsingCommentUtil.isCommentBlockStart(buffer, 1, 3));
        emitOutcome("blockStart.short", () -> TextParsingCommentUtil.isCommentBlockStart(buffer, 1, 2));
        emitOutcome("blockStart.false", () -> TextParsingCommentUtil.isCommentBlockStart(buffer, 2, 5));
        emitOutcome("blockEnd.true", () -> TextParsingCommentUtil.isCommentBlockEnd(buffer, 7, 9));
        emitOutcome("lineStart.true", () -> TextParsingCommentUtil.isCommentLineStart(buffer, 3, 5));
        emitOutcome("predicate.nullShort", () -> TextParsingCommentUtil.isCommentBlockStart(null, 0, 1));
        emitOutcome("predicate.nullLong", () -> TextParsingCommentUtil.isCommentBlockStart(null, 0, 2));
        emitOutcome("predicate.negative", () -> TextParsingCommentUtil.isCommentBlockStart(buffer, -1, 2));
        emitOutcome(
                "predicate.wrappedDifference",
                () -> TextParsingCommentUtil.isCommentBlockStart(buffer, Integer.MIN_VALUE, Integer.MAX_VALUE));
    }

    private static void literalCases() {
        emitLiteral("literal.paren", "(/", 1, 2);
        emitLiteral("literal.equalsWhitespace", "= \t/", 3, 4);
        emitLiteral("literal.commaUnicodeWhitespace", ",\u3000/", 2, 3);
        emitLiteral("literal.other", "a/", 1, 2);
        emitLiteral("literal.beginning", "/", 0, 1);
        emitLiteral("literal.notSlash", "(x", 1, 2);
        emitLiteral("literal.blockComment", "(/*", 1, 3);
        emitLiteral("literal.lineComment", "(//", 1, 3);
        emitLiteral("literal.onlyWhitespace", " \t/", 2, 3);
        emitLiteral("literal.surrogate", "(\uD800/", 2, 3);
        emitOutcome("literal.nullZero", () -> TextParsingLiteralUtil.isRegexLiteralStart(null, 0, 1));
        emitOutcome("literal.nullPositive", () -> TextParsingLiteralUtil.isRegexLiteralStart(null, 1, 2));
        emitOutcome(
                "literal.negative",
                () -> TextParsingLiteralUtil.isRegexLiteralStart("/".toCharArray(), -1, 1));
        emitOutcome(
                "literal.offsetPastEnd",
                () -> TextParsingLiteralUtil.isRegexLiteralStart("/".toCharArray(), 1, 2));
    }

    private static void exhaustiveCases() {
        long whitespaceHash = FNV_OFFSET;
        for (int unit = Character.MIN_VALUE; unit <= Character.MAX_VALUE; unit++) {
            final char[] buffer = {'(', (char) unit, '/'};
            whitespaceHash = mix(
                    whitespaceHash,
                    TextParsingLiteralUtil.isRegexLiteralStart(buffer, 2, 3) ? 1 : 0);
        }
        emit("exhaustive.whitespaceHash", hex(whitespaceHash));

        long predicateHash = FNV_OFFSET;
        final char[] buffer = "/*//*/".toCharArray();
        for (int offset = -2; offset <= 8; offset++) {
            for (int maxi = -2; maxi <= 8; maxi++) {
                final int currentOffset = offset;
                final int currentMaxi = maxi;
                predicateHash = mixOutcome(
                        predicateHash,
                        () -> TextParsingCommentUtil.isCommentBlockStart(
                                buffer, currentOffset, currentMaxi));
                predicateHash = mixOutcome(
                        predicateHash,
                        () -> TextParsingCommentUtil.isCommentBlockEnd(
                                buffer, currentOffset, currentMaxi));
                predicateHash = mixOutcome(
                        predicateHash,
                        () -> TextParsingCommentUtil.isCommentLineStart(
                                buffer, currentOffset, currentMaxi));
            }
        }
        emit("exhaustive.predicateHash", hex(predicateHash));
    }

    private static void emitParse(
            final String key,
            final char[] buffer,
            final int offset,
            final int len,
            final int line,
            final int col,
            final boolean mutate,
            final boolean fail) {
        final RecordingHandler handler = new RecordingHandler();
        handler.mutate = mutate;
        handler.fail = fail;
        try {
            TextParsingCommentUtil.parseComment(buffer, offset, len, line, col, handler);
            emit(key, "OK:" + handler.calls + ":" + toUtf16Hex(new String(buffer)));
        } catch (final Throwable throwable) {
            emit(key, throwable(throwable));
        }
    }

    private static void emitLiteral(
            final String key, final String value, final int offset, final int maxi) {
        emitOutcome(
                key,
                () -> TextParsingLiteralUtil.isRegexLiteralStart(value.toCharArray(), offset, maxi));
    }

    private static void emitOutcome(final String key, final ThrowingSupplier action) {
        try {
            emit(key, "OK:" + String.valueOf(action.get()));
        } catch (final Throwable throwable) {
            emit(key, throwable(throwable));
        }
    }

    private static long mixOutcome(long hash, final ThrowingSupplier action) {
        try {
            hash = mix(hash, 1);
            return mix(hash, Boolean.TRUE.equals(action.get()) ? 1 : 0);
        } catch (final Throwable throwable) {
            hash = mix(hash, 0);
            hash = mixString(hash, throwable.getClass().getName());
            return mixString(hash, String.valueOf(throwable.getMessage()));
        }
    }

    private static String throwable(final Throwable throwable) {
        return "ERR:" + throwable.getClass().getName() + ":"
                + toUtf16Hex(String.valueOf(throwable.getMessage()));
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

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    @FunctionalInterface
    private interface ThrowingSupplier {
        Object get() throws Throwable;
    }

    private static final class RecordingHandler implements ITextHandler {
        private final StringBuilder calls = new StringBuilder();
        private boolean mutate;
        private boolean fail;

        private void record(final String value) {
            if (calls.length() > 0) {
                calls.append('|');
            }
            calls.append(value);
        }

        @Override
        public void handleDocumentStart(final long startTimeNanos, final int line, final int col) {
            record("DS:" + startTimeNanos + ":" + line + ":" + col);
        }

        @Override
        public void handleDocumentEnd(
                final long endTimeNanos, final long totalTimeNanos, final int line, final int col) {
            record("DE:" + endTimeNanos + ":" + totalTimeNanos + ":" + line + ":" + col);
        }

        @Override
        public void handleText(
                final char[] buffer, final int offset, final int len, final int line, final int col) {
            record("T:" + offset + ":" + len + ":" + line + ":" + col);
        }

        @Override
        public void handleComment(
                final char[] buffer,
                final int contentOffset,
                final int contentLen,
                final int outerOffset,
                final int outerLen,
                final int line,
                final int col) throws TextParseException {
            record("C:" + contentOffset + ":" + contentLen + ":" + outerOffset + ":"
                    + outerLen + ":" + line + ":" + col);
            if (mutate) {
                buffer[contentOffset] = 'Z';
            }
            if (fail) {
                throw new TextParseException("handler", 41, 43);
            }
        }

        @Override
        public void handleStandaloneElementStart(
                final char[] buffer, final int nameOffset, final int nameLen,
                final boolean minimized, final int line, final int col) {
            record("SS:" + nameOffset + ":" + nameLen + ":" + minimized + ":" + line + ":" + col);
        }

        @Override
        public void handleStandaloneElementEnd(
                final char[] buffer, final int nameOffset, final int nameLen,
                final boolean minimized, final int line, final int col) {
            record("SE:" + nameOffset + ":" + nameLen + ":" + minimized + ":" + line + ":" + col);
        }

        @Override
        public void handleOpenElementStart(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) {
            record("OS:" + nameOffset + ":" + nameLen + ":" + line + ":" + col);
        }

        @Override
        public void handleOpenElementEnd(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) {
            record("OE:" + nameOffset + ":" + nameLen + ":" + line + ":" + col);
        }

        @Override
        public void handleCloseElementStart(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) {
            record("CS:" + nameOffset + ":" + nameLen + ":" + line + ":" + col);
        }

        @Override
        public void handleCloseElementEnd(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) {
            record("CE:" + nameOffset + ":" + nameLen + ":" + line + ":" + col);
        }

        @Override
        public void handleAttribute(
                final char[] buffer,
                final int nameOffset, final int nameLen, final int nameLine, final int nameCol,
                final int operatorOffset, final int operatorLen,
                final int operatorLine, final int operatorCol,
                final int valueContentOffset, final int valueContentLen,
                final int valueOuterOffset, final int valueOuterLen,
                final int valueLine, final int valueCol) {
            record("A:" + nameOffset + ":" + nameLen + ":" + nameLine + ":" + nameCol + ":"
                    + operatorOffset + ":" + operatorLen + ":" + operatorLine + ":"
                    + operatorCol + ":" + valueContentOffset + ":" + valueContentLen + ":"
                    + valueOuterOffset + ":" + valueOuterLen + ":" + valueLine + ":" + valueCol);
        }
    }
}
