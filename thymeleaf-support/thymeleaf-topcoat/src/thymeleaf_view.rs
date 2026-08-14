use std::sync::Arc;

use thymeleaf::TemplateEngine;
use thymeleaf::context::IContext;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{Body, IntoResponse, Response};

/// 可直接从 Topcoat route 返回的 Thymeleaf 渲染结果。
///
/// 该适配器使用 Topcoat 的公开 `http_body` 包装入口，保留状态码、Header、
/// 数据帧、背压和取消传播。
pub struct ThymeleafView {
    rendered_template: RenderedTemplate,
}

impl ThymeleafView {
    /// 包装框架中立渲染结果。
    ///
    /// # 参数
    /// - `rendered_template`：核心渲染器产生的响应。
    #[must_use]
    pub const fn new(rendered_template: RenderedTemplate) -> Self {
        Self { rendered_template }
    }

    /// 在 Tokio 阻塞线程池渲染模板并包装为视图。
    ///
    /// 异步策略第一步（见 superpowers spec `2026-08-15-web-adapter-p0-design.md`）：
    /// `spawn_blocking` 包同步渲染——每次渲染付一次线程切换成本，换取渲染
    /// 核心零改动；同步入口并存，由宿主按并发形态选择。Topcoat 为
    /// Tokio-first 全栈框架，直接使用 Tokio 阻塞池。
    ///
    /// # 参数
    /// - `engine`：模板引擎共享句柄。
    /// - `template`：模板文本。
    /// - `context`：线程安全渲染上下文的共享句柄（`IContext: Send + Sync`）。
    ///
    /// # 返回
    /// 渲染成功得到 200 全量 body 视图；失败得到引擎异常。
    pub async fn render_async(
        engine: Arc<TemplateEngine>,
        template: &str,
        context: Arc<dyn IContext>,
    ) -> Result<Self> {
        let template = template.to_owned();
        let rendered = tokio::task::spawn_blocking(move || {
            engine.process_template(&template, context.as_ref())
        })
        .await
        .map_err(|error| topcoat::Error::from(std::io::Error::other(error.to_string())))?
        .map_err(|error| topcoat::Error::from(std::io::Error::other(error.to_string())))?;
        let body = rendered.to_string_lossy().into_bytes();
        Ok(Self::new(RenderedTemplate::new(
            Default::default(),
            RenderedTemplateBody::Full(body.into()),
        )))
    }
}

impl From<RenderedTemplate> for ThymeleafView {
    fn from(rendered_template: RenderedTemplate) -> Self {
        Self::new(rendered_template)
    }
}

impl IntoResponse for ThymeleafView {
    fn into_response(self, _context: &Cx) -> Result<Response> {
        let (status, headers, body) = self.rendered_template.into_parts();
        let mut response = Response::new(Body::new(body));
        *response.status_mut() = status;
        *response.headers_mut() = headers;
        Ok(response)
    }
}
