package org.thymeleaf.engine;

/**
 * 从固定 Thymeleaf Java 源码导出 TemplateFlowController Golden。
 */
public final class TemplateFlowControllerGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private TemplateFlowControllerGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        final TemplateFlowController first = new TemplateFlowController();
        final TemplateFlowController second = new TemplateFlowController();
        emitState("default.first", first);
        emitState("default.second", second);

        first.stopProcessing = true;
        emitState("mutate.stop", first);
        first.processorTemplateHandlerPending = true;
        emitState("mutate.pending", first);
        emitState("independent.second", second);
    }

    private static void emitState(
            final String key, final TemplateFlowController controller) {
        emit(key,
                controller.stopProcessing + ","
                        + controller.processorTemplateHandlerPending);
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }
}
