import org.thymeleaf.util.LoggingUtils;

/**
 * 从固定 Thymeleaf Java 源码导出 LoggingUtils Golden。
 */
public final class LoggingUtilsGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private LoggingUtilsGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emit("null", Boolean.toString(LoggingUtils.loggifyTemplateName(null) == null));

        emitCase("empty", "");
        emitCase("short", "home");
        emitCase("short_lf", "home\npage");
        emitCase("short_cr", "home\rpage");
        emitCase("length_120", repeat('x', 120));
        emitCase("length_121", repeat('x', 121));
        emitCase("long_lf", repeat('a', 34) + "\n" + repeat('b', 90));

        final String prefixSplit =
                repeat('a', 34) + new String(Character.toChars(0x1F600)) + repeat('b', 90);
        emitCase("prefix_surrogate_split", prefixSplit);

        final String suffixSplit =
                repeat('a', 41) + new String(Character.toChars(0x1F600)) + repeat('b', 79);
        emitCase("suffix_surrogate_split", suffixSplit);
    }

    private static void emitCase(final String key, final String source) {
        final String result = LoggingUtils.loggifyTemplateName(source);
        emit(key + ".source_length", Integer.toString(source.length()));
        emit(key + ".result_length", Integer.toString(result.length()));
        emit(key + ".same", Boolean.toString(result == source));
        emit(key + ".utf16", utf16Hex(result));
    }

    private static String repeat(final char character, final int count) {
        final StringBuilder builder = new StringBuilder(count);
        for (int i = 0; i < count; i++) {
            builder.append(character);
        }
        return builder.toString();
    }

    private static String utf16Hex(final String value) {
        final StringBuilder builder = new StringBuilder(value.length() * 5);
        for (int i = 0; i < value.length(); i++) {
            if (i > 0) {
                builder.append(',');
            }
            appendHexDigit(builder, (value.charAt(i) >>> 12) & 0x0F);
            appendHexDigit(builder, (value.charAt(i) >>> 8) & 0x0F);
            appendHexDigit(builder, (value.charAt(i) >>> 4) & 0x0F);
            appendHexDigit(builder, value.charAt(i) & 0x0F);
        }
        return builder.toString();
    }

    private static void appendHexDigit(final StringBuilder builder, final int value) {
        builder.append((char)(value < 10 ? '0' + value : 'A' + value - 10));
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }
}
