import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.StringWriter;
import java.io.Writer;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.Collections;
import java.util.stream.Collectors;

import org.thymeleaf.IThrottledTemplateProcessor;
import org.thymeleaf.TemplateEngine;
import org.thymeleaf.context.Context;
import org.thymeleaf.linkbuilder.StandardLinkBuilder;
import org.thymeleaf.messageresolver.StandardMessageResolver;
import org.thymeleaf.templateresolver.StringTemplateResolver;

/**
 * TemplateEngine、ITemplateEngine 与 IThrottledTemplateProcessor 的 Java Golden。
 *
 * <p>输出只包含稳定、可由 Rust 公共 API 复现的可观察行为。</p>
 */
public final class TemplateEngineExecutionGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private TemplateEngineExecutionGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        emitInitializationAndOrdering();
        emitProcessingOverloads();
        emitThrottledCharacters();
        emitThrottledBytes();
        emitModeSwitchFailure();
    }

    private static void emitInitializationAndOrdering() {
        final TemplateEngine engine = new TemplateEngine();

        final StringTemplateResolver resolver20 = resolver(20);
        final StringTemplateResolver resolverNull = resolver(null);
        final StringTemplateResolver resolverMinus1 = resolver(-1);
        engine.setTemplateResolvers(
                new java.util.LinkedHashSet<>(Arrays.asList(resolver20, resolverNull, resolverMinus1)));

        final StandardMessageResolver message20 = messageResolver(20);
        final StandardMessageResolver messageNull = messageResolver(null);
        final StandardMessageResolver messageMinus1 = messageResolver(-1);
        engine.setMessageResolvers(
                new java.util.LinkedHashSet<>(Arrays.asList(message20, messageNull, messageMinus1)));

        final StandardLinkBuilder link20 = linkBuilder(20);
        final StandardLinkBuilder linkNull = linkBuilder(null);
        final StandardLinkBuilder linkMinus1 = linkBuilder(-1);
        engine.setLinkBuilders(
                new java.util.LinkedHashSet<>(Arrays.asList(link20, linkNull, linkMinus1)));

        emit("initialization.before", Boolean.toString(engine.isInitialized()));
        emit("ordering.template.before", orders(engine.getTemplateResolvers().stream()
                .map(value -> value.getOrder()).collect(Collectors.toList())));
        emit("ordering.message.before", orders(engine.getMessageResolvers().stream()
                .map(value -> value.getOrder()).collect(Collectors.toList())));
        emit("ordering.link.before", orders(engine.getLinkBuilders().stream()
                .map(value -> value.getOrder()).collect(Collectors.toList())));

        engine.getConfiguration();
        emit("initialization.after", Boolean.toString(engine.isInitialized()));
        emit("ordering.template.after", orders(engine.getTemplateResolvers().stream()
                .map(value -> value.getOrder()).collect(Collectors.toList())));
        emit("ordering.message.after", orders(engine.getMessageResolvers().stream()
                .map(value -> value.getOrder()).collect(Collectors.toList())));
        emit("ordering.link.after", orders(engine.getLinkBuilders().stream()
                .map(value -> value.getOrder()).collect(Collectors.toList())));

        try {
            engine.addTemplateResolver(new StringTemplateResolver());
            emit("initialization.freeze", "NO_ERROR");
        } catch (final RuntimeException exception) {
            emit("initialization.freeze",
                    exception.getClass().getName() + ":" + String.valueOf(exception.getMessage()));
        }
    }

    private static void emitProcessingOverloads() {
        final TemplateEngine engine = stringEngine();
        final Context context = new Context();
        context.setVariable("name", "Rust");
        final String template = "<p th:text=\"${name}\">fallback</p>";

        emit("process.string", engine.process(template, context));
        emit("process.spec", engine.process(
                new org.thymeleaf.TemplateSpec(
                        template, (org.thymeleaf.templatemode.TemplateMode) null),
                context));

        final TrackingWriter writer = new TrackingWriter();
        engine.process(template, context, writer);
        emit("process.writer.output", writer.toString());
        emit("process.writer.flush_count", Integer.toString(writer.flushCount));

        final String selected = engine.process(
                "<main><p id=\"a\">A</p><p id=\"b\">B</p></main>",
                Collections.singleton("#b"),
                context);
        emit("process.selectors", selected);

        final TrackingWriter selectedWriter = new TrackingWriter();
        engine.process(
                "<main><p id=\"a\">A</p><p id=\"b\">B</p></main>",
                Collections.singleton("#a"),
                context,
                selectedWriter);
        emit("process.selectors_writer.output", selectedWriter.toString());
        emit("process.selectors_writer.flush_count", Integer.toString(selectedWriter.flushCount));

        final IThrottledTemplateProcessor selectedProcessor = engine.processThrottled(
                "<main><p id=\"a\">A</p><p id=\"b\">B</p></main>",
                Collections.singleton("#b"),
                context);
        final TrackingWriter throttledSelectedWriter = new TrackingWriter();
        emit("process.selectors_throttled.count",
                Integer.toString(selectedProcessor.processAll(throttledSelectedWriter)));
        emit("process.selectors_throttled.output", throttledSelectedWriter.toString());

        final TemplateEngine emptyEngine = stringEngine();
        emit("process.empty", emptyEngine.process("", new Context()));

        final TemplateEngine failingEngine = stringEngine();
        try {
            failingEngine.process("output", new Context(), new FlushFailingWriter());
            emit("process.flush_failure", "NO_ERROR");
        } catch (final RuntimeException exception) {
            emit("process.flush_failure",
                    exception.getClass().getName() + ":" + String.valueOf(exception.getMessage()));
        }
    }

    private static void emitThrottledCharacters() {
        final TemplateEngine engine = stringEngine();
        final Context context = new Context();
        final IThrottledTemplateProcessor processor =
                engine.processThrottled("<p>abcdef</p>", context);
        final TrackingWriter writer = new TrackingWriter();

        emit("throttle.chars.identifier_nonempty",
                Boolean.toString(!processor.getProcessorIdentifier().isEmpty()));
        emit("throttle.chars.spec", processor.getTemplateSpec().toString());
        emit("throttle.chars.initial_finished", Boolean.toString(processor.isFinished()));
        emit("throttle.chars.zero", Integer.toString(processor.process(0, writer)));

        final StringBuilder counts = new StringBuilder();
        int guard = 0;
        while (!processor.isFinished() && guard++ < 100) {
            if (counts.length() > 0) {
                counts.append(',');
            }
            counts.append(processor.process(3, writer));
        }
        emit("throttle.chars.counts", counts.toString());
        emit("throttle.chars.output", writer.toString());
        emit("throttle.chars.final_finished", Boolean.toString(processor.isFinished()));
        emit("throttle.chars.after_finished", Integer.toString(processor.process(3, writer)));

        final IThrottledTemplateProcessor all =
                stringEngine().processThrottled("all-at-once", new Context());
        final TrackingWriter allWriter = new TrackingWriter();
        emit("throttle.chars.all_count", Integer.toString(all.processAll(allWriter)));
        emit("throttle.chars.all_output", allWriter.toString());
    }

    private static void emitThrottledBytes() {
        final IThrottledTemplateProcessor processor =
                stringEngine().processThrottled("Aé中B", new Context());
        final ByteArrayOutputStream output = new ByteArrayOutputStream();
        emit("throttle.bytes.initial_finished", Boolean.toString(processor.isFinished()));

        final StringBuilder counts = new StringBuilder();
        int guard = 0;
        while (!processor.isFinished() && guard++ < 100) {
            if (counts.length() > 0) {
                counts.append(',');
            }
            counts.append(processor.process(3, output, StandardCharsets.UTF_8));
        }
        emit("throttle.bytes.counts", counts.toString());
        emit("throttle.bytes.output", new String(output.toByteArray(), StandardCharsets.UTF_8));
        emit("throttle.bytes.final_finished", Boolean.toString(processor.isFinished()));

        final IThrottledTemplateProcessor all =
                stringEngine().processThrottled("Aé中B", new Context());
        final ByteArrayOutputStream allOutput = new ByteArrayOutputStream();
        emit("throttle.bytes.all_count",
                Integer.toString(all.processAll(allOutput, StandardCharsets.UTF_8)));
        emit("throttle.bytes.all_output",
                new String(allOutput.toByteArray(), StandardCharsets.UTF_8));
    }

    private static void emitModeSwitchFailure() {
        final IThrottledTemplateProcessor processor =
                stringEngine().processThrottled("abcdef", new Context());
        processor.process(1, new TrackingWriter());
        try {
            processor.process(1, new ByteArrayOutputStream(), StandardCharsets.UTF_8);
            emit("throttle.mode_switch", "NO_ERROR");
        } catch (final RuntimeException exception) {
            emit("throttle.mode_switch",
                    exception.getClass().getName() + ":" + String.valueOf(exception.getMessage()));
        }
    }

    private static TemplateEngine stringEngine() {
        final TemplateEngine engine = new TemplateEngine();
        engine.setTemplateResolver(new StringTemplateResolver());
        return engine;
    }

    private static StringTemplateResolver resolver(final Integer order) {
        final StringTemplateResolver resolver = new StringTemplateResolver();
        resolver.setOrder(order);
        return resolver;
    }

    private static StandardMessageResolver messageResolver(final Integer order) {
        final StandardMessageResolver resolver = new StandardMessageResolver();
        resolver.setOrder(order);
        return resolver;
    }

    private static StandardLinkBuilder linkBuilder(final Integer order) {
        final StandardLinkBuilder builder = new StandardLinkBuilder();
        builder.setOrder(order);
        return builder;
    }

    private static String orders(final java.util.List<Integer> orders) {
        return orders.stream().map(String::valueOf).collect(Collectors.joining(","));
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "\t" + escape(value));
    }

    private static String escape(final String value) {
        return value
                .replace("\\", "\\\\")
                .replace("\t", "\\t")
                .replace("\r", "\\r")
                .replace("\n", "\\n");
    }

    private static final class TrackingWriter extends StringWriter {
        private int flushCount;

        @Override
        public void flush() {
            this.flushCount++;
        }
    }

    private static final class FlushFailingWriter extends Writer {
        @Override
        public void write(final char[] characters, final int offset, final int length) {
            // 接收输出，专门在 flush 阶段失败。
        }

        @Override
        public void flush() throws IOException {
            throw new IOException("golden flush failure");
        }

        @Override
        public void close() {
            // 无需关闭外部资源。
        }
    }
}
