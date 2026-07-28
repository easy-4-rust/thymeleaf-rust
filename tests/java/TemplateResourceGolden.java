import java.io.File;
import java.io.IOException;
import java.io.Reader;
import java.nio.charset.Charset;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Comparator;

import org.thymeleaf.templateresource.FileTemplateResource;
import org.thymeleaf.templateresource.ITemplateResource;
import org.thymeleaf.templateresource.StringTemplateResource;

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
