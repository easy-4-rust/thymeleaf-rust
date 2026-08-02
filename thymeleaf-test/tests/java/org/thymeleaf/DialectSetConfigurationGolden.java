package org.thymeleaf;

import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Collectors;

import org.thymeleaf.context.IExpressionContext;
import org.thymeleaf.dialect.AbstractDialect;
import org.thymeleaf.dialect.IExecutionAttributeDialect;
import org.thymeleaf.dialect.IExpressionObjectDialect;
import org.thymeleaf.dialect.IPostProcessorDialect;
import org.thymeleaf.dialect.IPreProcessorDialect;
import org.thymeleaf.engine.AbstractTemplateHandler;
import org.thymeleaf.engine.ITemplateHandler;
import org.thymeleaf.expression.IExpressionObjectFactory;
import org.thymeleaf.postprocessor.IPostProcessor;
import org.thymeleaf.preprocessor.IPreProcessor;
import org.thymeleaf.templatemode.TemplateMode;

/**
 * 从固定 Thymeleaf 3.1.5.RELEASE 导出 Dialect 能力聚合的完整可观察合同。
 */
public final class DialectSetConfigurationGolden {

    private static final String BASELINE =
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private DialectSetConfigurationGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        exportShape();
        exportEmpty();
        exportExecutionAttributes();
        exportExpressionFactories();
        exportPrePostProcessors();
        exportValidation();
        exportGetterValidation();
    }

    private static void exportShape() {
        emit("shape.execution", signatures(IExecutionAttributeDialect.class));
        emit("shape.expression", signatures(IExpressionObjectDialect.class));
        emit("shape.pre", signatures(IPreProcessorDialect.class));
        emit("shape.post", signatures(IPostProcessorDialect.class));
        emit("shape.configuration.public", Arrays.stream(DialectSetConfiguration.class.getDeclaredMethods())
                .filter(method -> java.lang.reflect.Modifier.isPublic(method.getModifiers()))
                .map(DialectSetConfigurationGolden::signature)
                .sorted()
                .collect(Collectors.joining(",")));
    }

    private static void exportEmpty() {
        final DialectSetConfiguration configuration =
                DialectSetConfiguration.build(Collections.emptySet());
        emit("empty.configurations", configuration.getDialectConfigurations().size());
        emit("empty.dialects", configuration.getDialects().size());
        emit("empty.standard", configuration.isStandardDialectPresent());
        emit("empty.prefix", configuration.getStandardDialectPrefix());
        emit("empty.attributes", configuration.getExecutionAttributes());
        emit("empty.expression.names",
                configuration.getExpressionObjectFactory().getAllExpressionObjectNames());
        emit("empty.pre.html", configuration.getPreProcessors(TemplateMode.HTML));
        emit("empty.post.raw", configuration.getPostProcessors(TemplateMode.RAW));
        emitFailure("empty.configurations.mutable",
                () -> configuration.getDialectConfigurations().clear());
        emitFailure("empty.attributes.mutable",
                () -> configuration.getExecutionAttributes().clear());
    }

    private static void exportExecutionAttributes() {
        final LinkedHashMap<String, Object> first = new LinkedHashMap<>();
        first.put(null, "null-key");
        first.put("null-value", null);
        first.put("alpha", 1);
        final LinkedHashMap<String, Object> second = new LinkedHashMap<>();
        second.put("beta", 2);

        final DialectSetConfiguration configuration = build(
                new CapabilityDialect("first", first, null, null, null),
                new CapabilityDialect("ignored-null-map", null, null, null, null),
                new CapabilityDialect("second", second, null, null, null));
        emit("attributes.entries", configuration.getExecutionAttributes());
        emit("attributes.null.present", configuration.hasExecutionAttribute(null));
        emit("attributes.null.value", configuration.getExecutionAttribute("null-value"));
        emit("attributes.missing.present", configuration.hasExecutionAttribute("missing"));

        final LinkedHashMap<String, Object> duplicate = new LinkedHashMap<>();
        duplicate.put("alpha", 9);
        emitFailure("attributes.conflict",
                () -> build(
                        new CapabilityDialect("a", first, null, null, null),
                        new CapabilityDialect("b", duplicate, null, null, null)));

        final LinkedHashMap<String, Object> duplicateNull = new LinkedHashMap<>();
        duplicateNull.put(null, "again");
        emitFailure("attributes.conflict.null",
                () -> build(
                        new CapabilityDialect("a", first, null, null, null),
                        new CapabilityDialect("b", duplicateNull, null, null, null)));
    }

    private static void exportExpressionFactories() {
        final ProbeFactory first = new ProbeFactory(
                "A", linkedSet("a", "shared"), true);
        final ProbeFactory second = new ProbeFactory(
                "B", linkedSet("b", "shared"), false);
        final DialectSetConfiguration aggregate = build(
                new CapabilityDialect("first", null, first, null, null),
                new CapabilityDialect("null", null, null, null, null),
                new CapabilityDialect("second", null, second, null, null));
        final IExpressionObjectFactory factory = aggregate.getExpressionObjectFactory();
        emit("expression.multi.names", factory.getAllExpressionObjectNames());
        emit("expression.multi.shared", factory.buildObject(null, "shared"));
        emit("expression.multi.a", factory.buildObject(null, "a"));
        emit("expression.multi.unknown", factory.buildObject(null, "unknown"));
        emit("expression.multi.cache.shared", factory.isCacheable("shared"));
        emit("expression.multi.calls", first.calls + "|" + second.calls);

        final ProbeFactory single = new ProbeFactory(
                "ONLY", linkedSet("known"), true);
        final IExpressionObjectFactory singleAggregate = build(
                new CapabilityDialect("single", null, single, null, null))
                .getExpressionObjectFactory();
        emit("expression.single.names", singleAggregate.getAllExpressionObjectNames());
        emit("expression.single.unknown", singleAggregate.buildObject(null, "unknown"));
        emit("expression.single.cache.unknown", singleAggregate.isCacheable("unknown"));
        emit("expression.single.calls", single.calls);
    }

    private static void exportPrePostProcessors() {
        final AtomicInteger prePrecedenceCalls = new AtomicInteger();
        final AtomicInteger postPrecedenceCalls = new AtomicInteger();
        final Set<IPreProcessor> firstPre = linkedSet(
                new BPreProcessor(TemplateMode.HTML, ProbeHandler.class, 20),
                new APreProcessor(TemplateMode.HTML, ProbeHandler.class, 20));
        final Set<IPreProcessor> secondPre = linkedSet(
                new APreProcessor(TemplateMode.HTML, ProbeHandler.class, 5));
        final Set<IPostProcessor> firstPost = linkedSet(
                new BPostProcessor(TemplateMode.HTML, ProbeHandler.class, 20),
                new APostProcessor(TemplateMode.HTML, ProbeHandler.class, 20));
        final Set<IPostProcessor> secondPost = linkedSet(
                new APostProcessor(TemplateMode.HTML, ProbeHandler.class, 5));

        final DialectSetConfiguration configuration = build(
                new CapabilityDialect(
                        "first", null, null,
                        new PreContribution(-100, prePrecedenceCalls, firstPre),
                        new PostContribution(-100, postPrecedenceCalls, firstPost)),
                new CapabilityDialect(
                        "ignored-null-sets", null, null,
                        new PreContribution(0, prePrecedenceCalls, null),
                        new PostContribution(0, postPrecedenceCalls, null)),
                new CapabilityDialect(
                        "second", null, null,
                        new PreContribution(100, prePrecedenceCalls, secondPre),
                        new PostContribution(100, postPrecedenceCalls, secondPost)));
        emit("pre.order", describePre(configuration.getPreProcessors(TemplateMode.HTML)));
        emit("post.order", describePost(configuration.getPostProcessors(TemplateMode.HTML)));
        emit("pre.dialect_precedence.calls", prePrecedenceCalls.get());
        emit("post.dialect_precedence.calls", postPrecedenceCalls.get());
        emit("pre.empty.xml", configuration.getPreProcessors(TemplateMode.XML));
        emit("post.empty.xml", configuration.getPostProcessors(TemplateMode.XML));
    }

    @SuppressWarnings({"rawtypes", "unchecked"})
    private static void exportValidation() {
        emitFailure("build.null", () -> DialectSetConfiguration.build(null));

        final Set nullPre = new LinkedHashSet();
        nullPre.add(null);
        emitFailure("pre.null.entry", () -> build(new CapabilityDialect(
                "bad", null, null,
                new PreContribution(0, new AtomicInteger(), nullPre), null)));
        emitFailure("pre.null.mode", () -> build(new CapabilityDialect(
                "bad", null, null,
                new PreContribution(0, new AtomicInteger(),
                        linkedSet(new APreProcessor(null, ProbeHandler.class, 0))), null)));
        emitFailure("pre.null.handler", () -> build(new CapabilityDialect(
                "bad", null, null,
                new PreContribution(0, new AtomicInteger(),
                        linkedSet(new APreProcessor(TemplateMode.HTML, null, 0))), null)));
        emitFailure("pre.wrong.handler", () -> build(new CapabilityDialect(
                "bad", null, null,
                new PreContribution(0, new AtomicInteger(),
                        linkedSet(new APreProcessor(
                                TemplateMode.HTML, (Class) String.class, 0))), null)));
        emitFailure("pre.no_zero_arg", () -> build(new CapabilityDialect(
                "bad", null, null,
                new PreContribution(0, new AtomicInteger(),
                        linkedSet(new APreProcessor(
                                TemplateMode.HTML, NoPublicConstructorHandler.class, 0))), null)));

        final Set nullPost = new LinkedHashSet();
        nullPost.add(null);
        emitFailure("post.null.entry", () -> build(new CapabilityDialect(
                "bad", null, null, null,
                new PostContribution(0, new AtomicInteger(), nullPost))));
        emitFailure("post.null.mode", () -> build(new CapabilityDialect(
                "bad", null, null, null,
                new PostContribution(0, new AtomicInteger(),
                        linkedSet(new APostProcessor(null, ProbeHandler.class, 0))))));
        emitFailure("post.null.handler", () -> build(new CapabilityDialect(
                "bad", null, null, null,
                new PostContribution(0, new AtomicInteger(),
                        linkedSet(new APostProcessor(TemplateMode.HTML, null, 0))))));
        emitFailure("post.wrong.handler", () -> build(new CapabilityDialect(
                "bad", null, null, null,
                new PostContribution(0, new AtomicInteger(),
                        linkedSet(new APostProcessor(
                                TemplateMode.HTML, (Class) String.class, 0))))));
        emitFailure("post.no_zero_arg", () -> build(new CapabilityDialect(
                "bad", null, null, null,
                new PostContribution(0, new AtomicInteger(),
                        linkedSet(new APostProcessor(
                                TemplateMode.HTML, NoPublicConstructorHandler.class, 0))))));
    }

    private static void exportGetterValidation() {
        final DialectSetConfiguration configuration =
                DialectSetConfiguration.build(Collections.emptySet());
        emitFailure("getter.boundaries.null",
                () -> configuration.getTemplateBoundariesProcessors(null));
        emitFailure("getter.cdata.null", () -> configuration.getCDATASectionProcessors(null));
        emitFailure("getter.comment.null", () -> configuration.getCommentProcessors(null));
        emitFailure("getter.doctype.null", () -> configuration.getDocTypeProcessors(null));
        emitFailure("getter.element.null", () -> configuration.getElementProcessors(null));
        emitFailure("getter.instruction.null",
                () -> configuration.getProcessingInstructionProcessors(null));
        emitFailure("getter.text.null", () -> configuration.getTextProcessors(null));
        emitFailure("getter.declaration.null",
                () -> configuration.getXMLDeclarationProcessors(null));
        emitFailure("getter.pre.null", () -> configuration.getPreProcessors(null));
        emitFailure("getter.post.null", () -> configuration.getPostProcessors(null));
    }

    private static DialectSetConfiguration build(final CapabilityDialect... dialects) {
        final Set<DialectConfiguration> configurations = new LinkedHashSet<>();
        for (final CapabilityDialect dialect : dialects) {
            configurations.add(new DialectConfiguration(dialect));
        }
        return DialectSetConfiguration.build(configurations);
    }

    private static String signatures(final Class<?> type) {
        return Arrays.stream(type.getDeclaredMethods())
                .map(DialectSetConfigurationGolden::signature)
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

    private static String describePre(final Set<IPreProcessor> processors) {
        return processors.stream()
                .map(processor -> processor.getClass().getSimpleName()
                        + ":" + processor.getPrecedence())
                .collect(Collectors.joining(",", "[", "]"));
    }

    private static String describePost(final Set<IPostProcessor> processors) {
        return processors.stream()
                .map(processor -> processor.getClass().getSimpleName()
                        + ":" + processor.getPrecedence())
                .collect(Collectors.joining(",", "[", "]"));
    }

    @SafeVarargs
    private static <T> LinkedHashSet<T> linkedSet(final T... values) {
        return new LinkedHashSet<>(Arrays.asList(values));
    }

    private static void emitFailure(final String key, final ThrowingRunnable runnable) {
        try {
            runnable.run();
            emit(key, "NO_ERROR");
        } catch (final Throwable throwable) {
            emit(key, throwable.getClass().getName() + ":" + throwable.getMessage()
                    + "|cause=" + (throwable.getCause() == null
                            ? "null" : throwable.getCause().getClass().getName()));
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private interface ThrowingRunnable {
        void run() throws Exception;
    }

    private static final class ProbeFactory implements IExpressionObjectFactory {
        private final String id;
        private final Set<String> names;
        private final boolean cacheable;
        private int calls;

        private ProbeFactory(
                final String id, final Set<String> names, final boolean cacheable) {
            this.id = id;
            this.names = names;
            this.cacheable = cacheable;
        }

        @Override
        public Set<String> getAllExpressionObjectNames() {
            return this.names;
        }

        @Override
        public Object buildObject(
                final IExpressionContext context, final String expressionObjectName) {
            this.calls++;
            return this.id + ":" + expressionObjectName;
        }

        @Override
        public boolean isCacheable(final String expressionObjectName) {
            this.calls++;
            return this.cacheable;
        }
    }

    private static final class PreContribution {
        private final int precedence;
        private final AtomicInteger calls;
        private final Set<IPreProcessor> processors;

        private PreContribution(
                final int precedence,
                final AtomicInteger calls,
                final Set<IPreProcessor> processors) {
            this.precedence = precedence;
            this.calls = calls;
            this.processors = processors;
        }
    }

    private static final class PostContribution {
        private final int precedence;
        private final AtomicInteger calls;
        private final Set<IPostProcessor> processors;

        private PostContribution(
                final int precedence,
                final AtomicInteger calls,
                final Set<IPostProcessor> processors) {
            this.precedence = precedence;
            this.calls = calls;
            this.processors = processors;
        }
    }

    private static final class CapabilityDialect extends AbstractDialect
            implements IExecutionAttributeDialect, IExpressionObjectDialect,
                       IPreProcessorDialect, IPostProcessorDialect {
        private final Map<String, Object> attributes;
        private final IExpressionObjectFactory factory;
        private final PreContribution pre;
        private final PostContribution post;

        private CapabilityDialect(
                final String name,
                final Map<String, Object> attributes,
                final IExpressionObjectFactory factory,
                final PreContribution pre,
                final PostContribution post) {
            super(name);
            this.attributes = attributes;
            this.factory = factory;
            this.pre = pre;
            this.post = post;
        }

        @Override
        public Map<String, Object> getExecutionAttributes() {
            return this.attributes;
        }

        @Override
        public IExpressionObjectFactory getExpressionObjectFactory() {
            return this.factory;
        }

        @Override
        public int getDialectPreProcessorPrecedence() {
            if (this.pre == null) {
                return 0;
            }
            this.pre.calls.incrementAndGet();
            return this.pre.precedence;
        }

        @Override
        public Set<IPreProcessor> getPreProcessors() {
            return this.pre == null ? null : this.pre.processors;
        }

        @Override
        public int getDialectPostProcessorPrecedence() {
            if (this.post == null) {
                return 0;
            }
            this.post.calls.incrementAndGet();
            return this.post.precedence;
        }

        @Override
        public Set<IPostProcessor> getPostProcessors() {
            return this.post == null ? null : this.post.processors;
        }
    }

    public static class ProbeHandler extends AbstractTemplateHandler {
        public ProbeHandler() {
            super();
        }
    }

    public static class NoPublicConstructorHandler extends AbstractTemplateHandler {
        private NoPublicConstructorHandler() {
            super();
        }
    }

    private abstract static class BasePreProcessor implements IPreProcessor {
        private final TemplateMode mode;
        private final Class<? extends ITemplateHandler> handlerClass;
        private final int precedence;

        private BasePreProcessor(
                final TemplateMode mode,
                final Class<? extends ITemplateHandler> handlerClass,
                final int precedence) {
            this.mode = mode;
            this.handlerClass = handlerClass;
            this.precedence = precedence;
        }

        @Override
        public TemplateMode getTemplateMode() {
            return this.mode;
        }

        @Override
        public Class<? extends ITemplateHandler> getHandlerClass() {
            return this.handlerClass;
        }

        @Override
        public int getPrecedence() {
            return this.precedence;
        }
    }

    private static final class APreProcessor extends BasePreProcessor {
        private APreProcessor(
                final TemplateMode mode,
                final Class<? extends ITemplateHandler> handlerClass,
                final int precedence) {
            super(mode, handlerClass, precedence);
        }
    }

    private static final class BPreProcessor extends BasePreProcessor {
        private BPreProcessor(
                final TemplateMode mode,
                final Class<? extends ITemplateHandler> handlerClass,
                final int precedence) {
            super(mode, handlerClass, precedence);
        }
    }

    private abstract static class BasePostProcessor implements IPostProcessor {
        private final TemplateMode mode;
        private final Class<? extends ITemplateHandler> handlerClass;
        private final int precedence;

        private BasePostProcessor(
                final TemplateMode mode,
                final Class<? extends ITemplateHandler> handlerClass,
                final int precedence) {
            this.mode = mode;
            this.handlerClass = handlerClass;
            this.precedence = precedence;
        }

        @Override
        public TemplateMode getTemplateMode() {
            return this.mode;
        }

        @Override
        public Class<? extends ITemplateHandler> getHandlerClass() {
            return this.handlerClass;
        }

        @Override
        public int getPrecedence() {
            return this.precedence;
        }
    }

    private static final class APostProcessor extends BasePostProcessor {
        private APostProcessor(
                final TemplateMode mode,
                final Class<? extends ITemplateHandler> handlerClass,
                final int precedence) {
            super(mode, handlerClass, precedence);
        }
    }

    private static final class BPostProcessor extends BasePostProcessor {
        private BPostProcessor(
                final TemplateMode mode,
                final Class<? extends ITemplateHandler> handlerClass,
                final int precedence) {
            super(mode, handlerClass, precedence);
        }
    }
}
