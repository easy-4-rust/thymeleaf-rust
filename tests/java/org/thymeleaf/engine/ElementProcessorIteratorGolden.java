package org.thymeleaf.engine;

import java.util.Collections;

import org.thymeleaf.IEngineConfiguration;
import org.thymeleaf.context.ITemplateContext;
import org.thymeleaf.context.TestTemplateEngineConfigurationBuilder;
import org.thymeleaf.dialect.IDialect;
import org.thymeleaf.dialect.IProcessorDialect;
import org.thymeleaf.model.IOpenElementTag;
import org.thymeleaf.templatemode.TemplateMode;
import org.thymeleaf.templateparser.markup.HTMLTemplateParser;
import org.thymeleaf.templateresource.StringTemplateResource;

/** 从上游 ElementProcessorIteratorTest 导出真实 parser/processor 聚合状态机。 */
public final class ElementProcessorIteratorGolden {
    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";
    private static final HTMLTemplateParser HTML_PARSER = new HTMLTemplateParser(2, 4096);

    private ElementProcessorIteratorGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emit("case01", case01());
        emit("case02", case02());
        emit("case03", case03());
        emit("case04", case04());
        emit("case06", case06());
        emit("case07", case07());
        emit("case08", case08());
        emit("case09", case09());
    }

    private static String case01() {
        final TagHandler h = tag("N-ELEMENT-10-null-src,N-ELEMENT-5-null-src");
        return next(h, h.tag) + "," + next(h, h.tag) + "," + next(h, h.tag);
    }

    private static String case02() {
        final TagHandler h = tag("N-ELEMENT-10-null-src,N-ELEMENT-5-null-src,N-ELEMENT-15-null-one");
        final String first = next(h, h.tag);
        final OpenElementTag changed = h.tag.setAttribute(h.definitions, null, "th:one", "somevalue", null);
        return first + "," + next(h, changed) + "," + next(h, changed) + "," + next(h, changed);
    }

    private static String case03() {
        final TagHandler h = tag("N-ELEMENT-10-null-src,N-ELEMENT-5-null-src,N-ELEMENT-7-null-one");
        final String first = next(h, h.tag);
        final OpenElementTag changed = h.tag.setAttribute(h.definitions, null, "th:one", "somevalue", null);
        return first + "," + next(h, changed) + "," + next(h, changed) + "," + next(h, changed);
    }

    private static String case04() {
        final TagHandler h = tag("N-ELEMENT-10-null-src,N-ELEMENT-5-null-src,N-ELEMENT-2-null-one");
        final String first = next(h, h.tag);
        final OpenElementTag changed = h.tag.setAttribute(h.definitions, null, "th:one", "somevalue", null);
        return first + "," + next(h, changed) + "," + next(h, changed) + "," + next(h, changed);
    }

    private static String case06() {
        final TagHandler h = tag("N-ELEMENT-10-null-src,N-ELEMENT-5-null-src,N-ELEMENT-2-null-one");
        return next(h, h.tag.removeAttribute("th:src"));
    }

    private static String case07() {
        final TagHandler h = tag("N-ELEMENT-10-null-src,N-ELEMENT-5-null-src,N-ELEMENT-2-null-one");
        OpenElementTag changed = h.tag.removeAttribute("th:src");
        changed = changed.setAttribute(h.definitions, null, "th:one", "somevalue", null);
        return next(h, changed) + "," + next(h, changed);
    }

    private static String case08() {
        final TagHandler h = tag("N-ELEMENT-10-null-src,N-ELEMENT-5-null-src,N-ELEMENT-2-null-one");
        final String first = next(h, h.tag);
        OpenElementTag changed = h.tag.setAttribute(h.definitions, null, "th:one", "somevalue", null);
        changed = changed.removeAttribute("data-th-src");
        return first + "," + next(h, changed) + "," + next(h, changed);
    }

    private static String case09() {
        final TagHandler h = tag("N-ELEMENT-10-null-src,N-ELEMENT-5-null-src,N-ELEMENT-2-null-one");
        final String first = next(h, h.tag);
        OpenElementTag changed = h.tag.setAttribute(h.definitions, null, "th:one", "somevalue", null);
        final String second = next(h, changed);
        changed = changed.removeAttribute("th:src");
        return first + "," + second + "," + next(h, changed);
    }

    private static TagHandler tag(final String specification) {
        final IProcessorDialect dialect = ProcessorAggregationTestDialect.buildHTMLDialect("standard", "th", specification);
        final TagHandler handler = new TagHandler();
        final IEngineConfiguration configuration = TestTemplateEngineConfigurationBuilder.build(Collections.<IDialect>singleton(dialect));
        handler.definitions = configuration.getAttributeDefinitions();
        HTML_PARSER.parseStandalone(configuration, "test", "test", null, new StringTemplateResource("<a th:src='hello'>"), TemplateMode.HTML, false, handler);
        return handler;
    }

    private static String next(final TagHandler handler, final OpenElementTag tag) {
        final Object processor = handler.iterator.next(tag);
        return processor == null ? "null" : processor.toString();
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private static final class TagHandler extends AbstractTemplateHandler {
        AttributeDefinitions definitions;
        OpenElementTag tag;
        final ElementProcessorIterator iterator = new ElementProcessorIterator();

        @Override
        public void setContext(final ITemplateContext context) {
            super.setContext(context);
            this.definitions = context.getConfiguration().getAttributeDefinitions();
        }

        @Override
        public void handleOpenElement(final IOpenElementTag tag) {
            this.tag = (OpenElementTag) tag;
            this.iterator.reset();
        }
    }
}
