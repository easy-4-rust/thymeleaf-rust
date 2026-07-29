package org.thymeleaf.templateparser.text;

import java.io.IOException;
import java.io.Reader;
import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.List;

/**
 * 固定 TextParser 的跨缓冲区、Reader 生命周期、异常与 BufferPool 可观察语义。
 */
public final class TextParserGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private TextParserGolden() {
    }

    public static void main(final String[] args) throws Exception {
        emit("baseline", BASELINE);
        validationCases();
        documentCases();
        splitMatrixCases();
        readerCases();
        handlerFailureCases();
        incompleteCases();
        bufferPoolCases();
        constructorCases();
    }

    private static void validationCases() {
        final TextParser parser = new TextParser(1, 4, true, true);
        emitThrowable("validation.nullDocument", () -> parser.parse((String) null, new RecordingHandler()));
        emitThrowable("validation.nullStringHandler", () -> parser.parse("x", null));
        emitThrowable("validation.nullReader", () -> parser.parse((Reader) null, new RecordingHandler()));
        emitThrowable(
                "validation.nullReaderHandler",
                () -> parser.parse(new ScriptedReader("x", 1, 0, -1, CloseMode.NONE), null));
    }

    private static void documentCases() {
        final String[] documents = {
                "",
                "plain",
                "a\nb\r\nc",
                "[#root]x[/root]",
                "[#img src=\"hello\" alt='x'/]",
                "[# one]",
                "[[#hello]]...[[/hello]]",
                "/*hello*/",
                "/*[#hello/]*/tail",
                "/*[(hello)]*/something;",
                "a//line\n[#x/]b",
                "\"[#not]\" [#yes/]",
                "'a\\'[#not]' [#yes/]",
                "`[#not]` [#yes/]",
                "/[#not]/ [#yes/]",
                "\ud800[#x/]\udc00"
        };
        for (final boolean processComments : new boolean[]{false, true}) {
            for (final boolean standardDialect : new boolean[]{false, true}) {
                for (int index = 0; index < documents.length; index++) {
                    final String document = documents[index];
                    final RecordingHandler handler = new RecordingHandler();
                    final TextParser parser = new TextParser(2, 3, processComments, standardDialect);
                    emitThrowable(
                            "document." + processComments + "." + standardDialect + "." + index + ".throwable",
                            () -> parser.parse(document, handler));
                    emit(
                            "document." + processComments + "." + standardDialect + "." + index + ".events",
                            handler.events());
                }
            }
        }
    }

    private static void splitMatrixCases() {
        final String[] documents = {
                "before[#root a=\"x]y\"]line\n[#single/][/root]after",
                "/*[(hello)]*/ [1,\n 2,3] tail;",
                "a//comment\nb/*[#x/]*/c",
                "\"quoted\\\\\\\" [#no]\" [#yes/]",
                "[#template a='zero' b='one']\n\naaaaa\n\n[/template]"
        };
        for (int index = 0; index < documents.length; index++) {
            for (final boolean processComments : new boolean[]{false, true}) {
                final String expected = parseWithBuffer(documents[index], 64, processComments);
                final StringBuilder digest = new StringBuilder();
                for (int bufferSize = 1; bufferSize <= 96; bufferSize++) {
                    final String actual = parseWithBuffer(documents[index], bufferSize, processComments);
                    if (!expected.equals(actual)) {
                        throw new AssertionError(
                                "split mismatch document=" + index + ", processComments=" + processComments
                                        + ", bufferSize=" + bufferSize + "\nexpected=" + expected + "\nactual=" + actual);
                    }
                    digest.append(actual.hashCode()).append(':').append(actual.length()).append(';');
                }
                emit(
                        "split." + index + "." + processComments,
                        expected + ";matrixHash=" + digest.toString().hashCode());
            }
        }
    }

    private static String parseWithBuffer(
            final String document,
            final int bufferSize,
            final boolean processComments) {
        final RecordingHandler handler = RecordingHandler.semantic();
        final TextParser parser = new TextParser(2, bufferSize, processComments, true);
        final ScriptedReader reader = new ScriptedReader(document, Integer.MAX_VALUE, 0, -1, CloseMode.NONE);
        ITextHandler chain = new EventProcessorTextHandler(handler);
        if (processComments) {
            chain = new CommentProcessorTextHandler(true, chain);
        }
        try {
            parser.parseDocument(reader, bufferSize, chain);
            return handler.events();
        } catch (final TextParseException exception) {
            return describeThrowable(exception);
        }
    }

    private static void readerCases() {
        runReader("reader.chunk1", new ScriptedReader("[#x/]tail", 1, 0, -1, CloseMode.NONE), 3);
        runReader("reader.chunk2", new ScriptedReader("[#x/]tail", 2, 0, -1, CloseMode.NONE), 3);
        runReader("reader.zeroThenData", new ScriptedReader("[#x/]", 2, 2, -1, CloseMode.NONE), 3);
        runReader("reader.readFailure", new ScriptedReader("[#x/]tail", 2, 0, 3, CloseMode.NONE), 3);
        runReader("reader.closeIOException", new ScriptedReader("plain", 2, 0, -1, CloseMode.IO), 3);
        runReader("reader.closeAssertion", new ScriptedReader("plain", 2, 0, -1, CloseMode.ASSERTION), 3);

        final ScriptedReader empty = new ScriptedReader("", 2, 0, -1, CloseMode.NONE);
        runReader("reader.empty", empty, 3);
    }

    private static void runReader(
            final String key,
            final ScriptedReader reader,
            final int bufferSize) {
        final TextParser parser = new TextParser(1, bufferSize, false, true);
        final RecordingHandler handler = new RecordingHandler();
        emitThrowable(key + ".throwable", () -> parser.parseDocument(reader, bufferSize, handler));
        emit(key + ".events", handler.events());
        emit(key + ".requests", reader.requests());
        emit(key + ".closeCount", reader.closeCount);
    }

    private static void handlerFailureCases() {
        for (final String event : new String[]{"documentStart", "text", "openStart", "documentEnd"}) {
            final ScriptedReader reader = new ScriptedReader("[#x]text[/x]", 2, 0, -1, CloseMode.NONE);
            final RecordingHandler handler = new RecordingHandler();
            handler.failEvent = event;
            final TextParser parser = new TextParser(1, 3, false, true);
            emitThrowable(
                    "handler.checked." + event,
                    () -> parser.parseDocument(reader, 3, handler));
            emit("handler.checked." + event + ".events", handler.events());
            emit("handler.checked." + event + ".closeCount", reader.closeCount);
        }

        for (final String event : new String[]{"documentStart", "text", "standaloneStart", "documentEnd"}) {
            final String document = "standaloneStart".equals(event) ? "[#x/]" : "text";
            final ScriptedReader reader = new ScriptedReader(document, 2, 0, -1, CloseMode.NONE);
            final RecordingHandler handler = new RecordingHandler();
            handler.runtimeFailEvent = event;
            final TextParser parser = new TextParser(1, 3, false, true);
            emitThrowable(
                    "handler.runtime." + event,
                    () -> parser.parseDocument(reader, 3, handler));
            emit("handler.runtime." + event + ".events", handler.events());
            emit("handler.runtime." + event + ".closeCount", reader.closeCount);
        }
    }

    private static void incompleteCases() {
        final String[] documents = {
                "[#open",
                "[/close",
                "/*block",
                "//line",
                "\"literal",
                "'literal",
                "`literal",
                "/regex"
        };
        for (final boolean processComments : new boolean[]{false, true}) {
            for (int index = 0; index < documents.length; index++) {
                final String document = documents[index];
                final ScriptedReader reader =
                        new ScriptedReader(document, 1, 0, -1, CloseMode.NONE);
                final TextParser parser = new TextParser(1, 2, processComments, true);
                final RecordingHandler handler = new RecordingHandler();
                emitThrowable(
                        "incomplete." + processComments + "." + index + ".throwable",
                        () -> parser.parseDocument(reader, 2, handler));
                emit(
                        "incomplete." + processComments + "." + index + ".events",
                        handler.events());
            }
        }
    }

    private static void bufferPoolCases() throws Exception {
        final Class<?> poolClass =
                Class.forName("org.thymeleaf.templateparser.text.TextParser$BufferPool");
        final Constructor<?> constructor = poolClass.getDeclaredConstructor(int.class, int.class);
        constructor.setAccessible(true);
        final Method allocate = poolClass.getDeclaredMethod("allocateBuffer", int.class);
        final Method release = poolClass.getDeclaredMethod("releaseBuffer", char[].class);
        allocate.setAccessible(true);
        release.setAccessible(true);

        final Object pool = constructor.newInstance(2, 4);
        final char[] first = (char[]) allocate.invoke(pool, 4);
        final char[] second = (char[]) allocate.invoke(pool, 4);
        final char[] overflow = (char[]) allocate.invoke(pool, 4);
        emit("pool.distinct", (first != second) + "," + (first != overflow) + "," + (second != overflow));
        release.invoke(pool, (Object) first);
        final char[] reusedFirst = (char[]) allocate.invoke(pool, 4);
        emit("pool.reusedFirst", reusedFirst == first);
        release.invoke(pool, (Object) new char[4]);
        final char[] stillOverflow = (char[]) allocate.invoke(pool, 4);
        emit("pool.foreignIgnored", stillOverflow != first && stillOverflow != second);
        release.invoke(pool, (Object) second);
        final char[] reusedSecond = (char[]) allocate.invoke(pool, 4);
        emit("pool.reusedSecond", reusedSecond == second);
        final char[] differentSizeOne = (char[]) allocate.invoke(pool, 3);
        final char[] differentSizeTwo = (char[]) allocate.invoke(pool, 3);
        emit(
                "pool.differentSize",
                differentSizeOne.length + "," + differentSizeTwo.length + "," + (differentSizeOne != differentSizeTwo));
        release.invoke(pool, new Object[]{null});
        emit("pool.releaseNull", "NO_ERROR");

        emitReflectiveThrowable("pool.negativeAllocate", () -> allocate.invoke(pool, -1));
        emitReflectiveThrowable(
                "pool.negativePoolSize",
                () -> constructor.newInstance(-1, 4));
        emitReflectiveThrowable(
                "pool.negativeBufferSize",
                () -> constructor.newInstance(1, -1));
        final Object zeroPoolNegativeBuffer = constructor.newInstance(0, -1);
        emit("pool.zeroPoolNegativeBuffer", zeroPoolNegativeBuffer != null);
        emitReflectiveThrowable(
                "pool.zeroPoolNegativeAllocate",
                () -> allocate.invoke(zeroPoolNegativeBuffer, -1));
    }

    private static void constructorCases() {
        emitThrowable("constructor.negativePool", () -> new TextParser(-1, 4, false, true));
        emitThrowable("constructor.negativeBuffer", () -> new TextParser(1, -1, false, true));
        emitThrowable("constructor.zeroPoolNegativeBuffer", () -> new TextParser(0, -1, false, true));
        final TextParser parser = new TextParser(0, 1, false, true);
        final RecordingHandler first = new RecordingHandler();
        final RecordingHandler second = new RecordingHandler();
        emitThrowable("constructor.zeroPool.first", () -> parser.parse("a", first));
        emitThrowable("constructor.zeroPool.second", () -> parser.parse("b", second));
        emit("constructor.zeroPool.events", first.events() + "|" + second.events());
    }

    private static void emitReflectiveThrowable(
            final String key,
            final ThrowingRunnable operation) {
        try {
            operation.run();
            emit(key, "NO_ERROR");
        } catch (final InvocationTargetException exception) {
            emit(key, describeThrowable(exception.getCause()));
        } catch (final Throwable throwable) {
            emit(key, describeThrowable(throwable));
        }
    }

    private static void emitThrowable(
            final String key,
            final ThrowingRunnable operation) {
        try {
            operation.run();
            emit(key, "NO_ERROR");
        } catch (final Throwable throwable) {
            emit(key, describeThrowable(throwable));
        }
    }

    private static String describeThrowable(final Throwable throwable) {
        final StringBuilder result = new StringBuilder();
        result.append(throwable.getClass().getName());
        result.append(";message=").append(nullableHex(throwable.getMessage()));
        if (throwable instanceof TextParseException) {
            final TextParseException parseException = (TextParseException) throwable;
            result.append(";line=").append(parseException.getLine());
            result.append(";col=").append(parseException.getCol());
        }
        final Throwable cause = throwable.getCause();
        if (cause != null) {
            result.append(";causeClass=").append(cause.getClass().getName());
            result.append(";causeMessage=").append(nullableHex(cause.getMessage()));
        }
        return result.toString();
    }

    private static String nullableHex(final String value) {
        return value == null ? "null" : hex(value.toCharArray(), 0, value.length());
    }

    private static String hex(final char[] value, final int offset, final int len) {
        final StringBuilder result = new StringBuilder();
        for (int index = offset; index < offset + len; index++) {
            if (result.length() > 0) {
                result.append(',');
            }
            result.append(String.format("%04x", (int) value[index]));
        }
        return result.toString();
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + value);
    }

    @FunctionalInterface
    private interface ThrowingRunnable {
        void run() throws Throwable;
    }

    private enum CloseMode {
        NONE,
        IO,
        ASSERTION
    }

    private static final class ScriptedReader extends Reader {
        private final char[] input;
        private final int maxChunk;
        private int zeroReads;
        private final int failCall;
        private final CloseMode closeMode;
        private int position;
        private int readCalls;
        private int closeCount;
        private final List<String> requests = new ArrayList<>();

        private ScriptedReader(
                final String input,
                final int maxChunk,
                final int zeroReads,
                final int failCall,
                final CloseMode closeMode) {
            this.input = input.toCharArray();
            this.maxChunk = maxChunk;
            this.zeroReads = zeroReads;
            this.failCall = failCall;
            this.closeMode = closeMode;
        }

        @Override
        public int read(final char[] target, final int offset, final int len) throws IOException {
            this.requests.add(offset + ":" + len);
            this.readCalls++;
            if (this.failCall == this.readCalls) {
                throw new IOException("reader-boom-" + this.readCalls);
            }
            if (this.zeroReads > 0) {
                this.zeroReads--;
                return 0;
            }
            if (this.position >= this.input.length) {
                return -1;
            }
            final int copied = Math.min(Math.min(len, this.maxChunk), this.input.length - this.position);
            System.arraycopy(this.input, this.position, target, offset, copied);
            this.position += copied;
            return copied;
        }

        @Override
        public int read(final char[] target) throws IOException {
            return read(target, 0, target.length);
        }

        @Override
        public int read() throws IOException {
            final char[] one = new char[1];
            final int read = read(one, 0, 1);
            return read == -1 ? -1 : one[0];
        }

        @Override
        public void close() throws IOException {
            this.closeCount++;
            if (this.closeMode == CloseMode.IO) {
                throw new IOException("close-boom");
            }
            if (this.closeMode == CloseMode.ASSERTION) {
                throw new AssertionError("close-error");
            }
        }

        private String requests() {
            return String.join(",", this.requests);
        }
    }

    private static final class RecordingHandler extends AbstractTextHandler {
        private final StringBuilder events = new StringBuilder();
        private final boolean semantic;
        private String failEvent;
        private String runtimeFailEvent;

        private RecordingHandler() {
            this(false);
        }

        private RecordingHandler(final boolean semantic) {
            this.semantic = semantic;
        }

        private static RecordingHandler semantic() {
            return new RecordingHandler(true);
        }

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
            } else if (offset >= 0 && len >= 0 && ((long) offset + len) <= buffer.length) {
                this.events.append(hex(buffer, offset, len));
            } else {
                this.events.append("range(").append(offset).append(',').append(len).append(')');
            }
            if (event.equals(this.failEvent)) {
                throw new TextParseException("checked-" + event, 71, 72);
            }
            if (event.equals(this.runtimeFailEvent)) {
                throw new IllegalStateException("runtime-" + event);
            }
        }

        @Override
        public void handleDocumentStart(
                final long startTimeNanos,
                final int line,
                final int col) throws TextParseException {
            record("documentStart", null, 0, 0, line + "," + col);
        }

        @Override
        public void handleDocumentEnd(
                final long endTimeNanos,
                final long totalTimeNanos,
                final int line,
                final int col) throws TextParseException {
            record("documentEnd", null, 0, 0, (totalTimeNanos >= 0) + "," + line + "," + col);
        }

        @Override
        public void handleText(
                final char[] buffer,
                final int offset,
                final int len,
                final int line,
                final int col) throws TextParseException {
            record(
                    "text",
                    buffer,
                    offset,
                    len,
                    this.semantic ? line + "," + col : offset + "," + len + "," + line + "," + col);
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
            record(
                    "comment",
                    buffer,
                    outerOffset,
                    outerLen,
                    this.semantic
                            ? line + "," + col
                            : contentOffset + "," + contentLen + "," + outerOffset + "," + outerLen + "," + line + "," + col);
        }

        @Override
        public void handleStandaloneElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final boolean minimized,
                final int line,
                final int col) throws TextParseException {
            record(
                    "standaloneStart",
                    buffer,
                    nameOffset,
                    nameLen,
                    this.semantic
                            ? minimized + "," + line + "," + col
                            : nameOffset + "," + nameLen + "," + minimized + "," + line + "," + col);
        }

        @Override
        public void handleStandaloneElementEnd(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final boolean minimized,
                final int line,
                final int col) throws TextParseException {
            record(
                    "standaloneEnd",
                    buffer,
                    nameOffset,
                    nameLen,
                    this.semantic
                            ? minimized + "," + line + "," + col
                            : nameOffset + "," + nameLen + "," + minimized + "," + line + "," + col);
        }

        @Override
        public void handleOpenElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col) throws TextParseException {
            record(
                    "openStart",
                    buffer,
                    nameOffset,
                    nameLen,
                    this.semantic ? line + "," + col : nameOffset + "," + nameLen + "," + line + "," + col);
        }

        @Override
        public void handleOpenElementEnd(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col) throws TextParseException {
            record(
                    "openEnd",
                    buffer,
                    nameOffset,
                    nameLen,
                    this.semantic ? line + "," + col : nameOffset + "," + nameLen + "," + line + "," + col);
        }

        @Override
        public void handleCloseElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col) throws TextParseException {
            record(
                    "closeStart",
                    buffer,
                    nameOffset,
                    nameLen,
                    this.semantic ? line + "," + col : nameOffset + "," + nameLen + "," + line + "," + col);
        }

        @Override
        public void handleCloseElementEnd(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col) throws TextParseException {
            record(
                    "closeEnd",
                    buffer,
                    nameOffset,
                    nameLen,
                    this.semantic ? line + "," + col : nameOffset + "," + nameLen + "," + line + "," + col);
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
                final int valueCol) throws TextParseException {
            record(
                    "attribute",
                    buffer,
                    nameOffset,
                    nameLen,
                    this.semantic
                            ? nameLine + "," + nameCol + "," + operatorLine + "," + operatorCol + ","
                                    + valueLine + "," + valueCol
                            : nameOffset + "," + nameLen + "," + nameLine + "," + nameCol + ","
                                    + operatorOffset + "," + operatorLen + "," + operatorLine + "," + operatorCol + ","
                                    + valueContentOffset + "," + valueContentLen + "," + valueOuterOffset + ","
                                    + valueOuterLen + "," + valueLine + "," + valueCol);
        }

        private String events() {
            return this.events.toString();
        }
    }
}
