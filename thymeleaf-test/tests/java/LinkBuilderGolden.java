import java.lang.reflect.Proxy;
import java.math.BigDecimal;
import java.math.BigInteger;
import java.util.Arrays;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Map;
import java.util.Set;

import org.thymeleaf.IEngineConfiguration;
import org.thymeleaf.context.IExpressionContext;
import org.thymeleaf.context.IWebContext;
import org.thymeleaf.expression.IExpressionObjects;
import org.thymeleaf.linkbuilder.StandardLinkBuilder;
import org.thymeleaf.web.IWebExchange;
import org.thymeleaf.web.IWebRequest;

/**
 * 从固定 Thymeleaf 3.1.5.RELEASE 导出标准链接构建器合同。
 */
public final class LinkBuilderGolden {

    private static final String JAVA_BASELINE =
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final IExpressionContext NON_WEB_CONTEXT = new NonWebContext();

    private LinkBuilderGolden() {
    }

    public static void main(final String[] args) {
        emit("java_baseline", JAVA_BASELINE);
        exportAbstractState();
        exportValidationAndClassification();
        exportQueryParameters();
        exportTemplateParameters();
        exportEscaping();
        exportWebAndExtensionPoints();
    }

    private static void exportAbstractState() {
        final StandardLinkBuilder builder = new StandardLinkBuilder();
        emit("state.name.default", builder.getName());
        emit("state.order.default", builder.getOrder());
        builder.setName(null);
        builder.setOrder(-17);
        emit("state.name.null", builder.getName());
        emit("state.order.negative", builder.getOrder());
    }

    private static void exportValidationAndClassification() {
        final StandardLinkBuilder builder = new StandardLinkBuilder();
        exportFailure("validation.context.null",
                () -> builder.buildLink(null, "/x", null));
        emit("validation.base.null",
                builder.buildLink(NON_WEB_CONTEXT, null, null));
        emitBuild("classification.empty", builder, NON_WEB_CONTEXT, "", null);
        emitBuild("classification.base", builder, NON_WEB_CONTEXT, "relative/path", null);
        emitBuild("classification.absolute.http", builder, NON_WEB_CONTEXT,
                "https://example.org/x", null);
        emitBuild("classification.absolute.embedded_scheme", builder, NON_WEB_CONTEXT,
                "prefix:https://example.org/x", null);
        emitBuild("classification.absolute.protocol_relative", builder, NON_WEB_CONTEXT,
                "//example.org/x", null);
        emitBuild("classification.absolute.mailto_case", builder, NON_WEB_CONTEXT,
                "MaIlTo:user@example.org", null);
        emitBuild("classification.server_relative", builder, NON_WEB_CONTEXT,
                "~/root/path", null);
        emitBuild("classification.fragment.last", builder, NON_WEB_CONTEXT,
                "path#first#last", map("q", "x"));
        emitBuild("classification.fragment.zero", builder, NON_WEB_CONTEXT,
                "#fragment", map("q", "x"));
        exportFailure("security.javascript.lower",
                () -> builder.buildLink(NON_WEB_CONTEXT, "javascript:alert(1)", null));
        exportFailure("security.javascript.mixed",
                () -> builder.buildLink(NON_WEB_CONTEXT, "JaVaScRiPt:alert(1)", null));
        emitBuild("security.javascript.leading_space", builder, NON_WEB_CONTEXT,
                " javascript:alert(1)", null);
        emitBuild("security.javascript_similar", builder, NON_WEB_CONTEXT,
                "javascriptx:alert(1)", null);
        exportFailure("classification.context_non_web",
                () -> builder.buildLink(NON_WEB_CONTEXT, "/context/path", null));
    }

    private static void exportQueryParameters() {
        final StandardLinkBuilder builder = new StandardLinkBuilder();
        emitBuild("query.scalar", builder, NON_WEB_CONTEXT, "path",
                map("name", "a b"));
        emitBuild("query.existing", builder, NON_WEB_CONTEXT, "path?fixed=yes",
                map("name", "a=b&c+d#e"));
        emitBuild("query.null_value", builder, NON_WEB_CONTEXT, "path",
                map("flag", null));
        emitBuild("query.empty_string", builder, NON_WEB_CONTEXT, "path",
                map("empty", ""));
        emitBuild("query.list", builder, NON_WEB_CONTEXT, "path",
                map("item", Arrays.asList("one", null, "two")));
        emitBuild("query.empty_list", builder, NON_WEB_CONTEXT, "path",
                map("item", Collections.emptyList()));
        final LinkedHashMap<String,Object> emptyThenNext = new LinkedHashMap<String,Object>();
        emptyThenNext.put("empty", Collections.emptyList());
        emptyThenNext.put("next", "value");
        emitBuild("query.empty_list_then_value", builder, NON_WEB_CONTEXT, "path",
                emptyThenNext);
        emitBuild("query.null_key", builder, NON_WEB_CONTEXT, "path",
                map(null, "value"));
        final LinkedHashMap<String,Object> ordered = new LinkedHashMap<String,Object>();
        ordered.put("first", "1");
        ordered.put("second", null);
        ordered.put("third", Arrays.asList("3", "4"));
        emitBuild("query.insertion_order", builder, NON_WEB_CONTEXT, "path", ordered);
        final LinkedHashMap<String,Object> defensive = map("id", "7");
        emitBuild("query.defensive.result", builder, NON_WEB_CONTEXT, "path/{id}", defensive);
        emit("query.defensive.size", defensive.size());
        emit("query.defensive.value", defensive.get("id"));
        emitBuild("query.number_types", builder, NON_WEB_CONTEXT, "path",
                map("n", Arrays.asList(
                        Long.valueOf(Long.MAX_VALUE),
                        new BigInteger("123456789012345678901234567890"),
                        new BigDecimal("1.2300"),
                        Boolean.TRUE)));
    }

    private static void exportTemplateParameters() {
        final StandardLinkBuilder builder = new StandardLinkBuilder();
        emitBuild("template.path", builder, NON_WEB_CONTEXT, "orders/{id}",
                map("id", "a/b c"));
        emitBuild("template.segment", builder, NON_WEB_CONTEXT, "orders{/id}",
                map("id", "a/b c"));
        emitBuild("template.query", builder, NON_WEB_CONTEXT, "orders?item={id}",
                map("id", "a=b&c+d#e"));
        emitBuild("template.repeated", builder, NON_WEB_CONTEXT, "{id}/x/{id}",
                map("id", "a b"));
        final LinkedHashMap<String,Object> directPreferred = new LinkedHashMap<String,Object>();
        directPreferred.put("id", "a/b");
        emitBuild("template.direct_preferred", builder, NON_WEB_CONTEXT, "{id}/x{/id}",
                directPreferred);
        emitBuild("template.list.leading_nulls", builder, NON_WEB_CONTEXT, "{id}",
                map("id", Arrays.asList(null, "", "x")));
        emitBuild("template.list.middle_null", builder, NON_WEB_CONTEXT, "{id}",
                map("id", Arrays.asList("a", null, "b")));
        emitBuild("template.null", builder, NON_WEB_CONTEXT, "{id}",
                map("id", null));
        emitBuild("template.replacement_contains_template", builder, NON_WEB_CONTEXT,
                "{id}/tail", map("id", "{id}"));
        emitBuild("template.path_and_remaining", builder, NON_WEB_CONTEXT,
                "orders/{id}", orderedMap("id", "a/b", "q", "x y"));
    }

    private static void exportEscaping() {
        final StandardLinkBuilder builder = new StandardLinkBuilder();
        emitBuild("escape.path_ascii", builder, NON_WEB_CONTEXT, "{v}",
                map("v", "-._~!$&'()*+,;=:@/ ?#[]"));
        emitBuild("escape.segment_ascii", builder, NON_WEB_CONTEXT, "{/v}",
                map("v", "-._~!$&'()*+,;=:@/ ?#[]"));
        emitBuild("escape.query_ascii", builder, NON_WEB_CONTEXT, "path",
                map("v", "-._~!$&'()*+,;=:@/ ?#[]"));
        emitBuild("escape.unicode", builder, NON_WEB_CONTEXT, "path/{v}",
                map("v", "中文😀"));
        final String isolated = new String(new char[] {'a', '\uD800', 'b', '\uDC00'});
        emit("escape.isolated.input_units", codeUnits(isolated));
        emitBuildUnits("escape.isolated.path_units", builder, NON_WEB_CONTEXT, "{v}",
                map("v", isolated));
        emitBuildUnits("escape.isolated.query_units", builder, NON_WEB_CONTEXT, "path",
                map("v", isolated));
    }

    private static void exportWebAndExtensionPoints() {
        final StandardLinkBuilder builder = new StandardLinkBuilder();
        emitBuild("web.null_application_path", builder,
                webContext(null, "T[%s]"), "/x", null);
        emitBuild("web.empty_application_path", builder,
                webContext("", "T[%s]"), "/x", null);
        emitBuild("web.root_application_path", builder,
                webContext("/", "T[%s]"), "/x", null);
        emitBuild("web.application_path", builder,
                webContext("/app", "T[%s]"), "/x", map("q", "v"));
        emitBuild("web.absolute_transformed", builder,
                webContext("/app", "T[%s]"), "https://example.org/x", null);

        final ProbeLinkBuilder probe = new ProbeLinkBuilder();
        final LinkedHashMap<String,Object> original = map("id", "7");
        emitBuild("hooks.result", probe, NON_WEB_CONTEXT, "/x/{id}", original);
        emit("hooks.context_calls", probe.contextCalls);
        emit("hooks.process_calls", probe.processCalls);
        emit("hooks.original_identity", probe.originalParameters == original);
        emit("hooks.original_size", probe.originalParameters.size());
        emit("hooks.process_input", probe.processInput);

        final ProbeLinkBuilder nullProcess = new ProbeLinkBuilder();
        nullProcess.returnNullFromProcess = true;
        emitBuild("hooks.process_null", nullProcess, NON_WEB_CONTEXT, "/x", null);
    }

    private static void emitBuild(
            final String key,
            final StandardLinkBuilder builder,
            final IExpressionContext context,
            final String base,
            final Map<String,Object> parameters) {
        emit(key, builder.buildLink(context, base, parameters));
    }

    private static void emitBuildUnits(
            final String key,
            final StandardLinkBuilder builder,
            final IExpressionContext context,
            final String base,
            final Map<String,Object> parameters) {
        emit(key, codeUnits(builder.buildLink(context, base, parameters)));
    }

    private static LinkedHashMap<String,Object> map(final String key, final Object value) {
        final LinkedHashMap<String,Object> result = new LinkedHashMap<String,Object>();
        result.put(key, value);
        return result;
    }

    private static LinkedHashMap<String,Object> orderedMap(
            final String firstKey, final Object firstValue,
            final String secondKey, final Object secondValue) {
        final LinkedHashMap<String,Object> result = map(firstKey, firstValue);
        result.put(secondKey, secondValue);
        return result;
    }

    private static IExpressionContext webContext(
            final String applicationPath,
            final String transformPattern) {
        final IWebRequest request = (IWebRequest) Proxy.newProxyInstance(
                LinkBuilderGolden.class.getClassLoader(),
                new Class<?>[] {IWebRequest.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("getApplicationPath")) {
                        return applicationPath;
                    }
                    return defaultValue(method.getReturnType());
                });
        final IWebExchange exchange = (IWebExchange) Proxy.newProxyInstance(
                LinkBuilderGolden.class.getClassLoader(),
                new Class<?>[] {IWebExchange.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("getRequest")) {
                        return request;
                    }
                    if (method.getName().equals("transformURL")) {
                        return String.format(transformPattern, args[0]);
                    }
                    return defaultValue(method.getReturnType());
                });
        return (IExpressionContext) Proxy.newProxyInstance(
                LinkBuilderGolden.class.getClassLoader(),
                new Class<?>[] {IExpressionContext.class, IWebContext.class},
                (proxy, method, args) -> {
                    if (method.getName().equals("getExchange")) {
                        return exchange;
                    }
                    if (method.getName().equals("getLocale")) {
                        return Locale.US;
                    }
                    if (method.getName().equals("getVariableNames")) {
                        return Collections.emptySet();
                    }
                    return defaultValue(method.getReturnType());
                });
    }

    private static Object defaultValue(final Class<?> type) {
        if (!type.isPrimitive()) {
            return null;
        }
        if (type == boolean.class) {
            return Boolean.FALSE;
        }
        if (type == char.class) {
            return Character.valueOf('\0');
        }
        if (type == byte.class) {
            return Byte.valueOf((byte) 0);
        }
        if (type == short.class) {
            return Short.valueOf((short) 0);
        }
        if (type == int.class) {
            return Integer.valueOf(0);
        }
        if (type == long.class) {
            return Long.valueOf(0L);
        }
        if (type == float.class) {
            return Float.valueOf(0.0f);
        }
        if (type == double.class) {
            return Double.valueOf(0.0d);
        }
        return null;
    }

    private static String codeUnits(final String value) {
        if (value == null) {
            return "null";
        }
        final StringBuilder result = new StringBuilder(value.length() * 5);
        for (int i = 0; i < value.length(); i++) {
            if (i > 0) {
                result.append(',');
            }
            result.append(String.format("%04X", Integer.valueOf(value.charAt(i))));
        }
        return result.toString();
    }

    private static void exportFailure(final String key, final ThrowingAction action) {
        try {
            action.run();
            emit(key, "NO_ERROR");
        } catch (final Throwable error) {
            emit(key, error.getClass().getName() + "|" + String.valueOf(error.getMessage()));
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private interface ThrowingAction {
        void run() throws Throwable;
    }

    private static class NonWebContext implements IExpressionContext {
        public IEngineConfiguration getConfiguration() {
            return null;
        }
        public IExpressionObjects getExpressionObjects() {
            return null;
        }
        public Locale getLocale() {
            return Locale.US;
        }
        public boolean containsVariable(final String name) {
            return false;
        }
        public Set<String> getVariableNames() {
            return Collections.emptySet();
        }
        public Object getVariable(final String name) {
            return null;
        }
    }

    private static final class ProbeLinkBuilder extends StandardLinkBuilder {
        int contextCalls;
        int processCalls;
        Map<String,Object> originalParameters;
        String processInput;
        boolean returnNullFromProcess;

        @Override
        protected String computeContextPath(
                final IExpressionContext context,
                final String base,
                final Map<String,Object> parameters) {
            this.contextCalls++;
            this.originalParameters = parameters;
            return "/hook";
        }

        @Override
        protected String processLink(final IExpressionContext context, final String link) {
            this.processCalls++;
            this.processInput = link;
            return this.returnNullFromProcess ? null : "P[" + link + "]";
        }
    }
}
