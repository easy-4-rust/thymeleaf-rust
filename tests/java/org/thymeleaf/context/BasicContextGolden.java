package org.thymeleaf.context;

import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
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

/**
 * 从固定上游导出基础 Context 六个对象的结构与可观察语义。
 */
public final class BasicContextGolden {

    private static final String BASELINE =
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private BasicContextGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emitShape("IContext", IContext.class);
        emitShape("AbstractContext", AbstractContext.class);
        emitShape("Context", Context.class);
        emitShape("IExpressionContext", IExpressionContext.class);
        emitShape("AbstractExpressionContext", AbstractExpressionContext.class);
        emitShape("ExpressionContext", ExpressionContext.class);

        exportConstructorsAndVariables();
        exportLiveVariableNames();
        exportMutationsAndErrors();
        exportAbstractContext();
        exportExpressionContexts();
    }

    private static void exportConstructorsAndVariables() {
        final Locale originalDefault = Locale.getDefault();
        try {
            Locale.setDefault(Locale.CANADA_FRENCH);
            final Context defaultContext = new Context();
            final Context nullLocaleContext = new Context(null);
            Locale.setDefault(Locale.JAPAN);
            emit("context.default.locale.snapshot", defaultContext.getLocale());
            emit("context.null.locale.snapshot", nullLocaleContext.getLocale());
            emit("context.new.default.locale", new Context().getLocale());
        } finally {
            Locale.setDefault(originalDefault);
        }

        final Marker marker = new Marker("shared");
        final Map<String,Object> source = new LinkedHashMap<>();
        source.put("first", marker);
        source.put(null, "null-key");
        source.put("nullable", null);
        final Context context = new Context(Locale.GERMANY, source);
        source.clear();
        source.put("later", "source-only");

        emit("context.explicit.locale", context.getLocale());
        emit("context.copy.names", context.getVariableNames());
        emit("context.copy.source.independent", !context.containsVariable("later"));
        emit("context.copy.value.identity", context.getVariable("first") == marker);
        emit("context.contains.null.key", context.containsVariable(null));
        emit("context.contains.null.value", context.containsVariable("nullable"));
        emit("context.contains.absent", context.containsVariable("absent"));
        emit("context.get.null.key", context.getVariable(null));
        emit("context.get.null.value", context.getVariable("nullable"));
        emit("context.get.absent", context.getVariable("absent"));
    }

    private static void exportLiveVariableNames() {
        final Context context = new Context(
                Locale.US, orderedMap("one", 1, "two", 2, "three", 3));
        final Set<String> names = context.getVariableNames();
        emit("names.identity", names == context.getVariableNames());
        emit("names.initial", names);

        context.setVariable("four", 4);
        emit("names.after.set", names);
        emit("names.remove.changed", names.remove("two"));
        emit("names.remove.backing", context.containsVariable("two"));
        emit("names.remove.absent", names.remove("absent"));
        emit(
                "names.contains.all",
                names.containsAll(Arrays.asList("one", "three", "four")));
        emit(
                "names.remove.all.changed",
                names.removeAll(Arrays.asList("one", "absent")));
        emit("names.after.remove.all", names);
        emit(
                "names.retain.all.changed",
                names.retainAll(Collections.singleton("four")));
        emit("names.after.retain.all", names);
        names.clear();
        emit("names.after.clear", names);
        emit("names.clear.backing.empty", context.getVariableNames().isEmpty());

        final Context unsupported = new Context();
        try {
            unsupported.getVariableNames().add("illegal");
            emit("names.add.error", "NONE");
        } catch (final RuntimeException exception) {
            emit("names.add.error", error(exception));
        }
    }

    private static void exportMutationsAndErrors() {
        final Context context = new Context();
        context.setVariable("first", 1);
        context.setVariable("second", 2);
        context.setVariable("first", 11);
        emit("mutate.replace.order", context.getVariableNames());
        emit("mutate.replace.value", context.getVariable("first"));

        final Map<String,Object> additions = new LinkedHashMap<>();
        additions.put("second", 22);
        additions.put("third", 3);
        context.setVariables(additions);
        context.setVariables(null);
        emit("mutate.put.all.order", context.getVariableNames());
        emit("mutate.put.all.second", context.getVariable("second"));
        context.removeVariable("absent");
        context.removeVariable("first");
        emit("mutate.after.remove", context.getVariableNames());
        context.clearVariables();
        emit("mutate.after.clear", context.getVariableNames());

        try {
            context.setLocale(null);
            emit("context.null.locale.error", "NONE");
        } catch (final RuntimeException exception) {
            emit("context.null.locale.error", error(exception));
        }
        context.setLocale(Locale.ITALY);
        emit("context.changed.locale", context.getLocale());
    }

    private static void exportAbstractContext() {
        final ProbeContext empty = new ProbeContext();
        final ProbeContext locale = new ProbeContext(Locale.UK);
        final ProbeContext populated = new ProbeContext(
                Locale.KOREA, orderedMap("alpha", "a", "beta", "b"));
        emit("abstract.default.locale.nonnull", empty.getLocale() != null);
        emit("abstract.locale", locale.getLocale());
        emit("abstract.variables", populated.getVariableNames());
        emit("abstract.variable.beta", populated.getVariable("beta"));
    }

    private static void exportExpressionContexts() {
        final ProbeFactory factory = new ProbeFactory();
        final TemplateEngine engine = new TemplateEngine();
        engine.addDialect(new ProbeDialect(factory));
        final IEngineConfiguration configuration = engine.getConfiguration();

        final Map<String,Object> variables = orderedMap("first", "one", "nullable", null);
        final ExpressionContext context =
                new ExpressionContext(configuration, Locale.FRANCE, variables);
        emit("expression.configuration.identity", context.getConfiguration() == configuration);
        emit("expression.before.builds", factory.builds);
        final IExpressionObjects firstObjects = context.getExpressionObjects();
        final IExpressionObjects secondObjects = context.getExpressionObjects();
        emit("expression.objects.identity", firstObjects == secondObjects);
        emit("expression.names.contains.probe", firstObjects.containsObject("probe"));
        final Object first = firstObjects.getObject("probe");
        final Object second = firstObjects.getObject("probe");
        emit("expression.object.value", first);
        emit("expression.object.identity", first == second);
        emit("expression.object.builds", factory.builds);
        emit("expression.factory.context.identity", factory.lastContext == context);
        emit("expression.factory.context.class", factory.lastContext.getClass().getName());
        emit("expression.locale", context.getLocale());
        emit("expression.variables", context.getVariableNames());

        final ProbeFactory abstractFactory = new ProbeFactory();
        final TemplateEngine abstractEngine = new TemplateEngine();
        abstractEngine.addDialect(new ProbeDialect(abstractFactory));
        final IEngineConfiguration abstractConfiguration = abstractEngine.getConfiguration();
        final ProbeExpressionContext abstractContext =
                new ProbeExpressionContext(abstractConfiguration, Locale.TAIWAN, variables);
        final IExpressionObjects abstractObjects = abstractContext.getExpressionObjects();
        abstractObjects.getObject("probe");
        emit(
                "abstract.expression.configuration.identity",
                abstractContext.getConfiguration() == abstractConfiguration);
        emit(
                "abstract.expression.objects.identity",
                abstractObjects == abstractContext.getExpressionObjects());
        emit(
                "abstract.expression.factory.context.identity",
                abstractFactory.lastContext == abstractContext);
        emit(
                "abstract.expression.factory.context.class",
                abstractFactory.lastContext.getClass().getName());

        emitConstructorError("expression.null.config.default", () -> new ExpressionContext(null));
        emitConstructorError(
                "expression.null.config.locale",
                () -> new ExpressionContext(null, Locale.US));
        emitConstructorError(
                "expression.null.config.variables",
                () -> new ExpressionContext(null, Locale.US, variables));
        emitConstructorError(
                "abstract.expression.null.config",
                () -> new ProbeExpressionContext(null, Locale.US, variables));
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

    private static Map<String,Object> orderedMap(final Object... pairs) {
        final Map<String,Object> map = new LinkedHashMap<>();
        for (int index = 0; index < pairs.length; index += 2) {
            map.put((String) pairs[index], pairs[index + 1]);
        }
        return map;
    }

    private static void emitConstructorError(final String key, final ThrowingAction action) {
        try {
            action.run();
            emit(key, "NONE");
        } catch (final RuntimeException exception) {
            emit(key, error(exception));
        }
    }

    private static String error(final RuntimeException exception) {
        return exception.getClass().getName() + ":" + exception.getMessage();
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

    private static final class ProbeContext extends AbstractContext {
        private ProbeContext() {
            super();
        }

        private ProbeContext(final Locale locale) {
            super(locale);
        }

        private ProbeContext(final Locale locale, final Map<String,Object> variables) {
            super(locale, variables);
        }
    }

    private static final class ProbeExpressionContext extends AbstractExpressionContext {
        private ProbeExpressionContext(
                final IEngineConfiguration configuration,
                final Locale locale,
                final Map<String,Object> variables) {
            super(configuration, locale, variables);
        }
    }

    private static final class ProbeDialect implements IExpressionObjectDialect {
        private final ProbeFactory factory;

        private ProbeDialect(final ProbeFactory factory) {
            this.factory = factory;
        }

        @Override
        public String getName() {
            return "probe";
        }

        @Override
        public IExpressionObjectFactory getExpressionObjectFactory() {
            return this.factory;
        }
    }

    private static final class ProbeFactory implements IExpressionObjectFactory {
        private final Set<String> names =
                Collections.unmodifiableSet(new LinkedHashSet<>(Collections.singleton("probe")));
        private int builds;
        private IExpressionContext lastContext;
        private Object cached;

        @Override
        public Set<String> getAllExpressionObjectNames() {
            return this.names;
        }

        @Override
        public Object buildObject(
                final IExpressionContext context, final String expressionObjectName) {
            this.builds++;
            this.lastContext = context;
            this.cached = new Marker(expressionObjectName);
            return this.cached;
        }

        @Override
        public boolean isCacheable(final String expressionObjectName) {
            return true;
        }
    }
}
