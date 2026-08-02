import org.thymeleaf.util.VersionUtils;

/**
 * 从固定 Thymeleaf Java 源码导出 VersionUtils 与 VersionSpec Golden。
 */
public final class VersionUtilsGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private VersionUtilsGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        emitSpec("null", VersionUtils.parseVersion(null));
        emitSpec("null_build", VersionUtils.parseVersion(null, "build-1"));
        emitSpec("empty", VersionUtils.parseVersion(""));
        emitSpec("ascii_blank", VersionUtils.parseVersion("\u0000\t "));
        emitSpec("nbsp", VersionUtils.parseVersion("\u00A0"));
        emitSpec("major", VersionUtils.parseVersion("7"));
        emitSpec("minor", VersionUtils.parseVersion("7.2"));
        emitSpec("patch", VersionUtils.parseVersion("7.2.4"));
        emitSpec("trimmed", VersionUtils.parseVersion(" \t3.1.5.RELEASE\r\n"));
        emitSpec("release_joined", VersionUtils.parseVersion("3.1.5RELEASE"));
        emitSpec("release_dash", VersionUtils.parseVersion("3.1.5-RELEASE"));
        emitSpec("release_lower", VersionUtils.parseVersion("3.1.5-release"));
        emitSpec("rc", VersionUtils.parseVersion("3.1.5.RC1"));
        emitSpec("letter", VersionUtils.parseVersion("2beta"));
        emitSpec("underscore", VersionUtils.parseVersion("2_beta"));
        emitSpec("one_dot_rc", VersionUtils.parseVersion("1.RC1"));
        emitSpec("leading_zeroes", VersionUtils.parseVersion("001.02.003"));
        emitSpec("max", VersionUtils.parseVersion("2147483647"));
        emitSpec("overflow", VersionUtils.parseVersion("2147483648"));
        emitSpec("negative", VersionUtils.parseVersion("-1"));
        emitSpec("trailing_dot", VersionUtils.parseVersion("1."));
        emitSpec("trailing_dash", VersionUtils.parseVersion("1-"));
        emitSpec("double_dot", VersionUtils.parseVersion("1..2"));
        emitSpec("four_parts", VersionUtils.parseVersion("1.2.3.4"));
        emitSpec("separator_blank", VersionUtils.parseVersion("1-\t"));
        emitSpec("separator_nbsp", VersionUtils.parseVersion("1-\u00A0"));
        emitSpec("qualifier_space", VersionUtils.parseVersion("1- RC "));
        emitSpec("arabic_digits", VersionUtils.parseVersion("\u0661.\u0662.\u0663"));
        emitSpec("fullwidth_digits", VersionUtils.parseVersion("\uFF19.\uFF18\u03B2"));
        emitSpec("unicode_modifier_letter", VersionUtils.parseVersion("1\u02B0"));
        emitSpec("unicode_mark_separator", VersionUtils.parseVersion("1\u0301mark"));
        emitSpec("supplementary_letter", VersionUtils.parseVersion("1\uD801\uDC00"));
        emitSpec("leading_dot_qualifier", VersionUtils.parseVersion(".RC"));
        emitSpec("empty_build", VersionUtils.parseVersion("1.2", ""));
        emitSpec("full_build", VersionUtils.parseVersion("1.2", "2026-07-29T00:00:00Z"));

        emit("character.digit_ranges", characterRanges(true));
        emit("character.letter_ranges", characterRanges(false));
    }

    private static void emitSpec(
            final String key, final VersionUtils.VersionSpec versionSpec) {
        final String prefix = "version." + key + ".";
        emit(prefix + "unknown", Boolean.toString(versionSpec.isUnknown()));
        emit(prefix + "major", Integer.toString(versionSpec.getMajor()));
        emit(prefix + "minor", Integer.toString(versionSpec.getMinor()));
        emit(prefix + "patch", Integer.toString(versionSpec.getPatch()));
        emit(prefix + "has_qualifier", Boolean.toString(versionSpec.hasQualifier()));
        emit(prefix + "qualifier", encode(versionSpec.getQualifier()));
        emit(prefix + "core", encode(versionSpec.getVersionCore()));
        emit(prefix + "version", encode(versionSpec.getVersion()));
        emit(prefix + "has_build", Boolean.toString(versionSpec.hasBuildTimestamp()));
        emit(prefix + "build", encode(versionSpec.getBuildTimestamp()));
        emit(prefix + "full", encode(versionSpec.getFullVersion()));
        emit(prefix + "stable", Boolean.toString(versionSpec.isStableRelease()));
        emit(prefix + "at_least_neg1", Boolean.toString(versionSpec.isAtLeast(-1)));
        emit(prefix + "at_least_0", Boolean.toString(versionSpec.isAtLeast(0)));
        emit(prefix + "at_least_3", Boolean.toString(versionSpec.isAtLeast(3)));
        emit(prefix + "at_least_3_1", Boolean.toString(versionSpec.isAtLeast(3, 1)));
        emit(prefix + "at_least_3_1_5", Boolean.toString(versionSpec.isAtLeast(3, 1, 5)));
        emit(prefix + "at_least_3_1_6", Boolean.toString(versionSpec.isAtLeast(3, 1, 6)));
        emit(prefix + "at_least_4", Boolean.toString(versionSpec.isAtLeast(4)));
    }

    private static String encode(final String value) {
        if (value == null) {
            return "<null>";
        }
        final StringBuilder encoded = new StringBuilder();
        for (int i = 0; i < value.length(); i++) {
            if (i > 0) {
                encoded.append(',');
            }
            encoded.append(String.format("%04X", Integer.valueOf(value.charAt(i))));
        }
        return encoded.toString();
    }

    private static String characterRanges(final boolean digits) {
        final StringBuilder ranges = new StringBuilder();
        int start = -1;
        int end = -1;
        for (int codeUnit = 0; codeUnit <= Character.MAX_VALUE; codeUnit++) {
            final char character = (char) codeUnit;
            final boolean matches = digits
                    ? Character.isDigit(character)
                    : Character.isLetter(character);
            if (matches) {
                if (start < 0) {
                    start = codeUnit;
                }
                end = codeUnit;
            } else if (start >= 0) {
                appendRange(ranges, start, end);
                start = -1;
            }
        }
        if (start >= 0) {
            appendRange(ranges, start, end);
        }
        return ranges.toString();
    }

    private static void appendRange(
            final StringBuilder ranges, final int start, final int end) {
        if (ranges.length() > 0) {
            ranges.append(';');
        }
        ranges.append(String.format("%04X", Integer.valueOf(start)));
        if (end != start) {
            ranges.append('-');
            ranges.append(String.format("%04X", Integer.valueOf(end)));
        }
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }
}
