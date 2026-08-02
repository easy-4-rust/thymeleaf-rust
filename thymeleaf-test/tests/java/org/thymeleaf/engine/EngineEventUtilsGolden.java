package org.thymeleaf.engine;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.List;
import java.util.stream.Collectors;

/** 固定 EngineEventUtils 对三类文本事件的 Java 可观察语义。 */
public final class EngineEventUtilsGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private EngineEventUtilsGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emit("shape", signatures(EngineEventUtils.class));
        emit("text.null.whitespace", EngineEventUtils.isWhitespace((Text) null));
        emit("text.empty.whitespace", EngineEventUtils.isWhitespace(new Text("")));
        emit("text.space.whitespace", EngineEventUtils.isWhitespace(new Text(" \t\n\u3000")));
        emit("text.figure-space.whitespace", EngineEventUtils.isWhitespace(new Text("\u2007")));
        emit("text.word.whitespace", EngineEventUtils.isWhitespace(new Text(" a ")));
        emit("text.bracket.inlineable", EngineEventUtils.isInlineable(new Text("x [[${name}]] y")));
        emit("text.paren.inlineable", EngineEventUtils.isInlineable(new Text("x [(${name})] y")));
        emit("text.malformed.inlineable", EngineEventUtils.isInlineable(new Text("[[$ {name}]")));
        emit("cdata.space.whitespace", EngineEventUtils.isWhitespace(new CDATASection("\t\u200A")));
        emit("cdata.inlineable", EngineEventUtils.isInlineable(new CDATASection("[[${name}]]")));
        emit("comment.space.whitespace", EngineEventUtils.isWhitespace(new Comment("\r\n")));
        emit("comment.inlineable", EngineEventUtils.isInlineable(new Comment("[(${name})]")));
    }

    private static String signatures(final Class<?> type) {
        final List<String> signatures = new ArrayList<>();
        Arrays.stream(type.getDeclaredConstructors()).filter(c -> !c.isSynthetic()).forEach(c -> signatures.add(
                Modifier.toString(c.getModifiers()) + " <init>(" + Arrays.stream(c.getParameterTypes())
                        .map(Class::getTypeName).collect(Collectors.joining(",")) + ")"));
        Arrays.stream(type.getDeclaredMethods()).filter(m -> !m.isSynthetic()).forEach(m -> signatures.add(
                Modifier.toString(m.getModifiers()) + " " + m.getReturnType().getTypeName() + " " + m.getName()
                        + "(" + Arrays.stream(m.getParameterTypes()).map(Class::getTypeName)
                                .collect(Collectors.joining(",")) + ")"));
        signatures.sort(Comparator.naturalOrder());
        return String.join("|", signatures);
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }
}
