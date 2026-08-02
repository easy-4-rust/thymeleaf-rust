package org.thymeleaf.engine;

/**
 * 从固定 Thymeleaf Java 源码导出 IEngineProcessable 动态调用 Golden。
 */
public final class EngineProcessableGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private EngineProcessableGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        final AlternatingProcessable concrete = new AlternatingProcessable();
        final IEngineProcessable dynamic = concrete;
        emit("process.1", dynamic.process());
        emit("process.2", dynamic.process());
        emit("process.3", dynamic.process());
        emit("process.4", dynamic.process());
        emit("process.calls", concrete.calls);
        emit("process.sameDynamicObject", dynamic == concrete);
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private static final class AlternatingProcessable implements IEngineProcessable {

        private int calls;

        @Override
        public boolean process() {
            this.calls++;
            return this.calls % 2 == 0;
        }
    }
}
