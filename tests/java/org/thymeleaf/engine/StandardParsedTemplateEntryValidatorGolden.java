package org.thymeleaf.engine;

import java.lang.reflect.Proxy;

import org.thymeleaf.IEngineConfiguration;
import org.thymeleaf.cache.ICacheEntryValidity;
import org.thymeleaf.cache.StandardParsedTemplateEntryValidator;

/**
 * 从固定 Thymeleaf Java 源码导出 StandardParsedTemplateEntryValidator Golden。
 */
public final class StandardParsedTemplateEntryValidatorGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private StandardParsedTemplateEntryValidatorGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        final StandardParsedTemplateEntryValidator validator =
                new StandardParsedTemplateEntryValidator();
        emit("constructor.class", validator.getClass().getName());
        emit("constructor.serializable", validator instanceof java.io.Serializable);

        final RecordingValidity valid = new RecordingValidity(true);
        final TemplateModel validModel = model(valid);
        emit("valid.first", validator.checkIsValueStillValid(null, validModel, Long.MIN_VALUE));
        emit("valid.second", validator.checkIsValueStillValid(null, validModel, Long.MAX_VALUE));
        emit("valid.calls", valid.calls);
        emit("valid.cacheableCalls", valid.cacheableCalls);

        final RecordingValidity invalid = new RecordingValidity(false);
        emit("invalid.result", validator.checkIsValueStillValid(null, model(invalid), 17L));
        emit("invalid.calls", invalid.calls);
        emit("invalid.cacheableCalls", invalid.cacheableCalls);

        emitThrowable(
                "throwing.validity",
                () -> validator.checkIsValueStillValid(null, model(new ThrowingValidity()), 0L));
        emitThrowable(
                "null.validity",
                () -> validator.checkIsValueStillValid(null, model(null), 0L));
        emitThrowable(
                "null.model",
                () -> validator.checkIsValueStillValid(null, null, 0L));
    }

    private static TemplateModel model(final ICacheEntryValidity validity) {
        final IEngineConfiguration configuration =
                (IEngineConfiguration) Proxy.newProxyInstance(
                        StandardParsedTemplateEntryValidatorGolden.class.getClassLoader(),
                        new Class<?>[] { IEngineConfiguration.class },
                        (proxy, method, args) -> {
                            throw new AssertionError("configuration method called: " + method.getName());
                        });
        final TemplateData templateData =
                new TemplateData("template", null, null, null, validity);
        return new TemplateModel(
                configuration,
                templateData,
                new IEngineTemplateEvent[] {
                    TemplateStart.TEMPLATE_START_INSTANCE,
                    TemplateEnd.TEMPLATE_END_INSTANCE
                });
    }

    private static void emitThrowable(final String key, final ThrowingAction action) {
        try {
            action.run();
            emit(key, "NO_ERROR");
        } catch (final Throwable throwable) {
            emit(
                    key,
                    throwable.getClass().getName() + ":" + String.valueOf(throwable.getMessage()));
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private interface ThrowingAction {
        void run();
    }

    private static final class RecordingValidity implements ICacheEntryValidity {
        private final boolean valid;
        private int calls;
        private int cacheableCalls;

        private RecordingValidity(final boolean valid) {
            this.valid = valid;
        }

        public boolean isCacheable() {
            this.cacheableCalls++;
            return true;
        }

        public boolean isCacheStillValid() {
            this.calls++;
            return this.valid;
        }
    }

    private static final class ThrowingValidity implements ICacheEntryValidity {
        public boolean isCacheable() {
            return true;
        }

        public boolean isCacheStillValid() {
            throw new IllegalStateException("boom");
        }
    }
}
