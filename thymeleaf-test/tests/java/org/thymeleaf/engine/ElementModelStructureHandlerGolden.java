package org.thymeleaf.engine;

import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.List;

import org.thymeleaf.context.IEngineContext;

/** 导出 ElementModelStructureHandler 的可组合上下文动作和固定应用顺序。 */
public final class ElementModelStructureHandlerGolden {
    private ElementModelStructureHandlerGolden() { }

    public static void main(final String[] args) {
        final ElementModelStructureHandler handler = new ElementModelStructureHandler();
        emit("initial", handler);
        handler.setLocalVariable("a", null);
        handler.setLocalVariable("b", "value");
        handler.setLocalVariable("old", "new-old");
        handler.removeLocalVariable("old");
        handler.setSelectionTarget(null);
        handler.setInliner(null);
        handler.setTemplateData(null);
        emit("combined", handler);

        final List<String> calls = new ArrayList<>();
        final IEngineContext context = (IEngineContext) Proxy.newProxyInstance(
                ElementModelStructureHandlerGolden.class.getClassLoader(),
                new Class<?>[] { IEngineContext.class },
                (proxy, method, arguments) -> {
                    if ("setVariables".equals(method.getName())) {
                        calls.add("setVariables:" + ((java.util.Map<?, ?>) arguments[0]).size());
                    } else if ("removeVariable".equals(method.getName())) {
                        calls.add("removeVariable:" + arguments[0]);
                    } else if ("setSelectionTarget".equals(method.getName())) {
                        calls.add("setSelectionTarget:" + arguments[0]);
                    } else if ("setInliner".equals(method.getName())) {
                        calls.add("setInliner:" + arguments[0]);
                    } else if ("setTemplateData".equals(method.getName())) {
                        calls.add("setTemplateData:" + arguments[0]);
                    }
                    return null;
                });
        handler.applyContextModifications(context);
        System.out.println("apply=" + String.join(",", calls));
        handler.reset();
        emit("reset", handler);
    }

    private static void emit(final String key, final ElementModelStructureHandler h) {
        System.out.println(key + "=" + h.setLocalVariable + "," + size(h.addedLocalVariables)
                + "," + h.removeLocalVariable + "," + size(h.removedLocalVariableNames)
                + "," + h.setSelectionTarget + "," + h.setInliner + "," + h.setTemplateData);
    }

    private static int size(final java.util.Collection<?> value) {
        return value == null ? 0 : value.size();
    }

    private static int size(final java.util.Map<?, ?> value) {
        return value == null ? 0 : value.size();
    }
}
