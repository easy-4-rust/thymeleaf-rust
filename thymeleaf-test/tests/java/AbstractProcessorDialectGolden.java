package org.thymeleaf.dialect;

import java.util.Collections;
import java.util.Set;

import org.thymeleaf.processor.IProcessor;

/**
 * 从固定 Thymeleaf Java 源码导出 AbstractProcessorDialect 基础状态合同的 Golden。
 */
public final class AbstractProcessorDialectGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private AbstractProcessorDialectGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emitFailure("nullName", () -> new ProbeDialect(null, "ignored", Integer.MAX_VALUE));

        emitCase("nullPrefix", "", null, Integer.MIN_VALUE, null);
        emitCase("emptyPrefix", "empty-prefix", "", 0, "");
        emitCase("unicode", "方言", "前缀", Integer.MAX_VALUE, "用户覆盖");
    }

    private static void emitCase(
            final String key,
            final String name,
            final String prefix,
            final int processorPrecedence,
            final String actualPrefix) {
        final ProbeDialect implementation =
                new ProbeDialect(name, prefix, processorPrecedence);
        final IDialect dialect = implementation;
        final IProcessorDialect processorDialect = implementation;
        final Set<IProcessor> processors = processorDialect.getProcessors(actualPrefix);

        emit(
                "case." + key,
                "name=" + implementation.getName()
                        + ",prefix=" + implementation.getPrefix()
                        + ",precedence=" + implementation.getDialectProcessorPrecedence()
                        + ",dialectName=" + dialect.getName()
                        + ",interfacePrefix=" + processorDialect.getPrefix()
                        + ",interfacePrecedence="
                        + processorDialect.getDialectProcessorPrecedence()
                        + ",processorsSize=" + processors.size()
                        + ",lastPrefix=" + implementation.lastPrefix
                        + ",calls=" + implementation.calls
                        + ",stable="
                        + (implementation.getName() == implementation.getName()
                                && implementation.getPrefix() == implementation.getPrefix()
                                && implementation.getDialectProcessorPrecedence()
                                        == processorDialect.getDialectProcessorPrecedence()));
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

    private static final class ProbeDialect extends AbstractProcessorDialect {

        private int calls;
        private String lastPrefix;

        private ProbeDialect(
                final String name,
                final String prefix,
                final int processorPrecedence) {
            super(name, prefix, processorPrecedence);
        }

        @Override
        public Set<IProcessor> getProcessors(final String dialectPrefix) {
            this.calls++;
            this.lastPrefix = dialectPrefix;
            return Collections.emptySet();
        }
    }
}
