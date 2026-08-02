# thymeleaf-examples —— GTVG（Good Thymes Virtual Grocery）示例移植

对应 Java [`examples/core`](https://github.com/thymeleaf/thymeleaf/tree/3.1.x/examples/core)
的 `thymeleaf-examples-gtvg-{jakarta,javax}` 模块（两变体仅 Servlet 包名不同，Rust
侧统一为一份）。

## 布局

| 目录 | 对应 Java |
|---|---|
| `templates/` | `src/main/webapp/WEB-INF/templates/`（8 个 HTML + 11 个 properties 1:1 字节复制，`diff -r` 验证） |
| `templates/css`、`templates/images` | `webapp/css/gtvg.css`、`webapp/images/gtvglogo.png` |
| `src/business/` | `business/`（6 实体 + 3 仓库含全部种子数据 + 3 服务 + CalendarUtil） |
| `src/controllers/` | `web/controller/`（8 控制器 + `IGTVGController` + `ControllerMappings`） |
| `src/web/` | `GTVGFilter` 的宿主角色（`JakartaServletWebApplication`/`Exchange`/`HttpServletRequest`/`HttpSession`） |
| `examples/gtvg.rs` | `GTVGFilter` 过滤器流程：session 注入 → URL 映射 → 渲染 |
| `tests/gtvg.rs` | `GTVGTest` 的示例级断言 + 映射 + 业务数据 + 模板字节校验 |

## 运行

```sh
cargo run -p thymeleaf-examples --example gtvg
```

依次渲染 7 个页面：`/`、`/product/list`、`/product/comments?prodId=13`、
`/order/list`、`/order/details?orderId=3`、`/subscribe`、`/userprofile`。

## 引擎侧零定制

- **消息**：引擎默认 `StandardMessageResolver` 按模板名读取并列的
  `home.properties`、`product/list.properties` 等（对应 Java 默认行为）；
  `home_en.properties` 的 Locale 覆盖同样生效
- **链接**：默认 `StandardLinkBuilder` 处理 `@{...}` 表达式
- **会话**：`session.user` 通过 `IWebSession` 属性作用域暴露（
  `GTVGFilter#addUserToSession` 的固定用户 `John Apricot, Antarctica`）
- **日期**：`#calendars.format` 处理 `MMMM dd'','' yyyy`（MessageFormat
  转义 → 日期模式引号语义）等 Java 模式

## 测试

```sh
cargo test -p thymeleaf-examples
```

12 项验收：页面渲染断言（固定“今天”2011-11-11 保持确定性）、URL 映射与
`;jsessionid` 剥离、业务种子数据（6 客户 / 30 产品 / 21 评论 / 3 订单）、
模板 SHA-256 与上游字节一致。
