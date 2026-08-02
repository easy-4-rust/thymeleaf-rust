use thiserror::Error;

/// 中立 Web 渲染阶段的错误。
///
/// 该对象是 Rust Web 整合扩展，不对应额外 Java 对象；内部保留 Thymeleaf
/// 异常文本，框架适配器可据此映射日志、状态码或 rejection。
/// 对应 Java 语义：Rust 侧内部类型（Java 无直接对应对象）。
#[derive(Debug, Error)]
#[error("{message}")]
pub struct RenderError {
    message: String,
}

impl RenderError {
    /// 使用可诊断消息创建渲染错误。
    /// 对应 Java 语义：Rust 侧辅助函数（Java 无直接对应）。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// 返回不含框架类型的错误消息。
    /// 对应 Java 语义：Java 接口/超类方法 `getMessage()` 的 Rust 移植（`None` 继承路径）。
    #[must_use]
    pub fn get_message(&self) -> &str {
        &self.message
    }
}
