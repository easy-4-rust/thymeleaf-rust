package org.slf4j;

/**
 * Golden 编译器使用的无输出日志工厂。
 */
public final class LoggerFactory {

    private static final Logger NO_OP = (format, arguments) -> {
    };

    private LoggerFactory() {
    }

    public static Logger getLogger(final Class<?> type) {
        return NO_OP;
    }
}
