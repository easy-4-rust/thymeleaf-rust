package org.thymeleaf.engine;

import java.util.Arrays;

/** 固定 XMLAttributeName 的名称、命名空间与 equality 合同。 */
public final class XMLAttributeNameGolden {
    private XMLAttributeNameGolden() { }

    public static void main(final String[] args) {
        final XMLAttributeName prefixed = AttributeNames.forXMLName("p:Code");
        final XMLAttributeName same = AttributeNames.forXMLName("p:Code");
        final XMLAttributeName differentCase = AttributeNames.forXMLName("p:code");
        final XMLAttributeName bare = AttributeNames.forXMLName("Code");
        emit("prefixed", prefixed);
        emit("bare", bare);
        System.out.println("equalsSame=" + prefixed.equals(same));
        System.out.println("equalsDifferentCase=" + prefixed.equals(differentCase));
        System.out.println("hashSame=" + (prefixed.hashCode() == same.hashCode()));
    }

    private static void emit(final String key, final XMLAttributeName name) {
        System.out.println(key + "=" + name.getAttributeName() + "," + name.isPrefixed() + ","
                + name.getPrefix() + "," + Arrays.toString(name.getCompleteAttributeNames()) + ","
                + name + "," + name.getCompleteNamespacedAttributeName());
    }
}
