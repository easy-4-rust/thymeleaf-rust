import java.nio.charset.Charset;

import org.thymeleaf.util.ContentTypeUtils;

/**
 * 从固定 Thymeleaf Java 源码导出 ContentTypeUtils Golden。
 */
public final class ContentTypeUtilsGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private ContentTypeUtilsGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emitContentTypes();
        emitTemplateNames();
        emitRequestPaths();
        emitCharsets();
        emitCombinations();
    }

    private static void emitContentTypes() {
        final String[] values = new String[] {
                null, "", " \t", "text/html", " APPLICATION/XHTML+XML ; q=1",
                "application/xml", "text/xml", "application/rss+xml",
                "application/atom+xml", "application/javascript",
                "application/x-javascript", "application/ecmascript",
                "text/javascript", "text/ecmascript", "application/json",
                "text/css", "text/plain", "text/event-stream",
                "application/octet-stream", "; TEXT/HTML ;; Charset=UTF-8", ";;;"
        };
        for (int index = 0; index < values.length; index++) {
            final String key = "content." + index;
            final String value = values[index];
            emitCall(key + ".html", () -> Boolean.toString(ContentTypeUtils.isContentTypeHTML(value)));
            emitCall(key + ".xml", () -> Boolean.toString(ContentTypeUtils.isContentTypeXML(value)));
            emitCall(key + ".rss", () -> Boolean.toString(ContentTypeUtils.isContentTypeRSS(value)));
            emitCall(key + ".atom", () -> Boolean.toString(ContentTypeUtils.isContentTypeAtom(value)));
            emitCall(key + ".javascript",
                    () -> Boolean.toString(ContentTypeUtils.isContentTypeJavaScript(value)));
            emitCall(key + ".json", () -> Boolean.toString(ContentTypeUtils.isContentTypeJSON(value)));
            emitCall(key + ".css", () -> Boolean.toString(ContentTypeUtils.isContentTypeCSS(value)));
            emitCall(key + ".text", () -> Boolean.toString(ContentTypeUtils.isContentTypeText(value)));
            emitCall(key + ".sse", () -> Boolean.toString(ContentTypeUtils.isContentTypeSSE(value)));
            emitCall(key + ".mode", () -> string(ContentTypeUtils.computeTemplateModeForContentType(value)));
        }
    }

    private static void emitTemplateNames() {
        final String[] values = new String[] {
                null, "", " \t", "index", "index.", "view.html", "view.HTML ",
                "archive.tar.xml", ".rss", "script.js", "data.json", "style.css",
                "plain.txt", "feed.atom", "unknown.bin", "name.xhtml"
        };
        for (int index = 0; index < values.length; index++) {
            final String key = "template." + index;
            final String value = values[index];
            emitCall(key + ".mode",
                    () -> string(ContentTypeUtils.computeTemplateModeForTemplateName(value)));
            emitCall(key + ".recognized",
                    () -> Boolean.toString(ContentTypeUtils.hasRecognizedFileExtension(value)));
            emitCall(key + ".plain",
                    () -> string(ContentTypeUtils.computeContentTypeForTemplateName(value, null)));
            emitCall(key + ".utf8", () -> string(ContentTypeUtils.computeContentTypeForTemplateName(
                    value, Charset.forName("UTF-8"))));
        }
    }

    private static void emitRequestPaths() {
        final String[] values = new String[] {
                null, "", "/", "/index", "/view.html", "/INDEX.HTML",
                "/asset/app.js?x=.css#part;v=1", "/style.css;v=2?x=1#part",
                "/feed.atom#fragment", "/data.json?x=1", "/plain.txt ",
                "relative.xml", "/dir.with.dot/file", "/dir/.rss"
        };
        for (int index = 0; index < values.length; index++) {
            final String key = "request." + index;
            final String value = values[index];
            emitCall(key + ".mode",
                    () -> string(ContentTypeUtils.computeTemplateModeForRequestPath(value)));
            emitCall(key + ".plain",
                    () -> string(ContentTypeUtils.computeContentTypeForRequestPath(value, null)));
            emitCall(key + ".latin1", () -> string(ContentTypeUtils.computeContentTypeForRequestPath(
                    value, Charset.forName("ISO-8859-1"))));
        }
    }

    private static void emitCharsets() {
        final String[] values = new String[] {
                null, "", "text/html", "text/html;charset=UTF-8",
                "text/html;CHARSET=latin1", "text/html;charset=UTF-16LE",
                "text/html;charset=UTF-32BE", "text/html;charset=windows-1252",
                "text/html;charset=Shift_JIS", "text/html;charset=x-no-such-charset",
                "text/html;charset=replacement",
                "text/html;charset=US-ASCII", "text/html;charset=ascii",
                "text/html;charset=iso646-us", "text/html;charset=646",
                "text/html;charset=iso-8859-1", "text/html;charset=iso_8859-1",
                "text/html;charset=l1", "text/html;charset=ibm819",
                "text/html;charset=cp819", "text/html;charset=utf8",
                "text/html;charset=unicode-1-1-utf-8", "text/html;charset=utf-16",
                "text/html;charset=utf16", "text/html;charset=unicode",
                "text/html;charset=utf-16be", "text/html;charset=utf_16be",
                "text/html;charset=unicodebigunmarked", "text/html;charset=utf_16le",
                "text/html;charset=unicodelittleunmarked", "text/html;charset=utf-32",
                "text/html;charset=utf32", "text/html;charset=utf_32be",
                "text/html;charset=utf-32le", "text/html;charset=utf_32le",
                "text/html;charset=csiso2022kr", "text/html;charset=hz-gb-2312",
                "text/html;charset=iso-2022-cn", "text/html;charset=iso-2022-cn-ext",
                "text/html;charset=iso-2022-kr",
                "text/html;charset=\"utf-8\"", "text/html;charset",
                "text/html;charset=;q=1", ";;;"
        };
        for (int index = 0; index < values.length; index++) {
            final String value = values[index];
            emitCall("charset." + index,
                    () -> {
                        final Charset charset = ContentTypeUtils.computeCharsetFromContentType(value);
                        return charset == null ? null : charset.name();
                    });
        }
    }

    private static void emitCombinations() {
        final String[] values = new String[] {
                null, "", " \t", "text/html", "TEXT/HTML;CHARSET=us-ascii;q=1",
                " Text/HTML ; Foo = A ; flag ; foo=B ", ";;;"
        };
        for (int index = 0; index < values.length; index++) {
            final String key = "combine." + index;
            final String value = values[index];
            emitCall(key + ".null", () -> string(
                    ContentTypeUtils.combineContentTypeAndCharset(value, null)));
            emitCall(key + ".utf16", () -> string(
                    ContentTypeUtils.combineContentTypeAndCharset(
                            value, Charset.forName("Unicode"))));
        }
    }

    private static String string(final Object value) {
        return value == null ? null : value.toString();
    }

    private static void emitCall(final String key, final Operation operation) {
        try {
            emit(key, "ok:" + encode(operation.run()));
        } catch (final Throwable throwable) {
            emit(key, "error:" + throwable.getClass().getSimpleName() + ":"
                    + encode(throwable.getMessage()));
        }
    }

    private static String encode(final String value) {
        if (value == null) {
            return "<null>";
        }
        final StringBuilder encoded = new StringBuilder();
        for (int index = 0; index < value.length(); index++) {
            if (index > 0) {
                encoded.append(',');
            }
            encoded.append(String.format("%04X", Integer.valueOf(value.charAt(index))));
        }
        return encoded.toString();
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private interface Operation {
        String run();
    }
}
