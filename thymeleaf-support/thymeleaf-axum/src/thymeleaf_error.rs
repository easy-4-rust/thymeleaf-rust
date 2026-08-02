use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use thymeleaf::web::RenderError;

/// Axum handler 使用的渲染错误响应。
///
/// 公开响应不泄漏内部模板名、表达式或文件路径；详细错误由应用通过
/// [`Self::get_cause`] 记录。
pub struct ThymeleafError {
    cause: RenderError,
}

impl ThymeleafError {
    /// 包装中立渲染错误。
    ///
    /// # 参数
    /// - `cause`：核心渲染失败。
    #[must_use]
    pub const fn new(cause: RenderError) -> Self {
        Self { cause }
    }

    /// 返回供应用日志记录的原始渲染错误。
    ///
    /// # 返回
    /// 保留完整原因链的核心错误。
    #[must_use]
    pub const fn get_cause(&self) -> &RenderError {
        &self.cause
    }
}

impl From<RenderError> for ThymeleafError {
    fn from(cause: RenderError) -> Self {
        Self::new(cause)
    }
}

impl IntoResponse for ThymeleafError {
    fn into_response(self) -> Response<Body> {
        let _cause = self.cause;
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Template rendering failed",
        )
            .into_response()
    }
}
