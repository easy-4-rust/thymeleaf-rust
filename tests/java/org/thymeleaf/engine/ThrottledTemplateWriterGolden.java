package org.thymeleaf.engine;

import java.io.StringWriter;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.io.Writer;
import java.nio.charset.StandardCharsets;

/** 固定字符模式节流 Writer 的额度、溢出与停止状态。 */
public final class ThrottledTemplateWriterGolden {
    private ThrottledTemplateWriterGolden() { }

    public static void main(final String[] args) throws Exception {
        final TemplateFlowController controller = new TemplateFlowController();
        final ThrottledTemplateWriter writer = new ThrottledTemplateWriter("template", controller);
        final StringWriter output = new StringWriter();
        writer.setOutput(output);
        System.out.println("initial=" + state(writer, controller, output));

        writer.allow(2);
        writer.write("abcd");
        System.out.println("first=" + state(writer, controller, output));

        writer.allow(1);
        System.out.println("second=" + state(writer, controller, output));

        writer.allow(Integer.MAX_VALUE);
        System.out.println("unlimited=" + state(writer, controller, output));

        writer.allow(0);
        writer.write("ef");
        System.out.println("zero=" + state(writer, controller, output));

        final TemplateFlowController byteController = new TemplateFlowController();
        final ThrottledTemplateWriter byteWriter = new ThrottledTemplateWriter("template", byteController);
        final ByteArrayOutputStream byteOutput = new ByteArrayOutputStream();
        byteWriter.setOutput(byteOutput, StandardCharsets.UTF_8, 2);
        byteWriter.allow(2);
        byteWriter.write("éx");
        System.out.println("bytesFirst=" + byteState(byteWriter, byteController, byteOutput));
        byteWriter.allow(1);
        System.out.println("bytesSecond=" + byteState(byteWriter, byteController, byteOutput));

        final TemplateFlowController sseController = new TemplateFlowController();
        final SSEThrottledTemplateWriter sseWriter = new SSEThrottledTemplateWriter("template", sseController);
        final StringWriter sseOutput = new StringWriter();
        sseWriter.setOutput(sseOutput);
        sseWriter.allow(Integer.MAX_VALUE);
        sseWriter.startEvent("id".toCharArray(), "event".toCharArray());
        sseWriter.write("a\nb");
        sseWriter.endEvent();
        System.out.println("sse=" + sseOutput.toString().replace("\n", "\\n"));

        final SSEThrottledTemplateWriter sseByteWriter = new SSEThrottledTemplateWriter(
                "template", new TemplateFlowController());
        final ByteArrayOutputStream sseByteOutput = new ByteArrayOutputStream();
        sseByteWriter.setOutput(sseByteOutput, StandardCharsets.UTF_8, Integer.MAX_VALUE);
        sseByteWriter.allow(Integer.MAX_VALUE);
        sseByteWriter.startEvent("id".toCharArray(), "event".toCharArray());
        sseByteWriter.write("x");
        sseByteWriter.endEvent();
        sseByteWriter.flush();
        System.out.println("sseBytes=" + hex(sseByteOutput));

        final SSEThrottledTemplateWriter invalidSseWriter = new SSEThrottledTemplateWriter(
                "template", new TemplateFlowController());
        invalidSseWriter.setOutput(new StringWriter());
        invalidSseWriter.allow(Integer.MAX_VALUE);
        invalidSseWriter.startEvent(null, "bad\nname".toCharArray());
        try {
            invalidSseWriter.write("x");
            System.out.println("sseInvalid=NONE");
        } catch (final RuntimeException exception) {
            System.out.println("sseInvalid=" + exception.getClass().getSimpleName() + ":" + exception.getMessage());
        }

        final SSEThrottledTemplateWriter invalidSseIdWriter = new SSEThrottledTemplateWriter(
                "template", new TemplateFlowController());
        invalidSseIdWriter.setOutput(new StringWriter());
        invalidSseIdWriter.allow(Integer.MAX_VALUE);
        invalidSseIdWriter.startEvent("bad\nid".toCharArray(), null);
        try {
            invalidSseIdWriter.write("x");
            System.out.println("sseInvalidId=NONE");
        } catch (final RuntimeException exception) {
            System.out.println("sseInvalidId=" + exception.getClass().getSimpleName() + ":" + exception.getMessage());
        }

        final SSEThrottledTemplateWriter emptySseWriter = new SSEThrottledTemplateWriter(
                "template", new TemplateFlowController());
        final StringWriter emptySseOutput = new StringWriter();
        emptySseWriter.setOutput(emptySseOutput);
        emptySseWriter.allow(Integer.MAX_VALUE);
        emptySseWriter.startEvent("id".toCharArray(), "event".toCharArray());
        emptySseWriter.write("");
        emptySseWriter.endEvent();
        System.out.println("sseEmpty=" + emptySseOutput.toString().replace("\n", "\\n") + ","
                + emptySseWriter.isOverflown() + "," + emptySseWriter.isStopped());

        final ThrottledTemplateWriter charFirst = new ThrottledTemplateWriter(
                "template", new TemplateFlowController());
        charFirst.setOutput(new StringWriter());
        try {
            charFirst.setOutput(new ByteArrayOutputStream(), StandardCharsets.UTF_8, 1);
            System.out.println("charThenBytes=NONE");
        } catch (final RuntimeException exception) {
            System.out.println("charThenBytes=" + exception.getClass().getSimpleName() + ":" + exception.getMessage());
        }
        final ThrottledTemplateWriter bytesFirst = new ThrottledTemplateWriter(
                "template", new TemplateFlowController());
        bytesFirst.setOutput(new ByteArrayOutputStream(), StandardCharsets.UTF_8, 1);
        try {
            bytesFirst.setOutput(new StringWriter());
            System.out.println("bytesThenChar=NONE");
        } catch (final RuntimeException exception) {
            System.out.println("bytesThenChar=" + exception.getClass().getSimpleName() + ":" + exception.getMessage());
        }

        final ThrottledTemplateWriter failing = new ThrottledTemplateWriter(
                "template", new TemplateFlowController());
        failing.setOutput(new FailOnSecondWriteWriter());
        failing.allow(2);
        failing.write("abc");
        try {
            failing.allow(1);
            System.out.println("overflowIo=NONE");
        } catch (final RuntimeException exception) {
            System.out.println("overflowIo=" + exception.getClass().getSimpleName() + ":" + exception.getMessage());
        }

        final ThrottledTemplateWriter bulk = new ThrottledTemplateWriter(
                "template", new TemplateFlowController());
        final StringWriter bulkOutput = new StringWriter();
        bulk.setOutput(bulkOutput);
        bulk.allow(0);
        bulk.write("a".repeat(600));
        bulk.write("b".repeat(200));
        System.out.println("bulkBuffered=" + bulkOutput.getBuffer().length() + "," + bulk.getWrittenCount()
                + "," + bulk.isOverflown() + "," + bulk.isStopped() + ","
                + bulk.getMaxOverflowSize() + "," + bulk.getOverflowGrowCount());
        bulk.allow(Integer.MAX_VALUE);
        System.out.println("bulkDrained=" + bulkOutput.getBuffer().length() + "," + bulk.getWrittenCount()
                + "," + bulk.isOverflown() + "," + bulk.isStopped() + ","
                + bulk.getMaxOverflowSize() + "," + bulk.getOverflowGrowCount());

        final ThrottledTemplateWriter resourceFailures = new ThrottledTemplateWriter(
                "template", new TemplateFlowController());
        resourceFailures.setOutput(new FlushCloseFailingWriter());
        try {
            resourceFailures.flush();
            System.out.println("flushIo=NONE");
        } catch (final IOException exception) {
            System.out.println("flushIo=" + exception.getClass().getSimpleName() + ":" + exception.getMessage());
        }
        try {
            resourceFailures.close();
            System.out.println("closeIo=NONE");
        } catch (final IOException exception) {
            System.out.println("closeIo=" + exception.getClass().getSimpleName() + ":" + exception.getMessage());
        }

        final ThrottledTemplateWriter overloads = new ThrottledTemplateWriter(
                "template", new TemplateFlowController());
        final StringWriter overloadOutput = new StringWriter();
        overloads.setOutput(overloadOutput);
        overloads.allow(Integer.MAX_VALUE);
        overloads.write('x');
        overloads.write("abcdef", 1, 3);
        overloads.write(new char[] {'q', 'r', 's'});
        overloads.write(new char[] {'q', 'r', 's'}, 1, 2);
        System.out.println("overloads=" + overloadOutput + "," + overloads.getWrittenCount());

        final ThrottledTemplateWriter uninitialized = new ThrottledTemplateWriter(
                "template", new TemplateFlowController());
        System.out.println("uninitializedOverflown=" + failure(() -> uninitialized.isOverflown()));
        System.out.println("uninitializedStopped=" + failure(() -> uninitialized.isStopped()));
        System.out.println("uninitializedWritten=" + failure(() -> uninitialized.getWrittenCount()));
        System.out.println("uninitializedMaxOverflow=" + failure(() -> uninitialized.getMaxOverflowSize()));
        System.out.println("uninitializedGrowCount=" + failure(() -> uninitialized.getOverflowGrowCount()));

        final TemplateFlowController adapterController = new TemplateFlowController();
        final ThrottledTemplateWriterWriterAdapter adapter = new ThrottledTemplateWriterWriterAdapter(
                "template", adapterController);
        final StringWriter adapterOutput = new StringWriter();
        adapter.setWriter(adapterOutput);
        adapter.allow(2);
        adapter.write("abcd");
        System.out.println("adapterFirst=" + adapterOutput + "," + adapter.getWrittenCount() + ","
                + adapter.isOverflown() + "," + adapter.isStopped() + ","
                + adapterController.stopProcessing + "," + adapter.getMaxOverflowSize() + ","
                + adapter.getOverflowGrowCount());
        adapter.allow(Integer.MAX_VALUE);
        System.out.println("adapterDrained=" + adapterOutput + "," + adapter.getWrittenCount() + ","
                + adapter.isOverflown() + "," + adapter.isStopped() + ","
                + adapterController.stopProcessing + "," + adapter.getMaxOverflowSize() + ","
                + adapter.getOverflowGrowCount());

        final TemplateFlowController byteAdapterController = new TemplateFlowController();
        final ThrottledTemplateWriterOutputStreamAdapter byteAdapter =
                new ThrottledTemplateWriterOutputStreamAdapter("template", byteAdapterController, 2);
        final ByteArrayOutputStream byteAdapterOutput = new ByteArrayOutputStream();
        byteAdapter.setOutputStream(byteAdapterOutput);
        byteAdapter.allow(2);
        byteAdapter.write(new byte[] {0x61, 0x62, 0x63, 0x64});
        System.out.println("byteAdapterFirst=" + hex(byteAdapterOutput) + ","
                + byteAdapter.getWrittenCount() + "," + byteAdapter.isOverflown() + ","
                + byteAdapter.isStopped() + "," + byteAdapterController.stopProcessing + ","
                + byteAdapter.getMaxOverflowSize() + "," + byteAdapter.getOverflowGrowCount());
        byteAdapter.allow(Integer.MAX_VALUE);
        System.out.println("byteAdapterDrained=" + hex(byteAdapterOutput) + ","
                + byteAdapter.getWrittenCount() + "," + byteAdapter.isOverflown() + ","
                + byteAdapter.isStopped() + "," + byteAdapterController.stopProcessing + ","
                + byteAdapter.getMaxOverflowSize() + "," + byteAdapter.getOverflowGrowCount());

        final TemplateFlowController byteGrowthController = new TemplateFlowController();
        final ThrottledTemplateWriterOutputStreamAdapter byteGrowth =
                new ThrottledTemplateWriterOutputStreamAdapter("template", byteGrowthController, 2);
        final ByteArrayOutputStream byteGrowthOutput = new ByteArrayOutputStream();
        byteGrowth.setOutputStream(byteGrowthOutput);
        byteGrowth.allow(0);
        byteGrowth.write(new byte[] {0, 1, 2, 3, 4, 5});
        byteGrowth.write(6);
        System.out.println("byteAdapterGrowth=" + hex(byteGrowthOutput) + ","
                + byteGrowth.getWrittenCount() + "," + byteGrowth.isOverflown() + ","
                + byteGrowth.isStopped() + "," + byteGrowthController.stopProcessing + ","
                + byteGrowth.getMaxOverflowSize() + "," + byteGrowth.getOverflowGrowCount());

        final ThrottledTemplateWriterWriterAdapter failingWriterAdapter =
                new ThrottledTemplateWriterWriterAdapter("template", new TemplateFlowController());
        failingWriterAdapter.setWriter(new FailOnSecondWriteWriter());
        failingWriterAdapter.allow(1);
        failingWriterAdapter.write("ab");
        try {
            failingWriterAdapter.allow(Integer.MAX_VALUE);
            System.out.println("adapterOverflowIo=NONE");
        } catch (final RuntimeException exception) {
            System.out.println("adapterOverflowIo=" + exception.getClass().getSimpleName() + ":" + exception.getMessage());
        }

        final ThrottledTemplateWriterOutputStreamAdapter failingByteAdapter =
                new ThrottledTemplateWriterOutputStreamAdapter(
                        "template", new TemplateFlowController(), 2);
        failingByteAdapter.setOutputStream(new FailOnSecondWriteOutputStream());
        failingByteAdapter.allow(1);
        failingByteAdapter.write(new byte[] {0x61, 0x62});
        try {
            failingByteAdapter.allow(Integer.MAX_VALUE);
            System.out.println("byteAdapterOverflowIo=NONE");
        } catch (final RuntimeException exception) {
            System.out.println("byteAdapterOverflowIo=" + exception.getClass().getSimpleName() + ":" + exception.getMessage());
        }
    }

    private static String state(
            final ThrottledTemplateWriter writer,
            final TemplateFlowController controller,
            final StringWriter output) throws Exception {
        return output + "," + writer.getWrittenCount() + "," + writer.isOverflown() + ","
                + writer.isStopped() + "," + controller.stopProcessing + ","
                + writer.getMaxOverflowSize() + "," + writer.getOverflowGrowCount();
    }

    private static String byteState(
            final ThrottledTemplateWriter writer,
            final TemplateFlowController controller,
            final ByteArrayOutputStream output) throws Exception {
        final StringBuilder hex = new StringBuilder();
        for (final byte value : output.toByteArray()) {
            hex.append(String.format("%02x", value & 0xff));
        }
        return hex + "," + writer.getWrittenCount() + "," + writer.isOverflown() + ","
                + writer.isStopped() + "," + controller.stopProcessing + ","
                + writer.getMaxOverflowSize() + "," + writer.getOverflowGrowCount();
    }

    private static String hex(final ByteArrayOutputStream output) {
        final StringBuilder hex = new StringBuilder();
        for (final byte value : output.toByteArray()) {
            hex.append(String.format("%02x", value & 0xff));
        }
        return hex.toString();
    }

    private static String failure(final ThrowingRunnable action) {
        try {
            action.run();
            return "NONE";
        } catch (final Exception exception) {
            return exception.getClass().getName() + ":" + exception.getMessage();
        }
    }

    @FunctionalInterface
    private interface ThrowingRunnable {
        void run() throws Exception;
    }

    private static final class FailOnSecondWriteWriter extends Writer {
        private int writes;

        @Override
        public void write(final char[] characters, final int offset, final int length) throws IOException {
            this.writes++;
            if (this.writes > 1) {
                throw new IOException("overflow sink failure");
            }
        }

        @Override
        public void flush() { }

        @Override
        public void close() { }
    }

    private static final class FlushCloseFailingWriter extends Writer {
        @Override
        public void write(final char[] characters, final int offset, final int length) { }

        @Override
        public void flush() throws IOException {
            throw new IOException("flush sink failure");
        }

        @Override
        public void close() throws IOException {
            throw new IOException("close sink failure");
        }
    }

    private static final class FailOnSecondWriteOutputStream extends OutputStream {
        private int writes;

        @Override
        public void write(final int value) throws IOException {
            this.writes++;
            if (this.writes > 1) {
                throw new IOException("overflow byte sink failure");
            }
        }

        @Override
        public void write(final byte[] bytes, final int offset, final int length) throws IOException {
            this.writes++;
            if (this.writes > 1) {
                throw new IOException("overflow byte sink failure");
            }
        }
    }
}
