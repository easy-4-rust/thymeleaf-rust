package org.thymeleaf.templateparser.text;

/**
 * 从固定 Thymeleaf Java 源码导出属性序列解析语义。
 */
public final class TextParsingAttributeSequenceGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final long FNV_OFFSET = 0xcbf29ce484222325L;
    private static final long FNV_PRIME = 0x100000001b3L;

    private TextParsingAttributeSequenceGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        fixedCases();
        handlerCases();
        runtimeCases();
        exhaustiveCases();
    }

    private static void fixedCases() {
        emitCase("fixed.empty", "", 0, 0, 1, 1, Mode.NORMAL);
        emitCase("fixed.whitespace", " \t\n", 0, 3, 2, 3, Mode.NORMAL);
        emitCase("fixed.nameOnly", "disabled", 0, 8, 1, 1, Mode.NORMAL);
        emitCase("fixed.twoNames", "a b", 0, 3, 4, 5, Mode.NORMAL);
        emitCase("fixed.equalsOnly", "a=", 0, 2, 1, 1, Mode.NORMAL);
        emitCase("fixed.equalsTrailingSpace", "a = \t", 0, 5, 1, 1, Mode.NORMAL);
        emitCase("fixed.unquoted", "a=b", 0, 3, 1, 1, Mode.NORMAL);
        emitCase("fixed.operatorSpaces", "a \t= \tb", 0, 7, 3, 7, Mode.NORMAL);
        emitCase("fixed.doubleQuoted", "a=\"x y\"", 0, 7, 1, 1, Mode.NORMAL);
        emitCase("fixed.singleQuoted", "a='x y'", 0, 7, 1, 1, Mode.NORMAL);
        emitCase("fixed.emptyDoubleQuoted", "a=\"\"", 0, 4, 1, 1, Mode.NORMAL);
        emitCase("fixed.emptySingleQuoted", "a=''", 0, 4, 1, 1, Mode.NORMAL);
        emitCase("fixed.unclosedDouble", "a=\"x y", 0, 6, 1, 1, Mode.NORMAL);
        emitCase("fixed.unclosedSingle", "a='x y", 0, 6, 1, 1, Mode.NORMAL);
        emitCase("fixed.adjacentAfterQuote", "a=\"x\"b=y", 0, 8, 1, 1, Mode.NORMAL);
        emitCase("fixed.multiple", "a=b c='d e' f = \"\" g", 0, 20, 5, 9, Mode.NORMAL);
        emitCase("fixed.multipleEquals", "a==b", 0, 4, 1, 1, Mode.NORMAL);
        emitCase("fixed.noEqualsThenValue", "a b=c", 0, 5, 1, 1, Mode.NORMAL);
        emitCase("fixed.leadingEquals", "=a", 0, 2, 7, 11, Mode.NORMAL);
        emitCase("fixed.onlyEquals", "=", 0, 1, -3, -5, Mode.NORMAL);
        emitCase("fixed.embeddedRange", "xxa=\"v w\"yy", 2, 7, 8, 13, Mode.NORMAL);
        emitCase("fixed.newlineBeforeName", "\n a=b", 0, 5, 10, 20, Mode.NORMAL);
        emitCase("fixed.newlineOperator", "a\n=\nb", 0, 5, 10, 20, Mode.NORMAL);
        emitCase("fixed.newlineQuotedValue", "a=\"x\ny\" z=q", 0, 11, 10, 20, Mode.NORMAL);
        emitCase("fixed.nulAndSurrogate", "a=\u0000 b=\uD800", 0, 7, 1, 1, Mode.NORMAL);
        emitCase("fixed.lineOverflow", "\na=b", 0, 4, Integer.MAX_VALUE, Integer.MAX_VALUE, Mode.NORMAL);
        emitCase("fixed.columnOverflow", "ab=c", 0, 4, 1, Integer.MAX_VALUE, Mode.NORMAL);
    }

    private static void handlerCases() {
        emitCase("handler.mutateFuture", "a=x b=y", 0, 7, 1, 1, Mode.MUTATE_FUTURE);
        emitCase("handler.checkedFirst", "a=x b=y", 0, 7, 1, 1, Mode.CHECKED_FIRST);
        emitCase("handler.checkedSecond", "a=x b=y", 0, 7, 1, 1, Mode.CHECKED_SECOND);
        emitCase("handler.checkedNameOnly", "a", 0, 1, 1, 1, Mode.CHECKED_FIRST);
        emitCase("handler.checkedNoOperator", "a b", 0, 3, 1, 1, Mode.CHECKED_FIRST);
        emitCase("handler.checkedNoValue", "a=", 0, 2, 1, 1, Mode.CHECKED_FIRST);
        emitCase("handler.runtimeFirst", "a=x b=y", 0, 7, 1, 1, Mode.RUNTIME_FIRST);

        emitAction("handler.nullEmpty", () ->
                TextParsingAttributeSequenceUtil.parseAttributeSequence(null, 0, 0, 1, 1, null));
        emitAction("handler.nullWhitespace", () ->
                TextParsingAttributeSequenceUtil.parseAttributeSequence(
                        " ".toCharArray(), 0, 1, 1, 1, null));
        emitAction("handler.nullAttribute", () ->
                TextParsingAttributeSequenceUtil.parseAttributeSequence(
                        "a".toCharArray(), 0, 1, 1, 1, null));
    }

    private static void runtimeCases() {
        emitCase("runtime.nullEmpty", null, 0, 0, 1, 1, Mode.NORMAL);
        emitCase("runtime.nullNegativeLen", null, 0, -1, 1, 1, Mode.NORMAL);
        emitCase("runtime.nullOne", null, 0, 1, 1, 1, Mode.NORMAL);
        emitCase("runtime.negativeOffset", "a", -1, 1, 1, 1, Mode.NORMAL);
        emitCase("runtime.offsetAtEndEmpty", "a", 1, 0, 1, 1, Mode.NORMAL);
        emitCase("runtime.offsetPastEndEmpty", "a", 2, 0, 1, 1, Mode.NORMAL);
        emitCase("runtime.offsetPastEnd", "a", 1, 1, 1, 1, Mode.NORMAL);
        emitCase("runtime.negativeLen", "a", 0, -1, 1, 1, Mode.NORMAL);
        emitCase("runtime.overflowRange", "a", 1, Integer.MAX_VALUE, 1, 1, Mode.NORMAL);
        emitCase("runtime.badNameStringRange", "=", 0, 2, 1, 1, Mode.NORMAL);
        emitCase("runtime.badNameNegativeStringRange", "=", 0, Integer.MAX_VALUE, 1, 1, Mode.NORMAL);
        emitCase("runtime.operatorPastEnd", "a=", 0, 3, 1, 1, Mode.NORMAL);
        emitCase("runtime.quotedValuePastEnd", "a=\"", 0, 4, 1, 1, Mode.NORMAL);
        emitCase("runtime.unquotedValuePastEnd", "a=b", 0, 4, 1, 1, Mode.NORMAL);
    }

    private static void exhaustiveCases() {
        long whitespaceHash = FNV_OFFSET;
        for (int unit = Character.MIN_VALUE; unit <= Character.MAX_VALUE; unit++) {
            final char[] buffer = {'a', (char) unit, 'b', '=', 'c'};
            whitespaceHash = mixString(
                    whitespaceHash,
                    outcome(buffer, 0, buffer.length, 3, 5, Mode.NORMAL));
        }
        emit("exhaustive.whitespaceHash", hex(whitespaceHash));

        long quotedHash = FNV_OFFSET;
        for (int unit = Character.MIN_VALUE; unit <= Character.MAX_VALUE; unit++) {
            final char[] buffer = {'a', '=', '"', (char) unit, '"', ' ', 'b', '=', 'z'};
            quotedHash = mixString(
                    quotedHash,
                    outcome(buffer, 0, buffer.length, 7, 11, Mode.NORMAL));
        }
        emit("exhaustive.quotedHash", hex(quotedHash));

        long grammarHash = FNV_OFFSET;
        final String[] names = {"a", "x:y", "data-x", "", "="};
        final String[] operators = {"", "=", " = ", "==", " \t"};
        final String[] values = {"", "v", "\"x y\"", "'x y'", "\"\"", "\"x", "/"};
        final String[] separators = {"", " ", "\n"};
        for (final String firstName : names) {
            for (final String firstOperator : operators) {
                for (final String firstValue : values) {
                    for (final String separator : separators) {
                        for (final String secondName : names) {
                            final String text =
                                    firstName + firstOperator + firstValue + separator + secondName + "=z";
                            grammarHash = mixString(
                                    grammarHash,
                                    outcome(
                                            text.toCharArray(),
                                            0,
                                            text.length(),
                                            -7,
                                            Integer.MAX_VALUE,
                                            Mode.NORMAL));
                        }
                    }
                }
            }
        }
        emit("exhaustive.grammarHash", hex(grammarHash));

        long rangeHash = FNV_OFFSET;
        final char[] rangeBuffer = "xxa = \"v w\" yy".toCharArray();
        for (int offset = -2; offset <= rangeBuffer.length + 2; offset++) {
            for (int len = -2; len <= rangeBuffer.length + 4; len++) {
                rangeHash = mixString(
                        rangeHash,
                        outcome(rangeBuffer.clone(), offset, len, 13, 17, Mode.NORMAL));
            }
        }
        emit("exhaustive.rangeHash", hex(rangeHash));
    }

    private static void emitCase(
            final String key,
            final String text,
            final int offset,
            final int len,
            final int line,
            final int col,
            final Mode mode) {
        emit(
                key,
                outcome(text == null ? null : text.toCharArray(), offset, len, line, col, mode));
    }

    private static String outcome(
            final char[] buffer,
            final int offset,
            final int len,
            final int line,
            final int col,
            final Mode mode) {
        final RecordingHandler handler = new RecordingHandler(mode);
        try {
            TextParsingAttributeSequenceUtil.parseAttributeSequence(
                    buffer, offset, len, line, col, handler);
            return "OK:" + handler.calls + ":" + bufferHex(buffer);
        } catch (final Throwable throwable) {
            return throwable(throwable) + ":" + handler.calls + ":" + bufferHex(buffer);
        }
    }

    private static void emitAction(final String key, final ThrowingAction action) {
        try {
            action.run();
            emit(key, "OK");
        } catch (final Throwable throwable) {
            emit(key, throwable(throwable));
        }
    }

    private static String throwable(final Throwable throwable) {
        return "ERR:" + throwable.getClass().getName() + ":"
                + toUtf16Hex(String.valueOf(throwable.getMessage()));
    }

    private static String bufferHex(final char[] buffer) {
        return buffer == null ? "null" : toUtf16Hex(new String(buffer));
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

    private enum Mode {
        NORMAL,
        MUTATE_FUTURE,
        CHECKED_FIRST,
        CHECKED_SECOND,
        RUNTIME_FIRST
    }

    @FunctionalInterface
    private interface ThrowingAction {
        void run() throws Throwable;
    }

    private static final class RecordingHandler implements ITextHandler {

        private final StringBuilder calls = new StringBuilder();
        private final Mode mode;
        private int callCount;

        private RecordingHandler(final Mode mode) {
            this.mode = mode;
        }

        private void unused() {
            // Keeps the ten unrelated interface callbacks visibly intentional in this Oracle.
        }

        @Override
        public void handleDocumentStart(final long startTimeNanos, final int line, final int col) {
            unused();
        }

        @Override
        public void handleDocumentEnd(
                final long endTimeNanos,
                final long totalTimeNanos,
                final int line,
                final int col) {
            unused();
        }

        @Override
        public void handleText(
                final char[] buffer,
                final int offset,
                final int len,
                final int line,
                final int col) {
            unused();
        }

        @Override
        public void handleComment(
                final char[] buffer,
                final int contentOffset,
                final int contentLen,
                final int outerOffset,
                final int outerLen,
                final int line,
                final int col) {
            unused();
        }

        @Override
        public void handleStandaloneElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final boolean minimized,
                final int line,
                final int col) {
            unused();
        }

        @Override
        public void handleStandaloneElementEnd(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final boolean minimized,
                final int line,
                final int col) {
            unused();
        }

        @Override
        public void handleOpenElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col) {
            unused();
        }

        @Override
        public void handleOpenElementEnd(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col) {
            unused();
        }

        @Override
        public void handleCloseElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col) {
            unused();
        }

        @Override
        public void handleCloseElementEnd(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col) {
            unused();
        }

        @Override
        public void handleAttribute(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int nameLine,
                final int nameCol,
                final int operatorOffset,
                final int operatorLen,
                final int operatorLine,
                final int operatorCol,
                final int valueContentOffset,
                final int valueContentLen,
                final int valueOuterOffset,
                final int valueOuterLen,
                final int valueLine,
                final int valueCol)
                throws TextParseException {
            callCount++;
            if (calls.length() > 0) {
                calls.append('|');
            }
            calls.append("A:")
                    .append(nameOffset).append(':').append(nameLen).append(':')
                    .append(nameLine).append(':').append(nameCol).append(':')
                    .append(operatorOffset).append(':').append(operatorLen).append(':')
                    .append(operatorLine).append(':').append(operatorCol).append(':')
                    .append(valueContentOffset).append(':').append(valueContentLen).append(':')
                    .append(valueOuterOffset).append(':').append(valueOuterLen).append(':')
                    .append(valueLine).append(':').append(valueCol);

            if (mode == Mode.MUTATE_FUTURE && callCount == 1 && buffer.length > 4) {
                buffer[4] = '=';
            }
            if ((mode == Mode.CHECKED_FIRST && callCount == 1)
                    || (mode == Mode.CHECKED_SECOND && callCount == 2)) {
                throw new TextParseException("handler", 41, 43);
            }
            if (mode == Mode.RUNTIME_FIRST && callCount == 1) {
                throw new IllegalStateException("runtime");
            }
        }
    }
}
