package org.thymeleaf.processor;

import org.thymeleaf.templatemode.TemplateMode;

/**
 * 从固定 Thymeleaf Java 源码导出 IProcessor 接口合同的 Golden。
 */
public final class ProcessorContractGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private ProcessorContractGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        final MutableProcessor implementation = new MutableProcessor(null, Integer.MIN_VALUE);
        final IProcessor processor = implementation;

        emit("initial.mode", processor.getTemplateMode());
        emit("initial.precedence", processor.getPrecedence());

        for (final TemplateMode templateMode : TemplateMode.values()) {
            implementation.templateMode = templateMode;
            emit("mode." + templateMode.name(), processor.getTemplateMode());
        }

        implementation.precedence = 0;
        emit("precedence.zero", processor.getPrecedence());
        implementation.precedence = Integer.MAX_VALUE;
        emit("precedence.max", processor.getPrecedence());
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private static final class MutableProcessor implements IProcessor {

        private TemplateMode templateMode;
        private int precedence;

        private MutableProcessor(final TemplateMode templateMode, final int precedence) {
            this.templateMode = templateMode;
            this.precedence = precedence;
        }

        @Override
        public TemplateMode getTemplateMode() {
            return this.templateMode;
        }

        @Override
        public int getPrecedence() {
            return this.precedence;
        }
    }
}
