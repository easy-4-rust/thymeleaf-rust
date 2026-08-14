# vernal-webmvc 视图解析对接验证笔记（Task 3.5 设计验证）

- **日期**：2026-08-15
- **状态**：设计验证（非实施）——对接缺口清单作为下一个计划的输入

## 1. 验证结论

`ThymeleafView`（thymeleaf-topcoat / thymeleaf-vernal）**可以**作为
vernal-webmvc 的 View 后端，但需要一层薄桥接，不是零成本直连。

## 2. 对接形态（三件适配器三种通道）

| 适配器 | 响应通道 | 对接 vernal-webmvc 的形态 |
|---|---|---|
| thymeleaf-axum | `IntoResponse → axum::response::Response` | vernal-axum 的 handler 直接返回 `ThymeleafView`（axum 生态内零桥接） |
| thymeleaf-actix-web | `Responder → HttpResponse` + `ThymeleafBody` 流式 | vernal-actix-web handler 直接返回（actix 生态内零桥接） |
| thymeleaf-topcoat | `IntoResponse(cx: &Cx)`（topcoat 特色签名） | vernal-web/webmvc 若走 Topcoat 轨道，route 返回 `ThymeleafView` |

**核心通道**：vernal-http 的中立 `HttpResponse`。`thymeleaf-vernal::ThymeleafView`
已实现 `RenderedTemplate → vernal HttpResponse` 协议转换（729 行 crate 的主线）。

## 3. 缺口清单（下一计划输入）

1. **ViewResolver 抽象缺失**：vernal-webmvc 尚无 `ViewResolver`/`View` trait
   定义（对标 Spring `org.springframework.web.servlet.ViewResolver`）。需要
   在 vernal-webmvc 定义 trait（`resolve_view_name(name, locale) -> View` +
   `View.render(model, exchange)`），thymeleaf-vernal 提供第一个实现
   `ThymeleafViewResolver`。
2. **模型桥**：Spring MVC 的 `Model`（attribute map）→ thymeleaf
   `WebContext` 变量集的映射函数（`IWebExchange::set_attribute_value` 已有，
   需批量入口）。
3. **Locale 协商**：vernal-webmvc 的 locale 解析结果传入
   `HostWebExchange::new(.., locale)`（构造参数已预留）。
4. **模板缓存配置**：Spring `spring.thymeleaf.cache` 配置项 → thymeleaf
   `TemplateCache` 开关的配置桥（vernal 配置体系 → thymeleaf Configurable）。
5. **视图名 → 模板资源解析**：`ViewResolver` 前缀/后缀语义（Spring
   `spring.thymeleaf.prefix/suffix`）→ `TemplateResolver` 的
   prefix/suffix（核心已实现，需配置暴露）。

## 4. 建议的下一计划

`vernal-webmvc ViewResolver 集成`：定义 View/ViewResolver trait + 上述 5 项
桥接 + GTVG 示例移植验证。规模约 6-8 Task。
