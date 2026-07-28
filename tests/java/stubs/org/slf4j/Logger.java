package org.slf4j;

/**
 * Golden 编译器使用的最小日志边界；不参与待迁移业务语义。
 */
public interface Logger {

    void warn(String format, Object... arguments);
}
