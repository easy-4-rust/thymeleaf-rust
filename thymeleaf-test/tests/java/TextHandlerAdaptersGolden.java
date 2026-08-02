package org.thymeleaf.templateparser.text;

/**
 * 从固定 Thymeleaf Java 源码导出文本 handler 基类的转发与异常语义。
 */
public final class TextHandlerAdaptersGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private static final String[] EVENTS = {
            "documentStart",
            "documentEnd",
            "text",
            "comment",
            "standaloneStart",
            "standaloneEnd",
            "openStart",
            "openEnd",
            "closeStart",
            "closeEnd",
            "attribute"
    };

    private TextHandlerAdaptersGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        abstractNoOpCases();
        chainedSuccessCases();
        chainedCheckedCases();
        chainedRuntimeCases();
        chainedNullCases();
    }

    private static void abstractNoOpCases() {
        final NoOpProbe handler = new NoOpProbe();
        final char[] buffer = {'a', '\uD800', 'z'};
        for (final String event : EVENTS) {
            invoke(handler, event, null);
            invoke(handler, event, buffer);
        }
        emit("abstract.allEvents", "OK:" + hex(buffer));
    }

    private static void chainedSuccessCases() {
        final RecordingHandler next = new RecordingHandler();
        final ChainedProbe handler = new ChainedProbe(next);
        final char[] buffer = {'a', '\uD800', 'z'};
        for (final String event : EVENTS) {
            invoke(handler, event, buffer);
        }
        emit("chained.identity", handler.exposeNext() == next);
        emit("chained.success.events", next.events.toString());
        emit("chained.success.buffer", hex(buffer));
    }

    private static void chainedCheckedCases() {
        for (final String event : EVENTS) {
            final TextParseException expected =
                    new TextParseException("checked-" + event, 101, 202);
            final RecordingHandler next = new RecordingHandler(event, expected, null);
            final ChainedProbe handler = new ChainedProbe(next);
            final char[] buffer = {'a', 'b'};
            try {
                invokeThrowable(handler, event, buffer);
                emit("chained.checked." + event, "NO_ERROR");
            } catch (final Throwable throwable) {
                emit(
                        "chained.checked." + event,
                        "same=" + (throwable == expected)
                                + ";class=" + throwable.getClass().getName()
                                + ";message=" + utf16Hex(throwable.getMessage())
                                + ";line=" + ((TextParseException) throwable).getLine()
                                + ";col=" + ((TextParseException) throwable).getCol()
                                + ";buffer=" + hex(buffer));
            }
        }
    }

    private static void chainedRuntimeCases() {
        for (final String event : EVENTS) {
            final IllegalStateException expected =
                    new IllegalStateException("runtime-" + event);
            final RecordingHandler next = new RecordingHandler(event, null, expected);
            final ChainedProbe handler = new ChainedProbe(next);
            final char[] buffer = {'a', 'b'};
            try {
                invokeThrowable(handler, event, buffer);
                emit("chained.runtime." + event, "NO_ERROR");
            } catch (final Throwable throwable) {
                emit(
                        "chained.runtime." + event,
                        "same=" + (throwable == expected)
                                + ";class=" + throwable.getClass().getName()
                                + ";message=" + utf16Hex(throwable.getMessage())
                                + ";buffer=" + hex(buffer));
            }
        }
    }

    private static void chainedNullCases() {
        final ChainedProbe handler = new ChainedProbe(null);
        emit("chained.null.identity", handler.exposeNext() == null);
        for (final String event : EVENTS) {
            final char[] buffer = {'a', 'b'};
            try {
                invokeThrowable(handler, event, buffer);
                emit("chained.null." + event, "NO_ERROR");
            } catch (final Throwable throwable) {
                emit(
                        "chained.null." + event,
                        "class=" + throwable.getClass().getName()
                                + ";message=" + utf16Hex(throwable.getMessage())
                                + ";buffer=" + hex(buffer));
            }
        }
    }

    private static void invoke(
            final ITextHandler handler, final String event, final char[] buffer) {
        try {
            invokeThrowable(handler, event, buffer);
        } catch (final Throwable throwable) {
            throw new AssertionError(event, throwable);
        }
    }

    private static void invokeThrowable(
            final ITextHandler handler, final String event, final char[] buffer)
            throws Throwable {
        switch (event) {
            case "documentStart":
                handler.handleDocumentStart(Long.MIN_VALUE, Integer.MIN_VALUE, Integer.MAX_VALUE);
                return;
            case "documentEnd":
                handler.handleDocumentEnd(
                        Long.MAX_VALUE, -7L, Integer.MAX_VALUE, Integer.MIN_VALUE);
                return;
            case "text":
                handler.handleText(buffer, -1, 7, 11, 13);
                return;
            case "comment":
                handler.handleComment(buffer, 1, 2, 3, 4, 5, 6);
                return;
            case "standaloneStart":
                handler.handleStandaloneElementStart(buffer, 7, 8, true, 9, 10);
                return;
            case "standaloneEnd":
                handler.handleStandaloneElementEnd(buffer, 11, 12, false, 13, 14);
                return;
            case "openStart":
                handler.handleOpenElementStart(buffer, 15, 16, 17, 18);
                return;
            case "openEnd":
                handler.handleOpenElementEnd(buffer, 19, 20, 21, 22);
                return;
            case "closeStart":
                handler.handleCloseElementStart(buffer, 23, 24, 25, 26);
                return;
            case "closeEnd":
                handler.handleCloseElementEnd(buffer, 27, 28, 29, 30);
                return;
            case "attribute":
                handler.handleAttribute(
                        buffer,
                        31, 32, 33, 34,
                        35, 36, 37, 38,
                        39, 40, 41, 42, 43, 44);
                return;
            default:
                throw new AssertionError("Unknown event " + event);
        }
    }

    private static String hex(final char[] value) {
        if (value == null) {
            return "null";
        }
        final StringBuilder result = new StringBuilder(value.length * 5);
        for (int index = 0; index < value.length; index++) {
            if (index > 0) {
                result.append(',');
            }
            result.append(String.format("%04x", (int) value[index]));
        }
        return result.toString();
    }

    private static String utf16Hex(final String value) {
        if (value == null) {
            return "null";
        }
        return hex(value.toCharArray());
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private static final class NoOpProbe extends AbstractTextHandler {
        private NoOpProbe() {
            super();
        }
    }

    private static final class ChainedProbe extends AbstractChainedTextHandler {
        private ChainedProbe(final ITextHandler next) {
            super(next);
        }

        private ITextHandler exposeNext() {
            return getNext();
        }
    }

    private static final class RecordingHandler implements ITextHandler {

        private final StringBuilder events = new StringBuilder();
        private final String failEvent;
        private final TextParseException checked;
        private final RuntimeException runtime;

        private RecordingHandler() {
            this(null, null, null);
        }

        private RecordingHandler(
                final String failEvent,
                final TextParseException checked,
                final RuntimeException runtime) {
            this.failEvent = failEvent;
            this.checked = checked;
            this.runtime = runtime;
        }

        private void record(
                final String event, final char[] buffer, final String arguments)
                throws TextParseException {
            if (this.events.length() > 0) {
                this.events.append('|');
            }
            this.events.append(event)
                    .append('(').append(arguments).append(')')
                    .append('@').append(hex(buffer));
            if (buffer != null && buffer.length > 0) {
                buffer[0]++;
            }
            if (event.equals(this.failEvent)) {
                if (this.checked != null) {
                    throw this.checked;
                }
                throw this.runtime;
            }
        }

        @Override
        public void handleDocumentStart(
                final long startTimeNanos, final int line, final int col)
                throws TextParseException {
            record("documentStart", null, startTimeNanos + "," + line + "," + col);
        }

        @Override
        public void handleDocumentEnd(
                final long endTimeNanos,
                final long totalTimeNanos,
                final int line,
                final int col)
                throws TextParseException {
            record(
                    "documentEnd",
                    null,
                    endTimeNanos + "," + totalTimeNanos + "," + line + "," + col);
        }

        @Override
        public void handleText(
                final char[] buffer,
                final int offset,
                final int len,
                final int line,
                final int col)
                throws TextParseException {
            record("text", buffer, offset + "," + len + "," + line + "," + col);
        }

        @Override
        public void handleComment(
                final char[] buffer,
                final int contentOffset,
                final int contentLen,
                final int outerOffset,
                final int outerLen,
                final int line,
                final int col)
                throws TextParseException {
            record(
                    "comment",
                    buffer,
                    contentOffset + "," + contentLen + "," + outerOffset + ","
                            + outerLen + "," + line + "," + col);
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
            record(
                    "standaloneStart",
                    buffer,
                    nameOffset + "," + nameLen + "," + minimized + "," + line + "," + col);
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
            record(
                    "standaloneEnd",
                    buffer,
                    nameOffset + "," + nameLen + "," + minimized + "," + line + "," + col);
        }

        @Override
        public void handleOpenElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col)
                throws TextParseException {
            record("openStart", buffer, nameOffset + "," + nameLen + "," + line + "," + col);
        }

        @Override
        public void handleOpenElementEnd(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col)
                throws TextParseException {
            record("openEnd", buffer, nameOffset + "," + nameLen + "," + line + "," + col);
        }

        @Override
        public void handleCloseElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col)
                throws TextParseException {
            record("closeStart", buffer, nameOffset + "," + nameLen + "," + line + "," + col);
        }

        @Override
        public void handleCloseElementEnd(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col)
                throws TextParseException {
            record("closeEnd", buffer, nameOffset + "," + nameLen + "," + line + "," + col);
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
            record(
                    "attribute",
                    buffer,
                    nameOffset + "," + nameLen + "," + nameLine + "," + nameCol + ","
                            + operatorOffset + "," + operatorLen + ","
                            + operatorLine + "," + operatorCol + ","
                            + valueContentOffset + "," + valueContentLen + ","
                            + valueOuterOffset + "," + valueOuterLen + ","
                            + valueLine + "," + valueCol);
        }
    }
}
