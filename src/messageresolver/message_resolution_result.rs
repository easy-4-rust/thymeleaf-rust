use super::MessageResolutionError;

/// 消息解析结果，保留 Java 运行时异常传播语义。
///
/// 对应 Java: `IMessageResolver` 方法的正常返回或运行时异常。
pub type MessageResolutionResult<T> = Result<T, MessageResolutionError>;
