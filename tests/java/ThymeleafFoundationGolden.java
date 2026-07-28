import org.thymeleaf.exceptions.AlreadyInitializedException;
import org.thymeleaf.exceptions.CacheConfigurationException;
import org.thymeleaf.exceptions.ConfigurationException;
import org.thymeleaf.exceptions.ParserInitializationException;
import org.thymeleaf.exceptions.TemplateAssertionException;
import org.thymeleaf.exceptions.TemplateInputException;
import org.thymeleaf.exceptions.TemplateOutputException;
import org.thymeleaf.exceptions.TemplateProcessingException;
import org.thymeleaf.templatemode.TemplateMode;

/**
 * 从固定 Thymeleaf Java 源码生成基础对象的可重复 Golden 输出。
 */
public final class ThymeleafFoundationGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private ThymeleafFoundationGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emitTemplateModes();
        emitSimpleExceptions();
        emitTemplateProcessingException();
        emitTemplateAssertionException();
        emitTemplateInputException();
        emitTemplateOutputException();
    }

    private static void emitTemplateModes() {
        for (final TemplateMode mode : TemplateMode.values()) {
            emit("mode." + mode + ".flags",
                    mode.isMarkup() + "," + mode.isText() + "," + mode.isCaseSensitive());
            emit("mode." + mode + ".display", mode.toString());
        }
        emitParsedMode("null", null);
        emitParsedMode("empty", "");
        emitParsedMode("blank", " \n\t");
        emitParsedMode("html", "html");
        emitParsedMode("XML", "XML");
        emitParsedMode("Text", "Text");
        emitParsedMode("javascript", "javascript");
        emitParsedMode("Css", "Css");
        emitParsedMode("raw", "raw");
        emitParsedMode("unknown", "MARKDOWN");
        emitParsedMode("padded_xml", " XML ");
    }

    private static void emitParsedMode(final String key, final String value) {
        try {
            emit("parse." + key, TemplateMode.parse(value));
        } catch (final RuntimeException exception) {
            emit("parse." + key,
                    exception.getClass().getSimpleName() + ":" + exception.getMessage());
        }
    }

    private static void emitSimpleExceptions() {
        final Throwable cause = new IllegalStateException("cause");

        final AlreadyInitializedException already =
                new AlreadyInitializedException("initialized", cause);
        emit("already.message", already.getMessage());
        emit("already.cause", already.getCause().getMessage());
        emit("already.null", new AlreadyInitializedException(null).getMessage());

        final ConfigurationException configuration =
                new ConfigurationException("configuration", cause);
        emit("configuration.message", configuration.getMessage());
        emit("configuration.cause", configuration.getCause().getMessage());
        emit("configuration.null", new ConfigurationException(null).getMessage());

        final CacheConfigurationException cache =
                new CacheConfigurationException("cache", cause);
        emit("cache.message", cache.getMessage());
        emit("cache.cause", cache.getCause().getMessage());
        emit("cache.null", new CacheConfigurationException(null).getMessage());

        final ParserInitializationException parser =
                new ParserInitializationException("parser", cause);
        emit("parser.message", parser.getMessage());
        emit("parser.cause", parser.getCause().getMessage());
        emit("parser.null", new ParserInitializationException(null).getMessage());
    }

    private static void emitTemplateProcessingException() {
        final Throwable cause = new IllegalStateException("cause");

        final TemplateProcessingException plain = new TemplateProcessingException("problem");
        emit("processing.plain.message", plain.getMessage());
        emit("processing.plain.template", plain.getTemplateName());
        emit("processing.plain.has_template", plain.hasTemplateName());
        emit("processing.plain.line", plain.getLine());
        emit("processing.plain.col", plain.getCol());
        emit("processing.plain.has_line_col", plain.hasLineAndCol());

        final TemplateProcessingException nullMessage = new TemplateProcessingException(null);
        emit("processing.null.message", nullMessage.getMessage());

        final TemplateProcessingException caused =
                new TemplateProcessingException("problem", cause);
        emit("processing.caused.cause", caused.getCause().getMessage());

        final TemplateProcessingException templateCaused =
                new TemplateProcessingException("problem", "index.html", cause);
        emit("processing.template_cause.message", templateCaused.getMessage());
        emit("processing.template_cause.cause", templateCaused.getCause().getMessage());

        emitProcessingLocation("complete", 7, 11);
        emitProcessingLocation("line_only", 7, -1);
        emitProcessingLocation("col_only", -1, 11);
        emitProcessingLocation("no_location", -1, -1);

        final TemplateProcessingException hiddenLocation =
                new TemplateProcessingException("problem", null, 1, 2);
        emit("processing.hidden_location.message", hiddenLocation.getMessage());

        final TemplateProcessingException mutable =
                new TemplateProcessingException("problem", "old.html", 1, 2, cause);
        mutable.setTemplateName("new.html");
        mutable.setLineAndCol(-1, 9);
        emit("processing.mutable.message", mutable.getMessage());
        emit("processing.mutable.template", mutable.getTemplateName());
        emit("processing.mutable.line", mutable.getLine());
        emit("processing.mutable.col", mutable.getCol());
        emit("processing.mutable.has_line_col", mutable.hasLineAndCol());
        emit("processing.mutable.cause", mutable.getCause().getMessage());
    }

    private static void emitProcessingLocation(final String key, final int line, final int col) {
        final TemplateProcessingException exception =
                new TemplateProcessingException("problem", "index.html", line, col);
        emit("processing." + key + ".message", exception.getMessage());
        emit("processing." + key + ".line", exception.getLine());
        emit("processing." + key + ".col", exception.getCol());
        emit("processing." + key + ".has_line_col", exception.hasLineAndCol());
    }

    private static void emitTemplateAssertionException() {
        emit("assertion.plain",
                new TemplateAssertionException("${user != null}", "index.html").getMessage());
        emit("assertion.located",
                new TemplateAssertionException("${user != null}", "index.html", 7, 3)
                        .getMessage());
        emit("assertion.null", new TemplateAssertionException(null, null).getMessage());
    }

    private static void emitTemplateInputException() {
        final Throwable cause = new IllegalStateException("cause");
        emit("input.plain", new TemplateInputException("input").getMessage());
        emit("input.caused.cause",
                new TemplateInputException("input", cause).getCause().getMessage());
        emit("input.template_cause",
                new TemplateInputException("input", "index.html", cause).getMessage());
        emit("input.location",
                new TemplateInputException("input", "index.html", 3, 4).getMessage());
        final TemplateInputException locatedCause =
                new TemplateInputException("input", "index.html", 5, 6, cause);
        emit("input.location_cause.message", locatedCause.getMessage());
        emit("input.location_cause.cause", locatedCause.getCause().getMessage());
    }

    private static void emitTemplateOutputException() {
        final TemplateOutputException output = new TemplateOutputException(
                "output", "index.html", 9, 10, new IllegalStateException("writer"));
        emit("output.message", output.getMessage());
        emit("output.template", output.getTemplateName());
        emit("output.line", output.getLine());
        emit("output.col", output.getCol());
        emit("output.has_line_col", output.hasLineAndCol());
        emit("output.cause", output.getCause().getMessage());
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }
}
