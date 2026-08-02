package org.thymeleaf.util;

import java.util.StringTokenizer;

/**
 * TemplateSpec Golden 编译所需的上游 StringUtils.split 精确切片。
 */
public final class StringUtils {

    private StringUtils() {
    }

    public static String[] split(final Object target, final String separator) {
        Validate.notNull(separator, "Separator cannot be null");
        if (target == null) {
            return null;
        }
        final StringTokenizer tokenizer = new StringTokenizer(target.toString(), separator);
        final String[] result = new String[tokenizer.countTokens()];
        for (int index = 0; index < result.length; index++) {
            result[index] = tokenizer.nextToken();
        }
        return result;
    }

    public static boolean isEmptyOrWhitespace(final String target) {
        if (target == null || target.length() == 0) {
            return true;
        }
        final char first = target.charAt(0);
        if ((first >= 'a' && first <= 'z') || (first >= 'A' && first <= 'Z')) {
            return false;
        }
        for (int index = 0; index < target.length(); index++) {
            final char character = target.charAt(index);
            if (character != ' ' && !Character.isWhitespace(character)) {
                return false;
            }
        }
        return true;
    }

    public static String replace(final Object target, final String before, final String after) {
        Validate.notNull(before, "Parameter before cannot be null");
        Validate.notNull(after, "Parameter after cannot be null");
        if (target == null) {
            return null;
        }
        return target.toString().replace(before, after);
    }
}
