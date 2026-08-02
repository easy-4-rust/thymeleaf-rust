package org.thymeleaf.templateparser.reader;

import java.io.IOException;
import java.io.Reader;
import java.io.StringReader;

/**
 * 固定标记注释 Reader 的定界符、跨缓冲区、异常和生命周期语义。
 */
public final class MarkupCommentReaderGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private MarkupCommentReaderGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        final String[] parserCases = {
                "",
                "plain",
                "<!-- hello -->",
                "a<!--/*hidden*/-->b",
                "<!-- <!--/* hello /*/--> -->",
                "\u4e2d<!--/*\ud83d\ude00*/-->\u6587"
        };
        final String[] prototypeCases = {
                "",
                "plain",
                "<!-- hello -->",
                "a<!--/*/shown/*/-->b",
                "<!-- <!--/*/ hello /*/--> */--> -->",
                "\u4e2d<!--/*/\ud83d\ude00/*/-->\u6587"
        };
        final int[][] requests = {
                {1, 0, 1},
                {4, 0, 4},
                {9, 2, 5},
                {15, 5, 8}
        };
        emitCases("parser", parserCases, requests, false);
        emitCases("prototype", prototypeCases, requests, true);

        final String[] combinedCases = {
                "a<!--/*/shown/*/-->b<!--/*hidden*/-->c",
                "<!--/*x<!--/*/not-shown/*/-->y*/-->tail"
        };
        for (int caseIndex = 0; caseIndex < combinedCases.length; caseIndex++) {
            for (int requestIndex = 0; requestIndex < requests.length; requestIndex++) {
                final int[] request = requests[requestIndex];
                final Reader reader = new ParserLevelCommentMarkupReader(
                        new PrototypeOnlyCommentMarkupReader(new StringReader(combinedCases[caseIndex])));
                emitRead(
                        "combined." + caseIndex + "." + requestIndex,
                        reader,
                        request[0],
                        request[1],
                        request[2]);
            }
        }

        emitRead(
                "unfinished.parser",
                new ParserLevelCommentMarkupReader(new StringReader("a<!--/*open")),
                3,
                0,
                3);
        emitRead(
                "unfinished.prototype",
                new PrototypeOnlyCommentMarkupReader(new StringReader("a<!--/*/open")),
                5,
                1,
                3);

        final TrackingReader closeDelegate = new TrackingReader();
        final Reader closeReader = new ParserLevelCommentMarkupReader(closeDelegate);
        try {
            closeReader.close();
            emit("close.throwable", "none");
        } catch (final Throwable throwable) {
            emit("close.throwable", describe(throwable));
        }
        emit("close.count", Integer.toString(closeDelegate.closeCount));
    }

    private static void emitCases(
            final String family,
            final String[] cases,
            final int[][] requests,
            final boolean prototype) {
        for (int caseIndex = 0; caseIndex < cases.length; caseIndex++) {
            for (int requestIndex = 0; requestIndex < requests.length; requestIndex++) {
                final int[] request = requests[requestIndex];
                final Reader reader = prototype
                        ? new PrototypeOnlyCommentMarkupReader(new StringReader(cases[caseIndex]))
                        : new ParserLevelCommentMarkupReader(new StringReader(cases[caseIndex]));
                emitRead(
                        family + "." + caseIndex + "." + requestIndex,
                        reader,
                        request[0],
                        request[1],
                        request[2]);
            }
        }
    }

    private static void emitRead(
            final String key,
            final Reader reader,
            final int bufferSize,
            final int offset,
            final int len) {
        final char[] buffer = new char[bufferSize];
        final StringBuilder result = new StringBuilder();
        final StringBuilder returns = new StringBuilder();
        String throwable = "none";
        try {
            int read = 0;
            while (read >= 0) {
                read = reader.read(buffer, offset, len);
                if (returns.length() > 0) {
                    returns.append(',');
                }
                returns.append(read);
                if (read > 0) {
                    result.append(buffer, offset, read);
                }
            }
        } catch (final Throwable failure) {
            throwable = describe(failure);
        }
        emit(key, escape(result.toString()) + "|returns=" + returns + "|throwable=" + escape(throwable));
    }

    private static String describe(final Throwable throwable) {
        return throwable.getClass().getName() + ":" + throwable.getMessage();
    }

    private static String escape(final String value) {
        return value
                .replace("\\", "\\\\")
                .replace("\r", "\\r")
                .replace("\n", "\\n")
                .replace("|", "\\|");
    }

    private static void emit(final String key, final String value) {
        System.out.println(key + "=" + value);
    }

    private static final class TrackingReader extends Reader {

        private int closeCount;

        @Override
        public int read(final char[] cbuf, final int off, final int len) {
            return -1;
        }

        @Override
        public void close() throws IOException {
            this.closeCount++;
            throw new IOException("close-boom");
        }
    }
}
