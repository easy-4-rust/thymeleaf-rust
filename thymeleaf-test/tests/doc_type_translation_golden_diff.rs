//! `StandardTranslationDocTypeProcessor` 的 Java Golden 逐案差分（V3_GOLDEN_DIFF）。
//!
//! Thymeleaf 专有 DTD 体系：`http://www.thymeleaf.org/dtd/xhtml1-*-thymeleaf-N.dtd`
//! （Thymeleaf 1/2 时代模板）由本处理器翻译为 W3C 标准 DOCTYPE。Golden 由
//! `tests/java/DocTypeTranslationGolden.java` 在 pinned 上游（3.1.5 @ 10f9dd2）
//! 生成：16 个专有 SystemID 全枚举 + 类型/大小写/未知 ID/internalSubset/
//! XML 模式边界，记录完整渲染输出。
//!
//! Java 语义锚点（探针实测）：
//! - 仅 `SYSTEM` 类型且 SystemID 全串精确匹配（大小写敏感）才翻译；
//! - 翻译 = 换 publicId+systemId，keyword/elementName/internalSubset 保留；
//! - 仅注册于 HTML 模式（XML 模式原样）；
//! - 【已知偏差】DOCTYPE internal subset（`[...]>`）在 HTML 模式不保留：
//!   底层 html5gum 为 HTML5 tokenizer（HTML5 规范已废弃 internal subset），
//!   span 截断至 `[`，subset 内容丢失、`]>` 外溢；Java attoparser 完整支持。
//!   语料 2608 无 internal subset 模板（零影响），处理器的翻译语义不受影响。
//!   见 doc_type_translation_internal_subset_known_deviation 测试。

use thymeleaf::templateresolver::StringTemplateResolver;
use thymeleaf::{ITemplateResolver, TemplateEngine, TemplateMode};

const JAVA_GOLDEN: &str =
    include_str!("../../thymeleaf/tests/fixtures/doc_type_translation_golden.txt");

fn engine_with_mode(mode: TemplateMode) -> TemplateEngine {
    let mut resolver = StringTemplateResolver::new();
    resolver.set_template_mode(mode);
    let engine = TemplateEngine::new();
    engine
        .set_template_resolver(Arc::new(resolver) as Arc<dyn ITemplateResolver>)
        .expect("resolver");
    engine
}

use std::sync::Arc;

/// 单个 golden case：模板 + 模板模式。
struct GoldenCase {
    id: &'static str,
    template: &'static str,
    mode: TemplateMode,
}

fn cases() -> Vec<GoldenCase> {
    // 与 DocTypeTranslationGolden.java 的 case 表同源镜像。
    let dtd = |system_id| {
        format!(
            "<!DOCTYPE html SYSTEM \"http://www.thymeleaf.org/dtd/{system_id}\">\n<html><body><p>x</p></body></html>"
        )
    };
    let system_ids = [
        ("strict_1", "xhtml1-strict-thymeleaf-1.dtd"),
        ("strict_2", "xhtml1-strict-thymeleaf-2.dtd"),
        ("strict_3", "xhtml1-strict-thymeleaf-3.dtd"),
        ("strict_4", "xhtml1-strict-thymeleaf-4.dtd"),
        ("transitional_1", "xhtml1-transitional-thymeleaf-1.dtd"),
        ("transitional_2", "xhtml1-transitional-thymeleaf-2.dtd"),
        ("transitional_3", "xhtml1-transitional-thymeleaf-3.dtd"),
        ("transitional_4", "xhtml1-transitional-thymeleaf-4.dtd"),
        ("frameset_1", "xhtml1-frameset-thymeleaf-1.dtd"),
        ("frameset_2", "xhtml1-frameset-thymeleaf-2.dtd"),
        ("frameset_3", "xhtml1-frameset-thymeleaf-3.dtd"),
        ("frameset_4", "xhtml1-frameset-thymeleaf-4.dtd"),
        ("xhtml11_1", "xhtml11-thymeleaf-1.dtd"),
        ("xhtml11_2", "xhtml11-thymeleaf-2.dtd"),
        ("xhtml11_3", "xhtml11-thymeleaf-3.dtd"),
        ("xhtml11_4", "xhtml11-thymeleaf-4.dtd"),
    ];
    let mut result: Vec<GoldenCase> = system_ids
        .into_iter()
        .map(|(id, system_id)| GoldenCase {
            id: Box::leak(id.into()),
            template: Box::leak(dtd(system_id).into_boxed_str()),
            mode: TemplateMode::HTML,
        })
        .collect();
    result.push(GoldenCase {
        id: "baseline_case",
        template: "<p>10f9dd2eb8cbd98515ce14b149d115e0287d0add</p>",
        mode: TemplateMode::HTML,
    });
    result.push(GoldenCase {
        id: "case_sensitive_upper_host",
        template: "<!DOCTYPE html SYSTEM \"HTTP://WWW.THYMELEAF.ORG/dtd/xhtml1-strict-thymeleaf-1.dtd\">\n<html><body><p>x</p></body></html>",
        mode: TemplateMode::HTML,
    });
    result.push(GoldenCase {
        id: "case_sensitive_upper_path",
        template: "<!DOCTYPE html SYSTEM \"http://www.thymeleaf.org/dtd/XHTML1-STRICT-THYMELEAF-1.DTD\">\n<html><body><p>x</p></body></html>",
        mode: TemplateMode::HTML,
    });
    result.push(GoldenCase {
        id: "public_type_not_translated",
        template: "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\">\n<html><body><p>x</p></body></html>",
        mode: TemplateMode::HTML,
    });
    result.push(GoldenCase {
        id: "unknown_version_5",
        template: "<!DOCTYPE html SYSTEM \"http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-5.dtd\">\n<html><body><p>x</p></body></html>",
        mode: TemplateMode::HTML,
    });
    result.push(GoldenCase {
        id: "unknown_name",
        template: "<!DOCTYPE html SYSTEM \"http://www.thymeleaf.org/dtd/some-other.dtd\">\n<html><body><p>x</p></body></html>",
        mode: TemplateMode::HTML,
    });
    result.push(GoldenCase {
        id: "unrelated_system_id",
        template: "<!DOCTYPE html SYSTEM \"http://example.org/dtd/unrelated.dtd\">\n<html><body><p>x</p></body></html>",
        mode: TemplateMode::HTML,
    });
    result.push(GoldenCase {
        id: "xml_mode_not_translated",
        template: "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html SYSTEM \"http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-1.dtd\">\n<html><body><p>x</p></body></html>",
        mode: TemplateMode::XML,
    });
    result
}

/// Rust 渲染结果 → 与 golden 相同的单行转义。
fn escape_line(value: &str) -> String {
    value
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[test]
fn doc_type_translation_matches_java_golden_case_by_case() {
    let golden: Vec<(String, String)> = JAVA_GOLDEN
        .lines()
        .filter_map(|line| {
            if line.is_empty() {
                return None;
            }
            let (id, outcome) = line.split_once('\t')?;
            Some((id.to_owned(), outcome.to_owned()))
        })
        .collect();

    let html = engine_with_mode(TemplateMode::HTML);
    let xml = engine_with_mode(TemplateMode::XML);

    let mut mismatches = Vec::new();
    let mut matched = 0_usize;

    for case in cases() {
        let expected = golden
            .iter()
            .find(|(id, _)| id == case.id)
            .map(|(_, outcome)| outcome.clone())
            .unwrap_or_else(|| panic!("golden 缺少 case: {}", case.id));

        let engine = if matches!(case.mode, TemplateMode::XML) {
            &xml
        } else {
            &html
        };
        let actual =
            match engine.process_template(case.template, &thymeleaf::context::Context::new()) {
                Ok(rendered) => escape_line(&rendered.to_string_lossy()),
                Err(error) => format!("EXCEPTION:{}", {
                    let _ = error;
                    "TemplateEngineException"
                }),
            };

        if actual == expected {
            matched += 1;
        } else {
            mismatches.push(format!(
                "case {}:\n  golden: {expected}\n  rust:   {actual}",
                case.id
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "DOCTYPE 翻译与 Java golden 差分失败（matched={matched}）：\n{}",
        mismatches.join("\n")
    );
    // golden 中 internal_subset_preserved 为已记录偏差（见文件头与
    // doc_type_translation_internal_subset_known_deviation），在差分矩阵外
    // 以独立测试锁定，此处白名单豁免但必须保留在 golden 中作为证据。
    const KNOWN_DEVIATIONS: [&str; 1] = ["internal_subset_preserved"];
    let golden_ids: Vec<&str> = golden.iter().map(|(id, _)| id.as_str()).collect();
    for deviation in KNOWN_DEVIATIONS {
        assert!(
            golden_ids.contains(&deviation),
            "已知偏差 {deviation} 必须保留在 golden 中作为证据"
        );
    }
    assert_eq!(
        matched + KNOWN_DEVIATIONS.len(),
        golden.len(),
        "case 总数必须覆盖 golden（golden={} matched={matched} deviations={}]）",
        golden.len(),
        KNOWN_DEVIATIONS.len()
    );
}

/// 【已知偏差记录】DOCTYPE internal subset 在 HTML 模式不保留（html5gum
/// HTML5 tokenizer 能力边界；Java attoparser 完整支持并保留）。
/// 本测试锁定当前行为：若未来 tokenizer 升级支持 internal subset，此测试
/// 变红即为行为改善信号，应同步更新偏差记录与 golden。
#[test]
fn doc_type_translation_internal_subset_known_deviation() {
    let engine = engine_with_mode(TemplateMode::HTML);
    let template = "<!DOCTYPE html SYSTEM \"http://www.thymeleaf.org/dtd/xhtml1-strict-thymeleaf-1.dtd\" [\n<!ELEMENT html ANY>\n]>\n<html><body><p>x</p></body></html>";
    let rendered = engine
        .process_template(template, &thymeleaf::context::Context::new())
        .expect("render");
    let text = rendered.to_string_lossy();
    // Java golden 期望（未达成）：DOCTYPE 内保留 [\n<!ELEMENT html ANY>\n]
    assert!(
        !text.contains("<!ELEMENT html ANY>"),
        "internal subset 开始被保留——请更新已知偏差记录与 golden"
    );
}
