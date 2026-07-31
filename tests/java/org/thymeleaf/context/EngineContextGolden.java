package org.thymeleaf.context;

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
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Collectors;

import org.thymeleaf.IEngineConfiguration;
import org.thymeleaf.TemplateEngine;
import org.thymeleaf.cache.AlwaysValidCacheEntryValidity;
import org.thymeleaf.engine.TemplateData;
import org.thymeleaf.templatemode.TemplateMode;
import org.thymeleaf.templateresource.StringTemplateResource;
import org.thymeleaf.web.IWebExchange;

/** 从固定上游导出 Engine/WebEngine Context 的层级与惰性语义。 */
public final class EngineContextGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private EngineContextGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emit("shape.abstract", signatures(AbstractEngineContext.class));
        emit("shape.engine", signatures(EngineContext.class));
        emit("shape.web", signatures(WebEngineContext.class));
        emit("shape.template.interface", signatures(ITemplateContext.class));
        emit("shape.engine.interface", signatures(IEngineContext.class));

        final IEngineConfiguration configuration = new TemplateEngine().getConfiguration();
        exportEngine(configuration);
        exportWeb(configuration);
        exportExpressionObjectsAreLazy(configuration);
    }

    private static void exportEngine(final IEngineConfiguration configuration) {
        final Map<String,Object> variables = new LinkedHashMap<>();
        variables.put("root", "one");
        variables.put("nullable", null);
        final EngineContext context = new EngineContext(
                configuration, templateData("root-template"), null, Locale.US, variables);
        emit("plain.root.names", sortedNames(context));
        emit("plain.root.value", context.getVariable("root"));
        emit("plain.root.nullable", context.getVariable("nullable"));
        emit("plain.root.stack", stack(context));
        emit("plain.root.selection.present", context.hasSelectionTarget());

        context.setSelectionTarget("root-target");
        context.increaseLevel();
        context.setVariable("root", "local");
        context.setVariable("local", "yes");
        context.setSelectionTarget(null);
        context.setTemplateData(templateData("nested-template"));
        emit("plain.nested.level", context.level());
        emit("plain.nested.root", context.getVariable("root"));
        emit("plain.nested.local", context.getVariable("local"));
        emit("plain.nested.root.local", context.isVariableLocal("root"));
        emit("plain.nested.selection.present", context.hasSelectionTarget());
        emit("plain.nested.selection", context.getSelectionTarget());
        emit("plain.nested.stack", stack(context));
        emit("plain.nested.representation", context.getStringRepresentationByLevel());
        context.decreaseLevel();
        emit("plain.restored.level", context.level());
        emit("plain.restored.root", context.getVariable("root"));
        emit("plain.restored.local", context.getVariable("local"));
        emit("plain.restored.selection", context.getSelectionTarget());
        emit("plain.restored.stack", stack(context));

        final AtomicInteger loads = new AtomicInteger();
        context.setVariable("lazy", new LazyContextVariable<String>() {
            @Override
            protected String loadValue() {
                return "lazy-" + loads.incrementAndGet();
            }
        });
        emit("plain.lazy.first", context.getVariable("lazy"));
        emit("plain.lazy.second", context.getVariable("lazy"));
        emit("plain.lazy.loads", loads.get());
    }

    private static void exportWeb(final IEngineConfiguration configuration) {
        final Map<String,Object> attributes = new LinkedHashMap<>();
        final IWebExchange exchange = exchange(attributes);
        final WebEngineContext context = new WebEngineContext(
                configuration, templateData("web-root"), null, exchange, Locale.CANADA, null);
        context.setVariable("value", "root");
        context.setSelectionTarget("root-target");
        context.increaseLevel();
        context.setVariable("value", "local");
        context.setVariable("local", "yes");
        context.setSelectionTarget(null);
        emit("web.nested.value", context.getVariable("value"));
        emit("web.nested.local", context.getVariable("local"));
        emit("web.nested.value.local", context.isVariableLocal("value"));
        emit("web.nested.selection.present", context.hasSelectionTarget());
        emit("web.nested.selection", context.getSelectionTarget());
        emit("web.nested.representation", context.getStringRepresentationByLevel());
        context.decreaseLevel();
        emit("web.restored.value", context.getVariable("value"));
        emit("web.restored.local", context.getVariable("local"));
        emit("web.restored.selection", context.getSelectionTarget());
        emit("web.restored.attributes", attributes);
    }

    private static void exportExpressionObjectsAreLazy(final IEngineConfiguration delegate) {
        final AtomicInteger factoryReads = new AtomicInteger();
        final IEngineConfiguration traced = (IEngineConfiguration) Proxy.newProxyInstance(
                IEngineConfiguration.class.getClassLoader(), new Class<?>[] { IEngineConfiguration.class },
                (proxy, method, arguments) -> {
                    if ("getExpressionObjectFactory".equals(method.getName())) {
                        factoryReads.incrementAndGet();
                    }
                    return method.invoke(delegate, arguments);
                });
        final EngineContext context = new EngineContext(
                traced, templateData("lazy-expression"), null, Locale.US, null);
        emit("expression.factory.before", factoryReads.get());
        final Object first = context.getExpressionObjects();
        emit("expression.factory.after.first", factoryReads.get());
        emit("expression.factory.same", first == context.getExpressionObjects());
        emit("expression.factory.after.second", factoryReads.get());
    }

    private static IWebExchange exchange(final Map<String,Object> attributes) {
        return (IWebExchange) Proxy.newProxyInstance(
                IWebExchange.class.getClassLoader(), new Class<?>[] { IWebExchange.class },
                (proxy, method, arguments) -> {
                    final String name = method.getName();
                    if ("setAttributeValue".equals(name)) {
                        if (arguments[1] == null) {
                            attributes.remove(arguments[0]);
                        } else {
                            attributes.put((String) arguments[0], arguments[1]);
                        }
                        return null;
                    }
                    if ("getAttributeValue".equals(name)) return attributes.get(arguments[0]);
                    if ("containsAttribute".equals(name)) return attributes.containsKey(arguments[0]);
                    if ("getAttributeCount".equals(name)) return attributes.size();
                    if ("getAllAttributeNames".equals(name)) return new LinkedHashSet<>(attributes.keySet());
                    if ("getAttributeMap".equals(name)) return Collections.unmodifiableMap(attributes);
                    if ("getLocale".equals(name)) return Locale.CANADA;
                    if ("transformURL".equals(name)) return arguments[0];
                    if ("toString".equals(name)) return "exchange";
                    if (method.getReturnType().equals(boolean.class)) return false;
                    if (method.getReturnType().equals(int.class)) return 0;
                    return null;
                });
    }

    private static TemplateData templateData(final String name) {
        return new TemplateData(name, null, new StringTemplateResource(name), TemplateMode.HTML,
                AlwaysValidCacheEntryValidity.INSTANCE);
    }

    private static String stack(final ITemplateContext context) {
        return context.getTemplateStack().stream().map(TemplateData::getTemplate)
                .collect(Collectors.joining(","));
    }

    private static String sortedNames(final IContext context) {
        return context.getVariableNames().stream().sorted(Comparator.nullsFirst(Comparator.naturalOrder()))
                .collect(Collectors.toList()).toString();
    }

    private static String signatures(final Class<?> type) {
        final List<String> signatures = new ArrayList<>();
        Arrays.stream(type.getDeclaredConstructors()).filter(c -> !c.isSynthetic()).forEach(c -> signatures.add(
                Modifier.toString(c.getModifiers()) + " <init>(" + Arrays.stream(c.getParameterTypes())
                        .map(Class::getTypeName).collect(Collectors.joining(",")) + ")"));
        Arrays.stream(type.getDeclaredMethods()).filter(m -> !m.isSynthetic())
                .forEach(m -> signatures.add(Modifier.toString(m.getModifiers()) + " "
                        + m.getReturnType().getTypeName() + " " + m.getName() + "(" + Arrays.stream(m.getParameterTypes())
                                .map(Class::getTypeName).collect(Collectors.joining(",")) + ")"));
        signatures.sort(Comparator.naturalOrder());
        return String.join("|", signatures);
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }
}
