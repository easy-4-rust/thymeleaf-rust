import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Set;
import java.util.TreeSet;

import org.thymeleaf.cache.ExpressionCacheKey;
import org.thymeleaf.cache.TemplateCacheKey;
import org.thymeleaf.templatemode.TemplateMode;

/**
 * 从固定 Thymeleaf Java 源码导出表达式与模板缓存键 Golden。
 */
public final class CacheKeyGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private CacheKeyGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        exportExpressionCacheKey();
        exportTemplateCacheKey();
    }

    private static void exportExpressionCacheKey() {
        final ExpressionCacheKey basic = new ExpressionCacheKey("EXPRESSION", "😀");
        emit("expression.basic.type", basic.getType());
        emit("expression.basic.expression0", basic.getExpression0());
        emit("expression.basic.expression1", basic.getExpression1());
        emit("expression.basic.string", basic.toString());
        emit("expression.basic.hash", basic.hashCode());

        final ExpressionCacheKey complete =
                new ExpressionCacheKey("PREPROCESS", "${user}", "*{name}");
        emit("expression.complete.expression1", complete.getExpression1());
        emit("expression.complete.string", complete.toString());
        emit("expression.complete.hash", complete.hashCode());

        final ExpressionCacheKey empty = new ExpressionCacheKey("", "", "");
        emit("expression.empty.string", empty.toString());
        emit("expression.empty.hash", empty.hashCode());

        emitFailure("expression.null_type", () -> new ExpressionCacheKey(null, "x"));
        emitFailure("expression.null_expression0", () -> new ExpressionCacheKey("T", null));

        final ExpressionCacheKey same =
                new ExpressionCacheKey("PREPROCESS", "${user}", "*{name}");
        emit("expression.equals.same", complete.equals(same));
        emit("expression.hash.same", complete.hashCode() == same.hashCode());
        emit(
                "expression.equals.type",
                complete.equals(new ExpressionCacheKey("OTHER", "${user}", "*{name}")));
        emit(
                "expression.equals.expression0",
                complete.equals(new ExpressionCacheKey("PREPROCESS", "other", "*{name}")));
        emit(
                "expression.equals.expression1",
                complete.equals(new ExpressionCacheKey("PREPROCESS", "${user}", "other")));
        emit("expression.equals.null_expression1", complete.equals(
                new ExpressionCacheKey("PREPROCESS", "${user}")));
        emit("expression.equals.other_type", complete.equals("PREPROCESS"));
        emit("expression.equals.null", complete.equals(null));

        final ExpressionCacheKey collision0 = new ExpressionCacheKey("T", "Aa");
        final ExpressionCacheKey collision1 = new ExpressionCacheKey("T", "BB");
        emit("expression.collision.hash", collision0.hashCode() == collision1.hashCode());
        emit("expression.collision.equals", collision0.equals(collision1));
    }

    private static void exportTemplateCacheKey() {
        emitFailure(
                "template.null_template",
                () -> new TemplateCacheKey(null, null, null, 0, 0, null, null));

        final TemplateCacheKey plain =
                new TemplateCacheKey(null, "", null, Integer.MIN_VALUE, Integer.MAX_VALUE, null, null);
        emit("template.plain.owner", plain.getOwnerTemplate());
        emit("template.plain.template", plain.getTemplate());
        emit("template.plain.selectors", plain.getTemplateSelectors());
        emit("template.plain.line", plain.getLineOffset());
        emit("template.plain.col", plain.getColOffset());
        emit("template.plain.mode", plain.getTemplateMode());
        emit("template.plain.attributes", plain.getTemplateResolutionAttributes());
        emit("template.plain.string", plain.toString());

        final Set<String> emptySelectors = new TreeSet<>();
        final Map<String,Object> emptyAttributes = new LinkedHashMap<>();
        final TemplateCacheKey emptyCollections =
                new TemplateCacheKey(null, "", emptySelectors, 0, 0, null, emptyAttributes);
        emit("template.empty.selectors_identity",
                emptyCollections.getTemplateSelectors() == emptySelectors);
        emit("template.empty.attributes_identity",
                emptyCollections.getTemplateResolutionAttributes() == emptyAttributes);
        emit("template.empty.string", emptyCollections.toString());
        emit("template.empty.equals_null_collections", emptyCollections.equals(
                new TemplateCacheKey(null, "", null, 0, 0, null, null)));

        final Set<String> selectors = selectors("footer", "article", "\uE000", "\uD800\uDC00");
        final Map<String,Object> attributes = attributes("tenant", "acme");
        final TemplateCacheKey full = new TemplateCacheKey(
                "owner\nname", "page\nname", selectors, -2, 7, TemplateMode.XML, attributes);
        emit("template.full.owner", full.getOwnerTemplate());
        emit("template.full.template", full.getTemplate());
        emit("template.full.selectors_identity", full.getTemplateSelectors() == selectors);
        emit("template.full.line", full.getLineOffset());
        emit("template.full.col", full.getColOffset());
        emit("template.full.mode", full.getTemplateMode());
        emit("template.full.attributes_identity",
                full.getTemplateResolutionAttributes() == attributes);
        emit("template.full.string", full.toString());

        final TemplateCacheKey same = new TemplateCacheKey(
                "owner\nname", "page\nname",
                selectors("\uD800\uDC00", "\uE000", "article", "footer"),
                -2, 7, TemplateMode.XML, attributes("tenant", "acme"));
        emit("template.equals.same", full.equals(same));
        emit("template.hash.same", full.hashCode() == same.hashCode());
        emit("template.equals.line", full.equals(new TemplateCacheKey(
                "owner\nname", "page\nname", selectors, -1, 7, TemplateMode.XML, attributes)));
        emit("template.equals.col", full.equals(new TemplateCacheKey(
                "owner\nname", "page\nname", selectors, -2, 8, TemplateMode.XML, attributes)));
        emit("template.equals.owner", full.equals(new TemplateCacheKey(
                "other", "page\nname", selectors, -2, 7, TemplateMode.XML, attributes)));
        emit("template.equals.template", full.equals(new TemplateCacheKey(
                "owner\nname", "other", selectors, -2, 7, TemplateMode.XML, attributes)));
        emit("template.equals.selectors", full.equals(new TemplateCacheKey(
                "owner\nname", "page\nname", selectors("other"), -2, 7, TemplateMode.XML, attributes)));
        emit("template.equals.mode", full.equals(new TemplateCacheKey(
                "owner\nname", "page\nname", selectors, -2, 7, TemplateMode.HTML, attributes)));
        emit("template.equals.attributes", full.equals(new TemplateCacheKey(
                "owner\nname", "page\nname", selectors, -2, 7, TemplateMode.XML,
                attributes("tenant", "other"))));
        emit("template.equals.other_type", full.equals("page"));
        emit("template.equals.null", full.equals(null));
    }

    private static Set<String> selectors(final String... values) {
        final Set<String> selectors = new TreeSet<>();
        for (final String value : values) {
            selectors.add(value);
        }
        return selectors;
    }

    private static Map<String,Object> attributes(final String key, final Object value) {
        final Map<String,Object> attributes = new LinkedHashMap<>();
        attributes.put(key, value);
        return attributes;
    }

    private static void emitFailure(final String key, final Operation operation) {
        try {
            operation.run();
            emit(key + ".class", "<none>");
            emit(key + ".message", "<none>");
        } catch (final RuntimeException exception) {
            emit(key + ".class", exception.getClass().getSimpleName());
            emit(key + ".message", exception.getMessage());
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + escape(String.valueOf(value)));
    }

    private static String escape(final String value) {
        return value.replace("\\", "\\\\").replace("\n", "\\n");
    }

    private interface Operation {
        void run();
    }
}
