import org.thymeleaf.standard.expression.Token;

/**
 * 从固定 Thymeleaf Java 源码导出 Token 与 TokenParsingTracer Golden。
 */
public final class TokenGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final long FNV_OFFSET = 0xcbf29ce484222325L;
    private static final long FNV_PRIME = 0x100000001b3L;

    private TokenGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        valueCases();
        exceptionCases();
        readableCharacterCases();
        exhaustiveCharacterCases();
        traceCases();
    }

    private static void valueCases() {
        final String source = new String("source");
        final TestToken stringToken = new TestToken(source);
        final SharedStringProbe probe = new SharedStringProbe();
        final TestToken objectToken = new TestToken(probe);

        emit("value.identity", stringToken.getValue() == source);
        emit("value.stringRepresentationIdentity", stringToken.getStringRepresentation() == source);
        emit("value.toStringIdentity", stringToken.toString() == source);
        emit("value.objectIdentity", objectToken.getValue() == probe);
        emit(
                "value.sharedRepresentationIdentity",
                objectToken.getStringRepresentation() == probe.shared);
        emit("value.sharedToStringIdentity", objectToken.toString() == probe.shared);
        emit("value.nullGet", new TestToken(null).getValue());
        emit("value.nullToStringResult", new TestToken(new NullStringProbe()).toString());
        emitClassOutcome(
                "value.nullFailure",
                () -> new TestToken(null).getStringRepresentation());
        emitOutcome(
                "value.runtimeFailure",
                () -> new TestToken(new ThrowingProbe()).getStringRepresentation());
    }

    private static void exceptionCases() {
        emitBooleanOutcome("char.null", () -> Token.isTokenChar(null, 0));
        emitBooleanOutcome("char.negative", () -> Token.isTokenChar("", -1));
        emitBooleanOutcome("char.empty", () -> Token.isTokenChar("", 0));
        emitBooleanOutcome("char.afterEnd", () -> Token.isTokenChar("a", 1));
        emitClassOutcome("trace.null", () -> Token.TokenParsingTracer.trace(null));
    }

    private static void readableCharacterCases() {
        final int[] boundaries = {
                0x0000, 0x000A, 0x0020, 0x002D, 0x002E, 0x0030, 0x0039,
                0x0041, 0x005A, 0x005B, 0x005D, 0x005F, 0x0061, 0x007A,
                0x00B6, 0x00B7, 0x00B8, 0x00BF, 0x00C0, 0x00D6, 0x00D7,
                0x00D8, 0x00F6, 0x00F7, 0x00F8, 0x02FF, 0x0300, 0x036F,
                0x0370, 0x037D, 0x037E, 0x037F, 0x1FFF, 0x2000, 0x200C,
                0x200D, 0x203E, 0x203F, 0x2040, 0x2041, 0x206F, 0x2070,
                0x218F, 0x2190, 0x2BFF, 0x2C00, 0x2FEF, 0x2FF0, 0x3000,
                0x3001, 0xD7FF, 0xD800, 0xF8FF, 0xF900, 0xFDCF, 0xFDD0,
                0xFDEF, 0xFDF0, 0xFFFD, 0xFFFE, 0xFFFF
        };
        final StringBuilder result = new StringBuilder(boundaries.length);
        for (final int boundary : boundaries) {
            result.append(Token.isTokenChar(String.valueOf((char) boundary), 0) ? '1' : '0');
        }
        emit("char.boundaries", result);

        final String[] dashContexts = {
                "-", "a-", "1-", "-a", "-1", "1-2", "a-1", "1-a",
                "--", "1--2", ".-.", "é-1", "1-é", "a - b", "a-+b",
                "foo-bar", "12.3-4", "12.-x", "x-.12"
        };
        for (int i = 0; i < dashContexts.length; i++) {
            emit(
                    "dash.trace." + i,
                    Token.TokenParsingTracer.trace(dashContexts[i]));
        }
    }

    private static void exhaustiveCharacterCases() {
        long singleHash = FNV_OFFSET;
        long leftDashHash = FNV_OFFSET;
        long rightDashHash = FNV_OFFSET;
        final char[] allBmp = new char[Character.MAX_VALUE + 1];

        for (int codeUnit = Character.MIN_VALUE;
                codeUnit <= Character.MAX_VALUE;
                codeUnit++) {
            final char value = (char) codeUnit;
            allBmp[codeUnit] = value;
            singleHash = mixBoolean(
                    singleHash,
                    Token.isTokenChar(String.valueOf(value), 0));
            leftDashHash = mixBoolean(
                    leftDashHash,
                    Token.isTokenChar(new String(new char[] {value, '-'}), 1));
            rightDashHash = mixBoolean(
                    rightDashHash,
                    Token.isTokenChar(new String(new char[] {'-', value}), 0));
        }

        emit("exhaustive.singleBmpHash", hex(singleHash));
        emit("exhaustive.leftDashBmpHash", hex(leftDashHash));
        emit("exhaustive.rightDashBmpHash", hex(rightDashHash));

        final String tracedBmp = Token.TokenParsingTracer.trace(new String(allBmp));
        emit("exhaustive.traceBmpHash", hex(hashString(tracedBmp)));

        long decisionHash = FNV_OFFSET;
        long traceHash = FNV_OFFSET;
        long state = 0x4d595df4d0f33173L;
        final char[] pool = {
                '-', '0', '1', '9', '.', 'a', 'Z', '_', '[', ']', ' ',
                '\n', '+', '#', '\u00B7', '\u00C0', '\u037E', '\u200C',
                '\uD800', '\uF900', '\uFFFD', '\uFFFF'
        };
        for (int sample = 0; sample < 20_000; sample++) {
            state = next(state);
            final int length = (int) (state >>> 60) + 1;
            final char[] units = new char[length];
            for (int i = 0; i < length; i++) {
                state = next(state);
                units[i] = pool[(int) Long.remainderUnsigned(state, pool.length)];
            }
            final String context = new String(units);
            for (int position = 0; position < length; position++) {
                decisionHash = mixBoolean(
                        decisionHash,
                        Token.isTokenChar(context, position));
            }
            traceHash = mixString(
                    traceHash,
                    Token.TokenParsingTracer.trace(context));
        }
        emit("exhaustive.contextDecisionHash", hex(decisionHash));
        emit("exhaustive.contextTraceHash", hex(traceHash));
    }

    private static void traceCases() {
        emit("trace.substitute", (int) Token.TokenParsingTracer.TOKEN_SUBSTITUTE);
        emit("trace.empty", Token.TokenParsingTracer.trace(""));
        emit(
                "trace.mixed",
                Token.TokenParsingTracer.trace("foo-bar + 12-3 -- .-. ${x}"));
        emit(
                "trace.utf16",
                toUtf16Hex(Token.TokenParsingTracer.trace("\u00b7\u037e\ud800\uf900")));
    }

    private static long next(final long state) {
        return state * 6364136223846793005L + 1442695040888963407L;
    }

    private static long mixBoolean(final long hash, final boolean value) {
        return (hash ^ (value ? 1L : 0L)) * FNV_PRIME;
    }

    private static long mixString(long hash, final String value) {
        for (int i = 0; i < value.length(); i++) {
            final char unit = value.charAt(i);
            hash = (hash ^ (unit & 0xffL)) * FNV_PRIME;
            hash = (hash ^ ((unit >>> 8) & 0xffL)) * FNV_PRIME;
        }
        return hash;
    }

    private static long hashString(final String value) {
        return mixString(FNV_OFFSET, value);
    }

    private static String hex(final long value) {
        return String.format("%016x", value);
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

    private static void emitBooleanOutcome(
            final String key,
            final BooleanOperation operation) {
        try {
            emit(key, "OK:" + operation.execute());
        } catch (final Throwable throwable) {
            emit(key, "ERR:" + throwable.getClass().getName());
        }
    }

    private static void emitOutcome(final String key, final Operation operation) {
        try {
            emit(key, "OK:" + String.valueOf(operation.execute()));
        } catch (final Throwable throwable) {
            emit(
                    key,
                    "ERR:"
                            + throwable.getClass().getName()
                            + ":"
                            + String.valueOf(throwable.getMessage()));
        }
    }

    private static void emitClassOutcome(final String key, final Operation operation) {
        try {
            emit(key, "OK:" + String.valueOf(operation.execute()));
        } catch (final Throwable throwable) {
            emit(key, "ERR:" + throwable.getClass().getName());
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private static final class TestToken extends Token {
        private TestToken(final Object value) {
            super(value);
        }
    }

    private static final class SharedStringProbe {
        private final String shared = new String("shared");

        @Override
        public String toString() {
            return this.shared;
        }
    }

    private static final class NullStringProbe {
        @Override
        public String toString() {
            return null;
        }
    }

    private static final class ThrowingProbe {
        @Override
        public String toString() {
            throw new IllegalStateException("boom");
        }
    }

    private interface BooleanOperation {
        boolean execute();
    }

    private interface Operation {
        Object execute();
    }
}
