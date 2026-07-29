package org.thymeleaf.dialect;

import java.util.LinkedHashSet;
import java.util.Set;

import org.thymeleaf.processor.IProcessor;
import org.thymeleaf.templatemode.TemplateMode;

/**
 * 从固定 Thymeleaf Java 源码导出 IProcessorDialect 接口合同的 Golden。
 */
public final class ProcessorDialectContractGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private ProcessorDialectContractGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        final ProbeDialect nullPrefix =
                new ProbeDialect("probe-null", null, Integer.MIN_VALUE);
        final ProbeDialect emptyPrefix =
                new ProbeDialect("probe-empty", "", 0);
        final ProbeDialect unicodePrefix =
                new ProbeDialect("方言", "前缀", Integer.MAX_VALUE);

        emitGetters("null", nullPrefix);
        emitGetters("empty", emptyPrefix);
        emitGetters("unicode", unicodePrefix);

        emitProcessors("null", nullPrefix, null);
        emitProcessors("empty", nullPrefix, "");
        emitProcessors("unicode", nullPrefix, "用户前缀");
        emitProcessors("nullSet", nullPrefix, "return-null");
    }

    private static void emitGetters(final String key, final IProcessorDialect dialect) {
        emit(
                "getters." + key,
                "name=" + dialect.getName()
                        + ",prefix=" + dialect.getPrefix()
                        + ",precedence=" + dialect.getDialectProcessorPrecedence());
    }

    private static void emitProcessors(
            final String key,
            final ProbeDialect implementation,
            final String dialectPrefix) {
        final IProcessorDialect dialect = implementation;
        final Set<IProcessor> processors = dialect.getProcessors(dialectPrefix);
        if (processors == null) {
            emit(
                    "processors." + key,
                    "set=null,lastPrefix=" + implementation.lastPrefix
                            + ",calls=" + implementation.calls);
            return;
        }

        final StringBuilder values = new StringBuilder();
        for (final IProcessor processor : processors) {
            if (values.length() > 0) {
                values.append('|');
            }
            if (processor == null) {
                values.append("null");
            } else {
                values.append(processor.getTemplateMode())
                        .append(':')
                        .append(processor.getPrecedence());
            }
        }

        emit(
                "processors." + key,
                "size=" + processors.size()
                        + ",values=" + values
                        + ",duplicateAdded=" + implementation.duplicateAdded
                        + ",lastPrefix=" + implementation.lastPrefix
                        + ",calls=" + implementation.calls);
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + String.valueOf(value));
    }

    private static final class ProbeDialect implements IProcessorDialect {

        private final String name;
        private final String prefix;
        private final int precedence;
        private int calls;
        private String lastPrefix;
        private boolean duplicateAdded;

        private ProbeDialect(final String name, final String prefix, final int precedence) {
            this.name = name;
            this.prefix = prefix;
            this.precedence = precedence;
        }

        @Override
        public String getName() {
            return this.name;
        }

        @Override
        public String getPrefix() {
            return this.prefix;
        }

        @Override
        public int getDialectProcessorPrecedence() {
            return this.precedence;
        }

        @Override
        public Set<IProcessor> getProcessors(final String dialectPrefix) {
            this.calls++;
            this.lastPrefix = dialectPrefix;
            if ("return-null".equals(dialectPrefix)) {
                return null;
            }

            final Set<IProcessor> processors = new LinkedHashSet<IProcessor>();
            final IProcessor first =
                    new ProbeProcessor(TemplateMode.HTML, Integer.MIN_VALUE);
            processors.add(null);
            processors.add(first);
            this.duplicateAdded = processors.add(first);
            processors.add(new ProbeProcessor(null, Integer.MAX_VALUE));
            return processors;
        }
    }

    private static final class ProbeProcessor implements IProcessor {

        private final TemplateMode templateMode;
        private final int precedence;

        private ProbeProcessor(final TemplateMode templateMode, final int precedence) {
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
