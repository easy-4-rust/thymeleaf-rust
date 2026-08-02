package org.thymeleaf.context;

import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;

import org.thymeleaf.IEngineConfiguration;
import org.thymeleaf.TemplateEngine;
import org.thymeleaf.dialect.IExpressionObjectDialect;
import org.thymeleaf.expression.IExpressionObjectFactory;
import org.thymeleaf.expression.IExpressionObjects;
import org.thymeleaf.web.IWebExchange;

/**
 * 从固定上游导出 Web Context 三个对象的结构与可观察语义。
 */
public final class WebContextGolden {

    private static final String BASELINE =
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private WebContextGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emitShape("IWebContext", IWebContext.class);
        emitShape("WebContext", WebContext.class);
        emitShape("WebExpressionContext", WebExpressionContext.class);
        exportWebContext();
        exportWebExpressionContext();
        exportValidation();
    }

    private static void exportWebContext() {
        final IWebExchange exchange = exchange();
        final Marker marker = new Marker("shared");
        final Map<String,Object> variables = new LinkedHashMap<>();
        variables.put("first", marker);
        variables.put(null, "null-key");
        variables.put("nullable", null);
        final WebContext context = new WebContext(exchange, Locale.GERMANY, variables);

        emit("web.exchange.identity", context.getExchange() == exchange);
        emit("web.interface.exchange.identity", ((IWebContext) context).getExchange() == exchange);
        emit("web.locale", context.getLocale());
        emit("web.names", context.getVariableNames());
        emit("web.value.identity", context.getVariable("first") == marker);
        emit("web.contains.null.key", context.containsVariable(null));
        emit("web.contains.null.value", context.containsVariable("nullable"));
        final Set<String> names = context.getVariableNames();
        emit("web.names.identity", names == context.getVariableNames());
        context.setVariable("later", "value");
        emit("web.names.live", names);
        emit("web.names.remove.changed", names.remove("first"));
        emit("web.names.remove.backing", context.containsVariable("first"));
    }

    private static void exportWebExpressionContext() {
        final IWebExchange exchange = exchange();
        final ProbeFactory factory = new ProbeFactory(exchange);
        final TemplateEngine engine = new TemplateEngine();
        engine.addDialect(new ProbeDialect(factory));
        final IEngineConfiguration configuration = engine.getConfiguration();
        final Map<String,Object> variables = new LinkedHashMap<>();
        variables.put("first", "one");
        variables.put("nullable", null);
        final WebExpressionContext context =
                new WebExpressionContext(configuration, exchange, Locale.FRANCE, variables);

        emit(
                "web.expression.configuration.identity",
                context.getConfiguration() == configuration);
        emit("web.expression.exchange.identity", context.getExchange() == exchange);
        emit(
                "web.expression.interface.exchange.identity",
                ((IWebContext) context).getExchange() == exchange);
        emit("web.expression.before.builds", factory.builds);
        final IExpressionObjects firstObjects = context.getExpressionObjects();
        final IExpressionObjects secondObjects = context.getExpressionObjects();
        emit("web.expression.objects.identity", firstObjects == secondObjects);
        emit("web.expression.names.contains.probe", firstObjects.containsObject("probe"));
        final Object first = firstObjects.getObject("probe");
        final Object second = firstObjects.getObject("probe");
        emit("web.expression.object.value", first);
        emit("web.expression.object.identity", first == second);
        emit("web.expression.object.builds", factory.builds);
        emit("web.expression.factory.context.identity", factory.lastContext == context);
        emit("web.expression.factory.context.class", factory.lastContext.getClass().getName());
        emit(
                "web.expression.factory.context.web",
                factory.lastContext instanceof IWebContext);
        emit(
                "web.expression.factory.exchange.identity",
                ((IWebContext) factory.lastContext).getExchange() == exchange);
        emit("web.expression.locale", context.getLocale());
        emit("web.expression.names", context.getVariableNames());
    }

    private static void exportValidation() {
        final IWebExchange exchange = exchange();
        final IEngineConfiguration configuration = new TemplateEngine().getConfiguration();
        final Map<String,Object> variables = Collections.singletonMap("first", "one");

        emitError("web.null.exchange.default", () -> new WebContext(null));
        emitError("web.null.exchange.locale", () -> new WebContext(null, Locale.US));
        emitError(
                "web.null.exchange.variables",
                () -> new WebContext(null, Locale.US, variables));

        emitError(
                "web.expression.null.configuration.default",
                () -> new WebExpressionContext(null, exchange));
        emitError(
                "web.expression.null.configuration.locale",
                () -> new WebExpressionContext(null, exchange, Locale.US));
        emitError(
                "web.expression.null.configuration.variables",
                () -> new WebExpressionContext(null, exchange, Locale.US, variables));
        emitError(
                "web.expression.null.exchange.default",
                () -> new WebExpressionContext(configuration, null));
        emitError(
                "web.expression.null.exchange.locale",
                () -> new WebExpressionContext(configuration, null, Locale.US));
        emitError(
                "web.expression.null.exchange.variables",
                () -> new WebExpressionContext(configuration, null, Locale.US, variables));
        emitError(
                "web.expression.both.null.precedence",
                () -> new WebExpressionContext(null, null));
    }

    private static IWebExchange exchange() {
        return (IWebExchange) Proxy.newProxyInstance(
                WebContextGolden.class.getClassLoader(),
                new Class<?>[] { IWebExchange.class },
                (proxy, method, arguments) -> {
                    if ("equals".equals(method.getName())) {
                        return proxy == arguments[0];
                    }
                    if ("hashCode".equals(method.getName())) {
                        return System.identityHashCode(proxy);
                    }
                    if ("toString".equals(method.getName())) {
                        return "ProbeWebExchange";
                    }
                    final Class<?> returnType = method.getReturnType();
                    if (returnType == boolean.class) {
                        return false;
                    }
                    if (returnType == int.class) {
                        return 0;
                    }
                    return null;
                });
    }

    private static void emitShape(final String name, final Class<?> type) {
        final List<String> signatures = new ArrayList<>();
        for (final Constructor<?> constructor : type.getDeclaredConstructors()) {
            if (!constructor.isSynthetic()) {
                signatures.add(
                        Modifier.toString(constructor.getModifiers()) + " <init>("
                                + typeNames(constructor.getParameterTypes()) + ")");
            }
        }
        for (final Method method : type.getDeclaredMethods()) {
            if (!method.isSynthetic()) {
                signatures.add(
                        Modifier.toString(method.getModifiers()) + " "
                                + method.getReturnType().getTypeName() + " "
                                + method.getName() + "("
                                + typeNames(method.getParameterTypes()) + ")");
            }
        }
        signatures.sort(Comparator.naturalOrder());
        emit("shape." + name + ".count", signatures.size());
        emit("shape." + name + ".signatures", String.join("|", signatures));
    }

    private static String typeNames(final Class<?>[] types) {
        return Arrays.stream(types).map(Class::getTypeName).collect(Collectors.joining(","));
    }

    private static void emitError(final String key, final ThrowingAction action) {
        try {
            action.run();
            emit(key, "NONE");
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getName() + ":" + exception.getMessage());
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private interface ThrowingAction {
        void run();
    }

    private static final class Marker {
        private final String value;

        private Marker(final String value) {
            this.value = value;
        }

        @Override
        public String toString() {
            return "Marker(" + this.value + ")";
        }
    }

    private static final class ProbeDialect implements IExpressionObjectDialect {
        private final ProbeFactory factory;

        private ProbeDialect(final ProbeFactory factory) {
            this.factory = factory;
        }

        @Override
        public String getName() {
            return "web-probe";
        }

        @Override
        public IExpressionObjectFactory getExpressionObjectFactory() {
            return this.factory;
        }
    }

    private static final class ProbeFactory implements IExpressionObjectFactory {
        private final Set<String> names =
                Collections.unmodifiableSet(new LinkedHashSet<>(Collections.singleton("probe")));
        private final IWebExchange expectedExchange;
        private int builds;
        private IExpressionContext lastContext;

        private ProbeFactory(final IWebExchange expectedExchange) {
            this.expectedExchange = expectedExchange;
        }

        @Override
        public Set<String> getAllExpressionObjectNames() {
            return this.names;
        }

        @Override
        public Object buildObject(
                final IExpressionContext context, final String expressionObjectName) {
            this.builds++;
            this.lastContext = context;
            if (!(context instanceof IWebContext)
                    || ((IWebContext) context).getExchange() != this.expectedExchange) {
                throw new IllegalStateException("factory lost IWebContext capability");
            }
            return new Marker(expressionObjectName);
        }

        @Override
        public boolean isCacheable(final String expressionObjectName) {
            return true;
        }
    }
}
