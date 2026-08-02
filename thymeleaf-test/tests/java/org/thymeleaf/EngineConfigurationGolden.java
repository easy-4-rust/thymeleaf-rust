package org.thymeleaf;

import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.CyclicBarrier;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;
import java.util.stream.Collectors;

import org.slf4j.Logger;
import org.thymeleaf.dialect.IExecutionAttributeDialect;
import org.thymeleaf.dialect.IExpressionObjectDialect;
import org.thymeleaf.dialect.IProcessorDialect;
import org.thymeleaf.engine.AbstractTemplateHandler;
import org.thymeleaf.linkbuilder.ILinkBuilder;
import org.thymeleaf.linkbuilder.StandardLinkBuilder;
import org.thymeleaf.messageresolver.IMessageResolver;
import org.thymeleaf.messageresolver.StandardMessageResolver;
import org.thymeleaf.model.IModelFactory;
import org.thymeleaf.postprocessor.IPostProcessor;
import org.thymeleaf.postprocessor.PostProcessor;
import org.thymeleaf.preprocessor.IPreProcessor;
import org.thymeleaf.preprocessor.PreProcessor;
import org.thymeleaf.standard.StandardDialect;
import org.thymeleaf.templatemode.TemplateMode;
import org.thymeleaf.templateresolver.ITemplateResolver;
import org.thymeleaf.templateresolver.StringTemplateResolver;

import sun.misc.Unsafe;

/**
 * Exports deterministic EngineConfiguration and ConfigurationPrinterHelper behavior.
 */
public final class EngineConfigurationGolden {

    private static final String BASELINE =
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private EngineConfigurationGolden() {
    }

    public static void main(final String[] args) throws Exception {
        emit("baseline", BASELINE);
        exportShape();

        final IEngineConfiguration seed = new TemplateEngine().getConfiguration();
        final EngineConfiguration configuration = orderedConfiguration(seed);
        exportOrderingAndSnapshots(configuration);
        exportDialectQueries(seed);
        exportFactoriesAndReshape(seed);
        exportConfigurationPrinter(seed);
        exportConfigLogBuilder();
        exportPreAndPostPrinter();
    }

    private static void exportShape() {
        emit("shape.engine.public", signatures(EngineConfiguration.class, true));
        emit("shape.interface", signatures(IEngineConfiguration.class, false));
    }

    private static EngineConfiguration orderedConfiguration(
            final IEngineConfiguration seed) {
        final Set<ITemplateResolver> templateResolvers = new LinkedHashSet<>();
        templateResolvers.add(templateResolver("template-null", null));
        templateResolvers.add(templateResolver("template-twenty-a", 20));
        templateResolvers.add(templateResolver("template-negative", -1));
        templateResolvers.add(templateResolver("template-twenty-b", 20));

        final Set<IMessageResolver> messageResolvers = new LinkedHashSet<>();
        messageResolvers.add(messageResolver("message-null", null));
        messageResolvers.add(messageResolver("message-five-a", 5));
        messageResolvers.add(messageResolver("message-min", Integer.MIN_VALUE));
        messageResolvers.add(messageResolver("message-five-b", 5));

        final Set<ILinkBuilder> linkBuilders = new LinkedHashSet<>();
        linkBuilders.add(linkBuilder("link-null", null));
        linkBuilders.add(linkBuilder("link-max", Integer.MAX_VALUE));
        linkBuilders.add(linkBuilder("link-zero-a", 0));
        linkBuilders.add(linkBuilder("link-zero-b", 0));

        final EngineConfiguration configuration = new EngineConfiguration(
                templateResolvers,
                messageResolvers,
                linkBuilders,
                new LinkedHashSet<>(seed.getDialectConfigurations()),
                seed.getCacheManager(),
                seed.getEngineContextFactory(),
                seed.getDecoupledTemplateLogicResolver());
        configuration.initialize();

        // Mutations after construction must not affect the frozen snapshots.
        templateResolvers.add(templateResolver("template-late", -100));
        messageResolvers.clear();
        linkBuilders.clear();
        return configuration;
    }

    private static void exportOrderingAndSnapshots(
            final EngineConfiguration configuration) {
        emit("order.template", configuration.getTemplateResolvers().stream()
                .map(ITemplateResolver::getName)
                .collect(Collectors.joining(",")));
        emit("order.message", configuration.getMessageResolvers().stream()
                .map(IMessageResolver::getName)
                .collect(Collectors.joining(",")));
        emit("order.link", configuration.getLinkBuilders().stream()
                .map(ILinkBuilder::getName)
                .collect(Collectors.joining(",")));
        emit("snapshot.template.size", configuration.getTemplateResolvers().size());
        emit("snapshot.message.size", configuration.getMessageResolvers().size());
        emit("snapshot.link.size", configuration.getLinkBuilders().size());
        emitFailure("snapshot.template.mutable",
                () -> configuration.getTemplateResolvers().clear());
        emitFailure("snapshot.message.mutable",
                () -> configuration.getMessageResolvers().clear());
        emitFailure("snapshot.link.mutable",
                () -> configuration.getLinkBuilders().clear());
    }

    private static void exportDialectQueries(final IEngineConfiguration configuration) {
        emit("dialect.all", configuration.getDialects().size());
        emit("dialect.standard",
                configuration.getDialectsOfType(StandardDialect.class).size());
        emit("dialect.processor",
                configuration.getDialectsOfType(IProcessorDialect.class).size());
        emit("dialect.expression",
                configuration.getDialectsOfType(IExpressionObjectDialect.class).size());
        emit("dialect.execution",
                configuration.getDialectsOfType(IExecutionAttributeDialect.class).size());
        emit("dialect.present", configuration.isStandardDialectPresent());
        emit("dialect.prefix", configuration.getStandardDialectPrefix());
        emit("definitions.element.identity",
                configuration.getElementDefinitions()
                        == configuration.getElementDefinitions());
        emit("definitions.attribute.identity",
                configuration.getAttributeDefinitions()
                        == configuration.getAttributeDefinitions());
    }

    private static void exportFactoriesAndReshape(
            final IEngineConfiguration configuration) throws Exception {
        final IModelFactory htmlFirst =
                configuration.getModelFactory(TemplateMode.HTML);
        final IModelFactory htmlSecond =
                configuration.getModelFactory(TemplateMode.HTML);
        final IModelFactory xml = configuration.getModelFactory(TemplateMode.XML);
        emit("manager.present", configuration.getTemplateManager() != null);
        emit("model.same", htmlFirst == htmlSecond);
        emit("model.different_mode", htmlFirst != xml);

        final int workers = 12;
        final CyclicBarrier barrier = new CyclicBarrier(workers);
        final Set<IModelFactory> identities =
                Collections.newSetFromMap(new ConcurrentHashMap<>());
        final List<Thread> threads = new ArrayList<>();
        for (int index = 0; index < workers; index++) {
            final Thread thread = new Thread(() -> {
                try {
                    barrier.await();
                    identities.add(
                            configuration.getModelFactory(TemplateMode.JAVASCRIPT));
                } catch (final Exception exception) {
                    throw new RuntimeException(exception);
                }
            });
            threads.add(thread);
            thread.start();
        }
        for (final Thread thread : threads) {
            thread.join();
        }
        emit("model.concurrent.identities", identities.size());

        final EngineConfiguration concrete = (EngineConfiguration) configuration;
        for (final TemplateMode mode : TemplateMode.values()) {
            emit("reshape." + mode, concrete.isModelReshapeable(mode));
            emit("bucket." + mode,
                    configuration.getTemplateBoundariesProcessors(mode).size() + ","
                    + configuration.getCDATASectionProcessors(mode).size() + ","
                    + configuration.getCommentProcessors(mode).size() + ","
                    + configuration.getDocTypeProcessors(mode).size() + ","
                    + configuration.getElementProcessors(mode).size() + ","
                    + configuration.getTextProcessors(mode).size() + ","
                    + configuration.getProcessingInstructionProcessors(mode).size() + ","
                    + configuration.getXMLDeclarationProcessors(mode).size() + ","
                    + configuration.getPreProcessors(mode).size() + ","
                    + configuration.getPostProcessors(mode).size());
        }
    }

    private static void exportConfigurationPrinter(
            final IEngineConfiguration configuration) throws Exception {
        final AtomicBoolean traceEnabled = new AtomicBoolean(false);
        final AtomicReference<String> event = new AtomicReference<>();
        final AtomicReference<String> message = new AtomicReference<>();
        final Logger logger = (Logger) Proxy.newProxyInstance(
                EngineConfigurationGolden.class.getClassLoader(),
                new Class<?>[]{Logger.class},
                (proxy, method, args) -> {
                    switch (method.getName()) {
                        case "getName":
                            return ConfigurationPrinterHelper.CONFIGURATION_LOGGER_NAME;
                        case "isDebugEnabled":
                            return true;
                        case "isTraceEnabled":
                            return traceEnabled.get();
                        case "debug":
                        case "trace":
                            if (args != null && args.length > 0
                                    && args[0] instanceof String) {
                                event.set(method.getName());
                                message.set((String) args[0]);
                            }
                            return null;
                        case "equals":
                            return proxy == args[0];
                        case "hashCode":
                            return System.identityHashCode(proxy);
                        case "toString":
                            return "GoldenLogger";
                        default:
                            if (method.getReturnType() == boolean.class) {
                                return false;
                            }
                            return null;
                    }
                });
        replaceStaticFinalLogger(logger);

        ConfigurationPrinterHelper.printConfiguration(configuration);
        emit("printer.debug.event", event.get());
        emit("printer.debug.output", normalizeConfigurationLog(message.get()));

        traceEnabled.set(true);
        event.set(null);
        message.set(null);
        ConfigurationPrinterHelper.printConfiguration(configuration);
        emit("printer.trace.event", event.get());
        emit("printer.trace.output", normalizeConfigurationLog(message.get()));
    }

    private static void exportConfigLogBuilder() throws Exception {
        final Class<?> builderType = Class.forName(
                "org.thymeleaf.ConfigurationPrinterHelper$ConfigLogBuilder");
        final Constructor<?> constructor = builderType.getDeclaredConstructor();
        constructor.setAccessible(true);
        final Object builder = constructor.newInstance();

        invokeBuilder(builderType, builder, "line",
                new Class<?>[]{String.class}, "plain");
        invokeBuilder(builderType, builder, "line",
                new Class<?>[]{String.class, Object.class},
                "single={}", "a$b");
        invokeBuilder(builderType, builder, "line",
                new Class<?>[]{String.class, Object.class, Object.class},
                "double={}|{}", null, "tail");
        invokeBuilder(builderType, builder, "line",
                new Class<?>[]{String.class, Object[].class},
                "array={}|{}|{}", (Object) new Object[]{"x", null, 3});
        invokeBuilder(builderType, builder, "end",
                new Class<?>[]{String.class}, "end");
        emit("builder.output", builder.toString());
    }

    private static void exportPreAndPostPrinter() throws Exception {
        final Set<IPreProcessor> preProcessors = new LinkedHashSet<>();
        preProcessors.add(new PreProcessor(
                TemplateMode.HTML, AbstractTemplateHandler.class, 20));
        preProcessors.add(new PreProcessor(
                TemplateMode.HTML, AbstractTemplateHandler.class, -1));
        preProcessors.add(new PreProcessor(
                TemplateMode.XML, AbstractTemplateHandler.class, 0));

        final Set<IPostProcessor> postProcessors = new LinkedHashSet<>();
        postProcessors.add(new PostProcessor(
                TemplateMode.HTML, AbstractTemplateHandler.class, 30));
        postProcessors.add(new PostProcessor(
                TemplateMode.HTML, AbstractTemplateHandler.class, 5));
        postProcessors.add(new PostProcessor(
                TemplateMode.XML, AbstractTemplateHandler.class, 0));

        emit("printer.pre.output", invokeProcessorPrinter(
                "printPreProcessorsForTemplateMode", IPreProcessor.class,
                preProcessors, TemplateMode.HTML));
        emit("printer.post.output", invokeProcessorPrinter(
                "printPostProcessorsForTemplateMode", IPostProcessor.class,
                postProcessors, TemplateMode.HTML));
    }

    private static String invokeProcessorPrinter(
            final String methodName,
            final Class<?> processorType,
            final Set<?> processors,
            final TemplateMode mode) throws Exception {
        final Class<?> builderType = Class.forName(
                "org.thymeleaf.ConfigurationPrinterHelper$ConfigLogBuilder");
        final Constructor<?> constructor = builderType.getDeclaredConstructor();
        constructor.setAccessible(true);
        final Object builder = constructor.newInstance();
        final Method method = ConfigurationPrinterHelper.class.getDeclaredMethod(
                methodName, builderType, Set.class, TemplateMode.class);
        method.setAccessible(true);
        method.invoke(null, builder, processors, mode);
        return builder.toString();
    }

    private static void invokeBuilder(
            final Class<?> type,
            final Object target,
            final String name,
            final Class<?>[] parameterTypes,
            final Object... arguments) throws Exception {
        final Method method = type.getDeclaredMethod(name, parameterTypes);
        method.setAccessible(true);
        method.invoke(target, arguments);
    }

    private static void replaceStaticFinalLogger(final Logger logger) throws Exception {
        final Field unsafeField = Unsafe.class.getDeclaredField("theUnsafe");
        unsafeField.setAccessible(true);
        final Unsafe unsafe = (Unsafe) unsafeField.get(null);
        final Field loggerField =
                ConfigurationPrinterHelper.class.getDeclaredField("configLogger");
        final Object base = unsafe.staticFieldBase(loggerField);
        final long offset = unsafe.staticFieldOffset(loggerField);
        unsafe.putObject(base, offset, logger);
    }

    private static String normalizeConfigurationLog(final String value) {
        if (value == null) {
            return null;
        }
        final String normalized = value
                .replaceAll(
                        "(?m)(\\[THYMELEAF\\] \\* Thymeleaf version:"
                                + " .* \\(built )[^)]*(\\))$",
                        "$1<BUILD_TIMESTAMP>$2")
                .replaceAll(
                "(?m)(\\[THYMELEAF\\]\\s+\\* \\\"[^\\\"]*\\\": ).*$",
                "$1<VALUE>");
        final List<String> output = new ArrayList<>();
        final List<String> processorLines = new ArrayList<>();
        for (final String line : normalized.split("\\n", -1)) {
            if (line.startsWith("[THYMELEAF]             * [")) {
                processorLines.add(line);
                continue;
            }
            if (!processorLines.isEmpty()) {
                Collections.sort(processorLines);
                output.addAll(processorLines);
                processorLines.clear();
            }
            output.add(line);
        }
        if (!processorLines.isEmpty()) {
            Collections.sort(processorLines);
            output.addAll(processorLines);
        }
        return String.join("\n", output);
    }

    private static StringTemplateResolver templateResolver(
            final String name, final Integer order) {
        final StringTemplateResolver resolver = new StringTemplateResolver();
        resolver.setName(name);
        resolver.setOrder(order);
        return resolver;
    }

    private static StandardMessageResolver messageResolver(
            final String name, final Integer order) {
        final StandardMessageResolver resolver = new StandardMessageResolver();
        resolver.setName(name);
        resolver.setOrder(order);
        return resolver;
    }

    private static StandardLinkBuilder linkBuilder(
            final String name, final Integer order) {
        final StandardLinkBuilder builder = new StandardLinkBuilder();
        builder.setName(name);
        builder.setOrder(order);
        return builder;
    }

    private static String signatures(
            final Class<?> type, final boolean publicOnly) {
        return Arrays.stream(type.getDeclaredMethods())
                .filter(method -> !publicOnly
                        || Modifier.isPublic(method.getModifiers()))
                .map(EngineConfigurationGolden::signature)
                .sorted()
                .collect(Collectors.joining(","));
    }

    private static String signature(final Method method) {
        return method.getName() + "("
                + Arrays.stream(method.getParameterTypes())
                        .map(Class::getSimpleName)
                        .collect(Collectors.joining("+"))
                + "):" + method.getReturnType().getSimpleName();
    }

    private static void emitFailure(
            final String key, final ThrowingRunnable action) {
        try {
            action.run();
            emit(key, "NO_ERROR");
        } catch (final Throwable throwable) {
            emit(key, throwable.getClass().getName() + ":"
                    + throwable.getMessage());
        }
    }

    private static void emit(final String key, final Object value) {
        final String text = String.valueOf(value)
                .replace("\\", "\\\\")
                .replace("\r", "\\r")
                .replace("\n", "\\n");
        System.out.println(key + "=" + text);
    }

    @FunctionalInterface
    private interface ThrowingRunnable {
        void run() throws Exception;
    }
}
