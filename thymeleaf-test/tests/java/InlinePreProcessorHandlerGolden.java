package org.thymeleaf.standard.inline;

/**
 * 从固定 Thymeleaf Java 源码导出内联预处理 SPI 的参数和动态分派语义。
 */
public final class InlinePreProcessorHandlerGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private static final String[] EVENTS = {
            "text",
            "standaloneStart",
            "standaloneEnd",
            "openStart",
            "openEnd",
            "autoOpenStart",
            "autoOpenEnd",
            "closeStart",
            "closeEnd",
            "autoCloseStart",
            "autoCloseEnd",
            "attribute"
    };

    private InlinePreProcessorHandlerGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        for (final String event : EVENTS) {
            final RecordingHandler handler = new RecordingHandler();
            final IInlinePreProcessorHandler dynamic = handler;
            final char[] buffer = {'a', '\uD800', 'z'};
            invoke(dynamic, event, buffer);
            emit(event + ".buffer", handler.event);
            invoke(dynamic, event, null);
            emit(event + ".null", handler.event);
        }
    }

    private static void invoke(
            final IInlinePreProcessorHandler handler, final String event, final char[] buffer) {
        switch (event) {
            case "text":
                handler.handleText(buffer, -1, 2, 3, 4);
                return;
            case "standaloneStart":
                handler.handleStandaloneElementStart(buffer, 5, 6, true, 7, 8);
                return;
            case "standaloneEnd":
                handler.handleStandaloneElementEnd(buffer, 9, 10, false, 11, 12);
                return;
            case "openStart":
                handler.handleOpenElementStart(buffer, 13, 14, 15, 16);
                return;
            case "openEnd":
                handler.handleOpenElementEnd(buffer, 17, 18, 19, 20);
                return;
            case "autoOpenStart":
                handler.handleAutoOpenElementStart(buffer, 21, 22, 23, 24);
                return;
            case "autoOpenEnd":
                handler.handleAutoOpenElementEnd(buffer, 25, 26, 27, 28);
                return;
            case "closeStart":
                handler.handleCloseElementStart(buffer, 29, 30, 31, 32);
                return;
            case "closeEnd":
                handler.handleCloseElementEnd(buffer, 33, 34, 35, 36);
                return;
            case "autoCloseStart":
                handler.handleAutoCloseElementStart(buffer, 37, 38, 39, 40);
                return;
            case "autoCloseEnd":
                handler.handleAutoCloseElementEnd(buffer, 41, 42, 43, 44);
                return;
            case "attribute":
                handler.handleAttribute(
                        buffer,
                        45, 46, 47, 48,
                        49, 50, 51, 52,
                        53, 54, 55, 56, 57, 58);
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

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private static final class RecordingHandler implements IInlinePreProcessorHandler {

        private String event;

        private void record(final String name, final char[] buffer, final Object... arguments) {
            if (buffer != null && buffer.length > 0) {
                buffer[0]++;
            }
            final StringBuilder result = new StringBuilder(name).append('(');
            for (int index = 0; index < arguments.length; index++) {
                if (index > 0) {
                    result.append(',');
                }
                result.append(arguments[index]);
            }
            this.event = result.append(")@").append(hex(buffer)).toString();
        }

        @Override
        public void handleText(
                final char[] buffer, final int offset, final int len,
                final int line, final int col) {
            record("text", buffer, offset, len, line, col);
        }

        @Override
        public void handleStandaloneElementStart(
                final char[] buffer, final int nameOffset, final int nameLen,
                final boolean minimized, final int line, final int col) {
            record("standaloneStart", buffer, nameOffset, nameLen, minimized, line, col);
        }

        @Override
        public void handleStandaloneElementEnd(
                final char[] buffer, final int nameOffset, final int nameLen,
                final boolean minimized, final int line, final int col) {
            record("standaloneEnd", buffer, nameOffset, nameLen, minimized, line, col);
        }

        @Override
        public void handleOpenElementStart(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) {
            record("openStart", buffer, nameOffset, nameLen, line, col);
        }

        @Override
        public void handleOpenElementEnd(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) {
            record("openEnd", buffer, nameOffset, nameLen, line, col);
        }

        @Override
        public void handleAutoOpenElementStart(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) {
            record("autoOpenStart", buffer, nameOffset, nameLen, line, col);
        }

        @Override
        public void handleAutoOpenElementEnd(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) {
            record("autoOpenEnd", buffer, nameOffset, nameLen, line, col);
        }

        @Override
        public void handleCloseElementStart(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) {
            record("closeStart", buffer, nameOffset, nameLen, line, col);
        }

        @Override
        public void handleCloseElementEnd(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) {
            record("closeEnd", buffer, nameOffset, nameLen, line, col);
        }

        @Override
        public void handleAutoCloseElementStart(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) {
            record("autoCloseStart", buffer, nameOffset, nameLen, line, col);
        }

        @Override
        public void handleAutoCloseElementEnd(
                final char[] buffer, final int nameOffset, final int nameLen,
                final int line, final int col) {
            record("autoCloseEnd", buffer, nameOffset, nameLen, line, col);
        }

        @Override
        public void handleAttribute(
                final char[] buffer,
                final int nameOffset, final int nameLen,
                final int nameLine, final int nameCol,
                final int operatorOffset, final int operatorLen,
                final int operatorLine, final int operatorCol,
                final int valueContentOffset, final int valueContentLen,
                final int valueOuterOffset, final int valueOuterLen,
                final int valueLine, final int valueCol) {
            record(
                    "attribute", buffer,
                    nameOffset, nameLen, nameLine, nameCol,
                    operatorOffset, operatorLen, operatorLine, operatorCol,
                    valueContentOffset, valueContentLen, valueOuterOffset, valueOuterLen,
                    valueLine, valueCol);
        }
    }
}
