import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.Arrays;
import java.util.Collections;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;

import org.thymeleaf.TemplateEngine;
import org.thymeleaf.context.Context;
import org.thymeleaf.dialect.AbstractDialect;
import org.thymeleaf.dialect.IPostProcessorDialect;
import org.thymeleaf.dialect.IPreProcessorDialect;
import org.thymeleaf.dialect.IProcessorDialect;
import org.thymeleaf.engine.AbstractTemplateHandler;
import org.thymeleaf.engine.ITemplateHandler;
import org.thymeleaf.postprocessor.IPostProcessor;
import org.thymeleaf.postprocessor.PostProcessor;
import org.thymeleaf.preprocessor.IPreProcessor;
import org.thymeleaf.preprocessor.PreProcessor;
import org.thymeleaf.processor.IProcessor;
import org.thymeleaf.templatemode.TemplateMode;
import org.thymeleaf.util.ProcessorComparators;
import org.thymeleaf.util.ProcessorConfigurationUtils;

/**
 * 从固定 Thymeleaf 3.1.5.RELEASE 导出 PreProcessor/PostProcessor 配置与运行时合同。
 */
public final class PrePostProcessorGolden {

    private static final String JAVA_BASELINE =
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private PrePostProcessorGolden() {
    }

    public static void main(final String[] args) throws Exception {
        emit("java_baseline", JAVA_BASELINE);
        exportShape();
        exportValidation();
        exportImmutableState();
        exportDynamicDispatch();
        exportOrdering();
        exportFreshHandlers();
        exportHandlerConstructionFailures();
    }

    private static void exportShape() {
        emit("shape.pre.class.final", Modifier.isFinal(PreProcessor.class.getModifiers()));
        emit("shape.post.class.final", Modifier.isFinal(PostProcessor.class.getModifiers()));
        emit("shape.pre.interface.methods", interfaceSignatures(IPreProcessor.class));
        emit("shape.post.interface.methods", interfaceSignatures(IPostProcessor.class));
        emit("shape.pre.constructor", constructorSignature(PreProcessor.class));
        emit("shape.post.constructor", constructorSignature(PostProcessor.class));
    }

    private static void exportValidation() {
        exportFailure("pre.validation.both_null",
                () -> new PreProcessor(null, null, 7));
        exportFailure("pre.validation.handler_null",
                () -> new PreProcessor(TemplateMode.HTML, null, 7));
        exportFailure("post.validation.both_null",
                () -> new PostProcessor(null, null, 7));
        exportFailure("post.validation.handler_null",
                () -> new PostProcessor(TemplateMode.HTML, null, 7));
    }

    private static void exportImmutableState() {
        final TemplateMode[] modes = TemplateMode.values();
        final int[] precedences = {
                Integer.MIN_VALUE, -1, 0, 1, 1000, Integer.MAX_VALUE
        };
        for (int index = 0; index < modes.length; index++) {
            final TemplateMode mode = modes[index];
            final int precedence = precedences[index];
            final PreProcessor pre = new PreProcessor(mode, ProbeHandler.class, precedence);
            final PostProcessor post = new PostProcessor(mode, ProbeHandler.class, precedence);
            emit("pre.state." + mode.name(),
                    state(pre.getTemplateMode(), pre.getPrecedence(), pre.getHandlerClass(),
                            pre.getHandlerClass() == pre.getHandlerClass()));
            emit("post.state." + mode.name(),
                    state(post.getTemplateMode(), post.getPrecedence(), post.getHandlerClass(),
                            post.getHandlerClass() == post.getHandlerClass()));
        }
    }

    private static void exportDynamicDispatch() {
        final IPreProcessor pre = new CustomPreProcessor(
                TemplateMode.CSS, ProbeHandler.class, -17);
        final IPostProcessor post = new CustomPostProcessor(
                TemplateMode.JAVASCRIPT, ProbeHandler.class, 23);
        emit("dynamic.pre",
                state(pre.getTemplateMode(), pre.getPrecedence(), pre.getHandlerClass(), true));
        emit("dynamic.post",
                state(post.getTemplateMode(), post.getPrecedence(), post.getHandlerClass(), true));
    }

    private static void exportOrdering() {
        final IPreProcessor preLow =
                new PreProcessor(TemplateMode.HTML, ProbeHandler.class, -1);
        final IPreProcessor preHigh =
                new PreProcessor(TemplateMode.HTML, ProbeHandler.class, 1);
        final IPostProcessor postLow =
                new PostProcessor(TemplateMode.HTML, ProbeHandler.class, -1);
        final IPostProcessor postHigh =
                new PostProcessor(TemplateMode.HTML, ProbeHandler.class, 1);
        emit("ordering.pre.self",
                sign(ProcessorComparators.PRE_PROCESSOR_COMPARATOR.compare(preLow, preLow)));
        emit("ordering.pre.precedence",
                sign(ProcessorComparators.PRE_PROCESSOR_COMPARATOR.compare(preLow, preHigh)));
        emit("ordering.post.self",
                sign(ProcessorComparators.POST_PROCESSOR_COMPARATOR.compare(postLow, postLow)));
        emit("ordering.post.precedence",
                sign(ProcessorComparators.POST_PROCESSOR_COMPARATOR.compare(postLow, postHigh)));

        final IPreProcessor preA = new APreProcessor();
        final IPreProcessor preB = new BPreProcessor();
        final IPostProcessor postA = new APostProcessor();
        final IPostProcessor postB = new BPostProcessor();
        emit("ordering.pre.implementation_class",
                sign(ProcessorComparators.PRE_PROCESSOR_COMPARATOR.compare(preA, preB)));
        emit("ordering.post.implementation_class",
                sign(ProcessorComparators.POST_PROCESSOR_COMPARATOR.compare(postA, postB)));

        final IProcessorDialect dialectLow = new ProbeProcessorDialect("low", -10);
        final IProcessorDialect dialectHigh = new ProbeProcessorDialect("high", 10);
        final IPreProcessor wrappedPreLow =
                ProcessorConfigurationUtils.wrap(preA, dialectLow);
        final IPreProcessor wrappedPreHigh =
                ProcessorConfigurationUtils.wrap(preB, dialectHigh);
        final IPostProcessor wrappedPostLow =
                ProcessorConfigurationUtils.wrap(postA, dialectLow);
        final IPostProcessor wrappedPostHigh =
                ProcessorConfigurationUtils.wrap(postB, dialectHigh);
        emit("ordering.pre.wrapped_dialect",
                sign(ProcessorComparators.PRE_PROCESSOR_COMPARATOR.compare(
                        wrappedPreLow, wrappedPreHigh)));
        emit("ordering.post.wrapped_dialect",
                sign(ProcessorComparators.POST_PROCESSOR_COMPARATOR.compare(
                        wrappedPostLow, wrappedPostHigh)));
        emit("ordering.pre.unwrap.identity",
                ProcessorConfigurationUtils.unwrap(wrappedPreLow) == preA);
        emit("ordering.post.unwrap.identity",
                ProcessorConfigurationUtils.unwrap(wrappedPostLow) == postA);
    }

    @SuppressWarnings("deprecation")
    private static void exportFreshHandlers() throws InstantiationException, IllegalAccessException {
        ProbeHandler.SEQUENCE.set(0);
        final PreProcessor pre =
                new PreProcessor(TemplateMode.HTML, ProbeHandler.class, 0);
        final ProbeHandler first = (ProbeHandler) pre.getHandlerClass().newInstance();
        final ProbeHandler second = (ProbeHandler) pre.getHandlerClass().newInstance();
        emit("handler.class.name", pre.getHandlerClass().getName());
        emit("handler.instances.distinct", first != second);
        emit("handler.instances.sequence", first.instanceId + "," + second.instanceId);
    }

    private static void exportHandlerConstructionFailures() {
        final TemplateEngine preEngine = new TemplateEngine();
        preEngine.addDialect(new ThrowingPreDialect());
        exportEngineFailure("handler.failure.pre",
                () -> preEngine.process("<p>pre</p>", new Context()));

        final TemplateEngine postEngine = new TemplateEngine();
        postEngine.addDialect(new ThrowingPostDialect());
        exportEngineFailure("handler.failure.post",
                () -> postEngine.process("<p>post</p>", new Context()));
    }

    private static String interfaceSignatures(final Class<?> type) {
        return Arrays.stream(type.getDeclaredMethods())
                .map(PrePostProcessorGolden::methodSignature)
                .sorted()
                .reduce((left, right) -> left + "," + right)
                .orElse("");
    }

    private static String methodSignature(final Method method) {
        return method.getName() + "():" + method.getReturnType().getSimpleName();
    }

    private static String constructorSignature(final Class<?> type) {
        final Constructor<?> constructor = type.getDeclaredConstructors()[0];
        final String parameters = Arrays.stream(constructor.getParameterTypes())
                .map(Class::getSimpleName)
                .reduce((left, right) -> left + "+" + right)
                .orElse("");
        return Modifier.toString(constructor.getModifiers()) + "(" + parameters + ")";
    }

    private static String state(
            final TemplateMode mode,
            final int precedence,
            final Class<? extends ITemplateHandler> handlerClass,
            final boolean stableIdentity) {
        return mode.name() + "|" + precedence + "|" + handlerClass.getName()
                + "|" + stableIdentity;
    }

    private static int sign(final int value) {
        return Integer.compare(value, 0);
    }

    private static void exportFailure(final String key, final ThrowingRunnable runnable) {
        try {
            runnable.run();
            emit(key, "NO_ERROR");
        } catch (final Throwable throwable) {
            emit(key, throwable.getClass().getName() + ":" + throwable.getMessage());
        }
    }

    private static void exportEngineFailure(
            final String key, final ThrowingRunnable runnable) {
        try {
            runnable.run();
            emit(key, "NO_ERROR");
        } catch (final Throwable throwable) {
            final Throwable cause = throwable.getCause();
            emit(key,
                    throwable.getClass().getName() + ":" + throwable.getMessage()
                            + "|cause="
                            + (cause == null
                                    ? "null"
                                    : cause.getClass().getName() + ":" + cause.getMessage()));
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private interface ThrowingRunnable {
        void run() throws Exception;
    }

    public static final class ProbeHandler extends AbstractTemplateHandler {
        private static final AtomicInteger SEQUENCE = new AtomicInteger();
        private final int instanceId;

        public ProbeHandler() {
            this.instanceId = SEQUENCE.incrementAndGet();
        }
    }

    public static final class ThrowingHandler extends AbstractTemplateHandler {
        public ThrowingHandler() {
            throw new IllegalStateException("handler boom");
        }
    }

    private static class CustomPreProcessor implements IPreProcessor {
        private final TemplateMode mode;
        private final Class<? extends ITemplateHandler> handlerClass;
        private final int precedence;

        CustomPreProcessor(
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
        public int getPrecedence() {
            return this.precedence;
        }

        @Override
        public Class<? extends ITemplateHandler> getHandlerClass() {
            return this.handlerClass;
        }
    }

    private static final class APreProcessor extends CustomPreProcessor {
        APreProcessor() {
            super(TemplateMode.HTML, ProbeHandler.class, 0);
        }
    }

    private static final class BPreProcessor extends CustomPreProcessor {
        BPreProcessor() {
            super(TemplateMode.HTML, ProbeHandler.class, 0);
        }
    }

    private static class CustomPostProcessor implements IPostProcessor {
        private final TemplateMode mode;
        private final Class<? extends ITemplateHandler> handlerClass;
        private final int precedence;

        CustomPostProcessor(
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
        public int getPrecedence() {
            return this.precedence;
        }

        @Override
        public Class<? extends ITemplateHandler> getHandlerClass() {
            return this.handlerClass;
        }
    }

    private static final class APostProcessor extends CustomPostProcessor {
        APostProcessor() {
            super(TemplateMode.HTML, ProbeHandler.class, 0);
        }
    }

    private static final class BPostProcessor extends CustomPostProcessor {
        BPostProcessor() {
            super(TemplateMode.HTML, ProbeHandler.class, 0);
        }
    }

    private static final class ProbeProcessorDialect extends AbstractDialect
            implements IProcessorDialect {
        private final int precedence;

        ProbeProcessorDialect(final String name, final int precedence) {
            super(name);
            this.precedence = precedence;
        }

        @Override
        public String getPrefix() {
            return null;
        }

        @Override
        public int getDialectProcessorPrecedence() {
            return this.precedence;
        }

        @Override
        public Set<IProcessor> getProcessors(final String dialectPrefix) {
            return Collections.emptySet();
        }
    }

    private static final class ThrowingPreDialect extends AbstractDialect
            implements IPreProcessorDialect {
        ThrowingPreDialect() {
            super("ThrowingPre");
        }

        @Override
        public int getDialectPreProcessorPrecedence() {
            return 0;
        }

        @Override
        public Set<IPreProcessor> getPreProcessors() {
            return Collections.singleton(
                    new PreProcessor(TemplateMode.HTML, ThrowingHandler.class, 0));
        }
    }

    private static final class ThrowingPostDialect extends AbstractDialect
            implements IPostProcessorDialect {
        ThrowingPostDialect() {
            super("ThrowingPost");
        }

        @Override
        public int getDialectPostProcessorPrecedence() {
            return 0;
        }

        @Override
        public Set<IPostProcessor> getPostProcessors() {
            return Collections.singleton(
                    new PostProcessor(TemplateMode.HTML, ThrowingHandler.class, 0));
        }
    }
}
