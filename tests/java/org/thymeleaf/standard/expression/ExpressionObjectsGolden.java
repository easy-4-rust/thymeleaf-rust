package org.thymeleaf.standard.expression;

import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.Collections;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Collectors;

import org.thymeleaf.IEngineConfiguration;
import org.thymeleaf.TemplateEngine;
import org.thymeleaf.context.ExpressionContext;
import org.thymeleaf.context.IExpressionContext;
import org.thymeleaf.context.ITemplateContext;
import org.thymeleaf.expression.ExpressionObjects;
import org.thymeleaf.expression.IExpressionObjectFactory;
import org.thymeleaf.expression.IExpressionObjects;

/**
 * 从固定上游导出表达式对象工厂、容器和 OGNL Map 包装器的可观察行为。
 */
public final class ExpressionObjectsGolden {

    private static final String BASELINE =
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private ExpressionObjectsGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emit("shape.factory.interface", signatures(IExpressionObjectFactory.class));
        emit("shape.objects.interface", signatures(IExpressionObjects.class));
        emit("shape.objects", signatures(ExpressionObjects.class));
        emit("shape.standard", signatures(StandardExpressionObjectFactory.class));
        emit("shape.wrapper", signatures(OGNLExpressionObjectsWrapper.class));

        exportExpressionObjects();
        exportStandardFactory();
        exportWrapper();
    }

    private static void exportExpressionObjects() {
        final IEngineConfiguration configuration = new TemplateEngine().getConfiguration();
        final IExpressionContext context = new ExpressionContext(configuration, Locale.US);
        final ProbeFactory factory = new ProbeFactory();
        final ExpressionObjects objects = new ExpressionObjects(context, factory);

        emit("container.names.identity", objects.getObjectNames() == factory.names);
        emit("container.names", join(objects.getObjectNames()));
        emit("container.size", objects.size());
        emit("container.contains.cached", objects.containsObject("cached"));
        emit("container.contains.null", objects.containsObject(null));
        emit("container.contains.unknown", objects.containsObject("unknown"));
        emit("container.unknown.value", objects.getObject("unknown"));
        emit("container.unknown.builds", factory.builds.get());

        final Object cachedOne = objects.getObject("cached");
        final Object cachedTwo = objects.getObject("cached");
        emit("container.cached.value", cachedOne);
        emit("container.cached.same", cachedOne == cachedTwo);
        emit("container.cached.builds", factory.buildsFor("cached"));
        emit("container.cached.cache_checks", factory.cacheChecksFor("cached"));
        emit("container.cached.context.same", factory.lastContext == context);

        final Object freshOne = objects.getObject("fresh");
        final Object freshTwo = objects.getObject("fresh");
        emit("container.fresh.first", freshOne);
        emit("container.fresh.second", freshTwo);
        emit("container.fresh.same", freshOne == freshTwo);
        emit("container.fresh.builds", factory.buildsFor("fresh"));
        emit("container.fresh.cache_checks", factory.cacheChecksFor("fresh"));

        emit("container.null.first", objects.getObject("cachedNull"));
        emit("container.null.second", objects.getObject("cachedNull"));
        emit("container.null.builds", factory.buildsFor("cachedNull"));
        emit("container.null.cache_checks", factory.cacheChecksFor("cachedNull"));

        emitFailure("container.null.context", () -> new ExpressionObjects(null, factory));
        emitFailure("container.null.factory", () -> new ExpressionObjects(context, null));
    }

    private static void exportStandardFactory() {
        final IEngineConfiguration configuration = new TemplateEngine().getConfiguration();
        final IExpressionContext context =
                new ExpressionContext(configuration, Locale.CANADA_FRENCH);
        final StandardExpressionObjectFactory first = new StandardExpressionObjectFactory();
        final StandardExpressionObjectFactory second = new StandardExpressionObjectFactory();

        emit("standard.names", join(first.getAllExpressionObjectNames()));
        emit("standard.names.count", first.getAllExpressionObjectNames().size());
        emit("standard.names.identity",
                first.getAllExpressionObjectNames() == second.getAllExpressionObjectNames());
        emit("standard.cache.null", first.isCacheable(null));
        emit("standard.cache.object", first.isCacheable("object"));
        emit("standard.cache.unknown", first.isCacheable("unknown"));

        emit("standard.ctx.same", first.buildObject(context, "ctx") == context);
        emit("standard.root.same", first.buildObject(context, "root") == context);
        emit("standard.vars.same", first.buildObject(context, "vars") == context);
        emit("standard.object.fallback.same", first.buildObject(context, "object") == context);
        emit("standard.locale", first.buildObject(context, "locale"));
        emit("standard.unknown", first.buildObject(context, "unknown"));
        emit("standard.null", first.buildObject(context, null));

        final String[] names = {
                "conversions", "uris", "temporals", "calendars", "dates", "bools",
                "numbers", "objects", "strings", "arrays", "lists", "sets", "maps",
                "aggregates", "messages", "ids", "execInfo"
        };
        for (final String name : names) {
            final Object value = first.buildObject(context, name);
            emit("standard.ordinary." + name,
                    value == null ? null : value.getClass().getName());
        }

        final Object selection = new Marker("selection");
        final ITemplateContext templateContext = templateContext(configuration, selection);
        emit("standard.template.object.same",
                first.buildObject(templateContext, "object") == selection);
        emit("standard.template.messages",
                first.buildObject(templateContext, "messages").getClass().getName());
        emit("standard.template.ids",
                first.buildObject(templateContext, "ids").getClass().getName());
        emit("standard.template.execInfo",
                first.buildObject(templateContext, "execInfo").getClass().getName());

        for (final String name :
                Arrays.asList("uris", "bools", "objects", "arrays", "lists", "sets", "maps",
                        "aggregates")) {
            emit("standard.singleton." + name,
                    first.buildObject(context, name) == second.buildObject(context, name));
        }
        for (final String name : Arrays.asList("strings", "numbers", "dates", "calendars")) {
            emit("standard.fresh." + name,
                    first.buildObject(context, name) == second.buildObject(context, name));
        }

        for (final String name :
                Arrays.asList("request", "response", "session", "servletContext")) {
            emitFailure("standard.removed." + name, () -> first.buildObject(context, name));
        }
    }

    private static void exportWrapper() {
        final WrapperObjects expressionObjects = new WrapperObjects();
        final OGNLExpressionObjectsWrapper wrapper =
                new OGNLExpressionObjectsWrapper(expressionObjects);

        emit("wrapper.restricted.names",
                join(Arrays.stream(new String[] {"ctx", "vars", "root", "this", "execInfo",
                                "custom", null})
                        .filter(OGNLExpressionObjectsWrapper::isRestricted)
                        .collect(Collectors.toCollection(LinkedHashSet::new))));
        emit("wrapper.initial.size", wrapper.size());
        emit("wrapper.initial.empty", wrapper.isEmpty());
        emit("wrapper.initial.keys.identity",
                wrapper.keySet() == expressionObjects.getObjectNames());
        emit("wrapper.initial.keys", join(wrapper.keySet()));
        emit("wrapper.contains.custom", wrapper.containsKey("custom"));
        emit("wrapper.contains.missing", wrapper.containsKey("missing"));
        emit("wrapper.get.custom", wrapper.get("custom"));
        emit("wrapper.custom.builds", expressionObjects.gets.get());

        emit("wrapper.put.first", wrapper.put("local", "one"));
        emit("wrapper.put.second", wrapper.put("local", "two"));
        emit("wrapper.get.local", wrapper.get("local"));
        emit("wrapper.size.local", wrapper.size());
        emit("wrapper.keys.local", join(wrapper.keySet()));
        emit("wrapper.values.local", sorted(wrapper.values()));
        emitFailure("wrapper.put.expression", () -> wrapper.put("custom", "bad"));
        emitFailure("wrapper.remove.expression", () -> wrapper.remove("custom"));
        emit("wrapper.remove.local", wrapper.remove("local"));

        final Map<String,Object> batch = new LinkedHashMap<>();
        batch.put("batch", "value");
        batch.put("custom", "forbidden");
        emitFailure("wrapper.putAll", () -> wrapper.putAll(batch));
        emit("wrapper.putAll.batch", wrapper.get("batch"));
        emit("wrapper.putAll.custom", wrapper.get("custom"));

        wrapper.put(OGNLContextPropertyAccessor.RESTRICT_EXPRESSION_OBJECTS, Boolean.TRUE);
        emitFailure("wrapper.restricted.ctx", () -> wrapper.get("ctx"));
        emit("wrapper.restricted.custom", wrapper.get("custom"));

        emitFailure("wrapper.null.get", () -> wrapper.get(null));
        emitFailure("wrapper.null.contains", () -> wrapper.containsKey(null));
        emitFailure("wrapper.null.put", () -> wrapper.put(null, "value"));
        emitFailure("wrapper.null.remove", () -> wrapper.remove(null));
        emitFailure("wrapper.clear", wrapper::clear);
        emitFailure("wrapper.containsValue", () -> wrapper.containsValue("value"));
        emitFailure("wrapper.clone", wrapper::clone);
        emitFailure("wrapper.entrySet", wrapper::entrySet);
        emitFailure("wrapper.equals", () -> wrapper.equals(Collections.emptyMap()));
        emitFailure("wrapper.hashCode", wrapper::hashCode);
        emit("wrapper.toString", wrapper.toString());
    }

    private static ITemplateContext templateContext(
            final IEngineConfiguration configuration,
            final Object selectionTarget) {
        return (ITemplateContext) Proxy.newProxyInstance(
                ExpressionObjectsGolden.class.getClassLoader(),
                new Class<?>[] {ITemplateContext.class},
                (proxy, method, args) -> {
                    switch (method.getName()) {
                        case "getConfiguration":
                            return configuration;
                        case "getLocale":
                            return Locale.JAPAN;
                        case "hasSelectionTarget":
                            return selectionTarget != null;
                        case "getSelectionTarget":
                            return selectionTarget;
                        case "containsVariable":
                            return false;
                        case "getVariableNames":
                            return Collections.emptySet();
                        case "getVariable":
                            return null;
                        case "toString":
                            return "TemplateContextProxy";
                        default:
                            return defaultValue(method.getReturnType());
                    }
                });
    }

    private static Object defaultValue(final Class<?> type) {
        if (!type.isPrimitive()) {
            return null;
        }
        if (type == boolean.class) {
            return false;
        }
        if (type == char.class) {
            return '\0';
        }
        return 0;
    }

    private static String signatures(final Class<?> type) {
        final List<String> values = new ArrayList<>();
        for (final Constructor<?> constructor : type.getDeclaredConstructors()) {
            values.add(Modifier.toString(constructor.getModifiers()) + " <init>("
                    + parameterTypes(constructor.getParameterTypes()) + ")");
        }
        for (final Method method : type.getDeclaredMethods()) {
            values.add(Modifier.toString(method.getModifiers()) + " "
                    + method.getReturnType().getTypeName() + " " + method.getName() + "("
                    + parameterTypes(method.getParameterTypes()) + ")");
        }
        values.sort(Comparator.naturalOrder());
        return String.join("|", values);
    }

    private static String parameterTypes(final Class<?>[] types) {
        return Arrays.stream(types).map(Class::getTypeName).collect(Collectors.joining(","));
    }

    private static String join(final Collection<?> values) {
        return values.stream().map(String::valueOf).collect(Collectors.joining(","));
    }

    private static String sorted(final Collection<?> values) {
        return values.stream().map(String::valueOf).sorted().collect(Collectors.joining(","));
    }

    private static void emitFailure(final String key, final Operation operation) {
        try {
            operation.run();
            emit(key, "NONE");
        } catch (final Throwable throwable) {
            emit(key, throwable.getClass().getName() + ":" + throwable.getMessage());
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private interface Operation {
        void run();
    }

    private static final class ProbeFactory implements IExpressionObjectFactory {
        private final Set<String> names =
                Collections.unmodifiableSet(new LinkedHashSet<>(
                        Arrays.asList("cached", "fresh", "cachedNull", null)));
        private final AtomicInteger builds = new AtomicInteger();
        private final Map<String,AtomicInteger> buildsByName = new LinkedHashMap<>();
        private final Map<String,AtomicInteger> cacheChecksByName = new LinkedHashMap<>();
        private IExpressionContext lastContext;

        @Override
        public Set<String> getAllExpressionObjectNames() {
            return this.names;
        }

        @Override
        public Object buildObject(
                final IExpressionContext context,
                final String expressionObjectName) {
            this.lastContext = context;
            this.builds.incrementAndGet();
            this.buildsByName
                    .computeIfAbsent(String.valueOf(expressionObjectName),
                            ignored -> new AtomicInteger())
                    .incrementAndGet();
            if ("cachedNull".equals(expressionObjectName)) {
                return null;
            }
            return new Marker(expressionObjectName + "-" + this.builds.get());
        }

        @Override
        public boolean isCacheable(final String expressionObjectName) {
            this.cacheChecksByName
                    .computeIfAbsent(String.valueOf(expressionObjectName),
                            ignored -> new AtomicInteger())
                    .incrementAndGet();
            return !"fresh".equals(expressionObjectName);
        }

        private int buildsFor(final String name) {
            return this.buildsByName.get(name).get();
        }

        private int cacheChecksFor(final String name) {
            return this.cacheChecksByName.get(name).get();
        }
    }

    private static final class WrapperObjects implements IExpressionObjects {
        private final Set<String> names = Collections.unmodifiableSet(
                new LinkedHashSet<>(Arrays.asList(
                        "ctx", "vars", "root", "this", "execInfo", "custom")));
        private final AtomicInteger gets = new AtomicInteger();

        @Override
        public int size() {
            return this.names.size();
        }

        @Override
        public boolean containsObject(final String name) {
            return this.names.contains(name);
        }

        @Override
        public Set<String> getObjectNames() {
            return this.names;
        }

        @Override
        public Object getObject(final String name) {
            this.gets.incrementAndGet();
            return "object:" + name;
        }
    }

    private static final class Marker {
        private final String text;

        private Marker(final String text) {
            this.text = text;
        }

        @Override
        public String toString() {
            return this.text;
        }
    }
}
