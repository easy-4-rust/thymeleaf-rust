package org.slf4j;

/**
 * Golden 编译器使用的最小日志边界；不参与待迁移业务语义。
 */
public interface Logger {

    void warn(String format, Object... arguments);

    default boolean isTraceEnabled() {
        return false;
    }

    default void trace(String format, Object... arguments) {
        // Golden 中默认关闭 trace；该方法只满足上游编译边界。
    }
}
