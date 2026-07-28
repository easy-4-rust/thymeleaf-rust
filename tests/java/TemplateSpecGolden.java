import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Set;

import org.thymeleaf.TemplateSpec;
import org.thymeleaf.templatemode.TemplateMode;

/**
 * 从固定 Thymeleaf Java 源码生成 TemplateSpec 的可重复 Golden 输出。
 */
public final class TemplateSpecGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private TemplateSpecGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emitConstructorShapes();
        emitContentTypes();
        emitSelectorValidation();
        emitAttributeSemantics();
        emitEqualitySemantics();
        emitDisplaySemantics();
    }

    private static void emitConstructorShapes() {
        emitException("constructor.null_template", () -> new TemplateSpec(null, TemplateMode.HTML));

        final TemplateSpec mode = new TemplateSpec("index", TemplateMode.XML);
        emitSpec("constructor.mode", mode);

        final TemplateSpec content = new TemplateSpec("index", "text/html");
        emitSpec("constructor.content", content);

        final Map<String, Object> attributes = new HashMap<>();
        attributes.put("tenant", "acme");
        final TemplateSpec resolved = new TemplateSpec("index", attributes);
        emitSpec("constructor.attributes", resolved);

        final Set<String> selectors = new HashSet<>();
        selectors.add("main");
        final TemplateSpec selectedMode =
                new TemplateSpec("index", selectors, TemplateMode.RAW, attributes);
        emitSpec("constructor.selected_mode", selectedMode);

        final TemplateSpec selectedContent =
                new TemplateSpec("index", selectors, "text/css", attributes);
        emitSpec("constructor.selected_content", selectedContent);
    }

    private static void emitContentTypes() {
        final String[] contentTypes = new String[] {
                "text/html",
                "application/xhtml+xml",
                "application/xml",
                "text/xml",
                "application/rss+xml",
                "application/atom+xml",
                "application/javascript",
                "application/x-javascript",
                "application/ecmascript",
                "text/javascript",
                "text/ecmascript",
                "application/json",
                "text/css",
                "text/plain",
                "text/event-stream",
                "application/octet-stream",
                "",
                " \t",
                "; TEXT/HTML ;; Charset=UTF-8"
        };
        for (int index = 0; index < contentTypes.length; index++) {
            emitSpec("content." + index, new TemplateSpec("index", contentTypes[index]));
        }
        emitException("content.malformed", () -> new TemplateSpec("index", ";;;"));
    }

    private static void emitSelectorValidation() {
        final TemplateSpec nullSelectors =
                new TemplateSpec("index", null, TemplateMode.HTML, null);
        emitSpec("selectors.null", nullSelectors);
        emitSpec("selectors.empty",
                new TemplateSpec("index", new HashSet<>(), TemplateMode.HTML, null));

        final Set<String> ordered = new HashSet<>();
        ordered.add("footer");
        ordered.add("article");
        emitSpec("selectors.ordered",
                new TemplateSpec("index", ordered, TemplateMode.HTML, null));

        final Set<String> utf16Order = new HashSet<>();
        utf16Order.add("\uE000");
        utf16Order.add("\uD800\uDC00");
        emitSpec("selectors.utf16_order",
                new TemplateSpec("index", utf16Order, TemplateMode.HTML, null));

        final Set<String> nullElement = new HashSet<>();
        nullElement.add(null);
        emitException("selectors.null_element",
                () -> new TemplateSpec("index", nullElement, TemplateMode.HTML, null));

        for (final String invalid : new String[] {"", " \n"}) {
            final Set<String> invalidSet = new HashSet<>();
            invalidSet.add(invalid);
            emitException("selectors.invalid." + invalid.length(),
                    () -> new TemplateSpec("index", invalidSet, TemplateMode.HTML, null));
        }

        final Set<String> nonBreakingSpace = new HashSet<>();
        nonBreakingSpace.add("\u00A0");
        emitException("selectors.nbsp",
                () -> new TemplateSpec("index", nonBreakingSpace, TemplateMode.HTML, null));

        final Set<String> emSpace = new HashSet<>();
        emSpace.add("\u2003");
        emitException("selectors.em_space",
                () -> new TemplateSpec("index", emSpace, TemplateMode.HTML, null));
    }

    private static void emitAttributeSemantics() {
        emitSpec("attributes.empty", new TemplateSpec("index", new HashMap<>()));

        final Map<String, Object> source = new LinkedHashMap<>();
        source.put("tenant", "acme");
        source.put("attempt", Integer.valueOf(3));
        source.put(null, null);
        final TemplateSpec copied = new TemplateSpec("index", source);
        source.clear();

        emit("attributes.copied.size", copied.getTemplateResolutionAttributes().size());
        emit("attributes.copied.tenant", copied.getTemplateResolutionAttributes().get("tenant"));
        emit("attributes.copied.attempt", copied.getTemplateResolutionAttributes().get("attempt"));
        emit("attributes.copied.null_key",
                copied.getTemplateResolutionAttributes().containsKey(null));
        emit("attributes.copied.null_value",
                copied.getTemplateResolutionAttributes().get(null));
        emitException("attributes.unmodifiable",
                () -> copied.getTemplateResolutionAttributes().put("new", "value"));
    }

    private static void emitEqualitySemantics() {
        final TemplateSpec noContent = new TemplateSpec("index", (TemplateMode) null);
        final TemplateSpec sameNoContent = new TemplateSpec("index", (TemplateMode) null);
        emit("equals.identity_without_content", noContent.equals(noContent));
        emit("equals.null", noContent.equals(null));
        emit("equals.other_type", noContent.equals("index"));
        emitException("equals.null_content_bug", () -> noContent.equals(sameNoContent));

        final TemplateSpec base = new TemplateSpec("index", "text/html");
        final TemplateSpec same = new TemplateSpec("index", "text/html");
        emit("equals.same", base.equals(same));
        emit("equals.same_hash", base.hashCode() == same.hashCode());
        emit("equals.template",
                base.equals(new TemplateSpec("other", "text/html")));

        final Set<String> selectors = new HashSet<>();
        selectors.add("main");
        emit("equals.selectors",
                base.equals(new TemplateSpec("index", selectors, "text/html", null)));
        emit("equals.mode",
                noContent.equals(new TemplateSpec("index", TemplateMode.XML)));
        emit("equals.content",
                base.equals(new TemplateSpec("index", "application/xhtml+xml")));

        final Map<String, Object> attributes = new HashMap<>();
        attributes.put("tenant", "acme");
        emit("equals.attributes",
                base.equals(new TemplateSpec("index", null, "text/html", attributes)));
    }

    private static void emitDisplaySemantics() {
        final Set<String> selectors = new HashSet<>();
        selectors.add("footer");
        selectors.add("article");
        final Map<String, Object> attributes = new HashMap<>();
        attributes.put("tenant", "acme");
        final TemplateSpec complete =
                new TemplateSpec("home\npage", selectors, "text/html;charset=UTF-8", attributes);
        emit("display.complete", complete);

        emit("display.short", new TemplateSpec(repeat("x", 120), (TemplateMode) null));
        final String longName = repeat("a", 34) + "\n" + repeat("b", 90) + "z";
        emit("display.long", new TemplateSpec(longName, (TemplateMode) null));
    }

    private static void emitSpec(final String key, final TemplateSpec spec) {
        emit(key + ".template", spec.getTemplate());
        emit(key + ".has_selectors", spec.hasTemplateSelectors());
        emit(key + ".selectors", spec.getTemplateSelectors());
        emit(key + ".has_mode", spec.hasTemplateMode());
        emit(key + ".mode", spec.getTemplateMode());
        emit(key + ".has_attributes", spec.hasTemplateResolutionAttributes());
        emit(key + ".attributes", spec.getTemplateResolutionAttributes());
        emit(key + ".content_type", spec.getOutputContentType());
        emit(key + ".sse", spec.isOutputSSE());
    }

    private static void emitException(final String key, final Runnable operation) {
        try {
            operation.run();
            emit(key, "NO_EXCEPTION");
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getSimpleName() + ":" + exception.getMessage());
        }
    }

    private static String repeat(final String value, final int count) {
        final StringBuilder builder = new StringBuilder(value.length() * count);
        for (int index = 0; index < count; index++) {
            builder.append(value);
        }
        return builder.toString();
    }

    private static void emit(final String key, final Object value) {
        final String escaped = String.valueOf(value)
                .replace("\\", "\\\\")
                .replace("\t", "\\t")
                .replace("\r", "\\r")
                .replace("\n", "\\n");
        System.out.println(key + "=" + escaped);
    }
}
