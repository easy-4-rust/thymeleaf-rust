package org.thymeleaf.engine;

import java.util.Collections;

import org.thymeleaf.IEngineConfiguration;
import org.thymeleaf.context.ITemplateContext;
import org.thymeleaf.context.TestTemplateEngineConfigurationBuilder;
import org.thymeleaf.model.AttributeValueQuotes;
import org.thymeleaf.model.IOpenElementTag;
import org.thymeleaf.templatemode.TemplateMode;
import org.thymeleaf.templateparser.markup.HTMLTemplateParser;
import org.thymeleaf.templateresource.StringTemplateResource;

/** 导出 ElementTagStructureHandler 的可观察动作状态。 */
public final class ElementTagStructureHandlerGolden {
    private ElementTagStructureHandlerGolden() { }

    public static void main(final String[] args) {
        final ElementTagStructureHandler handler = new ElementTagStructureHandler();
        emit("initial", handler);

        handler.setLocalVariable("a", null);
        handler.setLocalVariable("b", "value");
        handler.removeLocalVariable("old");
        handler.setAttribute("x", "1");
        handler.setAttribute("y", null, AttributeValueQuotes.SINGLE);
        handler.removeAttribute("gone");
        handler.removeAttribute("th", "each");
        emit("combined", handler);

        handler.removeElement();
        emit("removeElement", handler);

        handler.iterateElement("item", null, null);
        emit("iterate", handler);

        handler.reset();
        emit("reset", handler);

        System.out.println("attributes=" + applyAttributes());
    }

    /** 通过真实 parser、标签和 AttributeDefinitions 导出三个动作阶段的最终可观察结果。 */
    private static String applyAttributes() {
        final AttributeCaptureHandler capture = new AttributeCaptureHandler();
        final IEngineConfiguration configuration =
                TestTemplateEngineConfigurationBuilder.build(Collections.emptySet());
        capture.definitions = configuration.getAttributeDefinitions();
        new HTMLTemplateParser(2, 4096).parseStandalone(
                configuration, "test", "test", null,
                new StringTemplateResource("<element data-a='one' data-b='two'>"),
                TemplateMode.HTML, false, capture);

        final ElementTagStructureHandler handler = new ElementTagStructureHandler();
        // 即使 setAttribute 先调用，Java 的 applyAttributes 仍固定为 remove → replace → set。
        handler.setAttribute("data-c", "final");
        handler.removeAttribute("data-a");
        handler.replaceAttribute(
                AttributeNames.forName(TemplateMode.HTML, "data-b"),
                "data-c", "replacement", AttributeValueQuotes.SINGLE);
        handler.setAttribute("data-d", null);
        final OpenElementTag result = handler.applyAttributes(capture.definitions, capture.tag);
        return result.getAttributeMap().entrySet().stream()
                .map(entry -> entry.getKey() + "=" + entry.getValue())
                .reduce((left, right) -> left + "," + right)
                .orElse("");
    }

    private static void emit(final String key, final ElementTagStructureHandler h) {
        System.out.println(key + "="
                + h.setBodyText + "," + h.setBodyModel + "," + h.insertBeforeModel + ","
                + h.removeElement + "," + h.iterateElement + ","
                + h.setLocalVariable + "," + size(h.addedLocalVariables) + ","
                + h.removeLocalVariable + "," + size(h.removedLocalVariableNames) + ","
                + h.setAttribute + "," + h.setAttributeValuesSize + ","
                + h.removeAttribute + "," + h.removeAttributeValuesSize + ","
                + h.iterVariableName);
    }

    private static int size(final java.util.Collection<?> value) {
        return value == null ? 0 : value.size();
    }

    private static int size(final java.util.Map<?, ?> value) {
        return value == null ? 0 : value.size();
    }

    private static final class AttributeCaptureHandler extends AbstractTemplateHandler {
        AttributeDefinitions definitions;
        OpenElementTag tag;

        @Override
        public void setContext(final ITemplateContext context) {
            super.setContext(context);
            this.definitions = context.getConfiguration().getAttributeDefinitions();
        }

        @Override
        public void handleOpenElement(final IOpenElementTag tag) {
            this.tag = (OpenElementTag) tag;
        }
    }
}
