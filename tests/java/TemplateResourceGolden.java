import java.io.File;
import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.io.Reader;
import java.net.ServerSocket;
import java.net.Socket;
import java.net.SocketException;
import java.nio.charset.Charset;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Comparator;
import java.util.List;
import java.net.URL;

import org.thymeleaf.templateresource.FileTemplateResource;
import org.thymeleaf.templateresource.ITemplateResource;
import org.thymeleaf.templateresource.StringTemplateResource;
import org.thymeleaf.templateresource.UrlTemplateResource;

public final class TemplateResourceGolden {

    private static final String JAVA_BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final String RUST_BASELINE = "eca81ffdc14b721e60cbfc812cb701ffb8fae7ba";

    public static void main(final String[] args) throws Exception {
        emit("java_baseline", JAVA_BASELINE);
        emit("rust_baseline", RUST_BASELINE);
        exportNullConstructor();
        exportEmptyResource();
        exportUnicodeResource();
        exportRelativeFailures();
        exportFileValidation();
        exportFilePaths();
        exportFileReaders();
        exportUrlValidation();
        exportUrlPaths();
        exportUrlFiles();
        exportUrlHttp();
    }

    private static void exportNullConstructor() {
        try {
            new StringTemplateResource(null);
            emit("string.null", "<no-error>");
        } catch (final Throwable throwable) {
            emit("string.null.type", throwable.getClass().getName());
            emit("string.null.message", throwable.getMessage());
        }
    }

    private static void exportEmptyResource() throws IOException {
        final ITemplateResource resource = new StringTemplateResource("");
        emit("string.empty.description", resource.getDescription());
        emit("string.empty.base_name", resource.getBaseName());
        emit("string.empty.exists", resource.exists());
        emit("string.empty.reader", readAll(resource.reader()));
        emit("string.empty.fresh_readers", resource.reader() != resource.reader());
    }

    private static void exportUnicodeResource() throws IOException {
        final String contents = "<p>你好 😀</p>\r\n\u0000tail";
        final ITemplateResource resource = new StringTemplateResource(contents);
        emit("string.unicode.description", resource.getDescription());
        emit("string.unicode.base_name", resource.getBaseName());
        emit("string.unicode.exists", resource.exists());

        final Reader first = resource.reader();
        final char[] prefix = new char[3];
        emit("string.unicode.prefix_count", first.read(prefix));
        emit("string.unicode.prefix", new String(prefix));
        emit("string.unicode.second_full", readAll(resource.reader()));
        emit("string.unicode.first_remaining", readAll(first));
    }

    private static void exportRelativeFailures() {
        final ITemplateResource resource = new StringTemplateResource("line1\n\"line2\"");
        exportRelativeFailure(resource, "null", null);
        exportRelativeFailure(resource, "empty", "");
        exportRelativeFailure(resource, "child", "child.html");
    }

    private static void exportRelativeFailure(
            final ITemplateResource resource, final String name, final String relativeLocation) {
        try {
            resource.relative(relativeLocation);
            emit("string.relative." + name, "<no-error>");
        } catch (final Throwable throwable) {
            emit("string.relative." + name + ".type", throwable.getClass().getName());
            emit("string.relative." + name + ".message", throwable.getMessage());
        }
    }

    private static void exportFileValidation() {
        exportFileStringConstructorFailure("null", null);
        exportFileStringConstructorFailure("empty", "");
        exportFileStringConstructorFailure("whitespace", "\t \u3000");

        try {
            new FileTemplateResource((File) null, null);
            emit("file.null_file", "<no-error>");
        } catch (final Throwable throwable) {
            emit("file.null_file.type", throwable.getClass().getName());
            emit("file.null_file.message", throwable.getMessage());
        }

        final ITemplateResource emptyFile = new FileTemplateResource(new File(""), null);
        emit("file.empty_file.description_is_user_dir",
                emptyFile.getDescription().equals(System.getProperty("user.dir")));
        emit("file.empty_file.base_name", emptyFile.getBaseName());
        emit("file.empty_file.exists", emptyFile.exists());
    }

    private static void exportFileStringConstructorFailure(final String name, final String path) {
        try {
            new FileTemplateResource(path, null);
            emit("file.path." + name, "<no-error>");
        } catch (final Throwable throwable) {
            emit("file.path." + name + ".type", throwable.getClass().getName());
            emit("file.path." + name + ".message", throwable.getMessage());
        }
    }

    private static void exportFilePaths() {
        final ITemplateResource resource =
                new FileTemplateResource("something/else/../more.html", "ISO-8859-1");
        emit("file.path.description_suffix",
                slash(resource.getDescription()).endsWith("/something/else/../more.html"));
        emit("file.path.base_name", resource.getBaseName());
        emit("file.path.exists", resource.exists());

        final ITemplateResource duplicate = new FileTemplateResource("//something//else", null);
        emit("file.path.duplicate.description", slash(duplicate.getDescription()));
        emit("file.path.duplicate.base_name", duplicate.getBaseName());

        exportFileRelativeFailure(resource, "null", null);
        exportFileRelativeFailure(resource, "empty", "");
        exportFileRelativeFailure(resource, "whitespace", "\t \u3000");

        final ITemplateResource relative = resource.relative("../more_es.properties");
        emit("file.relative.valid.description_suffix",
                slash(relative.getDescription()).endsWith("/something/../more_es.properties"));
        emit("file.relative.valid.base_name", relative.getBaseName());
        emit("file.relative.valid.exists", relative.exists());
    }

    private static void exportFileRelativeFailure(
            final ITemplateResource resource, final String name, final String relativeLocation) {
        try {
            resource.relative(relativeLocation);
            emit("file.relative." + name, "<no-error>");
        } catch (final Throwable throwable) {
            emit("file.relative." + name + ".type", throwable.getClass().getName());
            emit("file.relative." + name + ".message", throwable.getMessage());
        }
    }

    private static void exportFileReaders() throws Exception {
        final Path directory = Files.createTempDirectory("thymeleaf-file-golden-");
        try {
            exportDecode(directory, "default", null, "默认😀".getBytes(Charset.forName("UTF-8")));
            exportDecode(directory, "blank", "\t \u3000", "默认😀".getBytes(Charset.forName("UTF-8")));
            exportDecode(directory, "utf8_bom", "UTF8",
                    new byte[] {(byte) 0xEF, (byte) 0xBB, (byte) 0xBF, 0x61});
            exportDecode(directory, "utf8_malformed", "UTF-8",
                    new byte[] {0x61, (byte) 0xC0, (byte) 0xAF, (byte) 0xE2, (byte) 0x82, 0x62});
            exportDecode(directory, "ascii", "ASCII",
                    new byte[] {0x61, (byte) 0x80, (byte) 0x81, 0x62});
            exportDecode(directory, "latin1", "ISO8859_1",
                    new byte[] {0x61, (byte) 0x80, (byte) 0xFF});
            exportDecode(directory, "utf16_bom_be", "UTF-16",
                    new byte[] {(byte) 0xFE, (byte) 0xFF, 0x00, 0x61});
            exportDecode(directory, "utf16_bom_le", "Unicode",
                    new byte[] {(byte) 0xFF, (byte) 0xFE, 0x61, 0x00});
            exportDecode(directory, "utf16_no_bom", "UTF-16", new byte[] {0x00, 0x61});
            exportDecode(directory, "utf16be_explicit_bom", "UnicodeBigUnmarked",
                    new byte[] {(byte) 0xFE, (byte) 0xFF, 0x00, 0x61});
            exportDecode(directory, "utf16le_explicit_bom", "UnicodeLittleUnmarked",
                    new byte[] {(byte) 0xFF, (byte) 0xFE, 0x61, 0x00});
            exportDecode(directory, "windows1252", "windows-1252",
                    new byte[] {0x61, (byte) 0x80, (byte) 0x81});
            exportDecode(directory, "gbk", "GBK",
                    new byte[] {(byte) 0xC4, (byte) 0xE3, (byte) 0xBA, (byte) 0xC3});

            final Path freshPath = directory.resolve("fresh.txt");
            Files.write(freshPath, "fresh-reader".getBytes(Charset.forName("UTF-8")));
            final ITemplateResource fresh =
                    new FileTemplateResource(freshPath.toFile(), "UTF-8");
            emit("file.reader.exists", fresh.exists());
            emit("file.reader.fresh", fresh.reader() != fresh.reader());
            emit("file.reader.first", readAll(fresh.reader()));
            emit("file.reader.second", readAll(fresh.reader()));

            final Path unsupportedPath = directory.resolve("unsupported.txt");
            Files.write(unsupportedPath, new byte[] {0x61});
            exportReaderFailure(
                    "file.reader.unsupported",
                    new FileTemplateResource(unsupportedPath.toFile(), " UTF-8 "));
            exportReaderFailure(
                    "file.reader.unknown",
                    new FileTemplateResource(unsupportedPath.toFile(), "not-a-charset"));

            exportMissingReaderFailure(
                    directory,
                    new FileTemplateResource(directory.resolve("missing.txt").toFile(), "not-a-charset"));
        } finally {
            try (java.util.stream.Stream<Path> paths = Files.walk(directory)) {
                paths.sorted(Comparator.reverseOrder()).forEach(path -> {
                    try {
                        Files.delete(path);
                    } catch (final IOException exception) {
                        throw new RuntimeException(exception);
                    }
                });
            }
        }
    }

    private static void exportDecode(
            final Path directory,
            final String name,
            final String characterEncoding,
            final byte[] bytes) throws IOException {
        final Path file = directory.resolve(name + ".txt");
        Files.write(file, bytes);
        final ITemplateResource resource =
                new FileTemplateResource(file.toFile(), characterEncoding);
        emit("file.decode." + name, codePoints(readAll(resource.reader())));
    }

    private static void exportReaderFailure(final String key, final ITemplateResource resource) {
        try {
            resource.reader();
            emit(key, "<no-error>");
        } catch (final Throwable throwable) {
            emit(key + ".type", throwable.getClass().getName());
            emit(key + ".message", codePoints(throwable.getMessage()));
        }
    }

    private static void exportMissingReaderFailure(
            final Path directory, final ITemplateResource resource) {
        try {
            resource.reader();
            emit("file.reader.missing_precedes_charset", "<no-error>");
        } catch (final Throwable throwable) {
            emit("file.reader.missing_precedes_charset.type", throwable.getClass().getName());
            emit("file.reader.missing_precedes_charset.message_mentions_file",
                    throwable.getMessage() != null
                            && slash(throwable.getMessage()).contains(
                                    slash(directory.resolve("missing.txt").toString())));
        }
    }

    private static void exportUrlValidation() {
        for (final Object[] value : new Object[][] {
                {"null", null},
                {"empty", ""},
                {"whitespace", "\t \u3000"}}) {
            try {
                new UrlTemplateResource((String) value[1], null);
                emit("url.path." + value[0], "<no-error>");
            } catch (final Throwable throwable) {
                emit("url.path." + value[0] + ".type", throwable.getClass().getName());
                emit("url.path." + value[0] + ".message", throwable.getMessage());
            }
        }

        try {
            new UrlTemplateResource((URL) null, null);
            emit("url.null_url", "<no-error>");
        } catch (final Throwable throwable) {
            emit("url.null_url.type", throwable.getClass().getName());
            emit("url.null_url.message", throwable.getMessage());
        }

        try {
            new UrlTemplateResource("not-a-url", null);
            emit("url.malformed", "<no-error>");
        } catch (final Throwable throwable) {
            emit("url.malformed.type", throwable.getClass().getName());
            emit("url.malformed.message_nonempty",
                    throwable.getMessage() != null && !throwable.getMessage().isEmpty());
        }
    }

    private static void exportUrlPaths() throws Exception {
        final String[] descriptions = {
                "http://www.thymeleaf.org/",
                "http://www.thymeleaf.org",
                "http://www.thymeleaf.org/something",
                "http://www.thymeleaf.org/something/",
                "http://www.thymeleaf.org/something/else",
                "http://www.thymeleaf.org/something/else.html",
                "http://www.thymeleaf.org/something/./else.html",
                "http://www.thymeleaf.org/something/more/../else.html",
                "http://www.thymeleaf.org/something/./more/../else.html"
        };
        for (int index = 0; index < descriptions.length; index++) {
            emit("url.description." + index,
                    new UrlTemplateResource(descriptions[index], null).getDescription());
        }

        final String[][] relatives = {
                {"http://www.thymeleaf.org/", "/"},
                {"http://www.thymeleaf.org", "/"},
                {"http://www.thymeleaf.org", "/something"},
                {"http://www.thymeleaf.org", "something"},
                {"http://www.thymeleaf.org/more", "something"},
                {"http://www.thymeleaf.org/more/", "something"},
                {"http://www.thymeleaf.org/something/else", "more"},
                {"http://www.thymeleaf.org/something/else.html", "more.html"},
                {"http://www.thymeleaf.org/something/else.html", "../more.html"},
                {"http://www.thymeleaf.org/something/more/../else.html", "../less.html"},
                {"http://www.thymeleaf.org/something/more/../else.html", "../even/less.html"},
                {"http://www.thymeleaf.org/something/./more/../else.html", "../even/./less.html"}
        };
        for (int index = 0; index < relatives.length; index++) {
            final ITemplateResource resource =
                    new UrlTemplateResource(relatives[index][0], "ISO-8859-1");
            emit("url.relative." + index, resource.relative(relatives[index][1]).getDescription());
        }

        final String[] baseNames = {
                "http://www.thymeleaf.org/",
                "http://www.thymeleaf.org",
                "http://www.thymeleaf.org/more",
                "http://www.thymeleaf.org/more/",
                "http://www.thymeleaf.org/something/else",
                "http://www.thymeleaf.org/something/else.html",
                "http://www.thymeleaf.org/something/more/../else.html",
                "http://www.thymeleaf.org/something/more/../else.html/",
                "http://www.thymeleaf.org/something/more/../else.html/a/..",
                "http://www.thymeleaf.org/something/./more/../else.html",
                "http://www.thymeleaf.org/something/./more/../else.html?param=a"
        };
        for (int index = 0; index < baseNames.length; index++) {
            emit("url.base_name." + index,
                    new UrlTemplateResource(baseNames[index], null).getBaseName());
        }

        final ITemplateResource failure =
                new UrlTemplateResource("http://www.thymeleaf.org/base.html", null);
        for (final Object[] value : new Object[][] {
                {"null", null},
                {"empty", ""},
                {"whitespace", "\t \u3000"}}) {
            try {
                failure.relative((String) value[1]);
                emit("url.relative_failure." + value[0], "<no-error>");
            } catch (final Throwable throwable) {
                emit("url.relative_failure." + value[0] + ".type",
                        throwable.getClass().getName());
                emit("url.relative_failure." + value[0] + ".message",
                        throwable.getMessage());
            }
        }

        try {
            failure.relative("http://[");
            emit("url.relative_failure.malformed", "<no-error>");
        } catch (final Throwable throwable) {
            emit("url.relative_failure.malformed.type", throwable.getClass().getName());
            emit("url.relative_failure.malformed.message",
                    throwable.getMessage());
            emit("url.relative_failure.malformed.cause_type",
                    throwable.getCause() == null ? null : throwable.getCause().getClass().getName());
        }
    }

    private static void exportUrlFiles() throws Exception {
        final Path directory = Files.createTempDirectory("thymeleaf-url-golden-");
        try {
            final Path parent = Files.createDirectory(directory.resolve("space dir"));
            final Path primary = parent.resolve("main.html");
            final Path sibling = parent.resolve("child.html");
            Files.write(primary, new byte[] {0x61, (byte) 0xE9});
            Files.write(sibling, new byte[] {0x62, (byte) 0xE9});

            final ITemplateResource resource =
                    new UrlTemplateResource(primary.toUri().toURL(), "ISO-8859-1");
            emit("url.file.description_scheme", resource.getDescription().startsWith("file:"));
            emit("url.file.description_has_escaped_space",
                    resource.getDescription().contains("space%20dir"));
            emit("url.file.base_name", resource.getBaseName());
            emit("url.file.exists", resource.exists());
            emit("url.file.reader", codePoints(readAll(resource.reader())));
            emit("url.file.reader_fresh", resource.reader() != resource.reader());

            final ITemplateResource relative = resource.relative("child.html");
            emit("url.file.relative.base_name", relative.getBaseName());
            emit("url.file.relative.exists", relative.exists());
            emit("url.file.relative.reader", codePoints(readAll(relative.reader())));

            final ITemplateResource missing = resource.relative("missing.html");
            emit("url.file.missing.exists", missing.exists());
            try {
                missing.reader();
                emit("url.file.missing.reader", "<no-error>");
            } catch (final Throwable throwable) {
                emit("url.file.missing.reader.type", throwable.getClass().getName());
            }
        } finally {
            try (java.util.stream.Stream<Path> paths = Files.walk(directory)) {
                paths.sorted(Comparator.reverseOrder()).forEach(path -> {
                    try {
                        Files.delete(path);
                    } catch (final IOException exception) {
                        throw new RuntimeException(exception);
                    }
                });
            }
        }
    }

    private static void exportUrlHttp() throws Exception {
        final LocalHttpServer server = new LocalHttpServer(10);
        try {
            emit("url.http.exists.ok",
                    new UrlTemplateResource(server.url("/ok"), null).exists());
            emit("url.http.exists.not_found",
                    new UrlTemplateResource(server.url("/not-found"), null).exists());
            emit("url.http.exists.other_with_length",
                    new UrlTemplateResource(server.url("/other-with-length"), null).exists());
            emit("url.http.exists.other_without_length",
                    new UrlTemplateResource(server.url("/other-without-length"), null).exists());

            emit("url.http.reader.ok",
                    readAll(new UrlTemplateResource(server.url("/ok"), "UTF-8").reader()));
            emit("url.http.reader.latin1",
                    codePoints(readAll(
                            new UrlTemplateResource(server.url("/latin1"), "ISO-8859-1").reader())));

            try {
                new UrlTemplateResource(server.url("/ok"), "not-a-charset").reader();
                emit("url.http.reader.unsupported", "<no-error>");
            } catch (final Throwable throwable) {
                emit("url.http.reader.unsupported.type", throwable.getClass().getName());
            }

            try {
                new UrlTemplateResource(server.url("/not-found"), "UTF-8").reader();
                emit("url.http.reader.not_found", "<no-error>");
            } catch (final Throwable throwable) {
                emit("url.http.reader.not_found.type", throwable.getClass().getName());
            }

            final ITemplateResource fresh =
                    new UrlTemplateResource(server.url("/ok"), "UTF-8");
            emit("url.http.reader.fresh", fresh.reader() != fresh.reader());

            server.await();
            emit("url.http.server.head_count", server.count("HEAD"));
            emit("url.http.server.get_count", server.count("GET"));
        } finally {
            server.close();
        }

        final int unavailablePort;
        try (ServerSocket unavailable = new ServerSocket(0)) {
            unavailablePort = unavailable.getLocalPort();
        }
        emit("url.http.exists.connection_refused",
                new UrlTemplateResource(
                        "http://127.0.0.1:" + unavailablePort + "/unavailable", null).exists());
    }

    private static final class LocalHttpServer implements AutoCloseable {

        private final ServerSocket serverSocket;
        private final Thread thread;
        private final int expectedRequests;
        private final List<String> methods =
                Collections.synchronizedList(new ArrayList<String>());
        private volatile Throwable failure;

        private LocalHttpServer(final int expectedRequests) throws IOException {
            this.serverSocket = new ServerSocket(0);
            this.expectedRequests = expectedRequests;
            this.thread = new Thread(this::serve, "thymeleaf-url-golden-http");
            this.thread.setDaemon(true);
            this.thread.start();
        }

        private String url(final String path) {
            return "http://127.0.0.1:" + this.serverSocket.getLocalPort() + path;
        }

        private int count(final String method) {
            synchronized (this.methods) {
                int count = 0;
                for (final String observed : this.methods) {
                    if (method.equals(observed)) {
                        count++;
                    }
                }
                return count;
            }
        }

        private void serve() {
            try {
                for (int index = 0; index < this.expectedRequests; index++) {
                    try (Socket socket = this.serverSocket.accept()) {
                        handle(socket);
                    }
                }
            } catch (final SocketException exception) {
                if (!this.serverSocket.isClosed()) {
                    this.failure = exception;
                }
            } catch (final Throwable throwable) {
                this.failure = throwable;
            }
        }

        private void handle(final Socket socket) throws IOException {
            final BufferedReader input = new BufferedReader(
                    new InputStreamReader(socket.getInputStream(), StandardCharsets.ISO_8859_1));
            final String requestLine = input.readLine();
            if (requestLine == null) {
                throw new IOException("Missing HTTP request line");
            }
            String line;
            while ((line = input.readLine()) != null && !line.isEmpty()) {
                // Consume all request headers before writing the response.
            }

            final String[] request = requestLine.split(" ");
            final String method = request[0];
            final String path = request[1];
            this.methods.add(method);

            final int status;
            final String reason;
            final byte[] body;
            final boolean includeLength;
            if ("/ok".equals(path)) {
                status = 200;
                reason = "OK";
                body = new byte[] {0x6F, 0x6B};
                includeLength = true;
            } else if ("/latin1".equals(path)) {
                status = 200;
                reason = "OK";
                body = new byte[] {0x61, (byte) 0xE9};
                includeLength = true;
            } else if ("/not-found".equals(path)) {
                status = 404;
                reason = "Not Found";
                body = new byte[0];
                includeLength = true;
            } else if ("/other-with-length".equals(path)) {
                status = 500;
                reason = "Server Error";
                body = new byte[] {0x65, 0x72};
                includeLength = true;
            } else {
                status = 204;
                reason = "No Content";
                body = new byte[0];
                includeLength = false;
            }

            final OutputStream output = socket.getOutputStream();
            output.write(("HTTP/1.1 " + status + " " + reason + "\r\n")
                    .getBytes(StandardCharsets.ISO_8859_1));
            if (includeLength) {
                output.write(("Content-Length: " + body.length + "\r\n")
                        .getBytes(StandardCharsets.ISO_8859_1));
            }
            output.write("Connection: close\r\n\r\n"
                    .getBytes(StandardCharsets.ISO_8859_1));
            if (!"HEAD".equals(method)) {
                output.write(body);
            }
            output.flush();
        }

        private void await() throws Exception {
            this.thread.join(10_000L);
            if (this.thread.isAlive()) {
                throw new IllegalStateException("HTTP server did not receive all requests");
            }
            if (this.failure != null) {
                throw new Exception(this.failure);
            }
        }

        @Override
        public void close() throws Exception {
            this.serverSocket.close();
            this.thread.join(10_000L);
            if (this.failure != null) {
                throw new Exception(this.failure);
            }
        }
    }

    private static String codePoints(final String value) {
        final StringBuilder result = new StringBuilder();
        value.codePoints().forEach(codePoint -> {
            if (result.length() > 0) {
                result.append(',');
            }
            result.append(String.format("%04X", codePoint));
        });
        return result.toString();
    }

    private static String slash(final String value) {
        return value.replace(File.separatorChar, '/');
    }

    private static String readAll(final Reader reader) throws IOException {
        try (Reader closeable = reader) {
            final StringBuilder result = new StringBuilder();
            final char[] buffer = new char[4];
            int count;
            while ((count = closeable.read(buffer)) != -1) {
                result.append(buffer, 0, count);
            }
            return result.toString();
        }
    }

    private static void emit(final String key, final boolean value) {
        emit(key, Boolean.toString(value));
    }

    private static void emit(final String key, final int value) {
        emit(key, Integer.toString(value));
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + escape(value));
    }

    private static String escape(final String value) {
        if (value == null) {
            return "null";
        }
        return value
                .replace("\\", "\\\\")
                .replace("\r", "\\r")
                .replace("\n", "\\n")
                .replace("\u0000", "\\0");
    }
}
