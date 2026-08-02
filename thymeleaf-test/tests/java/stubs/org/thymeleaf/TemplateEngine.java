package org.thymeleaf;

/**
 * 仅为编译 TemplateMode Golden Oracle 提供上游日志线程名边界。
 */
public final class TemplateEngine {

    private TemplateEngine() {
    }

    public static String threadIndex() {
        return Thread.currentThread().getName();
    }
}
