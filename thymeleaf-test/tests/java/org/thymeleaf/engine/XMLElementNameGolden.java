package org.thymeleaf.engine;

import java.util.Arrays;

/** 固定 XMLElementName 的名称、命名空间与 equality 合同。 */
public final class XMLElementNameGolden {
    private XMLElementNameGolden() { }

    public static void main(final String[] args) {
        final XMLElementName prefixed = ElementNames.forXMLName("p:Code");
        final XMLElementName same = ElementNames.forXMLName("p:Code");
        final XMLElementName differentCase = ElementNames.forXMLName("p:code");
        final XMLElementName bare = ElementNames.forXMLName("Code");
        emit("prefixed", prefixed);
        emit("bare", bare);
        System.out.println("equalsSame=" + prefixed.equals(same));
        System.out.println("equalsDifferentCase=" + prefixed.equals(differentCase));
        System.out.println("hashSame=" + (prefixed.hashCode() == same.hashCode()));
    }

    private static void emit(final String key, final XMLElementName name) {
        System.out.println(key + "=" + name.getElementName() + "," + name.isPrefixed() + ","
                + name.getPrefix() + "," + Arrays.toString(name.getCompleteElementNames()) + ","
                + name + "," + name.getCompleteNamespacedElementName());
    }
}
