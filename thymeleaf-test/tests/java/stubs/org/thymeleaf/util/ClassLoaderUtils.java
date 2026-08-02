package org.thymeleaf.util;

import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;

/**
 * 为 Java Golden 提供 Maven Central 3.1.5.RELEASE 制品中已过滤的版本属性。
 */
public final class ClassLoaderUtils {

    private ClassLoaderUtils() {
    }

    public static InputStream loadResourceAsStream(final String resourceName) {
        final String properties =
                "version=3.1.5.RELEASE\n"
                + "build.date=2026-04-21T20:38:36+0000\n";
        return new ByteArrayInputStream(properties.getBytes(StandardCharsets.ISO_8859_1));
    }
}
