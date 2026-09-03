# Changelog

本项目遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 与
[语义化版本](https://semver.org/lang/zh-CN/)（晋级规则见
[docs/superpowers/specs/2026-07-28-versioning-governance.md](docs/superpowers/specs/2026-07-28-versioning-governance.md)）。

## [Unreleased]

### Added

- **可选 DTD 验证**（`dtd-validation` feature gate，XML 模式）：新增
  `thymeleaf::dtd` 模块——内嵌 W3C XHTML 四族 DTD（xhtml1
  strict/transitional/frameset + xhtml11，`include_str!` + `MemoryResolver`
  解析 `.ent`/`.mod` 引用，SHA-256 完整性测试 14 文件）+ 实体展开预算
  （深度 10/次数 1000/1MB，反 expansion bomb）+ 有状态 push 验证器封装。
  配置面：`TemplateEngine::set_dtd_validation_policy` +
  `IEngineConfiguration::get_dtd_validation_policy`，三策略
  `Disabled`（默认，零开销零行为影响）/`Warn`（验证但不拒绝）/
  `Strict`（违反即 `TemplateParserError`）。DOCTYPE 缺失或含内部子集
  时跳过验证；DOCTYPE 无法解析时 Strict 失败关闭、Warn 降级。
  测试：`dtd_validation` 13 例 + `dtd_file_integrity` 3 例，
  dtd 模块行覆盖 100%（cargo-llvm-cov）

## [0.1.0-beta.0] - 2026-09-03

> 本版本相对 0.1.0-alpha.1：**文档↔代码核对收口 + 性能基线 + 覆盖率体系 +
> VALUE_ADD 97 测试 + web 桥接域交付**。alpha→beta 八门禁全绿
> （VERSION-PLAN §4.1），本版起进入 beta 通道。

### Added

- **VALUE_ADD 测试批（97 例）**：方言装饰契约（processor_configuration_utils 11）、
  数字格式边界（number_utils 26）、string_utils 分支（23）、资源加载工具（12）、
  web 渲染路径（thymeleaf_renderer 9）、WebExchange 参数契约（i_web_request 16）
  ——归因结论：Java 侧均无独立测试类，全部为缺口导向的合规 VALUE_ADD
- **criterion 性能基线**（`benches/render_baseline.rs` 三基准 +
  `docs/release/benchmarks.md` drift gate）：单变量插值 10.18µs /
  th:each×100 145µs / 混合文档 225.5µs
- **覆盖率体系**：`coverage-baseline`（79.89%→**80.20%** 行，--all-features）
  + `coverage-gap-attribution`（337 文件归因 + Top12 分类 + 5 批排期）
- **安全模型文档**（`docs/release/security.md`）：零 unsafe 实测 /
  表达式 ACL 禁止类型表 + 静态方法白名单 / deny 策略 / proptest fuzz 基线
- **web 桥接域**（8-15 计划线）：topcoat P0 四件套、actix HostWeb 契约、
  vernal ThymeleafViewResolver（Spring MVC 视图解析桥）
- **OGNL 求值器 V3 golden 差分**（46 case 端到端矩阵）

### Changed

- alpha→beta 门禁全表落定（VERSION-PLAN §4.1）：security.md 补建（修正虚标）、
  `cargo package --verify` 通过、S11 状态更正为完成
- CI：concurrency cancel-in-progress（同分支新推送取消旧排队 run）

### Fixed

- 内部 Elvis 语义偏差（`${a ?: b}` 对齐 Java OGNL 行为）
- 表达式求值器若干 parity 修正（随 golden 差分）

### Verification

- workspace `--all-features`：**1,534 passed / 0 failed**
- corpus `semantic_all`：2/2 批次（2,595 .thtest）
- 布局审计（CI 口径）：strict_blockers=0（113 豁免登记）
- `cargo package -p thymeleaf` 演练通过

## [Unreleased]

### Added

- **DOCTYPE 翻译（Thymeleaf 专有 DTD 体系）Java golden 差分（V3）**：
  `DocTypeTranslationGolden.java` 导出 24 case（16 个专有
  `thymeleaf.org/dtd` SystemID 全枚举 + 大小写/PUBLIC 类型/未知 ID/
  internalSubset/XML 模式边界），`doc_type_translation_golden_diff.rs`
  逐案断言 24/24 MATCH；处理器覆盖率 0→90.27%。已知偏差记录：HTML 模式
  DOCTYPE internal subset 不保留（html5gum HTML5 tokenizer 能力边界，
  Java attoparser 支持，语料零覆盖）。
- **OGNL 求值器 Java golden 差分（V3）**：`OgnlEvaluationGolden.java` 在 pinned
  上游（3.1.5 @ 10f9dd2）导出 46 case 端到端矩阵（属性导航/方法调用/集合/
  算术/比较/逻辑/三元/default/空值/字面量），`ognl_evaluation_golden_diff.rs`
  逐案断言 Rust 输出与 golden 一致（含异常类名归一化映射）。

### Fixed

- **内部 Elvis 语义偏差（Java parity）**：`${a ?: b}`（Elvis 简写在 `${}`
  内部）Java 3.1.5 原样交 OGNL 3.3.4 求值，OGNL 不支持 Elvis → 渲染期
  TemplateInputException；Rust 此前错误支持了该简写。移除 native 求值器的
  default 分支（外部 `${a} ?: b` DefaultExpression 不受影响，corpus 全部
  使用外部形式）。4 个测试文件 5 处断言修正为 Java parity。

- **vernal-webmvc ViewResolver 桥**（plan `2026-08-15-webmvc-viewresolver-bridge`）：
  - `ThymeleafViewResolver`（thymeleaf-vernal）：对标
    `org.thymeleaf.spring6.view.ThymeleafViewResolver` 核心子集——视图名
    prefix/suffix 映射（Spring 默认 `classpath:/templates/`+`.html`）、
    Model 弱类型桥（Any→TemplateValue）、Locale 协商（render(locale)→
    Context::set_locale）、cacheable=false 每渲染失效缓存。集成测试 5/5 绿。
  - vernal-web 新视图合同（`Model`/`RenderedView`/`View`/`ViewResolver`，
    dev `6d137da`）：对标 `org.springframework.ui.Model` 与 Web MVC
    View/ViewResolver；`RenderedView` 用 http/bytes 底层类型避免与
    vernal-http 环依赖。
  - vernal-aop 修复：aspect-core/std 本地 path 依赖改 aspect-rs git 依赖
    （rev 89beaa90），恢复 git 消费方（thymeleaf-vernal）可用性。

- **Web 适配器 P0 做厚**（superpowers plan `2026-08-15-web-adapter-p0`）：
  - `thymeleaf-axum` / `thymeleaf-actix-web` / `thymeleaf-topcoat` 补齐
    `HostWeb` 四件套（`IWebRequest` ~24 方法 + `IWebExchange`/`IWebSession`/
    `IWebApplication`），契约测试与 `thymeleaf-hyper` 标杆逐断言对齐
    （axum 13 / actix 13 / topcoat 12 测试全绿，含端到端渲染）。
  - 三件新增 `ThymeleafView::render_async`：axum/topcoat 走 tokio
    `spawn_blocking`，actix 走原生 `web::block`；同步入口并存。
  - topcoat API 勘察落档（`2026-08-15-topcoat-api-notes.md`）：topcoat 0.5.0
    router 层为 http 1.x 类型族 re-export，与 axum 路线同构；响应式 view
    宏与 Thymeleaf SSR 互补。
  - vernal-webmvc 对接缺口清单（`2026-08-15-webmvc-view-integration-notes.md`）：
    ViewResolver trait 缺失等 5 项桥接，作为下一计划输入。

### Changed

- `thymeleaf-gotham` / `thymeleaf-tide` / `thymeleaf-warp` 降级**实验性**
  （上游停滞/被取代；测试移入 `full` feature，CI `--all-features` 仍看护）。
- Vernal 约定文档新增 4.7 视图层选型节（thymeleaf-rust 替代
  ThymeleafViewResolver，状态 `[待验证]` → 本轮 P0 完成后 `[已验证]`）。



### Changed

- **命名 Rust 化（语义锁定，行为零变更）**：全量移除 `Java*` 前缀类型与
  `java_*` 边界方法，改用 Rust 领域命名——`JavaString`→`Utf16String`、
  `JavaLocale`→`Locale`、`JavaWriter`→`TemplateWriter`、`JavaTemporal`→
  `TemporalValue`、`JavaNumber`→`NumberValue`、`JavaList`→`ListValue`、
  `JavaDate`→`DateValue`、`JavaEvaluation*`→`Evaluation*` 等 40+ 类型；
  `ClassNotFoundException`→`ClassNotFoundError`、`NoSuchMethodException`→
  `NoSuchMethodError`、`OgnlException`→`OgnlError`；
  `TemplateObject` 边界方法 `java_class_name`→`class_name`、
  `java_invoke_method`→`invoke_method` 等 8 项。
  模板语言可见的 Java 对象契约（UTF-16 语义、toString 格式、corpus 断言、
  `%EXCEPTION` 字符串键）全部保持。另清理 60+ 个内部 `java_*` 辅助函数
  （`java_trim`→`trim`、`java_message`→`message`、`java_hash_code`→`hash_code`、
  `CharSequenceValue` 的 `java_length`/`java_char_at`/`java_to_string`→
  `length`/`char_at`/`to_utf16_string` 等）。API baseline 已用 cargo-public-api 再生成。
- **安全模型修正（预先存在的 CI 失败）**：`ExpressionUtils::is_type_forbidden`
  改为默认拒绝（仅白名单 + `java.time.*`/`org.thymeleaf.*` 受信前缀放行；
  无包名裸类名仍由解析器报 `ClassNotFound`，保持 corpus 契约）；
  `is_member_forbidden(None, ...)` 无目标上下文默认拒绝危险成员；
  GTVG 示例测试固定进程默认 Locale，消除 CI 平台 LANG 差异。

### Added

- **治理门禁**：通用迁移布局审计 vendor 为 `scripts/audit_migration_layout.py`
  （复制自 rust-java-migration 技能），新增批准清单机制
  （`docs/migration/layout_approvals.json`，含 reason/owner/exit_criteria 契约）
  与 no-op 来源证据识别；CI 新增通用审计 job 与 `migration-check` 双轨并存，
  `--require-source-comments --fail-on-warning` 全量启用。
- **来源注释生成器**：`scripts/generate_java_source_comments.py`，从对象级对照表
  与 Java API 清单推导"对应 Java"注释（可验证方法 → `Class#method()`；
  接口/继承方法 → `Java 接口/超类方法`；Rust 侧辅助 → 显式标注）。
- **独立对象拆分**：`JavaString`/`JavaStringResult` 从 `util/logging_utils.rs`
  拆分为 `util/java_string.rs`（重导出保持外部零 churn）。
- **发布配套文档**：`docs/superpowers/specs/2026-07-28-versioning-governance.md`（alpha→beta→1.0 晋级规则）、
  `docs/release/` 目录建立。

### Fixed

- **仓库键冲突竞态**：`ElementNames`/`AttributeNames` 全局仓库在并行访问下，
  `forHTMLName("xmlns:th")` 与 `forHTMLNameWithPrefix("xmlns","th")` 共享键
  `xmlns:th` 产生顺序依赖（Java 固有的首注册者胜语义被 Rust 并行测试暴露）。
  修复为 keep-first（返回既有绑定，对应 Java 读路径 short-circuit），并令
  HTML withPrefix(xmlns) 断言序无关；parity 测试以 `serial_test` 镜像 JUnit
  单线程模型。

### Governance

- `multiple_public_objects` 97 → 0、`missing_object_file` 17 → 0、
  `stub_logic` 22 → 0、`missing_java_source_comment` 1280 → 0
  （通用审计 `errors=0 warnings=0 strict_blockers=0`）。
- 15 个 `thymeleaf-support/*` crate 的 version/edition/rust-version/license
  改为 `[workspace.package]` 继承；`topcoat` rust-version 对齐 1.88。

### 测试（Java 1:1 差分补移植）

- `WebEngineContextTest` 14/14、`EngineContextTest` 10/10、
  `AttributeNamesTest` 6/6、`ElementNamesTest` 6/6、
  `LazyContextVariableTest` 10/10 全部方法 1:1 锁定（含 setVariable null 语义、
  表示串活 exchange 感知、别名 assertSame 系列）。

### Known Limitations（已修复）

- **TemplateEngine.render smoke 超时（真实 bug，已修复）**：根因有两个
  Rust 侧真实缺陷——
  1. `LiteralSubstitutionUtil` 对 `${${||}}` 以完全相同输入无限递归
     （Java 上游是单遍迭代状态机、零递归）。修复：步骤 2 递归加
     `substituted != selector` 进度守卫（相等时落到主状态机），递归入口
     加深度上限 16。Java 3.1.5 实测 ground truth：`th:text="${${||}}"` 在
     模板解析期抛 `TemplateInputException`（嵌套 `${||}` OGNL 语法错误），
     Rust 现在同样快速返回 Err（parity 锁定）。
  2. `markup_selector::parse_attributes` 对 `<L/ꟓ>`、`<L=x>` 类输入
     （自闭合斜杠/`=` 落在属性名位置）空名 push 后永不前进——无限
     `Vec::push` 内存膨胀（14GB） + 100% CPU 挂起。修复：空名时跳过该
     字符保证前进，属性合法性仍由 adapter 侧校验。
  附加结构防御：`ExpressionParsingUtil`/native OGNL 解析入口长度上限
  4096 UTF-16 units + 递归深度上限 256；`parse_internal` 模板字节上限
  64MB；`parse_html` token 进度守卫（span.end 连续不前进即中止）。
  render smoke proptest 与 html parser fuzz 均已恢复（DiscardingWriter +
  shrink 钳制 + proptest timeout 60s 兜底）。
- **html5gum tokenizer**：历史 SIGKILL 根因是输出侧无界缓冲（CapturedWriter，
  已由 DiscardingWriter 消除）；tokenizer 0.8.4 内部缓冲 O(n) 有界，隔离
  驱动验证 `<L/ꟓ>` 等输入 token 流正常。HTML parser fuzz 已恢复。
- **API baseline CI**：`cargo public-api` 需要 nightly toolchain——CI 新增
  固定 `nightly-2026-07-28`（与 `docs/release/api-baseline.txt` 生成版本
  一致，本地已验证 diff 完全匹配），public-api 步骤移除 `continue-on-error`
  改为硬门禁；alpha 阶段 API 漂移必须显式更新 baseline。
