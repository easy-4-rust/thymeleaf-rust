import java.io.IOException;
import java.io.Reader;

import org.thymeleaf.templateresource.ITemplateResource;
import org.thymeleaf.templateresource.StringTemplateResource;

public final class TemplateResourceGolden {

    private static final String JAVA_BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final String RUST_BASELINE = "b6c97b2df175370c8b6a94feaed0955af67712f9";

    public static void main(final String[] args) throws Exception {
        emit("java_baseline", JAVA_BASELINE);
        emit("rust_baseline", RUST_BASELINE);
        exportNullConstructor();
        exportEmptyResource();
        exportUnicodeResource();
        exportRelativeFailures();
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
