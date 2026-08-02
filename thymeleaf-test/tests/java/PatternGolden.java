import java.util.LinkedHashSet;
import java.util.regex.Pattern;
import java.util.regex.PatternSyntaxException;

import org.thymeleaf.util.PatternSpec;
import org.thymeleaf.util.PatternUtils;

/**
 * 从固定 Thymeleaf Java 源码导出 PatternUtils 与 PatternSpec Golden。
 */
public final class PatternGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private PatternGolden() {
    }

    public static void main(final String[] args) throws Exception {
        emit("baseline", BASELINE);

        emitPattern("glob", "*.html",
                new String[] {"index.html", "path/view.html", "index.htm"});
        emitPattern("escaped", "a.(b)[c]?$+*",
                new String[] {"a.(b)[c]?$+tail", "xa.(b)[c]?$+tail"});
        emitPattern("alternation", "foo|bar",
                new String[] {"foo", "bar", "foobar"});
        emitPattern("quantifier", "a{2}",
                new String[] {"aa", "a", "aaa"});
        emitPattern("digit", "\\d*",
                new String[] {"1tail", "\u0661tail", "xtail"});
        emitPattern("star", "*",
                new String[] {"plain", "line\nbreak", "line\rbreak", "line\u0085break",
                        "line\u2028break"});
        emitPattern("trailing_escape", "abc\\",
                new String[] {"abc$", "abc"});
        emitPattern("empty", "",
                new String[] {"", "value"});
        emitPattern("quoted", "\\Qfoo|*\\E",
                new String[] {"foo|(?:.*?)", "foo|anything", "foo|*"});
        emitPattern("unterminated_quote", "\\Qfoo",
                new String[] {"foo$", "foo"});

        for (final String regexClass : new String[] {
                "\\D", "\\w", "\\W", "\\s", "\\S", "\\h", "\\H", "\\v", "\\V", "\\R"
        }) {
            final String input;
            if ("\\s".equals(regexClass)) {
                input = "\t";
            } else if ("\\h".equals(regexClass)) {
                input = "\u3000";
            } else if ("\\v".equals(regexClass)) {
                input = "\u2028";
            } else if ("\\R".equals(regexClass)) {
                input = "\r\n";
            } else if ("\\W".equals(regexClass)) {
                input = "-";
            } else if ("\\w".equals(regexClass)) {
                input = "_";
            } else {
                input = "x";
            }
            emit("pattern.class." + printable(regexClass),
                    Boolean.toString(PatternUtils.strPatternToPattern(regexClass)
                            .matcher(input).matches()));
        }

        emitExceptionClass("pattern.null",
                () -> PatternUtils.strPatternToPattern(null));
        try {
            PatternUtils.strPatternToPattern("{");
            emit("pattern.syntax", "OK");
        } catch (final PatternSyntaxException exception) {
            emit("pattern.syntax",
                    exception.getClass().getName() + ":" + exception.getPattern());
        }

        final PatternSpec empty = new PatternSpec();
        emit("spec.new.empty", Boolean.toString(empty.isEmpty()));
        emit("spec.new.patterns", empty.getPatterns().toString());
        emit("spec.new.null_match", Boolean.toString(empty.matches(null)));
        emitExceptionClass("spec.new.unmodifiable",
                () -> empty.getPatterns().add("forbidden"));

        final PatternSpec set = new PatternSpec();
        final LinkedHashSet<String> ordered = new LinkedHashSet<String>();
        ordered.add("*.html");
        ordered.add("admin/*");
        set.setPatterns(ordered);
        emit("spec.set.patterns", set.getPatterns().toString());
        emit("spec.set.html", Boolean.toString(set.matches("index.html")));
        emit("spec.set.admin", Boolean.toString(set.matches("admin/users")));
        emit("spec.set.miss", Boolean.toString(set.matches("index.htm")));

        set.addPattern("*.html");
        emit("spec.duplicate.patterns", set.getPatterns().toString());

        final PatternSpec validation = new PatternSpec();
        emitException("spec.add.null", () -> validation.addPattern(null));
        emitException("spec.add.empty", () -> validation.addPattern(""));
        emitException("spec.add.whitespace", () -> validation.addPattern("\u2008"));
        emit("spec.add.validation_patterns", validation.getPatterns().toString());

        final PatternSpec addSyntax = new PatternSpec();
        emitExceptionClass("spec.add.syntax", () -> addSyntax.addPattern("{"));
        emit("spec.add.syntax_patterns", addSyntax.getPatterns().toString());
        emit("spec.add.syntax_empty", Boolean.toString(addSyntax.isEmpty()));

        final PatternSpec setSyntax = new PatternSpec();
        final LinkedHashSet<String> syntaxPatterns = new LinkedHashSet<String>();
        syntaxPatterns.add("*.html");
        syntaxPatterns.add("{");
        syntaxPatterns.add("*.txt");
        emitExceptionClass("spec.set.syntax", () -> setSyntax.setPatterns(syntaxPatterns));
        emit("spec.set.syntax_patterns", setSyntax.getPatterns().toString());
        emit("spec.set.syntax_html", Boolean.toString(setSyntax.matches("view.html")));
        emit("spec.set.syntax_txt", Boolean.toString(setSyntax.matches("view.txt")));

        final PatternSpec setNull = new PatternSpec();
        final LinkedHashSet<String> nullPatterns = new LinkedHashSet<String>();
        nullPatterns.add("*.html");
        nullPatterns.add(null);
        nullPatterns.add("*.txt");
        emitExceptionClass("spec.set.null_element", () -> setNull.setPatterns(nullPatterns));
        emit("spec.set.null_patterns", setNull.getPatterns().toString());

        final PatternSpec nullMatch = new PatternSpec();
        nullMatch.addPattern("*");
        emitExceptionClass("spec.matches.null", () -> nullMatch.matches(null));
        nullMatch.clearPatterns();
        emit("spec.clear.empty", Boolean.toString(nullMatch.isEmpty()));
        emit("spec.clear.patterns", nullMatch.getPatterns().toString());
        emit("spec.clear.null_match", Boolean.toString(nullMatch.matches(null)));
        nullMatch.setPatterns(ordered);
        nullMatch.setPatterns(null);
        emit("spec.set_null.empty", Boolean.toString(nullMatch.isEmpty()));
        emit("spec.set_null.patterns", nullMatch.getPatterns().toString());
    }

    private static void emitPattern(
            final String key, final String source, final String[] inputs) {
        final Pattern pattern = PatternUtils.strPatternToPattern(source);
        emit("pattern." + key + ".source", pattern.pattern());
        for (int i = 0; i < inputs.length; i++) {
            emit("pattern." + key + "." + i,
                    Boolean.toString(pattern.matcher(inputs[i]).matches()));
        }
    }

    private static String printable(final String value) {
        return value.substring(1);
    }

    private static void emitException(final String key, final ThrowingRunnable action) {
        try {
            action.run();
            emit(key, "OK");
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getName() + ":" + String.valueOf(exception.getMessage()));
        }
    }

    private static void emitExceptionClass(final String key, final ThrowingRunnable action) {
        try {
            action.run();
            emit(key, "OK");
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getName());
        }
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private interface ThrowingRunnable {
        void run();
    }
}
