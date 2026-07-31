package org.thymeleaf.engine;

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
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Collectors;

import org.thymeleaf.IEngineConfiguration;
import org.thymeleaf.TemplateEngine;
import org.thymeleaf.cache.AlwaysValidCacheEntryValidity;
import org.thymeleaf.context.Context;
import org.thymeleaf.context.EngineContext;
import org.thymeleaf.context.IContext;
import org.thymeleaf.context.IEngineContext;
import org.thymeleaf.context.IEngineContextFactory;
import org.thymeleaf.context.IWebContext;
import org.thymeleaf.context.StandardEngineContextFactory;
import org.thymeleaf.context.WebContext;
import org.thymeleaf.context.WebExpressionContext;
import org.thymeleaf.templatemode.TemplateMode;
import org.thymeleaf.templateresource.StringTemplateResource;
import org.thymeleaf.web.IWebExchange;

/**
 * 从固定上游导出 Engine Context 工厂与管理器的可观察行为。
 */
public final class EngineContextFactoryGolden {

    private static final String BASELINE =
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private EngineContextFactoryGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emit("shape.factory.interface", signatures(IEngineContextFactory.class));
        emit("shape.factory.standard", signatures(StandardEngineContextFactory.class));
        emit("shape.manager", signatures(EngineContextManager.class));

        final IEngineConfiguration configuration = new TemplateEngine().getConfiguration();
        exportStandardFactory(configuration);
        exportWebFactory(configuration);
        exportManagerLifecycle();
        exportNullContext(configuration);
    }

    private static void exportStandardFactory(final IEngineConfiguration configuration) {
        final StandardEngineContextFactory factory = new StandardEngineContextFactory();
        final TraceContext empty = new TraceContext(Locale.CANADA_FRENCH, Collections.emptyMap());
        final IEngineContext emptyResult = factory.createEngineContext(
                configuration, templateData("empty"), resolutionAttributes(), empty);
        emit("plain.empty.class", emptyResult.getClass().getName());
        emit("plain.empty.trace", empty.trace());
        emit("plain.empty.names", sortedNames(emptyResult));
        emit("plain.empty.locale", emptyResult.getLocale());
        emit("plain.empty.level", emptyResult.level());
        emit("plain.empty.template", emptyResult.getTemplateData().getTemplate());
        emit(
                "plain.empty.attributes.size",
                emptyResult.getTemplateResolutionAttributes().size());
        emit(
                "plain.empty.attributes.second",
                emptyResult.getTemplateResolutionAttributes().get("second"));
        emit(
                "plain.empty.attributes.first",
                emptyResult.getTemplateResolutionAttributes().get("first"));

        final Map<String,Object> variables = new LinkedHashMap<>();
        variables.put("second", 2);
        variables.put("first", "one");
        variables.put("nullable", null);
        final TraceContext populated = new TraceContext(Locale.JAPAN, variables);
        final IEngineContext populatedResult = factory.createEngineContext(
                configuration, templateData("plain"), resolutionAttributes(), populated);
        emit("plain.vars.class", populatedResult.getClass().getName());
        emit("plain.vars.trace", populated.trace());
        emit("plain.vars.names", sortedNames(populatedResult));
        emit("plain.vars.second", populatedResult.getVariable("second"));
        emit("plain.vars.first", populatedResult.getVariable("first"));
        emit("plain.vars.nullable", populatedResult.getVariable("nullable"));
        emit("plain.vars.level", populatedResult.level());
    }

    private static void exportWebFactory(final IEngineConfiguration configuration) {
        final StandardEngineContextFactory factory = new StandardEngineContextFactory();
        final ExchangeState exchangeState = new ExchangeState();
        final IWebExchange exchange = exchangeState.proxy();
        final Map<String,Object> variables = new LinkedHashMap<>();
        variables.put("webSecond", 22);
        variables.put("webFirst", "one");

        final WebContext webContext = new WebContext(exchange, Locale.GERMANY, variables);
        final IEngineContext webResult = factory.createEngineContext(
                configuration, templateData("web"), resolutionAttributes(), webContext);
        emit("web.context.class", webResult.getClass().getName());
        emit("web.context.exchange.same", ((IWebContext) webResult).getExchange() == exchange);
        emit("web.context.names", sortedNames(webResult));
        emit("web.context.second", webResult.getVariable("webSecond"));
        emit("web.context.first", webResult.getVariable("webFirst"));
        emit("web.context.level", webResult.level());

        final WebExpressionContext expressionContext = new WebExpressionContext(
                configuration, exchange, Locale.ITALY, variables);
        final IEngineContext expressionResult = factory.createEngineContext(
                configuration, templateData("web-expression"), null, expressionContext);
        emit("web.expression.class", expressionResult.getClass().getName());
        emit(
                "web.expression.exchange.same",
                ((IWebContext) expressionResult).getExchange() == exchange);
        emit("web.expression.locale", expressionResult.getLocale());
        emit("web.expression.names", sortedNames(expressionResult));
    }

    private static void exportManagerLifecycle() {
        final CountingFactory countingFactory = new CountingFactory();
        final TemplateEngine engine = new TemplateEngine();
        engine.setEngineContextFactory(countingFactory);
        final IEngineConfiguration configuration = engine.getConfiguration();

        final Context original = new Context(Locale.US, Collections.singletonMap("root", "value"));
        final IEngineContext created = EngineContextManager.prepareEngineContext(
                configuration, templateData("root"), resolutionAttributes(), original);
        emit("manager.created.factory.calls", countingFactory.calls.get());
        emit("manager.created.class", created.getClass().getName());
        emit("manager.created.level", created.level());
        emit("manager.created.template", created.getTemplateData().getTemplate());
        emit("manager.created.stack", templateStack(created));
        EngineContextManager.disposeEngineContext(created);
        emit("manager.created.disposed.level", created.level());
        emit("manager.created.disposed.stack", templateStack(created));

        final EngineContext existing = new EngineContext(
                configuration, templateData("existing-root"), resolutionAttributes(),
                Locale.UK, Collections.singletonMap("existing", "yes"));
        final IEngineContext reused = EngineContextManager.prepareEngineContext(
                configuration, templateData("nested"), Collections.singletonMap("nested", "attr"),
                existing);
        emit("manager.reused.same", reused == existing);
        emit("manager.reused.factory.calls", countingFactory.calls.get());
        emit("manager.reused.level", reused.level());
        emit("manager.reused.template", reused.getTemplateData().getTemplate());
        emit("manager.reused.stack", templateStack(reused));
        EngineContextManager.disposeEngineContext(reused);
        emit("manager.reused.disposed.level", reused.level());
        emit("manager.reused.disposed.template", reused.getTemplateData().getTemplate());
        emit("manager.reused.disposed.stack", templateStack(reused));
    }

    private static void exportNullContext(final IEngineConfiguration configuration) {
        try {
            new StandardEngineContextFactory().createEngineContext(
                    configuration, templateData("null"), null, null);
            emit("null.context", "NONE");
        } catch (final RuntimeException exception) {
            emit(
                    "null.context",
                    exception.getClass().getName() + ":" + exception.getMessage());
        }
    }

    private static TemplateData templateData(final String name) {
        return new TemplateData(
                name,
                null,
                new StringTemplateResource(name),
                TemplateMode.HTML,
                AlwaysValidCacheEntryValidity.INSTANCE);
    }

    private static Map<String,Object> resolutionAttributes() {
        final Map<String,Object> attributes = new LinkedHashMap<>();
        attributes.put("second", 2);
        attributes.put("first", "one");
        return attributes;
    }

    private static String templateStack(final IEngineContext context) {
        return context.getTemplateStack().stream()
                .map(TemplateData::getTemplate)
                .collect(Collectors.joining(","));
    }

    private static String sortedNames(final IContext context) {
        return context.getVariableNames().stream()
                .sorted(Comparator.nullsFirst(Comparator.naturalOrder()))
                .collect(Collectors.toList())
                .toString();
    }

    private static String signatures(final Class<?> type) {
        final List<String> signatures = new ArrayList<>();
        Arrays.stream(type.getDeclaredConstructors())
                .filter(constructor -> !constructor.isSynthetic())
                .forEach(constructor -> signatures.add(
                        Modifier.toString(constructor.getModifiers()) + " <init>("
                                + Arrays.stream(constructor.getParameterTypes())
                                        .map(Class::getTypeName)
                                        .collect(Collectors.joining(","))
                                + ")"));
        Arrays.stream(type.getDeclaredMethods())
                .filter(method -> !method.isSynthetic())
                .map(EngineContextFactoryGolden::signature)
                .forEach(signatures::add);
        signatures.sort(Comparator.naturalOrder());
        return String.join("|", signatures);
    }

    private static String signature(final Method method) {
        return Modifier.toString(method.getModifiers()) + " "
                + method.getReturnType().getTypeName() + " " + method.getName() + "("
                + Arrays.stream(method.getParameterTypes())
                        .map(Class::getTypeName)
                        .collect(Collectors.joining(","))
                + ")";
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private static class TraceContext implements IContext {

        private final Locale locale;
        private final LinkedHashMap<String,Object> variables;
        private final List<String> trace = new ArrayList<>();

        private TraceContext(final Locale locale, final Map<String,Object> variables) {
            this.locale = locale;
            this.variables = new LinkedHashMap<>(variables);
        }

        @Override
        public Locale getLocale() {
            this.trace.add("locale");
            return this.locale;
        }

        @Override
        public boolean containsVariable(final String name) {
            this.trace.add("contains:" + name);
            return this.variables.containsKey(name);
        }

        @Override
        public Set<String> getVariableNames() {
            this.trace.add("names");
            return new LinkedHashSet<>(this.variables.keySet());
        }

        @Override
        public Object getVariable(final String name) {
            this.trace.add("get:" + name);
            return this.variables.get(name);
        }

        private String trace() {
            return String.join(",", this.trace);
        }
    }

    private static final class CountingFactory implements IEngineContextFactory {

        private final AtomicInteger calls = new AtomicInteger();

        @Override
        public IEngineContext createEngineContext(
                final IEngineConfiguration configuration,
                final TemplateData templateData,
                final Map<String,Object> templateResolutionAttributes,
                final IContext context) {
            this.calls.incrementAndGet();
            final Map<String,Object> variables = new LinkedHashMap<>();
            for (final String name : context.getVariableNames()) {
                variables.put(name, context.getVariable(name));
            }
            return new EngineContext(
                    configuration,
                    templateData,
                    templateResolutionAttributes,
                    context.getLocale(),
                    variables);
        }
    }

    private static final class ExchangeState {

        private final LinkedHashMap<String,Object> attributes = new LinkedHashMap<>();

        private IWebExchange proxy() {
            return (IWebExchange) Proxy.newProxyInstance(
                    IWebExchange.class.getClassLoader(),
                    new Class<?>[] { IWebExchange.class },
                    (proxy, method, arguments) -> {
                        final String name = method.getName();
                        if ("setAttributeValue".equals(name)) {
                            if (arguments[1] == null) {
                                this.attributes.remove(arguments[0]);
                            } else {
                                this.attributes.put((String) arguments[0], arguments[1]);
                            }
                            return null;
                        }
                        if ("getAttributeValue".equals(name)) {
                            return this.attributes.get(arguments[0]);
                        }
                        if ("containsAttribute".equals(name)) {
                            return this.attributes.containsKey(arguments[0]);
                        }
                        if ("getAttributeCount".equals(name)) {
                            return this.attributes.size();
                        }
                        if ("getAllAttributeNames".equals(name)) {
                            return new LinkedHashSet<>(this.attributes.keySet());
                        }
                        if ("getAttributeMap".equals(name)) {
                            return Collections.unmodifiableMap(this.attributes);
                        }
                        if ("removeAttribute".equals(name)) {
                            this.attributes.remove(arguments[0]);
                            return null;
                        }
                        if ("getLocale".equals(name)) {
                            return Locale.GERMANY;
                        }
                        if ("transformURL".equals(name)) {
                            return arguments[0];
                        }
                        if ("toString".equals(name)) {
                            return "GoldenWebExchange";
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
    }
}
