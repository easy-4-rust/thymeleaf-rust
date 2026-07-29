package org.thymeleaf.templateparser.text;

import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.Arrays;

/**
 * 从固定 Thymeleaf Java 源码导出 CommentProcessorTextHandler 的注释解包和文本过滤语义。
 */
public final class CommentProcessorTextHandlerGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final Method COMPUTE_FILTER_OFFSET;

    static {
        try {
            COMPUTE_FILTER_OFFSET = CommentProcessorTextHandler.class.getDeclaredMethod(
                    "computeFilterOffset", char[].class, int.class, int.class, int[].class);
            COMPUTE_FILTER_OFFSET.setAccessible(true);
        } catch (final ReflectiveOperationException exception) {
            throw new ExceptionInInitializerError(exception);
        }
    }

    private CommentProcessorTextHandlerGolden() {
    }

    public static void main(final String[] args) throws Exception {
        emit("baseline", BASELINE);
        initialAndInheritedCases();
        ordinaryCommentCases();
        elementCommentCases();
        expressionCases();
        filterDelimiterCases();
        filterFlushCases();
        failureOrderingCases();
        bufferGrowthCases();
        computeCases();
        arraycopyCases();
        invalidCases();
    }

    private static void initialAndInheritedCases() throws Exception {
        final RecordingHandler next = new RecordingHandler();
        final CommentProcessorTextHandler handler = new CommentProcessorTextHandler(true, next);
        emit("initial.state", state(handler));
        handler.handleDocumentStart(1L, 2, 3);
        handler.handleText("plain".toCharArray(), 0, 5, 4, 5);
        handler.handleStandaloneElementEnd("s".toCharArray(), 0, 1, true, 6, 7);
        handler.handleOpenElementEnd("o".toCharArray(), 0, 1, 8, 9);
        handler.handleCloseElementEnd("c".toCharArray(), 0, 1, 10, 11);
        handler.handleAttribute(
                "a".toCharArray(), 0, 1, 12, 13,
                -1, 0, 12, 14, -1, 0, -1, 0, 12, 15);
        emit("inherited.events", next.events);
        emit("inherited.state", state(handler));
    }

    private static void ordinaryCommentCases() throws Exception {
        final String[] comments = {"x", "[x]", "[#x", "[?x]", "[#x]tail", ""};
        for (final boolean standard : new boolean[]{false, true}) {
            final RecordingHandler next = new RecordingHandler();
            final CommentProcessorTextHandler handler = new CommentProcessorTextHandler(standard, next);
            for (final String content : comments) {
                comment(handler, content, 20, 30);
            }
            emit("ordinary." + standard + ".events", next.events);
            emit("ordinary." + standard + ".state", state(handler));
        }

        final RecordingHandler shortNext = new RecordingHandler();
        final CommentProcessorTextHandler shortHandler =
                new CommentProcessorTextHandler(true, shortNext);
        shortHandler.handleComment(null, Integer.MIN_VALUE, 2, -7, 9, 1, 2);
        emit("ordinary.shortNull.events", shortNext.events);
    }

    private static void elementCommentCases() throws Exception {
        final RecordingHandler next = new RecordingHandler();
        final CommentProcessorTextHandler handler = new CommentProcessorTextHandler(true, next);
        comment(handler, "[#root]", 7, 9);
        comment(handler, "[#single/]", 11, 13);
        comment(handler, "[/root]", 17, 19);
        comment(handler, "[#item a='b']", 23, 29);
        emit("element.events", next.events);
        emit("element.state", state(handler));

        final RecordingHandler mutatingNext = new RecordingHandler();
        mutatingNext.mutateFirstUnit = true;
        final CommentProcessorTextHandler mutating =
                new CommentProcessorTextHandler(true, mutatingNext);
        comment(mutating, "[#x]", 1, 1);
        emit("element.mutation.events", mutatingNext.events);

        final RecordingHandler failedNext = new RecordingHandler();
        failedNext.failEvent = "openStart";
        final CommentProcessorTextHandler failed =
                new CommentProcessorTextHandler(true, failedNext);
        emitThrowable("element.checked", () -> comment(failed, "[#x]", 31, 32));
        emit("element.checked.state", state(failed));
    }

    private static void expressionCases() throws Exception {
        for (final boolean standard : new boolean[]{false, true}) {
            final RecordingHandler next = new RecordingHandler();
            final CommentProcessorTextHandler handler = new CommentProcessorTextHandler(standard, next);
            comment(handler, "[(${x})]", 41, Integer.MAX_VALUE);
            comment(handler, "[[${y}]]", 43, 44);
            emit("expression." + standard + ".events", next.events);
            emit("expression." + standard + ".state", state(handler));
        }

        final RecordingHandler failedNext = new RecordingHandler();
        failedNext.failEvent = "text";
        final CommentProcessorTextHandler failed =
                new CommentProcessorTextHandler(true, failedNext);
        emitThrowable("expression.checked", () -> comment(failed, "[(${x})]", 45, 46));
        emit("expression.checked.state", state(failed));
    }

    private static void filterDelimiterCases() throws Exception {
        final String[] texts = {
                "abc;rest",
                "abc,rest",
                "abc)rest",
                "abc}rest",
                "abc]rest",
                "abc\nrest",
                "abc//rest",
                "{a;b};rest",
                "[a,b],rest",
                "'a;b';rest",
                "\"a,b\",rest",
                "'a\\';b';rest",
                "all-filtered"
        };
        for (int index = 0; index < texts.length; index++) {
            final RecordingHandler next = new RecordingHandler();
            final CommentProcessorTextHandler handler = new CommentProcessorTextHandler(true, next);
            comment(handler, "[(${x})]", 1, 2);
            final char[] text = texts[index].toCharArray();
            handler.handleText(text, 0, text.length, 50 + index, 60 + index);
            handler.handleDocumentEnd(7L, 8L, 70, 80);
            emit("delimiter." + index + ".events", next.events);
            emit("delimiter." + index + ".state", state(handler));
        }
    }

    private static void filterFlushCases() throws Exception {
        final String[] triggers = {"documentEnd", "comment", "standaloneStart", "openStart", "closeStart"};
        for (final String trigger : triggers) {
            final RecordingHandler next = new RecordingHandler();
            final CommentProcessorTextHandler handler = new CommentProcessorTextHandler(true, next);
            comment(handler, "[(${x})]", 1, 2);
            handler.handleText("abc;rest".toCharArray(), 0, 8, 10, 20);
            fireTrigger(handler, trigger);
            emit("flush." + trigger + ".events", next.events);
            emit("flush." + trigger + ".state", state(handler));
        }

        final RecordingHandler delayedNext = new RecordingHandler();
        final CommentProcessorTextHandler delayed =
                new CommentProcessorTextHandler(true, delayedNext);
        comment(delayed, "[(${x})]", 1, 2);
        delayed.handleText("abc;rest".toCharArray(), 0, 8, 10, 20);
        delayed.handleOpenElementEnd("x".toCharArray(), 0, 1, 30, 31);
        delayed.handleAttribute(
                "a".toCharArray(), 0, 1, 32, 33,
                -1, 0, 32, 34, -1, 0, -1, 0, 32, 35);
        emit("flush.nonTriggers.before", delayedNext.events);
        emit("flush.nonTriggers.state", state(delayed));
        delayed.handleDocumentEnd(1, 2, 3, 4);
        emit("flush.nonTriggers.after", delayedNext.events);

        final RecordingHandler emptyNext = new RecordingHandler();
        final CommentProcessorTextHandler empty =
                new CommentProcessorTextHandler(true, emptyNext);
        comment(empty, "[(${x})]", 1, 2);
        empty.handleText(new char[0], 0, 0, 90, 91);
        empty.handleDocumentEnd(1, 2, 3, 4);
        emit("flush.empty.events", emptyNext.events);
        emit("flush.empty.state", state(empty));

        final RecordingHandler chunksNext = new RecordingHandler();
        final CommentProcessorTextHandler chunks =
                new CommentProcessorTextHandler(true, chunksNext);
        comment(chunks, "[(${x})]", 1, 2);
        chunks.handleText("ab".toCharArray(), 0, 2, 100, 101);
        chunks.handleText("c;rest".toCharArray(), 0, 6, 200, 201);
        chunks.handleDocumentEnd(1, 2, 3, 4);
        emit("flush.chunks.events", chunksNext.events);
        emit("flush.chunks.state", state(chunks));
    }

    private static void failureOrderingCases() throws Exception {
        final RecordingHandler next = new RecordingHandler();
        final CommentProcessorTextHandler handler = new CommentProcessorTextHandler(true, next);
        comment(handler, "[(${x})]", 1, 2);
        handler.handleText("abc;rest".toCharArray(), 0, 8, 10, 20);
        next.failEvent = "text";
        emitThrowable("failure.flush1", () -> handler.handleDocumentEnd(1, 2, 3, 4));
        emit("failure.flush1.state", state(handler));
        emitThrowable("failure.flush2", () -> handler.handleDocumentEnd(1, 2, 3, 4));
        emit("failure.flush2.state", state(handler));
        next.failEvent = null;
        handler.handleDocumentEnd(1, 2, 3, 4);
        emit("failure.recovered.events", next.events);
        emit("failure.recovered.state", state(handler));

        final RecordingHandler startNext = new RecordingHandler();
        final CommentProcessorTextHandler start =
                new CommentProcessorTextHandler(true, startNext);
        comment(start, "[(${x})]", 1, 2);
        start.handleText("abc;rest".toCharArray(), 0, 8, 10, 20);
        startNext.failEvent = "openStart";
        emitThrowable(
                "failure.afterFlush",
                () -> start.handleOpenElementStart("x".toCharArray(), 0, 1, 5, 6));
        emit("failure.afterFlush.state", state(start));
    }

    private static void bufferGrowthCases() throws Exception {
        final RecordingHandler next = new RecordingHandler();
        final CommentProcessorTextHandler handler = new CommentProcessorTextHandler(true, next);
        comment(handler, "[(${x})]", 1, 2);
        final char[] first = repeat('a', 200);
        final char[] second = repeat('b', 100);
        final char[] third = repeat('c', 500);
        handler.handleText(first, 0, first.length, 1, 2);
        first[0] = 'z';
        emit("growth.200", state(handler));
        handler.handleText(second, 0, second.length, 3, 4);
        emit("growth.300", state(handler));
        handler.handleText(third, 0, third.length, 5, 6);
        emit("growth.800", state(handler));
        emit("growth.copyHead", hex(Arrays.copyOf((char[]) field(handler, "filteredTextBuffer"), 3)));
    }

    private static void computeCases() {
        final String[] values = {
                "", "abc", ";x", "\nx", "//x", "{a;b};x", "[a,b],x",
                "'a;b';x", "\"a,b\",x", "'a\\';b';x", "{{a}b}c]x", "[[a]b]c)x"
        };
        for (int index = 0; index < values.length; index++) {
            final char[] chars = values[index].toCharArray();
            final int[] locator = {10, 20};
            emitCompute("compute." + index, chars, 0, chars.length, locator);
        }
        emitCompute("compute.emptyNull", null, 7, 7, null);
        emitCompute("compute.reverseNull", null, 8, 7, null);
    }

    private static void arraycopyCases() {
        emitThrowable("arraycopy.nullSource", () -> System.arraycopy(null, 0, new char[1], 0, 1));
        emitThrowable("arraycopy.negativeLen", () -> System.arraycopy(new char[1], 0, new char[1], 0, -1));
        emitThrowable("arraycopy.sourceIndex", () -> System.arraycopy(new char[1], -1, new char[1], 0, 1));
        emitThrowable("arraycopy.destinationIndex", () -> System.arraycopy(new char[1], 0, new char[1], -1, 1));
        emitThrowable("arraycopy.lastSource", () -> System.arraycopy(new char[1], 0, new char[2], 0, 2));
        emitThrowable("arraycopy.lastDestination", () -> System.arraycopy(new char[2], 0, new char[1], 0, 2));
    }

    private static void invalidCases() throws Exception {
        final CommentProcessorTextHandler predicate =
                new CommentProcessorTextHandler(true, new RecordingHandler());
        emitThrowable("invalid.comment.null", () -> predicate.handleComment(null, 0, 3, 0, 0, 1, 1));
        emitThrowable(
                "invalid.comment.negativeOffset",
                () -> predicate.handleComment("abc".toCharArray(), -1, 3, 0, 3, 1, 1));
        emitThrowable(
                "invalid.comment.overflow",
                () -> predicate.handleComment("abc".toCharArray(), Integer.MAX_VALUE, 3, 0, 3, 1, 1));

        final CommentProcessorTextHandler filter =
                new CommentProcessorTextHandler(true, new RecordingHandler());
        comment(filter, "[(${x})]", 1, 2);
        emitThrowable("invalid.filter.null", () -> filter.handleText(null, 0, 1, 1, 1));
        emit("invalid.filter.null.state", state(filter));

        final CommentProcessorTextHandler negative =
                new CommentProcessorTextHandler(true, new RecordingHandler());
        comment(negative, "[(${x})]", 1, 2);
        emitThrowable(
                "invalid.filter.negativeLen",
                () -> negative.handleText("a".toCharArray(), 0, -1, 1, 1));
        emit("invalid.filter.negativeLen.state", state(negative));

        emitCompute("invalid.compute.nullBuffer", null, 0, 1, new int[]{1, 2});
        emitCompute("invalid.compute.negativeOffset", "a".toCharArray(), -1, 1, new int[]{1, 2});
        emitCompute("invalid.compute.nullLocatorTerminator", ";".toCharArray(), 0, 1, null);
        emitCompute("invalid.compute.nullLocatorText", "a".toCharArray(), 0, 1, null);
        emitCompute("invalid.compute.shortLocatorLf", "\n".toCharArray(), 0, 1, new int[]{Integer.MAX_VALUE});

        final CommentProcessorTextHandler nullNext = new CommentProcessorTextHandler(true, null);
        emitThrowable("invalid.next.expression", () -> comment(nullNext, "[(${x})]", 1, 2));
        final CommentProcessorTextHandler nullNormal = new CommentProcessorTextHandler(true, null);
        emitThrowable("invalid.next.normal", () -> comment(nullNormal, "normal", 1, 2));
        final CommentProcessorTextHandler nullElement = new CommentProcessorTextHandler(true, null);
        emitThrowable("invalid.next.element", () -> comment(nullElement, "[#x]", 1, 2));
    }

    private static void fireTrigger(
            final CommentProcessorTextHandler handler,
            final String trigger) throws TextParseException {
        final char[] name = "x".toCharArray();
        switch (trigger) {
            case "documentEnd":
                handler.handleDocumentEnd(1, 2, 3, 4);
                return;
            case "comment":
                comment(handler, "normal", 30, 31);
                return;
            case "standaloneStart":
                handler.handleStandaloneElementStart(name, 0, 1, true, 30, 31);
                return;
            case "openStart":
                handler.handleOpenElementStart(name, 0, 1, 30, 31);
                return;
            case "closeStart":
                handler.handleCloseElementStart(name, 0, 1, 30, 31);
                return;
            default:
                throw new AssertionError(trigger);
        }
    }

    private static void comment(
            final CommentProcessorTextHandler handler,
            final String content,
            final int line,
            final int col) throws TextParseException {
        final char[] buffer = ("/*" + content + "*/").toCharArray();
        handler.handleComment(buffer, 2, content.length(), 0, buffer.length, line, col);
    }

    private static String state(final CommentProcessorTextHandler handler) throws Exception {
        final char[] buffer = (char[]) field(handler, "filteredTextBuffer");
        final int size = (Integer) field(handler, "filteredTextSize");
        final int[] locator = (int[]) field(handler, "filteredTextLocator");
        return "standard=" + field(handler, "standardDialectPresent")
                + ";filter=" + field(handler, "filterTexts")
                + ";size=" + size
                + ";buffer=" + (buffer == null ? "null" : buffer.length + ":" + hex(Arrays.copyOf(buffer, Math.min(size, 12))))
                + ";locator=" + (locator == null ? "null" : Arrays.toString(locator));
    }

    private static Object field(final Object target, final String name) throws Exception {
        final Field field = target.getClass().getDeclaredField(name);
        field.setAccessible(true);
        return field.get(target);
    }

    private static void emitCompute(
            final String key,
            final char[] buffer,
            final int offset,
            final int maxi,
            final int[] locator) {
        try {
            final Object result = COMPUTE_FILTER_OFFSET.invoke(null, buffer, offset, maxi, locator);
            emit(key, "offset=" + result + ";locator=" + (locator == null ? "null" : Arrays.toString(locator)));
        } catch (final InvocationTargetException exception) {
            emit(key, describeThrowable(exception.getCause())
                    + ";locator=" + (locator == null ? "null" : Arrays.toString(locator)));
        } catch (final ReflectiveOperationException exception) {
            throw new AssertionError(exception);
        }
    }

    private static char[] repeat(final char value, final int count) {
        final char[] result = new char[count];
        Arrays.fill(result, value);
        return result;
    }

    private static void emitThrowable(final String key, final ThrowingRunnable runnable) {
        try {
            runnable.run();
            emit(key, "NO_ERROR");
        } catch (final Throwable throwable) {
            emit(key, describeThrowable(throwable));
        }
    }

    private static String describeThrowable(final Throwable throwable) {
        String location = "";
        if (throwable instanceof TextParseException) {
            final TextParseException parse = (TextParseException) throwable;
            location = ";line=" + parse.getLine() + ";col=" + parse.getCol();
        }
        return throwable.getClass().getName()
                + ";message=" + utf16Hex(throwable.getMessage())
                + location;
    }

    private static String hex(final char[] value) {
        if (value == null) {
            return "null";
        }
        final StringBuilder result = new StringBuilder();
        for (int index = 0; index < value.length; index++) {
            if (index > 0) {
                result.append(',');
            }
            result.append(String.format("%04x", (int) value[index]));
        }
        return result.toString();
    }

    private static String utf16Hex(final String value) {
        return value == null ? "null" : hex(value.toCharArray());
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + value);
    }

    @FunctionalInterface
    private interface ThrowingRunnable {
        void run() throws Throwable;
    }

    private static final class RecordingHandler extends AbstractTextHandler {

        private final StringBuilder events = new StringBuilder();
        private String failEvent;
        private boolean mutateFirstUnit;

        private void record(
                final String event,
                final char[] buffer,
                final int offset,
                final int len,
                final String arguments) throws TextParseException {
            if (this.events.length() > 0) {
                this.events.append('|');
            }
            this.events.append(event).append('(').append(arguments).append(")@");
            if (buffer == null) {
                this.events.append("null");
            } else if (offset >= 0 && len >= 0 && offset + len <= buffer.length) {
                this.events.append(hex(Arrays.copyOfRange(buffer, offset, offset + len)));
            } else {
                this.events.append("range(").append(offset).append(',').append(len).append(')');
            }
            if (this.mutateFirstUnit && buffer != null && len > 0) {
                buffer[offset] = '!';
            }
            if (event.equals(this.failEvent)) {
                throw new TextParseException("downstream-" + event, 71, 72);
            }
        }

        @Override
        public void handleDocumentStart(final long startTimeNanos, final int line, final int col)
                throws TextParseException {
            record("documentStart", null, 0, 0, startTimeNanos + "," + line + "," + col);
        }

        @Override
        public void handleDocumentEnd(
                final long endTimeNanos, final long totalTimeNanos, final int line, final int col)
                throws TextParseException {
            record("documentEnd", null, 0, 0, endTimeNanos + "," + totalTimeNanos + "," + line + "," + col);
        }

        @Override
        public void handleText(
                final char[] buffer, final int offset, final int len, final int line, final int col)
                throws TextParseException {
            record("text", buffer, offset, len, offset + "," + len + "," + line + "," + col);
        }

        @Override
        public void handleStandaloneElementStart(
                final char[] buffer, final int nameOffset, final int nameLen, final boolean minimized,
                final int line, final int col) throws TextParseException {
            record("standaloneStart", buffer, nameOffset, nameLen,
                    nameOffset + "," + nameLen + "," + minimized + "," + line + "," + col);
        }

        @Override
        public void handleStandaloneElementEnd(
                final char[] buffer, final int nameOffset, final int nameLen, final boolean minimized,
                final int line, final int col) throws TextParseException {
            record("standaloneEnd", buffer, nameOffset, nameLen,
                    nameOffset + "," + nameLen + "," + minimized + "," + line + "," + col);
        }

        @Override
        public void handleOpenElementStart(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) throws TextParseException {
            record("openStart", buffer, nameOffset, nameLen,
                    nameOffset + "," + nameLen + "," + line + "," + col);
        }

        @Override
        public void handleOpenElementEnd(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) throws TextParseException {
            record("openEnd", buffer, nameOffset, nameLen,
                    nameOffset + "," + nameLen + "," + line + "," + col);
        }

        @Override
        public void handleCloseElementStart(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) throws TextParseException {
            record("closeStart", buffer, nameOffset, nameLen,
                    nameOffset + "," + nameLen + "," + line + "," + col);
        }

        @Override
        public void handleCloseElementEnd(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) throws TextParseException {
            record("closeEnd", buffer, nameOffset, nameLen,
                    nameOffset + "," + nameLen + "," + line + "," + col);
        }

        @Override
        public void handleAttribute(
                final char[] buffer,
                final int nameOffset, final int nameLen, final int nameLine, final int nameCol,
                final int operatorOffset, final int operatorLen, final int operatorLine, final int operatorCol,
                final int valueContentOffset, final int valueContentLen,
                final int valueOuterOffset, final int valueOuterLen,
                final int valueLine, final int valueCol) throws TextParseException {
            record("attribute", buffer, nameOffset, nameLen,
                    nameOffset + "," + nameLen + "," + nameLine + "," + nameCol + ","
                            + operatorOffset + "," + operatorLen + "," + operatorLine + "," + operatorCol + ","
                            + valueContentOffset + "," + valueContentLen + ","
                            + valueOuterOffset + "," + valueOuterLen + "," + valueLine + "," + valueCol);
        }
    }
}
