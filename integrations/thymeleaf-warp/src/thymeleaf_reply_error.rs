use thiserror::Error;

/// Warp 公共 Reply 边界无法表示 Thymeleaf 流式 Body 时的显式错误。
///
/// Warp 0.4 的响应 Body 类型是私有类型，外部 `Reply` 只能从有限字节构造响应；
/// 适配器因此拒绝流式输入，不会静默收集或破坏背压。
#[derive(Debug, Error)]
pub enum ThymeleafReplyError {
    /// 调用方把流式渲染结果交给只支持有限 Body 的 Warp Reply。
    #[error(
        "Warp 0.4 Reply uses a private body type and cannot preserve Thymeleaf streaming; use render_full for a Reply response"
    )]
    StreamingUnsupported,
}
