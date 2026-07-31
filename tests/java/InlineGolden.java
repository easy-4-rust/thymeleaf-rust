import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.Proxy;
import java.util.Arrays;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.CountDownLatch;

import org.thymeleaf.context.ITemplateContext;
import org.thymeleaf.inline.IInliner;
import org.thymeleaf.inline.NoOpInliner;
import org.thymeleaf.model.ICDATASection;
import org.thymeleaf.model.IComment;
import org.thymeleaf.model.IText;

/**
 * 从固定 Thymeleaf 3.1.5.RELEASE 导出基础 Inliner SPI 与 NoOp 单例合同。
 */
public final class InlineGolden {

    private static final String JAVA_BASELINE =
            "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private InlineGolden() {
    }

    public static void main(final String[] args) throws Exception {
        emit("java_baseline", JAVA_BASELINE);
        exportShape();
        exportNoOp();
        exportDynamicDispatch();
        exportConcurrentSingleton();
    }

    private static void exportShape() throws ReflectiveOperationException {
        final Constructor<?>[] constructors = NoOpInliner.class.getDeclaredConstructors();
        emit("shape.class.final", Modifier.isFinal(NoOpInliner.class.getModifiers()));
        emit("shape.constructor.count", constructors.length);
        emit("shape.constructor.private", Modifier.isPrivate(constructors[0].getModifiers()));
        emit("shape.instance.public_static_final",
                Modifier.isPublic(NoOpInliner.class.getField("INSTANCE").getModifiers())
                        && Modifier.isStatic(NoOpInliner.class.getField("INSTANCE").getModifiers())
                        && Modifier.isFinal(NoOpInliner.class.getField("INSTANCE").getModifiers()));

        final String[] signatures = Arrays.stream(IInliner.class.getDeclaredMethods())
                .map(InlineGolden::signature)
                .sorted()
                .toArray(String[]::new);
        emit("shape.interface.methods", String.join(",", signatures));
    }

    private static void exportNoOp() {
        final NoOpInliner first = NoOpInliner.INSTANCE;
        final NoOpInliner second = NoOpInliner.INSTANCE;
        emit("noop.instance.same", first == second);
        emit("noop.name", first.getName());

        emit("noop.null.text", first.inline(null, (IText) null));
        emit("noop.null.cdata", first.inline(null, (ICDATASection) null));
        emit("noop.null.comment", first.inline(null, (IComment) null));

        final ITemplateContext context = proxy(ITemplateContext.class);
        emit("noop.non_null.text", first.inline(context, proxy(IText.class)));
        emit("noop.non_null.cdata", first.inline(context, proxy(ICDATASection.class)));
        emit("noop.non_null.comment", first.inline(context, proxy(IComment.class)));
    }

    private static void exportDynamicDispatch() {
        final ProbeInliner probe = new ProbeInliner();
        final IInliner inliner = probe;
        final ITemplateContext context = proxy(ITemplateContext.class);

        emit("probe.name", inliner.getName());
        emit("probe.text", inliner.inline(context, proxy(IText.class)));
        emit("probe.cdata", inliner.inline(context, proxy(ICDATASection.class)));
        emit("probe.comment", inliner.inline(context, proxy(IComment.class)));
        emit("probe.calls", probe.textCalls + "," + probe.cdataCalls + "," + probe.commentCalls);
    }

    private static void exportConcurrentSingleton() throws InterruptedException {
        final int workers = 8;
        final CountDownLatch ready = new CountDownLatch(workers);
        final CountDownLatch start = new CountDownLatch(1);
        final CountDownLatch done = new CountDownLatch(workers);
        final ConcurrentHashMap<Integer,Boolean> identities =
                new ConcurrentHashMap<Integer,Boolean>();
        final ConcurrentHashMap<String,Integer> names =
                new ConcurrentHashMap<String,Integer>();

        for (int index = 0; index < workers; index++) {
            final Thread thread = new Thread(() -> {
                ready.countDown();
                try {
                    start.await();
                    identities.put(System.identityHashCode(NoOpInliner.INSTANCE), Boolean.TRUE);
                    names.merge(NoOpInliner.INSTANCE.getName(), 1, Integer::sum);
                } catch (final InterruptedException exception) {
                    Thread.currentThread().interrupt();
                    throw new IllegalStateException(exception);
                } finally {
                    done.countDown();
                }
            });
            thread.start();
        }

        ready.await();
        start.countDown();
        done.await();
        emit("concurrent.identity_count", identities.size());
        emit("concurrent.name_count", names.get("NOOP"));
    }

    private static String signature(final Method method) {
        final String parameters = Arrays.stream(method.getParameterTypes())
                .map(Class::getSimpleName)
                .reduce((left, right) -> left + "+" + right)
                .orElse("");
        return method.getName() + "(" + parameters + "):" + method.getReturnType().getSimpleName();
    }

    @SuppressWarnings("unchecked")
    private static <T> T proxy(final Class<T> type) {
        return (T) Proxy.newProxyInstance(
                InlineGolden.class.getClassLoader(),
                new Class<?>[] {type},
                (instance, method, args) -> {
                    final Class<?> returnType = method.getReturnType();
                    if (!returnType.isPrimitive()) {
                        return null;
                    }
                    if (returnType == boolean.class) {
                        return Boolean.FALSE;
                    }
                    if (returnType == char.class) {
                        return Character.valueOf('\0');
                    }
                    return Integer.valueOf(0);
                });
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private static final class ProbeInliner implements IInliner {
        private int textCalls;
        private int cdataCalls;
        private int commentCalls;

        @Override
        public String getName() {
            return "PROBE";
        }

        @Override
        public CharSequence inline(final ITemplateContext context, final IText text) {
            this.textCalls++;
            return "TEXT";
        }

        @Override
        public CharSequence inline(
                final ITemplateContext context, final ICDATASection cdataSection) {
            this.cdataCalls++;
            return "CDATA";
        }

        @Override
        public CharSequence inline(final ITemplateContext context, final IComment comment) {
            this.commentCalls++;
            return "COMMENT";
        }
    }
}
