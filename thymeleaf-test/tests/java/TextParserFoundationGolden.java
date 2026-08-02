package org.thymeleaf.templateparser.text;

/**
 * 从固定 Thymeleaf Java 源码导出 text parser 基础状态与定位语义 Golden。
 */
public final class TextParserFoundationGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private TextParserFoundationGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        statusCases();
        locatorCases();
    }

    private static void statusCases() {
        final TextParseStatus first = new TextParseStatus();
        final TextParseStatus second = new TextParseStatus();
        emit("status.default", describe(first));
        first.offset = -1;
        first.line = Integer.MAX_VALUE;
        first.col = Integer.MIN_VALUE;
        first.inStructure = true;
        first.inCommentLine = true;
        first.literalMarker = '\uD800';
        emit("status.mutated", describe(first));
        emit("status.independent", describe(second));
    }

    private static void locatorCases() {
        final int[] locator = {0, 0};
        countAndEmit("locator.ascii", locator, 'A');
        countAndEmit("locator.lf", locator, '\n');
        countAndEmit("locator.cr", locator, '\r');
        countAndEmit("locator.nul", locator, '\0');
        countAndEmit("locator.surrogate", locator, '\uD800');

        final int[] lineOverflow = {Integer.MAX_VALUE, 7};
        countAndEmit("locator.lineOverflow", lineOverflow, '\n');
        final int[] columnOverflow = {9, Integer.MAX_VALUE};
        countAndEmit("locator.columnOverflow", columnOverflow, 'x');

        emitLocatorOutcome("locator.nullLf", null, '\n');
        emitLocatorOutcome("locator.nullAscii", null, 'x');
        emitLocatorOutcome("locator.emptyLf", new int[0], '\n');
        emitLocatorOutcome("locator.emptyAscii", new int[0], 'x');
        emitLocatorOutcome("locator.oneLf", new int[]{5}, '\n');
        emitLocatorOutcome("locator.oneAscii", new int[]{5}, 'x');
        emitLocatorOutcome("locator.extra", new int[]{2, 3, 99}, '\n');
    }

    private static void countAndEmit(final String key, final int[] locator, final char value) {
        ParsingLocatorUtil.countChar(locator, value);
        emit(key, describe(locator));
    }

    private static void emitLocatorOutcome(
            final String key, final int[] locator, final char value) {
        try {
            ParsingLocatorUtil.countChar(locator, value);
            emit(key, "OK:" + describe(locator));
        } catch (final Throwable throwable) {
            emit(
                    key,
                    "ERR:" + throwable.getClass().getName() + ":"
                            + toUtf16Hex(String.valueOf(throwable.getMessage())) + ":"
                            + describeNullable(locator));
        }
    }

    private static String describe(final TextParseStatus status) {
        return status.offset + "," + status.line + "," + status.col + ","
                + status.inStructure + "," + status.inCommentLine + ","
                + String.format("%04x", (int) status.literalMarker);
    }

    private static String describe(final int[] locator) {
        final StringBuilder result = new StringBuilder();
        for (int index = 0; index < locator.length; index++) {
            if (index > 0) {
                result.append(',');
            }
            result.append(locator[index]);
        }
        return result.toString();
    }

    private static String describeNullable(final int[] locator) {
        return locator == null ? "null" : describe(locator);
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
}
