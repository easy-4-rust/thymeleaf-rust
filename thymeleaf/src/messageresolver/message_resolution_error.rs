/// 消息解析过程可观察的动态错误。
///
/// 对应 Java: `IMessageResolver` 方法可以传播的运行时异常。
pub type MessageResolutionError = Box<dyn std::error::Error + Send + Sync>;
