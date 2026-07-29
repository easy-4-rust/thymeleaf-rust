package org.thymeleaf.templateparser.text;

import java.lang.reflect.Field;
import java.util.Arrays;

/**
 * 从固定 Thymeleaf Java 源码导出 EventProcessorTextHandler 的栈、属性和名称仓库语义。
 */
public final class EventProcessorTextHandlerGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private EventProcessorTextHandlerGolden() {
    }

    public static void main(final String[] args) throws Exception {
        emit("baseline", BASELINE);
        happyPath();
        stackErrors();
        attributeRules();
        failureOrdering();
        growthRules();
        repositoryRules();
        invalidRanges();
    }

    private static void happyPath() throws Exception {
        final RecordingHandler next = new RecordingHandler();
        final EventProcessorTextHandler handler = new EventProcessorTextHandler(next);
        final char[] root = "root".toCharArray();
        final char[] id = "id".toCharArray();
        final char[] child = "child".toCharArray();

        handler.handleOpenElementStart(root, 0, root.length, 1, 2);
        handler.handleAttribute(id, 0, id.length, 3, 4, -1, 0, 3, 6, -1, 0, -1, 0, 3, 7);
        handler.handleOpenElementStart(child, 0, child.length, 5, 6);
        handler.handleCloseElementStart(child, 0, child.length, 7, 8);
        handler.handleStandaloneElementStart("single".toCharArray(), 0, 6, true, 9, 10);
        handler.handleCloseElementStart(root, 0, root.length, 11, 12);
        handler.handleDocumentEnd(13L, 14L, 15, 16);

        emit("happy.events", next.events);
        emit("happy.state", state(handler));
    }

    private static void stackErrors() throws Exception {
        final EventProcessorTextHandler empty = new EventProcessorTextHandler(new RecordingHandler());
        emitThrowable("stack.closeEmpty", () -> empty.handleCloseElementStart(null, -9, -8, 21, 22));
        emit("stack.closeEmpty.state", state(empty));

        final EventProcessorTextHandler mismatch = new EventProcessorTextHandler(new RecordingHandler());
        mismatch.handleOpenElementStart("alpha".toCharArray(), 0, 5, 1, 1);
        emitThrowable(
                "stack.mismatch",
                () -> mismatch.handleCloseElementStart("beta".toCharArray(), 0, 4, 23, 24));
        emit("stack.mismatch.state", state(mismatch));
        emitThrowable("stack.documentEnd1", () -> mismatch.handleDocumentEnd(1, 2, 3, 4));
        emit("stack.documentEnd1.state", state(mismatch));
        mismatch.handleDocumentEnd(1, 2, 3, 4);
        emit("stack.documentEnd2.state", state(mismatch));

        final EventProcessorTextHandler unnamed = new EventProcessorTextHandler(new RecordingHandler());
        unnamed.handleOpenElementStart(new char[0], 0, 0, 1, 1);
        emitThrowable(
                "stack.unnamed",
                () -> unnamed.handleCloseElementStart("x".toCharArray(), 0, 1, 25, 26));

        final EventProcessorTextHandler drain = new EventProcessorTextHandler(new RecordingHandler());
        drain.handleOpenElementStart("a".toCharArray(), 0, 1, 1, 1);
        drain.handleOpenElementStart("b".toCharArray(), 0, 1, 1, 1);
        emitThrowable("stack.drain1", () -> drain.handleDocumentEnd(1, 2, 3, 4));
        emitThrowable("stack.drain2", () -> drain.handleDocumentEnd(1, 2, 3, 4));
        drain.handleDocumentEnd(1, 2, 3, 4);
        emit("stack.drain.state", state(drain));
    }

    private static void attributeRules() throws Exception {
        final RecordingHandler next = new RecordingHandler();
        final EventProcessorTextHandler handler = new EventProcessorTextHandler(next);
        handler.handleStandaloneElementStart("s".toCharArray(), 0, 1, false, 1, 1);
        attribute(handler, "name", 31, 32);
        emitThrowable("attribute.duplicate", () -> attribute(handler, "name", 33, 34));
        attribute(handler, "Name", 35, 36);
        attribute(handler, "a", 1, 1);
        attribute(handler, "b", 1, 1);
        emit("attribute.caseAndGrowth.state", state(handler));
        emit("attribute.caseAndGrowth.events", next.events);

        final char[] mutable = "a".toCharArray();
        final RecordingHandler mutating = new RecordingHandler();
        mutating.mutateAttributeName = true;
        final EventProcessorTextHandler mutation = new EventProcessorTextHandler(mutating);
        mutation.handleStandaloneElementStart("s".toCharArray(), 0, 1, false, 1, 1);
        attribute(mutation, mutable, 0, 1, 1, 1);
        attribute(mutation, "b", 2, 2);
        emit("attribute.mutation.buffer", hex(mutable));
        emit("attribute.mutation.state", state(mutation));
    }

    private static void failureOrdering() throws Exception {
        final RecordingHandler openNext = new RecordingHandler();
        openNext.failEvent = "openStart";
        final EventProcessorTextHandler open = new EventProcessorTextHandler(openNext);
        emitThrowable(
                "ordering.open.checked",
                () -> open.handleOpenElementStart("x".toCharArray(), 0, 1, 1, 2));
        emit("ordering.open.state", state(open));

        final RecordingHandler closeNext = new RecordingHandler();
        final EventProcessorTextHandler close = new EventProcessorTextHandler(closeNext);
        close.handleOpenElementStart("x".toCharArray(), 0, 1, 1, 1);
        attribute(close, "old", 1, 1);
        closeNext.failEvent = "closeStart";
        emitThrowable(
                "ordering.close.checked",
                () -> close.handleCloseElementStart("x".toCharArray(), 0, 1, 3, 4));
        emit("ordering.close.state", state(close));

        final RecordingHandler attributeNext = new RecordingHandler();
        final EventProcessorTextHandler attribute = new EventProcessorTextHandler(attributeNext);
        attribute.handleStandaloneElementStart("s".toCharArray(), 0, 1, false, 1, 1);
        attributeNext.failEvent = "attribute";
        emitThrowable("ordering.attribute.checked", () -> attribute(attribute, "x", 5, 6));
        attributeNext.failEvent = null;
        emitThrowable("ordering.attribute.retry", () -> attribute(attribute, "x", 7, 8));
        emit("ordering.attribute.state", state(attribute));

        final RecordingHandler standaloneNext = new RecordingHandler();
        final EventProcessorTextHandler standalone = new EventProcessorTextHandler(standaloneNext);
        standalone.handleStandaloneElementStart("s".toCharArray(), 0, 1, false, 1, 1);
        attribute(standalone, "old", 1, 1);
        standaloneNext.failEvent = "standaloneStart";
        emitThrowable(
                "ordering.standalone.checked",
                () -> standalone.handleStandaloneElementStart("t".toCharArray(), 0, 1, false, 1, 1));
        emit("ordering.standalone.state", state(standalone));
    }

    private static void growthRules() throws Exception {
        final EventProcessorTextHandler handler = new EventProcessorTextHandler(new RecordingHandler());
        for (int index = 0; index < 11; index++) {
            final char[] name = ("e" + index).toCharArray();
            handler.handleOpenElementStart(name, 0, name.length, 1, 1);
        }
        emit("growth.stack.open", state(handler));
        for (int index = 10; index >= 0; index--) {
            final char[] name = ("e" + index).toCharArray();
            handler.handleCloseElementStart(name, 0, name.length, 1, 1);
        }
        emit("growth.stack.closed", state(handler));
    }

    private static void repositoryRules() throws Exception {
        final EventProcessorTextHandler.StructureNamesRepository repository =
                new EventProcessorTextHandler.StructureNamesRepository();
        final char[] source = {'x', 'b', '\u0000', '\uD800', 'z'};
        final char[] first = repository.getStructureName(source, 1, 3);
        final char[] same = repository.getStructureName(new char[]{'b', '\u0000', '\uD800'}, 0, 3);
        source[1] = 'q';
        emit("repository.identity", first == same);
        emit("repository.copy", hex(first));

        final String[] names = {"z", "a", "m", "", "A", "\uD800", "\u0000"};
        for (final String name : names) {
            final char[] chars = name.toCharArray();
            repository.getStructureName(chars, 0, chars.length);
        }
        emit("repository.sorted", repositoryState(repository));

        for (int index = 0; index < 20; index++) {
            final char[] chars = ("n" + index).toCharArray();
            repository.getStructureName(chars, 0, chars.length);
        }
        emit("repository.grown", repositoryState(repository));
    }

    private static void invalidRanges() throws Exception {
        final EventProcessorTextHandler.StructureNamesRepository repository =
                new EventProcessorTextHandler.StructureNamesRepository();
        emitThrowable("invalid.repository.null", () -> repository.getStructureName(null, 0, 0));
        emitThrowable("invalid.repository.negativeOffset", () -> repository.getStructureName("a".toCharArray(), -1, 1));
        emitThrowable("invalid.repository.longRange", () -> repository.getStructureName("a".toCharArray(), 0, 2));
        emitThrowable("invalid.repository.negativeLen", () -> repository.getStructureName("a".toCharArray(), 0, -1));
        emit("invalid.repository.state", repositoryState(repository));

        final EventProcessorTextHandler.StructureNamesRepository populated =
                new EventProcessorTextHandler.StructureNamesRepository();
        populated.getStructureName("a".toCharArray(), 0, 1);
        emitThrowable("invalid.repository.populatedNegativeLenDifferent", () -> populated.getStructureName("b".toCharArray(), 0, -1));
        emitThrowable("invalid.repository.populatedNegativeLenEqualPrefix", () -> populated.getStructureName("a".toCharArray(), 0, -1));
        emit("invalid.repository.populatedState", repositoryState(populated));

        final EventProcessorTextHandler open = new EventProcessorTextHandler(new RecordingHandler());
        emitThrowable("invalid.open.null", () -> open.handleOpenElementStart(null, 0, 0, 1, 1));
        emit("invalid.open.state", state(open));

        final EventProcessorTextHandler close = new EventProcessorTextHandler(new RecordingHandler());
        close.handleOpenElementStart("x".toCharArray(), 0, 1, 1, 1);
        emitThrowable("invalid.close.null", () -> close.handleCloseElementStart(null, 0, 0, 1, 1));
        emit("invalid.close.state", state(close));

        final EventProcessorTextHandler attribute = new EventProcessorTextHandler(new RecordingHandler());
        emitThrowable("invalid.attribute.null", () -> attribute.handleAttribute(
                null, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1));
        emit("invalid.attribute.state", state(attribute));
    }

    private static void attribute(
            final EventProcessorTextHandler handler,
            final String name,
            final int line,
            final int col) throws TextParseException {
        final char[] chars = name.toCharArray();
        attribute(handler, chars, 0, chars.length, line, col);
    }

    private static void attribute(
            final EventProcessorTextHandler handler,
            final char[] chars,
            final int offset,
            final int len,
            final int line,
            final int col) throws TextParseException {
        handler.handleAttribute(chars, offset, len, line, col, -1, 0, line, col, -1, 0, -1, 0, line, col);
    }

    private static String state(final EventProcessorTextHandler handler) throws Exception {
        final char[][] stack = (char[][]) field(handler, "elementStack");
        final int stackSize = (Integer) field(handler, "elementStackSize");
        final char[][] attributes = (char[][]) field(handler, "currentElementAttributeNames");
        final int attributeSize = (Integer) field(handler, "currentElementAttributeNamesSize");
        final Object repository = field(handler, "structureNamesRepository");
        return "stack=" + names(stack, stackSize)
                + "/" + stack.length
                + ";attrs=" + (attributes == null ? "null" : names(attributes, attributeSize) + "/" + attributes.length)
                + ";repo=" + repositoryState(repository);
    }

    private static String repositoryState(final Object repository) throws Exception {
        final char[][] values = (char[][]) field(repository, "repository");
        final int size = (Integer) field(repository, "repositorySize");
        return names(values, size) + "/" + values.length;
    }

    private static Object field(final Object target, final String name) throws Exception {
        final Field field = target.getClass().getDeclaredField(name);
        field.setAccessible(true);
        return field.get(target);
    }

    private static String names(final char[][] values, final int size) {
        final String[] names = new String[size];
        for (int index = 0; index < size; index++) {
            names[index] = hex(values[index]);
        }
        return Arrays.toString(names);
    }

    private static void emitThrowable(final String key, final ThrowingRunnable runnable) {
        try {
            runnable.run();
            emit(key, "NO_ERROR");
        } catch (final Throwable throwable) {
            String location = "";
            if (throwable instanceof TextParseException) {
                final TextParseException parse = (TextParseException) throwable;
                location = ";line=" + parse.getLine() + ";col=" + parse.getCol();
            }
            emit(
                    key,
                    throwable.getClass().getName()
                            + ";message=" + utf16Hex(throwable.getMessage())
                            + location);
        }
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
        private boolean mutateAttributeName;

        private void record(final String event, final char[] buffer, final int offset, final int len)
                throws TextParseException {
            if (this.events.length() > 0) {
                this.events.append('|');
            }
            this.events.append(event).append('@');
            if (buffer == null) {
                this.events.append("null");
            } else if (offset >= 0 && len >= 0 && offset + len <= buffer.length) {
                this.events.append(hex(Arrays.copyOfRange(buffer, offset, offset + len)));
            } else {
                this.events.append("range(").append(offset).append(',').append(len).append(')');
            }
            if (this.mutateAttributeName && "attribute".equals(event) && len > 0) {
                buffer[offset] = 'b';
            }
            if (event.equals(this.failEvent)) {
                throw new TextParseException("downstream-" + event, 71, 72);
            }
        }

        @Override
        public void handleDocumentEnd(
                final long endTimeNanos,
                final long totalTimeNanos,
                final int line,
                final int col) throws TextParseException {
            record("documentEnd", null, 0, 0);
        }

        @Override
        public void handleStandaloneElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final boolean minimized,
                final int line,
                final int col) throws TextParseException {
            record("standaloneStart", buffer, nameOffset, nameLen);
        }

        @Override
        public void handleOpenElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col) throws TextParseException {
            record("openStart", buffer, nameOffset, nameLen);
        }

        @Override
        public void handleCloseElementStart(
                final char[] buffer,
                final int nameOffset,
                final int nameLen,
                final int line,
                final int col) throws TextParseException {
            record("closeStart", buffer, nameOffset, nameLen);
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
            record("attribute", buffer, nameOffset, nameLen);
        }
    }
}
