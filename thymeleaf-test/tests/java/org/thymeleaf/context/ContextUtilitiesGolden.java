package org.thymeleaf.context;

import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Collectors;

import org.thymeleaf.web.IWebExchange;

/**
 * 从固定上游导出 Context 工具对象的结构和可观察运行时语义。
 */
public final class ContextUtilitiesGolden {

    private static final String BASELINE =
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private ContextUtilitiesGolden() {
    }

    public static void main(final String[] args) throws Exception {
        emit("baseline", BASELINE);
        emitShape("ILazyContextVariable", ILazyContextVariable.class);
        emitShape("LazyContextVariable", LazyContextVariable.class);
        emitShape("IdentifierSequences", IdentifierSequences.class);
        emitShape("Contexts", Contexts.class);
        exportLazyVariables();
        exportIdentifierSequences();
        exportContexts();
    }

    private static void exportLazyVariables() {
        final AtomicInteger loads = new AtomicInteger();
        final Object marker = new Marker("shared");
        final LazyContextVariable<Object> variable = new LazyContextVariable<Object>() {
            @Override
            protected Object loadValue() {
                loads.incrementAndGet();
                return marker;
            }
        };
        emit("lazy.before.loads", loads.get());
        final Object first = variable.getValue();
        final Object second = variable.getValue();
        emit("lazy.value", first);
        emit("lazy.identity", first == second && first == marker);
        emit("lazy.after.loads", loads.get());

        final AtomicInteger nullLoads = new AtomicInteger();
        final LazyContextVariable<Object> nullVariable = new LazyContextVariable<Object>() {
            @Override
            protected Object loadValue() {
                nullLoads.incrementAndGet();
                return null;
            }
        };
        emit("lazy.null.first", nullVariable.getValue());
        emit("lazy.null.second", nullVariable.getValue());
        emit("lazy.null.loads", nullLoads.get());

        final AtomicInteger retryLoads = new AtomicInteger();
        final LazyContextVariable<Integer> retryVariable = new LazyContextVariable<Integer>() {
            @Override
            protected Integer loadValue() {
                final int invocation = retryLoads.incrementAndGet();
                if (invocation == 1) {
                    throw new IllegalStateException("first load fails");
                }
                return Integer.valueOf(7);
            }
        };
        emitError("lazy.retry.first", retryVariable::getValue);
        emit("lazy.retry.second", retryVariable.getValue());
        emit("lazy.retry.third", retryVariable.getValue());
        emit("lazy.retry.loads", retryLoads.get());
    }

    private static void exportIdentifierSequences() throws Exception {
        final IdentifierSequences sequences = new IdentifierSequences();
        emit("ids.next.empty", sequences.getNextIDSeq("item"));
        emitError("ids.previous.empty", () -> sequences.getPreviousIDSeq("item"));
        emit("ids.increment.one", sequences.getAndIncrementIDSeq("item"));
        emit("ids.increment.two", sequences.getAndIncrementIDSeq("item"));
        emit("ids.next.after", sequences.getNextIDSeq("item"));
        emit("ids.previous.after", sequences.getPreviousIDSeq("item"));
        emit("ids.other.first", sequences.getAndIncrementIDSeq("其他"));
        emitError("ids.null.increment", () -> sequences.getAndIncrementIDSeq(null));
        emitError("ids.null.next", () -> sequences.getNextIDSeq(null));
        emitError("ids.null.previous", () -> sequences.getPreviousIDSeq(null));

        final Field field = IdentifierSequences.class.getDeclaredField("idCounts");
        field.setAccessible(true);
        @SuppressWarnings("unchecked")
        final Map<String,Integer> counts = (Map<String,Integer>) field.get(sequences);
        counts.put("max", Integer.valueOf(Integer.MAX_VALUE));
        emit("ids.max.increment", sequences.getAndIncrementIDSeq("max"));
        emit("ids.max.next", sequences.getNextIDSeq("max"));
        emit("ids.max.previous", sequences.getPreviousIDSeq("max"));
    }

    private static void exportContexts() {
        final Context plain = new Context();
        final IWebExchange exchange = exchange();
        final WebContext web = new WebContext(exchange);

        emit("contexts.null.engine", Contexts.isEngineContext(null));
        emit("contexts.null.web", Contexts.isWebContext(null));
        emit("contexts.plain.engine", Contexts.isEngineContext(plain));
        emit("contexts.plain.web", Contexts.isWebContext(plain));
        emit("contexts.web.engine", Contexts.isEngineContext(web));
        emit("contexts.web.web", Contexts.isWebContext(web));
        emit("contexts.web.as.identity", Contexts.asWebContext(web) == web);
        emit("contexts.web.exchange.identity", Contexts.getWebExchange(web) == exchange);
        emit("contexts.web.servlet", Contexts.isServletWebContext(web));
        emitErrorClass("contexts.plain.as.engine", () -> Contexts.asEngineContext(plain));
        emitErrorClass("contexts.plain.as.web", () -> Contexts.asWebContext(plain));
        emitErrorClass("contexts.plain.exchange", () -> Contexts.getWebExchange(plain));
        emitErrorClass("contexts.web.servlet.exchange", () -> Contexts.getServletWebExchange(web));
    }

    private static IWebExchange exchange() {
        return (IWebExchange) Proxy.newProxyInstance(
                ContextUtilitiesGolden.class.getClassLoader(),
                new Class<?>[] { IWebExchange.class },
                (proxy, method, arguments) -> {
                    if ("equals".equals(method.getName())) {
                        return proxy == arguments[0];
                    }
                    if ("hashCode".equals(method.getName())) {
                        return System.identityHashCode(proxy);
                    }
                    if ("toString".equals(method.getName())) {
                        return "ProbeWebExchange";
                    }
                    final Class<?> returnType = method.getReturnType();
                    if (returnType == boolean.class) {
                        return false;
                    }
                    if (returnType == int.class) {
                        return 0;
                    }
                    return null;
                });
    }

    private static void emitShape(final String name, final Class<?> type) {
        final List<String> signatures = new ArrayList<>();
        for (final Constructor<?> constructor : type.getDeclaredConstructors()) {
            if (!constructor.isSynthetic()) {
                signatures.add(
                        Modifier.toString(constructor.getModifiers()) + " <init>("
                                + typeNames(constructor.getParameterTypes()) + ")");
            }
        }
        for (final Method method : type.getDeclaredMethods()) {
            if (!method.isSynthetic()) {
                signatures.add(
                        Modifier.toString(method.getModifiers()) + " "
                                + method.getReturnType().getTypeName() + " "
                                + method.getName() + "("
                                + typeNames(method.getParameterTypes()) + ")");
            }
        }
        signatures.sort(Comparator.naturalOrder());
        emit("shape." + name + ".count", signatures.size());
        emit("shape." + name + ".signatures", String.join("|", signatures));
    }

    private static String typeNames(final Class<?>[] types) {
        return Arrays.stream(types).map(Class::getTypeName).collect(Collectors.joining(","));
    }

    private static void emitError(final String key, final ThrowingAction action) {
        try {
            action.run();
            emit(key, "NONE");
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getName() + ":" + exception.getMessage());
        }
    }

    private static void emitErrorClass(final String key, final ThrowingAction action) {
        try {
            action.run();
            emit(key, "NONE");
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getName());
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private interface ThrowingAction {
        void run();
    }

    private static final class Marker {
        private final String value;

        private Marker(final String value) {
            this.value = value;
        }

        @Override
        public String toString() {
            return "Marker(" + this.value + ")";
        }
    }
}
