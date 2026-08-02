import java.io.Reader;
import java.io.StringReader;
import java.lang.reflect.Proxy;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.Collections;
import java.util.HashMap;
import java.util.LinkedHashSet;
import java.util.Map;
import java.util.Set;

import org.thymeleaf.IEngineConfiguration;
import org.thymeleaf.TemplateEngine;
import org.thymeleaf.cache.AlwaysValidCacheEntryValidity;
import org.thymeleaf.cache.ICacheEntryValidity;
import org.thymeleaf.cache.TTLCacheEntryValidity;
import org.thymeleaf.templatemode.TemplateMode;
import org.thymeleaf.templateresolver.AbstractConfigurableTemplateResolver;
import org.thymeleaf.templateresolver.ClassLoaderTemplateResolver;
import org.thymeleaf.templateresolver.DefaultTemplateResolver;
import org.thymeleaf.templateresolver.FileTemplateResolver;
import org.thymeleaf.templateresolver.ITemplateResolver;
import org.thymeleaf.templateresolver.StringTemplateResolver;
import org.thymeleaf.templateresolver.TemplateResolution;
import org.thymeleaf.templateresolver.UrlTemplateResolver;
import org.thymeleaf.templateresolver.WebApplicationTemplateResolver;
import org.thymeleaf.templateresource.ITemplateResource;
import org.thymeleaf.web.IWebApplication;

/**
 * 从固定 Thymeleaf 3.1.5.RELEASE 导出模板解析器合同。
 */
public final class TemplateResolverGolden {

    private static final String JAVA_BASELINE =
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final IEngineConfiguration CONFIGURATION =
            new TemplateEngine().getConfiguration();

    private TemplateResolverGolden() {
    }

    public static void main(final String[] args) throws Exception {
        emit("java_baseline", JAVA_BASELINE);
        exportAbstractResolver();
        exportConfigurableResolver();
        exportDefaultResolver();
        exportStringResolver();
        exportFileResolver();
        exportClassLoaderResolver();
        exportUrlResolver();
        exportWebResolver();
    }

    private static void exportAbstractResolver() {
        final ProbeResolver resolver = new ProbeResolver();
        emit("abstract.name.default", resolver.getName());
        emit("abstract.order.default", resolver.getOrder());
        emit("abstract.resolvable.empty", resolver.getResolvablePatternSpec().isEmpty());
        emit("abstract.check.default", resolver.getCheckExistence());
        emit("abstract.decoupled.default", resolver.getUseDecoupledLogic());

        resolver.setName(null);
        resolver.setOrder(-2);
        resolver.setUseDecoupledLogic(true);
        emit("abstract.name.null", resolver.getName());
        emit("abstract.order.negative", resolver.getOrder());
        emit("abstract.decoupled.true", resolver.getUseDecoupledLogic());

        exportFailure(
                "abstract.resolve.null_configuration",
                () -> resolver.resolveTemplate(null, null, "name", null));
        exportFailure(
                "abstract.resolve.null_template",
                () -> resolver.resolveTemplate(CONFIGURATION, null, null, null));

        resolver.resetCalls();
        resolver.setResolvablePatterns(linkedSet("admin/*"));
        emit("abstract.resolve.pattern_rejected",
                resolver.resolveTemplate(CONFIGURATION, null, "public/x", null));
        emit("abstract.resolve.pattern_rejected.resource_calls", resolver.resourceCalls);

        resolver.resetCalls();
        resolver.returnNull = true;
        emit("abstract.resolve.null_resource",
                resolver.resolveTemplate(CONFIGURATION, null, "admin/x", null));
        emit("abstract.resolve.null_resource.mode_calls", resolver.modeCalls);
        emit("abstract.resolve.null_resource.validity_calls", resolver.validityCalls);

        resolver.returnNull = false;
        resolver.resourceExists = false;
        resolver.setCheckExistence(true);
        resolver.resetCalls();
        emit("abstract.resolve.missing",
                resolver.resolveTemplate(CONFIGURATION, null, "admin/x", null));
        emit("abstract.resolve.missing.resource_calls", resolver.resourceCalls);
        emit("abstract.resolve.missing.mode_calls", resolver.modeCalls);
        emit("abstract.resolve.missing.validity_calls", resolver.validityCalls);

        resolver.setCheckExistence(false);
        resolver.resetCalls();
        final TemplateResolution unchecked =
                resolver.resolveTemplate(CONFIGURATION, null, "admin/x", null);
        emit("abstract.resolve.unchecked.present", unchecked != null);
        emit("abstract.resolve.unchecked.verified",
                unchecked.isTemplateResourceExistenceVerified());
        emit("abstract.resolve.unchecked.mode_calls", resolver.modeCalls);
        emit("abstract.resolve.unchecked.validity_calls", resolver.validityCalls);
    }

    private static void exportConfigurableResolver() {
        final ProbeResolver resolver = new ProbeResolver();
        emit("config.prefix.default", resolver.getPrefix());
        emit("config.suffix.default", resolver.getSuffix());
        emit("config.force_suffix.default", resolver.getForceSuffix());
        emit("config.encoding.default", resolver.getCharacterEncoding());
        emit("config.mode.default", resolver.getTemplateMode());
        emit("config.force_mode.default", resolver.getForceTemplateMode());
        emit("config.cacheable.default", resolver.isCacheable());
        emit("config.ttl.default", resolver.getCacheTTLMs());
        emit("config.aliases.default_size", resolver.getTemplateAliases().size());

        resolver.setPrefix("/views/");
        resolver.setSuffix(".html");
        emit("config.resource.basic", resolver.resourceName("page"));
        emit("config.resource.known_extension", resolver.resourceName("page.xml"));
        resolver.setForceSuffix(true);
        emit("config.resource.force_suffix", resolver.resourceName("page.xml"));
        resolver.setPrefix("\t \u3000");
        resolver.setSuffix("\t \u3000");
        emit("config.resource.blank_affixes", resolver.resourceName("page"));

        resolver.setPrefix(null);
        resolver.setSuffix(null);
        resolver.setForceSuffix(false);
        final Map<String, String> first = new HashMap<String, String>();
        first.put("short", "first");
        resolver.setTemplateAliases(first);
        final Map<String, String> second = new HashMap<String, String>();
        second.put("short", "override");
        second.put("other", "second");
        resolver.setTemplateAliases(second);
        resolver.setTemplateAliases(null);
        emit("config.aliases.merge_size", resolver.getTemplateAliases().size());
        emit("config.aliases.override", resolver.resourceName("short"));
        emit("config.aliases.preserve", resolver.resourceName("other"));
        exportFailure("config.alias.null", () -> resolver.addTemplateAlias(null, "value"));
        exportFailure("config.alias.value_null", () -> resolver.addTemplateAlias("x", null));

        final String isolated = new String(new char[] {'p', '\uD800', 'x'});
        resolver.setPrefix(new String(new char[] {'\uD801'}));
        resolver.setSuffix(new String(new char[] {'\uD802'}));
        resolver.setForceSuffix(true);
        emit("config.resource.utf16", codeUnits(resolver.resourceName(isolated)));

        exportFailure(
                "config.mode.enum_null",
                () -> resolver.setTemplateMode((TemplateMode) null));
        exportFailure(
                "config.mode.string_null",
                () -> resolver.setTemplateMode((String) null));

        final ProbeResolver modes = new ProbeResolver();
        modes.setXmlTemplateModePatterns(linkedSet("*.data"));
        modes.setHtmlTemplateModePatterns(linkedSet("*.data"));
        emit("config.mode.pattern_precedence", modes.baseMode("sample.data"));
        emit("config.mode.auto_text", modes.baseMode("sample.txt"));
        modes.setTemplateMode(TemplateMode.CSS);
        modes.setForceTemplateMode(true);
        emit("config.mode.forced", modes.baseMode("sample.html"));

        final ProbeResolver validity = new ProbeResolver();
        validity.setCacheablePatterns(linkedSet("*.both"));
        validity.setNonCacheablePatterns(linkedSet("*.both", "*.none"));
        emitValidity("config.validity.both", validity.baseValidity("x.both"));
        emitValidity("config.validity.non_cacheable", validity.baseValidity("x.none"));
        validity.setCacheable(false);
        emitValidity("config.validity.default_false", validity.baseValidity("x.other"));
        validity.setCacheable(true);
        validity.setCacheTTLMs(-7L);
        emitValidity("config.validity.ttl", validity.baseValidity("x.other"));
    }

    private static void exportDefaultResolver() throws Exception {
        final DefaultTemplateResolver resolver = new DefaultTemplateResolver();
        emit("default.name", resolver.getName());
        emit("default.mode", resolver.getTemplateMode());
        emit("default.template", resolver.getTemplate());
        final TemplateResolution first =
                resolver.resolveTemplate(CONFIGURATION, null, "ignored", null);
        emit("default.reader.empty", readAll(first.getTemplateResource().reader()));
        emitValidity("default.validity", first.getValidity());

        resolver.setTemplate("fixed");
        resolver.setTemplateMode(TemplateMode.TEXT);
        final ITemplateResolver dynamic = resolver;
        final TemplateResolution fixed =
                dynamic.resolveTemplate(CONFIGURATION, "owner", "ignored-again", null);
        emit("default.dynamic.reader", readAll(fixed.getTemplateResource().reader()));
        emit("default.dynamic.mode", fixed.getTemplateMode());

        exportFailure(
                "default.mode.enum_null",
                () -> resolver.setTemplateMode((TemplateMode) null));
        exportFailure(
                "default.mode.string_null",
                () -> resolver.setTemplateMode((String) null));
        resolver.setTemplate(null);
        exportFailure(
                "default.template.null_resolution",
                () -> resolver.resolveTemplate(CONFIGURATION, null, "ignored", null));
    }

    private static void exportStringResolver() throws Exception {
        final StringTemplateResolver resolver = new StringTemplateResolver();
        emit("string_resolver.name", resolver.getName());
        emit("string_resolver.mode", resolver.getTemplateMode());
        emit("string_resolver.cacheable", resolver.isCacheable());
        emit("string_resolver.ttl", resolver.getCacheTTLMs());
        final String contents = "<p>你好 😀</p>";
        final TemplateResolution first =
                resolver.resolveTemplate(CONFIGURATION, null, contents, null);
        emit("string_resolver.reader", readAll(first.getTemplateResource().reader()));
        emitValidity("string_resolver.validity.default", first.getValidity());

        resolver.setCacheable(true);
        emitValidity(
                "string_resolver.validity.always",
                resolver.resolveTemplate(CONFIGURATION, null, "x", null).getValidity());
        resolver.setCacheTTLMs(-5L);
        emitValidity(
                "string_resolver.validity.ttl",
                resolver.resolveTemplate(CONFIGURATION, null, "x", null).getValidity());
        resolver.setUseDecoupledLogic(false);
        exportFailure(
                "string_resolver.decoupled.true",
                () -> resolver.setUseDecoupledLogic(true));
        exportFailure(
                "string_resolver.mode.enum_null",
                () -> resolver.setTemplateMode((TemplateMode) null));
        exportFailure(
                "string_resolver.mode.string_null",
                () -> resolver.setTemplateMode((String) null));
    }

    private static void exportFileResolver() throws Exception {
        final Path directory = Files.createTempDirectory("thymeleaf-file-resolver-");
        try {
            Files.write(
                    directory.resolve("main.txt"),
                    "file-body".getBytes(StandardCharsets.UTF_8));
            final FileTemplateResolver resolver = new FileTemplateResolver();
            resolver.setPrefix(directory.toString() + "/");
            resolver.setSuffix(".txt");
            resolver.setCheckExistence(true);
            final TemplateResolution resolution =
                    resolver.resolveTemplate(CONFIGURATION, null, "main", null);
            emit("file_resolver.present", resolution != null);
            emit("file_resolver.base_name", resolution.getTemplateResource().getBaseName());
            emit("file_resolver.reader", readAll(resolution.getTemplateResource().reader()));
            emit("file_resolver.verified", resolution.isTemplateResourceExistenceVerified());
        } finally {
            deleteTree(directory);
        }
        exportFailure(
                "file_resolver.empty_template",
                () -> new FileTemplateResolver()
                        .resolveTemplate(CONFIGURATION, null, "", null));
    }

    private static void exportClassLoaderResolver() throws Exception {
        final Path root = Files.createTempDirectory("thymeleaf-class-resolver-");
        try {
            final Path templates = Files.createDirectories(root.resolve("templates"));
            Files.write(
                    templates.resolve("main.txt"),
                    "class-body".getBytes(StandardCharsets.UTF_8));
            try (URLClassLoader loader =
                    new URLClassLoader(new URL[] {root.toUri().toURL()}, null)) {
                final ClassLoaderTemplateResolver resolver =
                        new ClassLoaderTemplateResolver(loader);
                resolver.setPrefix("templates/");
                resolver.setSuffix(".txt");
                resolver.setCheckExistence(true);
                final TemplateResolution resolution =
                        resolver.resolveTemplate(CONFIGURATION, null, "main", null);
                emit("class_resolver.name", resolver.getName());
                emit("class_resolver.present", resolution != null);
                emit("class_resolver.base_name", resolution.getTemplateResource().getBaseName());
                emit("class_resolver.reader", readAll(resolution.getTemplateResource().reader()));
                emit("class_resolver.verified",
                        resolution.isTemplateResourceExistenceVerified());
            }
        } finally {
            deleteTree(root);
        }
        exportFailure(
                "class_resolver.empty_template",
                () -> new ClassLoaderTemplateResolver()
                        .resolveTemplate(CONFIGURATION, null, "", null));
    }

    private static void exportUrlResolver() {
        final UrlTemplateResolver resolver = new UrlTemplateResolver();
        emit("url_resolver.name", resolver.getName());
        emit("url_resolver.malformed",
                resolver.resolveTemplate(CONFIGURATION, null, "not-a-url", null));
        exportFailure(
                "url_resolver.empty_template",
                () -> resolver.resolveTemplate(CONFIGURATION, null, "", null));

        final UrlProbeResolver probe = new UrlProbeResolver();
        emitValidity(
                "url_resolver.jsessionid",
                probe.validity("HTTP://example.test/a;JSESSIONID=1"));
        emitValidity(
                "url_resolver.jsessionid_newline",
                probe.validity("http://example.test/a;jsessionid=1\nx"));
    }

    private static void exportWebResolver() {
        exportFailure("web_resolver.null_application",
                () -> new WebApplicationTemplateResolver(null));
        final IWebApplication application = proxyApplication();
        final WebApplicationTemplateResolver resolver =
                new WebApplicationTemplateResolver(application);
        resolver.setPrefix("templates/");
        resolver.setSuffix(".html");
        final TemplateResolution resolution =
                resolver.resolveTemplate(CONFIGURATION, null, "main", null);
        emit("web_resolver.name", resolver.getName());
        emit("web_resolver.description", resolution.getTemplateResource().getDescription());
        emit("web_resolver.verified", resolution.isTemplateResourceExistenceVerified());
        exportFailure(
                "web_resolver.empty_template",
                () -> new WebApplicationTemplateResolver(application)
                        .resolveTemplate(CONFIGURATION, null, "", null));
    }

    private static IWebApplication proxyApplication() {
        return (IWebApplication) Proxy.newProxyInstance(
                TemplateResolverGolden.class.getClassLoader(),
                new Class<?>[] {IWebApplication.class},
                (proxy, method, arguments) -> {
                    final Class<?> type = method.getReturnType();
                    if (type == boolean.class) {
                        return false;
                    }
                    if (type == int.class) {
                        return 0;
                    }
                    if (Set.class.isAssignableFrom(type)) {
                        return Collections.emptySet();
                    }
                    return null;
                });
    }

    private static Set<String> linkedSet(final String... values) {
        return new LinkedHashSet<String>(Arrays.asList(values));
    }

    private static void emitValidity(final String key, final ICacheEntryValidity validity) {
        emit(key + ".type", validity.getClass().getSimpleName());
        if (validity instanceof TTLCacheEntryValidity) {
            emit(key + ".ttl", ((TTLCacheEntryValidity) validity).getCacheTTLMs());
        }
    }

    private static void exportFailure(final String key, final ThrowingAction action) {
        try {
            action.run();
            emit(key, "<no-error>");
        } catch (final Throwable throwable) {
            emit(key + ".type", throwable.getClass().getName());
            emit(key + ".message", throwable.getMessage());
        }
    }

    private static String readAll(final Reader reader) throws Exception {
        try (Reader closeable = reader) {
            final StringBuilder result = new StringBuilder();
            final char[] buffer = new char[64];
            int read;
            while ((read = closeable.read(buffer)) >= 0) {
                result.append(buffer, 0, read);
            }
            return result.toString();
        }
    }

    private static String codeUnits(final String value) {
        final StringBuilder result = new StringBuilder();
        for (int index = 0; index < value.length(); index++) {
            if (index > 0) {
                result.append(',');
            }
            result.append(String.format("%04X", (int) value.charAt(index)));
        }
        return result.toString();
    }

    private static void deleteTree(final Path root) throws Exception {
        if (!Files.exists(root)) {
            return;
        }
        try (java.util.stream.Stream<Path> paths = Files.walk(root)) {
            final Path[] ordered = paths
                    .sorted(Collections.reverseOrder())
                    .toArray(Path[]::new);
            for (final Path path : ordered) {
                Files.deleteIfExists(path);
            }
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    @FunctionalInterface
    private interface ThrowingAction {
        void run() throws Exception;
    }

    private static final class ProbeResource implements ITemplateResource {
        private final String description;
        private final boolean exists;

        private ProbeResource(final String description, final boolean exists) {
            this.description = description;
            this.exists = exists;
        }

        @Override
        public String getDescription() {
            return this.description;
        }

        @Override
        public String getBaseName() {
            return this.description;
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
            return new ProbeResource(relativeLocation, this.exists);
        }
    }

    private static class ProbeResolver extends AbstractConfigurableTemplateResolver {
        private boolean returnNull;
        private boolean resourceExists = true;
        private int resourceCalls;
        private int modeCalls;
        private int validityCalls;

        private void resetCalls() {
            this.resourceCalls = 0;
            this.modeCalls = 0;
            this.validityCalls = 0;
        }

        private String resourceName(final String template) {
            return super.computeResourceName(
                    CONFIGURATION,
                    null,
                    template,
                    getPrefix(),
                    getSuffix(),
                    getForceSuffix(),
                    getTemplateAliases(),
                    null);
        }

        private TemplateMode baseMode(final String template) {
            return super.computeTemplateMode(CONFIGURATION, null, template, null);
        }

        private ICacheEntryValidity baseValidity(final String template) {
            return super.computeValidity(CONFIGURATION, null, template, null);
        }

        @Override
        protected ITemplateResource computeTemplateResource(
                final IEngineConfiguration configuration,
                final String ownerTemplate,
                final String template,
                final String resourceName,
                final String characterEncoding,
                final Map<String, Object> templateResolutionAttributes) {
            this.resourceCalls++;
            return this.returnNull ? null : new ProbeResource(resourceName, this.resourceExists);
        }

        @Override
        protected TemplateMode computeTemplateMode(
                final IEngineConfiguration configuration,
                final String ownerTemplate,
                final String template,
                final Map<String, Object> templateResolutionAttributes) {
            this.modeCalls++;
            return super.computeTemplateMode(
                    configuration, ownerTemplate, template, templateResolutionAttributes);
        }

        @Override
        protected ICacheEntryValidity computeValidity(
                final IEngineConfiguration configuration,
                final String ownerTemplate,
                final String template,
                final Map<String, Object> templateResolutionAttributes) {
            this.validityCalls++;
            return super.computeValidity(
                    configuration, ownerTemplate, template, templateResolutionAttributes);
        }
    }

    private static final class UrlProbeResolver extends UrlTemplateResolver {
        private ICacheEntryValidity validity(final String template) {
            return super.computeValidity(CONFIGURATION, null, template, null);
        }
    }
}
