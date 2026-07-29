package org.thymeleaf.processor;

import org.thymeleaf.templatemode.TemplateMode;

/**
 * 从固定 Thymeleaf Java 源码导出 AbstractProcessor 基础状态合同的 Golden。
 */
public final class AbstractProcessorGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private AbstractProcessorGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emitFailure("null", () -> new ProbeProcessor(null, 123));

        emitCase("html", TemplateMode.HTML, Integer.MIN_VALUE);
        emitCase("xml", TemplateMode.XML, -1);
        emitCase("text", TemplateMode.TEXT, 0);
        emitCase("javascript", TemplateMode.JAVASCRIPT, 1);
        emitCase("css", TemplateMode.CSS, 1000);
        emitCase("raw", TemplateMode.RAW, Integer.MAX_VALUE);
    }

    private static void emitCase(
            final String key,
            final TemplateMode templateMode,
            final int precedence) {
        final ProbeProcessor implementation = new ProbeProcessor(templateMode, precedence);
        final IProcessor processor = implementation;

        emit(
                "case." + key,
                "mode=" + implementation.getTemplateMode()
                        + ",precedence=" + implementation.getPrecedence()
                        + ",interfaceMode=" + processor.getTemplateMode()
                        + ",interfacePrecedence=" + processor.getPrecedence()
                        + ",modeIdentity=" + (implementation.getTemplateMode() == templateMode)
                        + ",stable="
                        + (implementation.getTemplateMode() == processor.getTemplateMode()
                                && implementation.getPrecedence() == processor.getPrecedence()));
    }

    private static void emitFailure(final String key, final Operation operation) {
        try {
            operation.run();
            emit(key, "<none>");
        } catch (final RuntimeException exception) {
            emit(
                    key,
                    "ERR:" + exception.getClass().getName() + ":" + exception.getMessage());
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private interface Operation {
        void run();
    }

    private static final class ProbeProcessor extends AbstractProcessor {

        private ProbeProcessor(final TemplateMode templateMode, final int precedence) {
            super(templateMode, precedence);
        }
    }
}
