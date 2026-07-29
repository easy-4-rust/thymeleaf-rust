package org.thymeleaf.templateparser.text;

/**
 * 从固定 Thymeleaf Java 源码导出文本元素解析语义。
 */
public final class TextParsingElementGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final long FNV_OFFSET = 0xcbf29ce484222325L;
    private static final long FNV_PRIME = 0x100000001b3L;

    private TextParsingElementGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        predicateCases();
        standaloneCases();
        openCases();
        closeCases();
        handlerCases();
        runtimeCases();
        exhaustiveCases();
    }

    private static void predicateCases() {
        emitPredicates("predicate.open", "[#name]", 0, 7);
        emitPredicates("predicate.close", "[/name]", 0, 7);
        emitPredicates("predicate.noNameOpen", "[#]", 0, 3);
        emitPredicates("predicate.noNameStandalone", "[#/]", 0, 4);
        emitPredicates("predicate.whitespaceName", "[# a=b]", 0, 7);
        emitPredicates("predicate.forbiddenDash", "[#-x]", 0, 5);
        emitPredicates("predicate.forbiddenBang", "[#!x]", 0, 5);
        emitPredicates("predicate.forbiddenSlash", "[#/x]", 0, 5);
        emitPredicates("predicate.forbiddenQuestion", "[#?x]", 0, 5);
        emitPredicates("predicate.forbiddenBracket", "[#[x]", 0, 5);
        emitPredicates("predicate.forbiddenBrace", "[#{x]", 0, 5);
        emitPredicates("predicate.nulName", "[#\u0000]", 0, 4);
        emitPredicates("predicate.surrogateName", "[#\uD800]", 0, 4);
        emitPredicates("predicate.openEnd", "]", 0, 1);
        emitPredicates("predicate.standaloneEnd", "/]", 0, 2);
        emitPredicates("predicate.badStandaloneEnd", "/x", 0, 2);
        emitPredicates("predicate.emptyEnd", "", 0, 0);
        emitPredicateAction("predicate.nullShort", null, 0, 2);
        emitPredicateAction("predicate.nullLong", null, 0, 3);
        emitPredicateAction("predicate.negativeOffset", "[#x]", -1, 4);
        emitPredicateAction("predicate.pastEnd", "[#x]", 4, 8);
        emitPredicateAction("predicate.wrappedMax", "[#x]", Integer.MAX_VALUE, Integer.MIN_VALUE);
    }

    private static void standaloneCases() {
        emitParse("standalone.name", Kind.STANDALONE, "[#x/]", 0, 5, 1, 1, Mode.NORMAL);
        emitParse("standalone.noName", Kind.STANDALONE, "[#/]", 0, 4, 2, 3, Mode.NORMAL);
        emitParse("standalone.attributes", Kind.STANDALONE, "[#x a=b c='d e'/]", 0, 17, 4, 5, Mode.NORMAL);
        emitParse("standalone.noNameAttributes", Kind.STANDALONE, "[# a=b/]", 0, 8, 7, 11, Mode.NORMAL);
        emitParse("standalone.multiline", Kind.STANDALONE, "[#x\n a=\"b\nc\"/]", 0, 14, 10, 20, Mode.NORMAL);
        emitParse("standalone.quotedEnd", Kind.STANDALONE, "[#x a=\"/\"/]", 0, 11, 1, 1, Mode.NORMAL);
        emitParse("standalone.embedded", Kind.STANDALONE, "zz[#x a=b/]yy", 2, 9, 3, 9, Mode.NORMAL);
        emitParse("standalone.invalidShort", Kind.STANDALONE, "[#]", 0, 3, 1, 1, Mode.NORMAL);
        emitParse("standalone.invalidPrefix", Kind.STANDALONE, "[/x/]", 0, 5, 1, 1, Mode.NORMAL);
        emitParse("standalone.invalidEnd", Kind.STANDALONE, "[#x]", 0, 4, 1, 1, Mode.NORMAL);
        emitParse("standalone.invalidName", Kind.STANDALONE, "[#-x/]", 0, 6, 1, 1, Mode.NORMAL);
    }

    private static void openCases() {
        emitParse("open.name", Kind.OPEN, "[#x]", 0, 4, 1, 1, Mode.NORMAL);
        emitParse("open.noName", Kind.OPEN, "[#]", 0, 3, 2, 3, Mode.NORMAL);
        emitParse("open.attributes", Kind.OPEN, "[#x a=b c=\"d e\"]", 0, 16, 4, 5, Mode.NORMAL);
        emitParse("open.noNameAttributes", Kind.OPEN, "[# a=b]", 0, 7, 7, 11, Mode.NORMAL);
        emitParse("open.trailingWhitespace", Kind.OPEN, "[#x \t]", 0, 6, 1, 1, Mode.NORMAL);
        emitParse("open.multiline", Kind.OPEN, "[#x\n a=\"b\nc\"]", 0, 13, 10, 20, Mode.NORMAL);
        emitParse("open.quotedEnd", Kind.OPEN, "[#x a=\"]\"]", 0, 10, 1, 1, Mode.NORMAL);
        emitParse("open.doubleQuoteInName", Kind.OPEN, "[#x\" y\" a=b]", 0, 12, 1, 1, Mode.NORMAL);
        emitParse("open.singleQuoteInName", Kind.OPEN, "[#x' y' a=b]", 0, 12, 1, 1, Mode.NORMAL);
        emitParse("open.innerStructureEnd", Kind.OPEN, "[#x a=b]c]", 0, 10, 1, 1, Mode.NORMAL);
        emitParse("open.nulSurrogate", Kind.OPEN, "[#\uD800 a=\u0000]", 0, 8, 1, 1, Mode.NORMAL);
        emitParse("open.lineOverflow", Kind.OPEN, "[#x\n a=b]", 0, 9, Integer.MAX_VALUE, Integer.MAX_VALUE, Mode.NORMAL);
        emitParse("open.invalidShort", Kind.OPEN, "[]", 0, 2, 1, 1, Mode.NORMAL);
        emitParse("open.invalidPrefix", Kind.OPEN, "[/x]", 0, 4, 1, 1, Mode.NORMAL);
        emitParse("open.invalidEnd", Kind.OPEN, "[#x/", 0, 4, 1, 1, Mode.NORMAL);
        emitParse("open.invalidName", Kind.OPEN, "[#{x]", 0, 5, 1, 1, Mode.NORMAL);
    }

    private static void closeCases() {
        emitParse("close.name", Kind.CLOSE, "[/x]", 0, 4, 1, 1, Mode.NORMAL);
        emitParse("close.noName", Kind.CLOSE, "[/]", 0, 3, 2, 3, Mode.NORMAL);
        emitParse("close.trailingWhitespace", Kind.CLOSE, "[/x \t]", 0, 6, 4, 5, Mode.NORMAL);
        emitParse("close.noNameWhitespace", Kind.CLOSE, "[/ ]", 0, 4, 7, 11, Mode.NORMAL);
        emitParse("close.multiline", Kind.CLOSE, "[/x\n \t]", 0, 7, 10, 20, Mode.NORMAL);
        emitParse("close.attributesRejected", Kind.CLOSE, "[/x a=b]", 0, 8, 3, 9, Mode.NORMAL);
        emitParse("close.noNameAttributeRejected", Kind.CLOSE, "[/ a=b]", 0, 7, 3, 9, Mode.NORMAL);
        emitParse("close.invalidShort", Kind.CLOSE, "[]", 0, 2, 1, 1, Mode.NORMAL);
        emitParse("close.invalidPrefix", Kind.CLOSE, "[#x]", 0, 4, 1, 1, Mode.NORMAL);
        emitParse("close.invalidEnd", Kind.CLOSE, "[/x/", 0, 4, 1, 1, Mode.NORMAL);
        emitParse("close.invalidName", Kind.CLOSE, "[/?x]", 0, 5, 1, 1, Mode.NORMAL);
    }

    private static void handlerCases() {
        for (final Kind kind : Kind.values()) {
            final String prefix = "handler." + kind.name().toLowerCase();
            final String text =
                    kind == Kind.STANDALONE ? "[#x a=b c=d/]" :
                    kind == Kind.OPEN ? "[#x a=b c=d]" : "[/x ]";
            emitParse(prefix + ".checkedStart", kind, text, 0, text.length(), 1, 1, Mode.CHECKED_START);
            emitParse(prefix + ".checkedEnd", kind, text, 0, text.length(), 1, 1, Mode.CHECKED_END);
            emitParse(prefix + ".runtimeStart", kind, text, 0, text.length(), 1, 1, Mode.RUNTIME_START);
            emitParse(prefix + ".runtimeEnd", kind, text, 0, text.length(), 1, 1, Mode.RUNTIME_END);
            emitNullHandler(prefix + ".null", kind, text);
        }
        emitParse(
                "handler.standalone.checkedAttribute",
                Kind.STANDALONE,
                "[#x a=b c=d/]",
                0,
                13,
                1,
                1,
                Mode.CHECKED_ATTRIBUTE);
        emitParse(
                "handler.open.checkedAttribute",
                Kind.OPEN,
                "[#x a=b c=d]",
                0,
                12,
                1,
                1,
                Mode.CHECKED_ATTRIBUTE);
        emitParse(
                "handler.open.runtimeAttribute",
                Kind.OPEN,
                "[#x a=b c=d]",
                0,
                12,
                1,
                1,
                Mode.RUNTIME_ATTRIBUTE);
        emitParse(
                "handler.open.mutateStart",
                Kind.OPEN,
                "[#x a=b c=d]",
                0,
                12,
                1,
                1,
                Mode.MUTATE_START);
        emitParse(
                "handler.open.mutateAttribute",
                Kind.OPEN,
                "[#x a=b c=d]",
                0,
                12,
                1,
                1,
                Mode.MUTATE_ATTRIBUTE);
    }

    private static void runtimeCases() {
        emitParse("runtime.standaloneNullShort", Kind.STANDALONE, null, 0, 0, 1, 1, Mode.NORMAL);
        emitParse("runtime.standaloneNullLong", Kind.STANDALONE, null, 0, 4, 1, 1, Mode.NORMAL);
        emitParse("runtime.openNullShort", Kind.OPEN, null, 0, 0, 1, 1, Mode.NORMAL);
        emitParse("runtime.openNullLong", Kind.OPEN, null, 0, 3, 1, 1, Mode.NORMAL);
        emitParse("runtime.closeNullShort", Kind.CLOSE, null, 0, 0, 1, 1, Mode.NORMAL);
        emitParse("runtime.closeNullLong", Kind.CLOSE, null, 0, 3, 1, 1, Mode.NORMAL);
        emitParse("runtime.negativeOffset", Kind.OPEN, "[#x]", -1, 4, 1, 1, Mode.NORMAL);
        emitParse("runtime.offsetPastEnd", Kind.OPEN, "[#x]", 4, 3, 1, 1, Mode.NORMAL);
        emitParse("runtime.invalidStringNegativeOffset", Kind.OPEN, "x", -1, 0, 1, 1, Mode.NORMAL);
        emitParse("runtime.invalidStringPastEnd", Kind.OPEN, "x", 2, 0, 1, 1, Mode.NORMAL);
        emitParse("runtime.wrappedRange", Kind.OPEN, "[#x]", 1, Integer.MAX_VALUE, 1, 1, Mode.NORMAL);
        emitParse("runtime.scanPastEnd", Kind.OPEN, "[#x]", 0, 5, 1, 1, Mode.NORMAL);
        emitParse("runtime.endPastEnd", Kind.OPEN, "[#x", 0, 4, 1, 1, Mode.NORMAL);
        emitParse("runtime.closeEndPastEnd", Kind.CLOSE, "[/x", 0, 4, 1, 1, Mode.NORMAL);
    }

    private static void exhaustiveCases() {
        long nameHash = FNV_OFFSET;
        for (int unit = Character.MIN_VALUE; unit <= Character.MAX_VALUE; unit++) {
            final char[] open = {'[', '#', (char) unit, ']'};
            final char[] close = {'[', '/', (char) unit, ']'};
            nameHash = mixString(nameHash, predicates(open, 0, open.length));
            nameHash = mixString(nameHash, predicates(close, 0, close.length));
        }
        emit("exhaustive.nameUnitHash", hex(nameHash));

        long predicateRangeHash = FNV_OFFSET;
        final char[] predicateBuffer = "xx[#a/]yy[/b]zz".toCharArray();
        for (int offset = -2; offset <= predicateBuffer.length + 2; offset++) {
            for (int maxi = -2; maxi <= predicateBuffer.length + 4; maxi++) {
                predicateRangeHash =
                        mixString(predicateRangeHash, predicates(predicateBuffer, offset, maxi));
            }
        }
        emit("exhaustive.predicateRangeHash", hex(predicateRangeHash));

        long grammarHash = FNV_OFFSET;
        final String[] names = {"", "x", "x:y", "-", "{", "\uD800"};
        final String[] attributes = {"", " ", " a=b", "\n a=\"c d\"", " a=", " a b=c"};
        for (final Kind kind : Kind.values()) {
            for (final String name : names) {
                for (final String attributesValue : attributes) {
                    final String text =
                            kind == Kind.STANDALONE
                                    ? "[#" + name + attributesValue + "/]"
                                    : kind == Kind.OPEN
                                    ? "[#" + name + attributesValue + "]"
                                    : "[/" + name + attributesValue + "]";
                    grammarHash = mixString(
                            grammarHash,
                            outcome(
                                    text.toCharArray(),
                                    kind,
                                    0,
                                    text.length(),
                                    -7,
                                    Integer.MAX_VALUE,
                                    Mode.NORMAL));
                }
            }
        }
        emit("exhaustive.grammarHash", hex(grammarHash));

        long parseRangeHash = FNV_OFFSET;
        final char[] rangeBuffer = "xx[#a b=\"c d\"/]yy".toCharArray();
        for (int offset = -2; offset <= rangeBuffer.length + 2; offset++) {
            for (int len = -2; len <= rangeBuffer.length + 4; len++) {
                for (final Kind kind : Kind.values()) {
                    parseRangeHash = mixString(
                            parseRangeHash,
                            outcome(
                                    rangeBuffer.clone(),
                                    kind,
                                    offset,
                                    len,
                                    13,
                                    17,
                                    Mode.NORMAL));
                }
            }
        }
        emit("exhaustive.parseRangeHash", hex(parseRangeHash));
    }

    private static void emitPredicates(
            final String key, final String text, final int offset, final int maxi) {
        emit(key, predicates(text == null ? null : text.toCharArray(), offset, maxi));
    }

    private static void emitPredicateAction(
            final String key, final String text, final int offset, final int maxi) {
        emitPredicates(key, text, offset, maxi);
    }

    private static String predicates(
            final char[] buffer, final int offset, final int maxi) {
        return "O=" + predicate(() -> TextParsingElementUtil.isOpenElementStart(buffer, offset, maxi))
                + ",C=" + predicate(() -> TextParsingElementUtil.isCloseElementStart(buffer, offset, maxi))
                + ",E0=" + predicate(() -> TextParsingElementUtil.isElementEnd(buffer, offset, maxi, false))
                + ",E1=" + predicate(() -> TextParsingElementUtil.isElementEnd(buffer, offset, maxi, true));
    }

    private static String predicate(final BooleanAction action) {
        try {
            return String.valueOf(action.run());
        } catch (final Throwable throwable) {
            return throwable(throwable);
        }
    }

    private static void emitParse(
            final String key,
            final Kind kind,
            final String text,
            final int offset,
            final int len,
            final int line,
            final int col,
            final Mode mode) {
        emit(
                key,
                outcome(
                        text == null ? null : text.toCharArray(),
                        kind,
                        offset,
                        len,
                        line,
                        col,
                        mode));
    }

    private static String outcome(
            final char[] buffer,
            final Kind kind,
            final int offset,
            final int len,
            final int line,
            final int col,
            final Mode mode) {
        final RecordingHandler handler = new RecordingHandler(mode);
        try {
            parse(kind, buffer, offset, len, line, col, handler);
            return "OK:" + handler.calls + ":" + bufferHex(buffer);
        } catch (final Throwable throwable) {
            return throwable(throwable) + ":" + handler.calls + ":" + bufferHex(buffer);
        }
    }

    private static void emitNullHandler(final String key, final Kind kind, final String text) {
        try {
            parse(kind, text.toCharArray(), 0, text.length(), 1, 1, null);
            emit(key, "OK");
        } catch (final Throwable throwable) {
            emit(key, throwable(throwable));
        }
    }

    private static void parse(
            final Kind kind,
            final char[] buffer,
            final int offset,
            final int len,
            final int line,
            final int col,
            final ITextHandler handler)
            throws TextParseException {
        switch (kind) {
            case STANDALONE:
                TextParsingElementUtil.parseStandaloneElement(
                        buffer, offset, len, line, col, handler);
                return;
            case OPEN:
                TextParsingElementUtil.parseOpenElement(
                        buffer, offset, len, line, col, handler);
                return;
            case CLOSE:
                TextParsingElementUtil.parseCloseElement(
                        buffer, offset, len, line, col, handler);
                return;
            default:
                throw new AssertionError(kind);
        }
    }

    private static String throwable(final Throwable throwable) {
        final String base = "ERR:" + throwable.getClass().getName() + ":"
                + toUtf16Hex(String.valueOf(throwable.getMessage()));
        if (throwable instanceof TextParseException) {
            final TextParseException textParseException = (TextParseException) throwable;
            return base + ":" + textParseException.getLine() + ":" + textParseException.getCol();
        }
        return base;
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

    private enum Kind {
        STANDALONE,
        OPEN,
        CLOSE
    }

    private enum Mode {
        NORMAL,
        MUTATE_START,
        MUTATE_ATTRIBUTE,
        CHECKED_START,
        CHECKED_ATTRIBUTE,
        CHECKED_END,
        RUNTIME_START,
        RUNTIME_ATTRIBUTE,
        RUNTIME_END
    }

    @FunctionalInterface
    private interface BooleanAction {
        boolean run();
    }

    private static final class RecordingHandler implements ITextHandler {

        private final StringBuilder calls = new StringBuilder();
        private final Mode mode;
        private int attributeCount;

        private RecordingHandler(final Mode mode) {
            this.mode = mode;
        }

        private void record(final String value) {
            if (calls.length() > 0) {
                calls.append('|');
            }
            calls.append(value);
        }

        private void afterStart(final char[] buffer) throws TextParseException {
            fail(Mode.CHECKED_START, Mode.RUNTIME_START);
            if (mode == Mode.MUTATE_START) {
                mutateNext(buffer, 'a');
            }
        }

        private void afterEnd() throws TextParseException {
            fail(Mode.CHECKED_END, Mode.RUNTIME_END);
        }

        private void fail(final Mode checked, final Mode runtime) throws TextParseException {
            if (mode == checked) {
                throw new TextParseException("handler", 41, 43);
            }
            if (mode == runtime) {
                throw new IllegalStateException("runtime");
            }
        }

        private static void mutateNext(final char[] buffer, final char expected) {
            for (int index = 0; index < buffer.length; index++) {
                if (buffer[index] == expected) {
                    buffer[index] = '=';
                    return;
                }
            }
        }

        @Override
        public void handleDocumentStart(final long startTimeNanos, final int line, final int col) {
            // Not emitted by TextParsingElementUtil.
        }

        @Override
        public void handleDocumentEnd(
                final long endTimeNanos,
                final long totalTimeNanos,
                final int line,
                final int col) {
            // Not emitted by TextParsingElementUtil.
        }

        @Override
        public void handleText(
                final char[] buffer,
                final int offset,
                final int len,
                final int line,
                final int col) {
            // Not emitted by TextParsingElementUtil.
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
            // Not emitted by TextParsingElementUtil.
        }

        @Override
        public void handleStandaloneElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final boolean minimized,
                final int line,
                final int col)
                throws TextParseException {
            record("SS:" + nameOffset + ":" + nameLen + ":" + minimized + ":" + line + ":" + col);
            afterStart(buffer);
        }

        @Override
        public void handleStandaloneElementEnd(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final boolean minimized,
                final int line,
                final int col)
                throws TextParseException {
            record("SE:" + nameOffset + ":" + nameLen + ":" + minimized + ":" + line + ":" + col);
            afterEnd();
        }

        @Override
        public void handleOpenElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col)
                throws TextParseException {
            record("OS:" + nameOffset + ":" + nameLen + ":" + line + ":" + col);
            afterStart(buffer);
        }

        @Override
        public void handleOpenElementEnd(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col)
                throws TextParseException {
            record("OE:" + nameOffset + ":" + nameLen + ":" + line + ":" + col);
            afterEnd();
        }

        @Override
        public void handleCloseElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col)
                throws TextParseException {
            record("CS:" + nameOffset + ":" + nameLen + ":" + line + ":" + col);
            afterStart(buffer);
        }

        @Override
        public void handleCloseElementEnd(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col)
                throws TextParseException {
            record("CE:" + nameOffset + ":" + nameLen + ":" + line + ":" + col);
            afterEnd();
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
            attributeCount++;
            record(
                    "A:" + nameOffset + ":" + nameLen + ":" + nameLine + ":" + nameCol
                            + ":" + operatorOffset + ":" + operatorLen + ":" + operatorLine
                            + ":" + operatorCol + ":" + valueContentOffset + ":"
                            + valueContentLen + ":" + valueOuterOffset + ":" + valueOuterLen
                            + ":" + valueLine + ":" + valueCol);
            fail(Mode.CHECKED_ATTRIBUTE, Mode.RUNTIME_ATTRIBUTE);
            if (mode == Mode.MUTATE_ATTRIBUTE && attributeCount == 1) {
                mutateNext(buffer, 'c');
            }
        }
    }
}
