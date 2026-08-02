package org.thymeleaf.templateparser.reader;

import java.io.IOException;
import java.io.Reader;
import java.io.StringReader;

/**
 * 固定文本注释 Reader 的跨缓冲区、异常和生命周期可观察语义。
 */
public final class TextCommentReaderGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private TextCommentReaderGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);

        final String[] parserCases = {
                "",
                "plain",
                "/* hello */",
                "a/*[-hidden-]*/b",
                "/*[-all-]*/",
                "/* /*[- hello -]]*/ -]*/",
                "/* /*[- hello -]]*/ -]*/ */",
                "/* /*[[- hello -]]*/ -]*/ */",
                "/***[- hello -]***/ -]*/",
                "x/*[-a-]*/y/*[-b-]*/z",
                "x/*[",
                "x/*[-a-]*",
                "\u4e2d/*[-\ud83d\ude00-]*/\u6587"
        };
        final String[] prototypeCases = {
                "",
                "plain",
                "/* hello */",
                "a/*[+shown+]*/b",
                "/*[+all+]*/",
                "/* /*[+ hello +]*/ +]]]*/// */",
                "/*[+hello+]*/ +]]]*/// aa",
                "/*[[[/*[+hello +]*/ +]]]*///",
                "x/*[+a+]*/y/*[+b+]*/z",
                "x/*[",
                "x/*[+a+]*",
                "\u4e2d/*[+\ud83d\ude00+]*/\u6587"
        };
        final int[][] requests = {
                {1, 0, 1},
                {3, 0, 3},
                {7, 2, 3},
                {13, 4, 7}
        };

        emitCases("parser", parserCases, requests, false);
        emitCases("prototype", prototypeCases, requests, true);
        emitCombined(requests);
        emitSpecialCases();
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
                        ? new PrototypeOnlyCommentTextReader(new StringReader(cases[caseIndex]))
                        : new ParserLevelCommentTextReader(new StringReader(cases[caseIndex]));
                emitRead(
                        family + "." + caseIndex + "." + requestIndex,
                        reader,
                        request[0],
                        request[1],
                        request[2]);
            }
        }
    }

    private static void emitCombined(final int[][] requests) {
        final String[] cases = {
                "a/*[+shown+]*/b/*[-hidden-]*/c",
                "/*[-x/*[+not-shown+]*/y-]*/tail",
                "head/*[+x/*[-hidden-]*/y+]*/tail"
        };
        for (int caseIndex = 0; caseIndex < cases.length; caseIndex++) {
            for (int requestIndex = 0; requestIndex < requests.length; requestIndex++) {
                final int[] request = requests[requestIndex];
                final Reader reader = new ParserLevelCommentTextReader(
                        new PrototypeOnlyCommentTextReader(new StringReader(cases[caseIndex])));
                emitRead(
                        "combined." + caseIndex + "." + requestIndex,
                        reader,
                        request[0],
                        request[1],
                        request[2]);
            }
        }
    }

    private static void emitSpecialCases() {
        emitRead(
                "unfinished.parser",
                new ParserLevelCommentTextReader(new StringReader("a/*[-open")),
                3,
                0,
                3);
        emitRead(
                "unfinished.prototype",
                new PrototypeOnlyCommentTextReader(new StringReader("a/*[+open")),
                4,
                1,
                2);

        final TrackingReader zeroReader = new TrackingReader("a/*[-x-]*/b", -1, true, false);
        final Reader parser = new ParserLevelCommentTextReader(zeroReader);
        final char[] buffer = new char[4];
        try {
            final int zero = parser.read(buffer, 2, 0);
            emit("zero.return", Integer.toString(zero));
        } catch (final Throwable throwable) {
            emit("zero.return", describe(throwable));
        }
        emitRead("zero.after", parser, 4, 1, 2);

        emitRead(
                "delegate.readFailure",
                new ParserLevelCommentTextReader(new TrackingReader("abcdef", 2, false, false)),
                3,
                0,
                3);

        final TrackingReader closeDelegate = new TrackingReader("plain", -1, false, true);
        final Reader closeReader = new PrototypeOnlyCommentTextReader(closeDelegate);
        try {
            closeReader.close();
            emit("close.throwable", "none");
        } catch (final Throwable throwable) {
            emit("close.throwable", describe(throwable));
        }
        emit("close.count", Integer.toString(closeDelegate.closeCount));
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
            int guard = 0;
            while (read >= 0 && guard++ < 1000) {
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

        private final String value;
        private final int failAtPosition;
        private final boolean zeroOnce;
        private final boolean failClose;
        private int position;
        private boolean zeroReturned;
        private int closeCount;

        private TrackingReader(
                final String value,
                final int failAtPosition,
                final boolean zeroOnce,
                final boolean failClose) {
            this.value = value;
            this.failAtPosition = failAtPosition;
            this.zeroOnce = zeroOnce;
            this.failClose = failClose;
        }

        @Override
        public int read(final char[] cbuf, final int off, final int len) throws IOException {
            if (this.zeroOnce && !this.zeroReturned) {
                this.zeroReturned = true;
                return 0;
            }
            if (this.failAtPosition >= 0 && this.position >= this.failAtPosition) {
                throw new IOException("read-boom");
            }
            if (this.position >= this.value.length()) {
                return -1;
            }
            final int copied = Math.min(len, this.value.length() - this.position);
            this.value.getChars(this.position, this.position + copied, cbuf, off);
            this.position += copied;
            return copied;
        }

        @Override
        public void close() throws IOException {
            this.closeCount++;
            if (this.failClose) {
                throw new IOException("close-boom");
            }
        }
    }
}
