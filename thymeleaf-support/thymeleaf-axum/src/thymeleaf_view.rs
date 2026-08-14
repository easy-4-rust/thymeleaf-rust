use std::sync::Arc;

use axum::body::Body;
use axum::http::Response;
use axum::response::IntoResponse;
use thymeleaf::TemplateEngine;
use thymeleaf::context::IContext;
use thymeleaf::web::RenderedTemplate;

/// 可作为 Axum handler 返回值的 Thymeleaf 渲染结果。
///
/// 对应 Thymeleaf Web 集成中的视图响应职责。
pub struct ThymeleafView {
    rendered_template: RenderedTemplate,
}

impl ThymeleafView {
    /// 包装框架中立渲染结果。
    ///
    /// # 参数
    /// - `rendered_template`：核心渲染器产生的状态、Header 与 Body。
    #[must_use]
    pub const fn new(rendered_template: RenderedTemplate) -> Self {
        Self { rendered_template }
    }

    /// 消费适配器并返回中立结果。
    ///
    /// # 返回
    /// 未丢失状态、Header 或流式 Body 的 `RenderedTemplate`。
    #[must_use]
    pub fn into_rendered_template(self) -> RenderedTemplate {
        self.rendered_template
    }

    /// 在阻塞线程池渲染模板并把结果包装为 Axum 响应值。
    ///
    /// 异步策略第一步（见 superpowers spec `2026-08-15-web-adapter-p0-design.md`）：
    /// `spawn_blocking` 包同步渲染——每次渲染付一次线程切换成本，换取渲染
    /// 核心零改动、正确性零风险；同步入口并存，由宿主按并发形态选择。
    ///
    /// # 参数
    /// - `engine`：线程安全的模板引擎共享句柄。
    /// - `template`：模板文本。
    /// - `context`：线程安全渲染上下文的共享句柄（`IContext: Send + Sync`）。
    ///
    /// # 返回
    /// 渲染成功得到 200 全量 body 视图；失败得到携带原因的诊断错误。
    pub async fn render_async(
        engine: Arc<TemplateEngine>,
        template: &str,
        context: Arc<dyn IContext>,
    ) -> Result<Self, crate::ThymeleafError> {
        let template = template.to_owned();
        // 上下文以共享句柄跨线程（spawn_blocking 闭包要求 'static）。
        let rendered = tokio::task::spawn_blocking(move || {
            engine.process_template(&template, context.as_ref())
        })
        .await
        .expect("blocking render task joins")?;
        let body = rendered.to_string_lossy().into_bytes();
        Ok(Self::new(RenderedTemplate::new(
            Default::default(),
            thymeleaf::web::RenderedTemplateBody::Full(body.into()),
        )))
    }
}

impl From<RenderedTemplate> for ThymeleafView {
    fn from(rendered_template: RenderedTemplate) -> Self {
        Self::new(rendered_template)
    }
}

impl IntoResponse for ThymeleafView {
    fn into_response(self) -> Response<Body> {
        let (status, headers, body) = self.rendered_template.into_parts();
        let mut response = Response::new(Body::new(body));
        *response.status_mut() = status;
        *response.headers_mut() = headers;
        response
    }
}
