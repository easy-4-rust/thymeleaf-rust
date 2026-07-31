package org.thymeleaf.engine;

import java.util.Collections;

/** 固定 XMLAttributeDefinition 的继承属性定义合同。 */
public final class XMLAttributeDefinitionGolden {
    private XMLAttributeDefinitionGolden() { }

    public static void main(final String[] args) {
        final XMLAttributeDefinition first = new XMLAttributeDefinition(
                AttributeNames.forXMLName("p:code"), Collections.emptySet());
        final XMLAttributeDefinition same = new XMLAttributeDefinition(
                AttributeNames.forXMLName("p:code"), Collections.emptySet());
        final XMLAttributeDefinition different = new XMLAttributeDefinition(
                AttributeNames.forXMLName("code"), Collections.emptySet());

        System.out.println("name=" + first.getAttributeName());
        System.out.println("hasProcessors=" + first.hasAssociatedProcessors());
        System.out.println("processorCount=" + first.getAssociatedProcessors().size());
        System.out.println("string=" + first);
        System.out.println("equalsSelf=" + first.equals(first));
        System.out.println("equalsSame=" + first.equals(same));
        System.out.println("equalsDifferent=" + first.equals(different));
        System.out.println("hashSame=" + (first.hashCode() == same.hashCode()));
        System.out.println("hashDifferent=" + (first.hashCode() == different.hashCode()));
    }
}
