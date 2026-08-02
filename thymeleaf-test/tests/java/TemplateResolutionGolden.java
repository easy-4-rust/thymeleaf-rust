import java.io.Reader;
import java.io.StringReader;

import org.thymeleaf.cache.AlwaysValidCacheEntryValidity;
import org.thymeleaf.cache.ICacheEntryValidity;
import org.thymeleaf.cache.NonCacheableCacheEntryValidity;
import org.thymeleaf.templatemode.TemplateMode;
import org.thymeleaf.templateresolver.TemplateResolution;
import org.thymeleaf.templateresource.ITemplateResource;

/**
 * 从固定 Thymeleaf Java 源码导出 TemplateResolution Golden。
 */
public final class TemplateResolutionGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private TemplateResolutionGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        exportValidation();
        exportDefaultsAndIdentity();
        exportFullFlagsAndModes();
    }

    private static void exportValidation() {
        final ITemplateResource resource = new TestResource("resource", false);
        final ICacheEntryValidity validity = new AlwaysValidCacheEntryValidity();

        emitFailure(
                "null.resource",
                () -> new TemplateResolution(null, TemplateMode.HTML, validity));
        emitFailure(
                "null.mode",
                () -> new TemplateResolution(resource, null, validity));
        emitFailure(
                "null.validity",
                () -> new TemplateResolution(resource, TemplateMode.HTML, null));
        emitFailure(
                "null.validation_order",
                () -> new TemplateResolution(null, true, null, true, null));
    }

    private static void exportDefaultsAndIdentity() {
        final ITemplateResource resource = new TestResource("missing", false);
        final ICacheEntryValidity validity = new AlwaysValidCacheEntryValidity();
        final TemplateResolution resolution =
                new TemplateResolution(resource, TemplateMode.HTML, validity);

        emit("default.resource_identity", resolution.getTemplateResource() == resource);
        emit("default.resource_description", resolution.getTemplateResource().getDescription());
        emit("default.resource_exists", resolution.getTemplateResource().exists());
        emit(
                "default.existence_verified",
                resolution.isTemplateResourceExistenceVerified());
        emit("default.mode", resolution.getTemplateMode());
        emit("default.use_decoupled_logic", resolution.getUseDecoupledLogic());
        emit("default.validity_identity", resolution.getValidity() == validity);
        emit("default.validity_cacheable", resolution.getValidity().isCacheable());
        emit("default.validity_still_valid", resolution.getValidity().isCacheStillValid());
    }

    private static void exportFullFlagsAndModes() {
        for (final TemplateMode mode : TemplateMode.values()) {
            final boolean existenceVerified = mode.isMarkup();
            final boolean useDecoupledLogic = mode.isText();
            final ITemplateResource resource =
                    new TestResource("mode-" + mode, !existenceVerified);
            final ICacheEntryValidity validity = new NonCacheableCacheEntryValidity();
            final TemplateResolution resolution = new TemplateResolution(
                    resource,
                    existenceVerified,
                    mode,
                    useDecoupledLogic,
                    validity);
            final String prefix = "full." + mode;

            emit(prefix + ".resource_identity", resolution.getTemplateResource() == resource);
            emit(prefix + ".resource_exists", resolution.getTemplateResource().exists());
            emit(
                    prefix + ".existence_verified",
                    resolution.isTemplateResourceExistenceVerified());
            emit(prefix + ".mode", resolution.getTemplateMode());
            emit(prefix + ".use_decoupled_logic", resolution.getUseDecoupledLogic());
            emit(prefix + ".validity_identity", resolution.getValidity() == validity);
            emit(prefix + ".validity_cacheable", resolution.getValidity().isCacheable());
            emit(
                    prefix + ".validity_still_valid",
                    resolution.getValidity().isCacheStillValid());
        }
    }

    private static void emitFailure(final String key, final Operation operation) {
        try {
            operation.run();
            emit(key + ".class", "<none>");
            emit(key + ".message", "<none>");
        } catch (final RuntimeException exception) {
            emit(key + ".class", exception.getClass().getSimpleName());
            emit(key + ".message", exception.getMessage());
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private interface Operation {
        void run();
    }

    private static final class TestResource implements ITemplateResource {

        private final String description;
        private final boolean exists;

        private TestResource(final String description, final boolean exists) {
            this.description = description;
            this.exists = exists;
        }

        @Override
        public String getDescription() {
            return this.description;
        }

        @Override
        public String getBaseName() {
            return null;
        }

        @Override
        public boolean exists() {
            return this.exists;
        }

        @Override
        public Reader reader() {
            return new StringReader(this.description);
        }

        @Override
        public ITemplateResource relative(final String relativeLocation) {
            return this;
        }
    }
}
