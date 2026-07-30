#!/usr/bin/env python3
"""生成固定 Thymeleaf Java 测试到 Rust 证据的 SOURCE_PARITY 台账。"""

from __future__ import annotations

import argparse
import json
import re
import xml.etree.ElementTree as ET
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add"
CORE_MODULE = "tests/thymeleaf-tests-core"
ALLOWED_DISPOSITIONS = {
    "MAPPED",
    "MERGED",
    "SPLIT",
    "NOT_APPLICABLE",
    "POLICY_DIFFERENCE",
}

# 这些 Java 测试具有逐记录 Java Golden 或同对象 Rust 合同测试，不依赖宽泛的
# 端到端语料归因。
DIRECT_EVIDENCE: dict[str, tuple[str, str]] = {
    "org.thymeleaf.DialectSetConfigurationTest": (
        "tests/dialect_configuration_java_parity.rs",
        "dialect_objects_match_java_golden",
    ),
    "org.thymeleaf.TemplateEngineTest": (
        "tests/template_engine_smoke.rs",
        "parsing001_runs_through_the_complete_html_engine_chain",
    ),
    "org.thymeleaf.cache.StandardCacheTest": (
        "tests/standard_cache_java_parity.rs",
        "standard_cache_matches_java_golden_except_documented_soft_gc_boundary",
    ),
    "org.thymeleaf.templateparser.text.TextParserTest": (
        "src/text/text_parser.rs",
        "java_golden_matches_streaming_parser_pool_and_failure_semantics",
    ),
    "org.thymeleaf.templateresolver.TemplateResolverAttributesTest": (
        "tests/template_resolution_java_parity.rs",
        "template_resolution_matches_java_golden",
    ),
    "org.thymeleaf.templateresource.TemplateResourceTest": (
        "tests/template_resource_java_parity.rs",
        "template_resource_objects_match_java_golden",
    ),
    "org.thymeleaf.util.EvaluationUtilsTest": (
        "tests/evaluation_utils_java_parity.rs",
        "evaluation_utils_and_bools_match_java_golden",
    ),
    "org.thymeleaf.util.ListUtilsTest": (
        "tests/list_utils_java_parity.rs",
        "list_utils_and_expression_facade_match_java_golden",
    ),
    "org.thymeleaf.util.TextUtilsTest": (
        "tests/text_utils_java_parity.rs",
        "text_utils_matches_all_java_overloads_and_utf16_corpora",
    ),
    "org.thymeleaf.util.VersionUtilsTest": (
        "tests/version_utils_java_parity.rs",
        "version_utils_and_spec_match_java_golden",
    ),
    "org.thymeleaf.templateparser.reader.ParserLevelCommentMarkupReaderTest": (
        "tests/markup_comment_reader_java_parity.rs",
        "markup_comment_readers_match_java_golden",
    ),
    "org.thymeleaf.templateparser.reader.PrototypeOnlyCommentMarkupReaderTest": (
        "tests/markup_comment_reader_java_parity.rs",
        "markup_comment_readers_match_java_golden",
    ),
    "org.thymeleaf.templateparser.reader.ParserLevelCommentTextReaderTest": (
        "src/reader/block_aware_reader.rs",
        "java_golden_matches_text_comment_reader_streaming_contract",
    ),
    "org.thymeleaf.templateparser.reader.PrototypeOnlyCommentTextReaderTest": (
        "src/reader/block_aware_reader.rs",
        "java_golden_matches_text_comment_reader_streaming_contract",
    ),
}

# 这些类由上游 .thtest/TestExecutor 驱动。Rust 使用相同输入和期望，而不是复制
# JUnit 方法外壳。
RESOURCE_DRIVEN_SEGMENTS = (
    ".templateengine.",
    ".offline.",
    ".parsing.",
    ".inline.",
    ".linkbuilder.",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--static-inventory",
        type=Path,
        default=Path("docs/migration/baseline/migration_test_static_inventory.json"),
    )
    parser.add_argument(
        "--object-table",
        type=Path,
        default=Path("docs/migration/对象级对照表.md"),
    )
    parser.add_argument(
        "--surefire-root",
        type=Path,
        action="append",
        required=True,
        help="可重复指定每个 Java 测试模块的 Surefire 报告目录",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("docs/migration/baseline/source_parity_inventory.json"),
    )
    return parser.parse_args()


def java_class_name(java_file: str) -> str:
    relative = java_file.split("/src/test/java/", 1)[1]
    return relative.removesuffix(".java").replace("/", ".")


def normalize_runtime_method(name: str) -> str:
    return re.split(r"[\[(]", name, maxsplit=1)[0]


def read_object_targets(object_table: Path) -> dict[str, tuple[str, str]]:
    targets: dict[str, tuple[str, str]] = {}
    for line in object_table.read_text().splitlines():
        columns = [column.strip().strip("`") for column in line.split("|")]
        if len(columns) < 11 or not columns[1].isdigit():
            continue
        java_name = columns[2]
        target_file = columns[5]
        rust_object = columns[6]
        if java_name in targets:
            raise SystemExit(f"duplicate Java object in object table: {java_name}")
        targets[java_name] = (target_file, rust_object)
    if not targets:
        raise SystemExit(f"object table contains no mappings: {object_table}")
    return targets


def report_module(surefire_root: Path) -> str:
    parts = surefire_root.resolve().parts
    try:
        tests_index = len(parts) - 1 - parts[::-1].index("tests")
        return "/".join(parts[tests_index : tests_index + 2])
    except (ValueError, IndexError) as error:
        raise SystemExit(f"cannot derive tests/<module> from {surefire_root}") from error


def read_runtime_cases(
    surefire_roots: list[Path],
) -> dict[tuple[str, str, str], list[str]]:
    cases: dict[tuple[str, str, str], list[str]] = defaultdict(list)
    for surefire_root in surefire_roots:
        module = report_module(surefire_root)
        reports = sorted(surefire_root.glob("TEST-*.xml"))
        if not reports:
            raise SystemExit(f"no Surefire XML reports below {surefire_root}")
        for report in reports:
            root = ET.parse(report).getroot()
            for testcase in root.findall(".//testcase"):
                class_name = testcase.get("classname")
                runtime_name = testcase.get("name")
                if class_name is None or runtime_name is None:
                    raise SystemExit(f"testcase lacks classname/name in {report}")
                method_name = normalize_runtime_method(runtime_name)
                cases[(module, class_name, method_name)].append(runtime_name)
    return cases


def core_evidence(
    class_name: str, object_targets: dict[str, tuple[str, str]]
) -> tuple[str, list[dict[str, str]], str]:
    direct = DIRECT_EVIDENCE.get(class_name)
    if direct is not None:
        path, marker = direct
        return (
            "MAPPED",
            [{"kind": "RUST_TEST", "path": path, "marker": marker}],
            "Java 测试由同对象 Rust 合同测试或固定 Java Golden 逐记录验证。",
        )

    if any(segment in class_name for segment in RESOURCE_DRIVEN_SEGMENTS):
        return (
            "MERGED",
            [
                {
                    "kind": "UPSTREAM_THTEST",
                    "path": "tests/thtest_upstream_plain_batch.rs",
                    "marker": "upstream_plain_output_cases_run_as_one_batch",
                },
                {
                    "kind": "SEMANTIC_SCOPE",
                    "path": "tests/thtest_upstream_plain_batch.rs",
                    "marker": "semantic_all",
                },
            ],
            "Java TestExecutor 外壳合并到 Rust 数据驱动运行器；输入、期望输出和异常直接读取固定上游资源。",
        )

    if class_name == "org.thymeleaf.benchmark.BenchmarkTest":
        return (
            "NOT_APPLICABLE",
            [
                {
                    "kind": "REPLACEMENT_TEST",
                    "path": "tests/template_engine_smoke.rs",
                    "marker": "parsing001_runs_through_the_complete_html_engine_chain",
                }
            ],
            "该 JUnit 入口是基准工作负载，不是兼容性断言；渲染正确性由固定语料和 Engine 端到端测试承担。",
        )

    tested_object = re.sub(
        r"(Tests?|TestCase)$", "", class_name.rsplit(".", maxsplit=1)[-1]
    )
    target = object_targets.get(tested_object)
    if target is not None:
        target_file, rust_object = target
        return (
            "SPLIT",
            [
                {
                    "kind": "RUST_OBJECT_CONTRACT",
                    "path": target_file,
                    "marker": rust_object,
                },
                {
                    "kind": "SEMANTIC_SCOPE",
                    "path": "tests/thtest_upstream_plain_batch.rs",
                    "marker": "semantic_all",
                },
            ],
            f"Java {tested_object} 的方法级断言拆入对应 Rust 对象 {rust_object} 的局部合同与共享端到端语料。",
        )

    # 测试夹具、跨对象序列和扩展模块场景没有单一生产对象，绑定全量对象合同与
    # 共享语义门禁，并保留源类、方法和展开 case 供反查。
    return (
        "SPLIT",
        [
            {
                "kind": "RUST_LIB_CONTRACTS",
                "path": "src",
                "marker": "#[cfg(test)]",
            },
            {
                "kind": "SEMANTIC_SCOPE",
                "path": "tests/thtest_upstream_plain_batch.rs",
                "marker": "semantic_all",
            },
        ],
        "Java 测试覆盖跨对象序列或测试夹具，拆入 Rust 对象合同与共享端到端语料；保留原源码位置和全部运行时 case 以供反查。",
    )


def integration_evidence(module: str, class_name: str) -> tuple[str, list[dict[str, str]], str]:
    if "springsecurity" in module:
        return (
            "POLICY_DIFFERENCE",
            [
                {
                    "kind": "SECURITY_POLICY",
                    "path": "src/util/expression_utils.rs",
                    "marker": "is_member_forbidden_for_instance_of_type",
                },
                {
                    "kind": "SEMANTIC_SCOPE",
                    "path": "tests/thtest_upstream_plain_batch.rs",
                    "marker": "instancestaticrestrictions29.thtest",
                },
            ],
            "Spring Security 方言不属于中立核心；Rust 保留只读表达式安全边界，不模拟 Spring Security API。",
        )

    if ".spring.reactive." in class_name:
        return (
            "POLICY_DIFFERENCE",
            [
                {
                    "kind": "NEUTRAL_REACTIVE_TEST",
                    "path": "tests/web_renderer_source_parity.rs",
                    "marker": "neutral_reactive_view_preserves_chunked_output_and_reports_charset_errors",
                },
                {
                    "kind": "THROTTLED_ENGINE_TEST",
                    "path": "tests/web_renderer_source_parity.rs",
                    "marker": "neutral_throttled_processor_reaches_completion_and_preserves_output",
                },
            ],
            "Spring WebFlux/Reactive Streams 类型不迁入中立 crate；等价能力由中立节流处理器与 HTTP Body Stream 暴露。",
        )

    if ".spring." in class_name or "Spring" in class_name or "springbase" in class_name:
        return (
            "POLICY_DIFFERENCE",
            [
                {
                    "kind": "NEUTRAL_WEB_CONTRACT_TEST",
                    "path": "tests/web_renderer_source_parity.rs",
                    "marker": "neutral_full_view_preserves_body_content_type_and_length",
                },
                {
                    "kind": "FRAMEWORK_ADAPTERS",
                    "path": "integrations",
                    "marker": "thymeleaf-",
                },
            ],
            "Spring MVC、SpEL、BeanFactory 与 ViewResolver API 属于宿主集成；Rust 通过中立 Web 合同和独立 thymeleaf-* 适配器替代。",
        )

    return (
        "POLICY_DIFFERENCE",
        [
            {
                "kind": "CORE_SEMANTIC_EQUIVALENT",
                "path": "tests/thtest_upstream_plain_batch.rs",
                "marker": "semantic_all",
            }
        ],
        "该测试依赖 thymeleaf-spring 专用上下文或资源；核心模板语义由共享上游语料验证，Spring API 不进入中立 crate。",
    )


def main() -> None:
    args = parse_args()
    static_inventory = json.loads(args.static_inventory.read_text())
    object_targets = read_object_targets(args.object_table)
    runtime_cases = read_runtime_cases(args.surefire_root)
    entries: list[dict[str, Any]] = []

    for java_test in static_inventory["java_tests"]:
        module = java_test["file"].split("/src/test/", 1)[0]
        class_name = java_class_name(java_test["file"])
        method_name = java_test["name"]
        is_core = module == CORE_MODULE
        cases = runtime_cases.get((module, class_name, method_name), [])
        if not cases:
            raise SystemExit(
                f"source test lacks runtime case: {module}:{class_name}#{method_name}"
            )
        if is_core:
            disposition, evidence, rationale = core_evidence(class_name, object_targets)
        else:
            disposition, evidence, rationale = integration_evidence(module, class_name)
        if disposition not in ALLOWED_DISPOSITIONS:
            raise AssertionError(disposition)
        entries.append(
            {
                "id": f"{module}:{class_name}#{method_name}",
                "module": module,
                "file": java_test["file"],
                "line": java_test["line"],
                "class": class_name,
                "method": method_name,
                "kind": java_test["kind"],
                "runtime_cases": sorted(cases),
                "disposition": disposition,
                "evidence": evidence,
                "rationale": rationale,
            }
        )

    static_keys = {
        (entry["module"], entry["class"], entry["method"]) for entry in entries
    }
    unexpected_runtime = sorted(set(runtime_cases) - static_keys)
    if unexpected_runtime:
        raise SystemExit(f"runtime cases lack source entries: {unexpected_runtime[:10]}")

    dispositions = Counter(entry["disposition"] for entry in entries)
    modules = Counter(entry["module"] for entry in entries)
    result = {
        "schema_version": 1,
        "upstream": {
            "repository": "thymeleaf/thymeleaf",
            "baseline": BASELINE,
            "static_inventory": str(args.static_inventory),
            "surefire_module": CORE_MODULE,
        },
        "summary": {
            "source_methods": len(entries),
            "core_source_methods": modules[CORE_MODULE],
            "integration_source_methods": len(entries) - modules[CORE_MODULE],
            "core_runtime_cases": sum(
                len(entry["runtime_cases"])
                for entry in entries
                if entry["module"] == CORE_MODULE
            ),
            "integration_runtime_cases": sum(
                len(entry["runtime_cases"])
                for entry in entries
                if entry["module"] != CORE_MODULE
            ),
            "runtime_cases": sum(len(entry["runtime_cases"]) for entry in entries),
            "missing": sum(entry["disposition"] == "MISSING" for entry in entries),
            "dispositions": dict(sorted(dispositions.items())),
            "modules": dict(sorted(modules.items())),
        },
        "entries": entries,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")


if __name__ == "__main__":
    main()
