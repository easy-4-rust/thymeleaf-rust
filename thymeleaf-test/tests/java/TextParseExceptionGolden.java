package org.thymeleaf.templateparser.text;

/**
 * 从固定 Thymeleaf Java 源码导出 TextParseException Golden。
 */
public final class TextParseExceptionGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private TextParseExceptionGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        basicConstructors();
        causeConstructors();
        locationConstructors();
        inheritedLocationConstructors();
    }

    private static void basicConstructors() {
        emitException("default", new TextParseException());
        emitException("message", new TextParseException("problem"));
        emitException("nullMessage", new TextParseException((String) null));
        emitException("surrogateMessage", new TextParseException("\uD800"));
    }

    private static void causeConstructors() {
        final PlainThrowable cause = new PlainThrowable("cause");
        final PlainThrowable nullMessageCause = new PlainThrowable(null);
        emitException("messageCause", new TextParseException("outer", cause));
        emitException("nullMessageCause", new TextParseException(null, cause));
        emitException("messageNullCause", new TextParseException("outer", null));
        emitException("nullMessageNullCause", new TextParseException(null, null));
        emitException("cause", new TextParseException(cause));
        emitException("nullCause", new TextParseException((Throwable) null));
        emitException("nullCauseMessage", new TextParseException(nullMessageCause));
        final TextParseException caused = new TextParseException("outer", cause);
        emit("cause.identity", caused.getCause() == cause);
    }

    private static void locationConstructors() {
        emitException("location", new TextParseException(7, 11));
        emitException("negativeLocation", new TextParseException(-1, Integer.MIN_VALUE));
        emitException("messageLocation", new TextParseException("problem", 7, 11));
        emitException("nullMessageLocation", new TextParseException((String) null, 7, 11));
        final PlainThrowable cause = new PlainThrowable("cause");
        emitException("causeLocation", new TextParseException(cause, 7, 11));
        emitException("nullCauseLocation", new TextParseException((Throwable) null, 7, 11));
        emitException(
                "messageCauseLocation",
                new TextParseException("problem", cause, 7, 11));
        emitException(
                "nullMessageCauseLocation",
                new TextParseException(null, cause, 7, 11));
    }

    private static void inheritedLocationConstructors() {
        final TextParseException located = new TextParseException("inner", 3, 5);
        emitException("inherit.messageCause", new TextParseException("outer", located));
        emitException("inherit.nullMessageCause", new TextParseException(null, located));
        emitException("inherit.cause", new TextParseException(located));

        final TextParseException unlocated = new TextParseException("inner");
        emitException("inherit.unlocatedMessage", new TextParseException("outer", unlocated));
        emitException("inherit.unlocatedCause", new TextParseException(unlocated));

        final TextParseException nullLocated = new TextParseException((String) null, 3, 5);
        emitException("inherit.nullInnerMessage", new TextParseException(null, nullLocated));
    }

    private static void emitException(final String key, final TextParseException exception) {
        emit(
                key,
                toUtf16Hex(String.valueOf(exception.getMessage())) + ":"
                        + String.valueOf(exception.getLine()) + ":"
                        + String.valueOf(exception.getCol()) + ":"
                        + (exception.getCause() == null
                                ? "null"
                                : exception.getCause().getClass().getName()));
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

    private static final class PlainThrowable extends Exception {
        private PlainThrowable(final String message) {
            super(message);
        }
    }
}
