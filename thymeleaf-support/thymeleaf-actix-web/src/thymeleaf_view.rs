use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{HttpRequest, HttpResponse, Responder};
use thymeleaf::TemplateEngine;
use thymeleaf::context::IContext;
use thymeleaf::web::{RenderedTemplate, RenderedTemplateBody};

use crate::ThymeleafBody;

/// 可直接从 Actix Web handler 返回的 Thymeleaf 视图。
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

    /// 在 Actix blocking 线程池渲染模板并包装为视图。
    ///
    /// 异步策略第一步（见 superpowers spec `2026-08-15-web-adapter-p0-design.md`）：
    /// `web::block`（Actix 原生阻塞池）包同步渲染——每次渲染付一次线程切换
    /// 成本，换取渲染核心零改动；同步入口并存，由宿主按并发形态选择。
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
    ) -> Result<Self, Box<dyn thymeleaf::TemplateEngineException + Send + Sync>> {
        let template = template.to_owned();
        let rendered =
            actix_web::web::block(move || engine.process_template(&template, context.as_ref()))
                .await
                .expect("blocking render task joins")?;
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

impl Responder for ThymeleafView {
    type Body = ThymeleafBody;

    fn respond_to(self, _request: &HttpRequest) -> HttpResponse<Self::Body> {
        let (status, headers, body) = self.rendered_template.into_parts();
        let status =
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response = HttpResponse::with_body(status, ThymeleafBody::new(body));
        for (name, value) in &headers {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(name.as_str().as_bytes()),
                HeaderValue::from_bytes(value.as_bytes()),
            ) {
                response.headers_mut().append(name, value);
            }
        }
        response
    }
}
