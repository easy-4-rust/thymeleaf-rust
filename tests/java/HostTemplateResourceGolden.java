import java.io.ByteArrayInputStream;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStream;
import java.io.Reader;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Set;

import org.thymeleaf.templateresource.ClassLoaderTemplateResource;
import org.thymeleaf.templateresource.ITemplateResource;
import org.thymeleaf.templateresource.WebApplicationTemplateResource;
import org.thymeleaf.web.IWebApplication;

/**
 * 从固定 Thymeleaf 3.1.5.RELEASE 导出宿主模板资源行为。
 */
public final class HostTemplateResourceGolden {

    private static final String JAVA_BASELINE =
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private HostTemplateResourceGolden() {
    }

    public static void main(final String[] args) throws Exception {
        emit("java_baseline", JAVA_BASELINE);
        exportClassLoaderValidation();
        exportClassLoaderResource();
        exportWebApplicationValidation();
        exportWebApplicationResource();
    }

    private static void exportClassLoaderValidation() {
        exportFailure(
                "class_loader.path.null",
                () -> new ClassLoaderTemplateResource((ClassLoader) null, null, null));
        exportFailure(
                "class_loader.path.empty",
                () -> new ClassLoaderTemplateResource((ClassLoader) null, "", null));
        exportFailure(
                "class_loader.path.whitespace",
                () -> new ClassLoaderTemplateResource((ClassLoader) null, "\t \u3000", null));
    }

    private static void exportClassLoaderResource() throws Exception {
        final Path root = Files.createTempDirectory("thymeleaf-class-loader-golden-");
        try {
            final Path templates = Files.createDirectories(root.resolve("templates"));
            Files.write(
                    templates.resolve("main-latin1.txt"),
                    new byte[] {'c', 'a', 'f', (byte) 0xE9});

            try (URLClassLoader loader =
                    new URLClassLoader(new URL[] {root.toUri().toURL()}, null)) {
                final ITemplateResource resource = new ClassLoaderTemplateResource(
                        loader,
                        "/templates/../templates/main-latin1.txt",
                        "ISO-8859-1");
                emit("class_loader.description", resource.getDescription());
                emit("class_loader.base_name", resource.getBaseName());
                emit("class_loader.exists", resource.exists());
                emit("class_loader.reader", readAll(resource.reader()));
                emit("class_loader.fresh_readers", resource.reader() != resource.reader());

                final ITemplateResource relative = resource.relative("child.html");
                emit("class_loader.relative.description", relative.getDescription());
                emit("class_loader.relative.base_name", relative.getBaseName());
                emit("class_loader.relative.exists", relative.exists());

                exportFailure("class_loader.relative.null", () -> resource.relative(null));
                exportFailure("class_loader.relative.empty", () -> resource.relative(""));
                exportFailure(
                        "class_loader.relative.whitespace",
                        () -> resource.relative("\t \u3000"));

                final ITemplateResource missing =
                        new ClassLoaderTemplateResource(loader, "templates/missing.txt", "bad");
                emit("class_loader.missing.exists", missing.exists());
                exportFailure("class_loader.missing.reader", missing::reader);
            }
        } finally {
            deleteTree(root);
        }
    }

    private static void exportWebApplicationValidation() {
        exportFailure(
                "web.validation.order",
                () -> new WebApplicationTemplateResource(null, null, null));
        final TestWebApplication application = new TestWebApplication();
        exportFailure(
                "web.path.null",
                () -> new WebApplicationTemplateResource(application, null, null));
        exportFailure(
                "web.path.empty",
                () -> new WebApplicationTemplateResource(application, "", null));
        exportFailure(
                "web.path.whitespace",
                () -> new WebApplicationTemplateResource(application, "\t \u3000", null));
    }

    private static void exportWebApplicationResource() throws Exception {
        final TestWebApplication application = new TestWebApplication();
        application.resources.put(
                "/templates/main-latin1.txt",
                new byte[] {'c', 'a', 'f', (byte) 0xE9});

        final ITemplateResource resource = new WebApplicationTemplateResource(
                application,
                "templates/./other/../main-latin1.txt",
                "ISO-8859-1");
        emit("web.description", resource.getDescription());
        emit("web.base_name", resource.getBaseName());
        emit("web.exists", resource.exists());
        emit("web.exists.path", application.lastExistsPath);
        emit("web.reader", readAll(resource.reader()));
        emit("web.reader.path", application.lastReaderPath);
        emit("web.fresh_readers", resource.reader() != resource.reader());

        final ITemplateResource relative = resource.relative("../messages.properties");
        emit("web.relative.description", relative.getDescription());
        emit("web.relative.base_name", relative.getBaseName());
        emit("web.relative.exists", relative.exists());
        emit("web.relative.path", application.lastExistsPath);

        exportFailure("web.relative.null", () -> resource.relative(null));
        exportFailure("web.relative.empty", () -> resource.relative(""));
        exportFailure("web.relative.whitespace", () -> resource.relative("\t \u3000"));

        final ITemplateResource missing =
                new WebApplicationTemplateResource(application, "/missing.html", "bad");
        emit("web.missing.exists", missing.exists());
        exportFailure("web.missing.reader", missing::reader);
    }

    private static String readAll(final Reader reader) throws IOException {
        try (Reader current = reader) {
            final StringBuilder result = new StringBuilder();
            final char[] buffer = new char[32];
            int count;
            while ((count = current.read(buffer)) >= 0) {
                result.append(buffer, 0, count);
            }
            return result.toString();
        }
    }

    private static void exportFailure(final String key, final Operation operation) {
        try {
            operation.run();
            emit(key + ".class", "<none>");
            emit(key + ".message", "<none>");
        } catch (final Throwable throwable) {
            emit(key + ".class", throwable.getClass().getName());
            emit(key + ".message", throwable.getMessage());
        }
    }

    private static void deleteTree(final Path root) throws IOException {
        try (java.util.stream.Stream<Path> paths = Files.walk(root)) {
            paths.sorted(Collections.reverseOrder()).forEach(path -> {
                try {
                    Files.delete(path);
                } catch (final IOException exception) {
                    throw new RuntimeException(exception);
                }
            });
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private interface Operation {
        void run() throws Exception;
    }

    private static final class TestWebApplication implements IWebApplication {

        private final Map<String, byte[]> resources = new LinkedHashMap<>();
        private String lastExistsPath;
        private String lastReaderPath;

        @Override
        public boolean containsAttribute(final String name) {
            return false;
        }

        @Override
        public int getAttributeCount() {
            return 0;
        }

        @Override
        public Set<String> getAllAttributeNames() {
            return Collections.emptySet();
        }

        @Override
        public Map<String, Object> getAttributeMap() {
            return Collections.emptyMap();
        }

        @Override
        public Object getAttributeValue(final String name) {
            return null;
        }

        @Override
        public void setAttributeValue(final String name, final Object value) {
        }

        @Override
        public void removeAttribute(final String name) {
        }

        @Override
        public boolean resourceExists(final String path) {
            this.lastExistsPath = path;
            return this.resources.containsKey(path);
        }

        @Override
        public InputStream getResourceAsStream(final String path) {
            this.lastReaderPath = path;
            final byte[] contents = this.resources.get(path);
            return contents == null ? null : new ByteArrayInputStream(contents);
        }
    }
}
