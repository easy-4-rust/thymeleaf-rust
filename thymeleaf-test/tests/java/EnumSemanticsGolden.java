import org.thymeleaf.engine.HTMLElementType;
import org.thymeleaf.model.AttributeValueQuotes;
import org.thymeleaf.standard.inline.StandardInlineMode;

/**
 * 从固定 Thymeleaf Java 源码导出剩余三个 enum 的 Golden。
 */
public final class EnumSemanticsGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final long FNV_OFFSET = 0xcbf29ce484222325L;
    private static final long FNV_PRIME = 0x100000001b3L;

    private EnumSemanticsGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        attributeValueQuotes();
        htmlElementTypes();
        standardInlineModes();
        parseCases();
        exhaustiveInlineParsing();
    }

    private static void attributeValueQuotes() {
        emit("quotes.count", AttributeValueQuotes.values().length);
        for (final AttributeValueQuotes value : AttributeValueQuotes.values()) {
            emit("quotes." + value.name(),
                    value.ordinal() + "," + value.name() + "," + value.toString());
        }
    }

    private static void htmlElementTypes() {
        emit("html.count", HTMLElementType.values().length);
        for (final HTMLElementType value : HTMLElementType.values()) {
            emit("html." + value.name(),
                    value.ordinal() + "," + value.name() + "," + value.toString()
                            + "," + value.isVoid());
        }
    }

    private static void standardInlineModes() {
        emit("inline.count", StandardInlineMode.values().length);
        for (final StandardInlineMode value : StandardInlineMode.values()) {
            emit("inline." + value.name(),
                    value.ordinal() + "," + value.name() + "," + value.toString());
        }
    }

    private static void parseCases() {
        emitParse("null", null);
        emitParse("empty", "");
        emitParse("space", " ");
        emitParse("controls", "\u0000\u0009\u0020");
        emitParse("nbsp", "\u00A0");
        emitParse("raw", "RAW");
        emitParse("noneLower", "none");
        emitParse("htmlMixed", "hTmL");
        emitParse("xmlLower", "xml");
        emitParse("textMixed", "TeXt");
        emitParse("javascriptLower", "javascript");
        emitParse("cssLower", "css");
        emitParse("cssLongS", "C\u017F\u017F");
        emitParse("javascriptDotlessI", "JAVASCR\u0131PT");
        emitParse("javascriptDottedI", "JAVASCR\u0130PT");
        emitParseUtf16("paddedHtml", " HTML ");
        emitParseUtf16("isolatedHighSurrogate", "\uD800");
    }

    private static void exhaustiveInlineParsing() {
        long singleCodeUnitHash = FNV_OFFSET;
        for (int codeUnit = Character.MIN_VALUE;
                codeUnit <= Character.MAX_VALUE;
                codeUnit++) {
            singleCodeUnitHash = mix(singleCodeUnitHash, parseCode(String.valueOf((char) codeUnit)));
        }
        emit("exhaustive.singleCodeUnitHash", hex(singleCodeUnitHash));

        for (final StandardInlineMode mode : StandardInlineMode.values()) {
            final char[] chars = mode.name().toCharArray();
            for (int position = 0; position < chars.length; position++) {
                final char original = chars[position];
                long hash = FNV_OFFSET;
                for (int codeUnit = Character.MIN_VALUE;
                        codeUnit <= Character.MAX_VALUE;
                        codeUnit++) {
                    chars[position] = (char) codeUnit;
                    hash = mix(hash, parseCode(new String(chars)));
                }
                chars[position] = original;
                emit("exhaustive." + mode.name() + "." + position, hex(hash));
            }
        }
    }

    private static int parseCode(final String input) {
        try {
            return StandardInlineMode.parse(input).ordinal();
        } catch (final IllegalArgumentException exception) {
            if ("Inline mode cannot be null or empty".equals(exception.getMessage())) {
                return 6;
            }
            return 7;
        }
    }

    private static void emitParse(final String key, final String input) {
        try {
            emit("parse." + key, "OK:" + StandardInlineMode.parse(input));
        } catch (final RuntimeException exception) {
            emit("parse." + key,
                    "ERR:" + exception.getClass().getName() + ":" + exception.getMessage());
        }
    }

    private static void emitParseUtf16(final String key, final String input) {
        try {
            emit("parse." + key, "OK:" + StandardInlineMode.parse(input));
        } catch (final RuntimeException exception) {
            emit("parse." + key,
                    "ERR:" + exception.getClass().getName() + ":"
                            + toUtf16Hex(exception.getMessage()));
        }
    }

    private static String toUtf16Hex(final String value) {
        final StringBuilder result = new StringBuilder(value.length() * 5);
        for (int i = 0; i < value.length(); i++) {
            if (i > 0) {
                result.append(',');
            }
            result.append(String.format("%04x", (int) value.charAt(i)));
        }
        return result.toString();
    }

    private static long mix(final long hash, final int value) {
        return (hash ^ value) * FNV_PRIME;
    }

    private static String hex(final long value) {
        return String.format("%016x", value);
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }
}
