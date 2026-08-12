#!/usr/bin/env python3
"""生成 docs/superpowers/specs/2026-07-28-java-test-method-mapping.md —— 上游测试类**方法级**映射表。

数据源：docs/migration/baseline/source_parity_inventory.json（由
generate_source_parity_inventory.py 生成，875 个源码入口方法级处置，MISSING=0）。

每个核心方法一行：方法名、处置、运行时 case、Rust 覆盖证据（RUST_TEST 的
parity 文件与 marker / RUST_LIB_CONTRACTS 的对象合同 / UPSTREAM_THTEST 的
语料运行器）。Spring 集成模块按类汇总为 POLICY_DIFFERENCE。

用法：
    python3 scripts/generate_java_test_method_matrix.py
"""

from __future__ import annotations

import json
import pathlib
from collections import Counter, defaultdict

ROOT = pathlib.Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "docs" / "migration" / "baseline" / "source_parity_inventory.json"
OUTPUT = ROOT / "docs" / "migration" / "Java测试方法对照.md"

CORE_MODULE = "tests/thymeleaf-tests-core"

DISPOSITION_ORDER = ["SPLIT", "MERGED", "MAPPED", "NOT_APPLICABLE", "POLICY_DIFFERENCE"]


def evidence_text(entries: list[dict]) -> str:
    """把证据条目压成一行可读文本。"""
    parts = []
    for ev in entries:
        kind = ev["kind"]
        path = ev.get("path", "")
        marker = ev.get("marker", "")
        if kind == "RUST_TEST":
            name = pathlib.Path(path).name
            parts.append(f"`{name}` `{marker}`")
        elif kind == "RUST_LIB_CONTRACTS":
            parts.append("`thymeleaf/src` `#[cfg(test)]`")
        elif kind == "RUST_OBJECT_CONTRACT":
            parts.append(f"`{pathlib.Path(path)}` 对象合同")
        elif kind == "UPSTREAM_THTEST":
            parts.append("`thtest_upstream_plain_batch.rs`（语料运行器）")
        elif kind == "SEMANTIC_SCOPE":
            continue  # 由 UPSTREAM_THTEST 概括
        elif kind == "REPLACEMENT_TEST":
            parts.append(f"`{pathlib.Path(path).name}`（替代验证）")
        else:
            parts.append(f"{kind}:{path}")
    return "; ".join(parts) if parts else "—"


def render_class_table(methods: list[dict]) -> str:
    rows = []
    for m in methods:
        method = m["method"]
        runtime = ", ".join(m.get("runtime_cases", []))
        if len(runtime) > 60:
            runtime = runtime[:57] + "..."
        disp = m["disposition"]
        rows.append(f"| `{method}` | {disp} | {runtime} | {evidence_text(m.get('evidence', []))} |")
    return "\n".join(rows)


def main() -> None:
    inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
    entries = inventory["entries"]
    summary = inventory["summary"]
    upstream = inventory["upstream"]

    core = [e for e in entries if e["module"] == CORE_MODULE]
    integration = [e for e in entries if e["module"] != CORE_MODULE]

    core_disp = Counter(e["disposition"] for e in core)
    integ_disp = Counter(e["disposition"] for e in integration)

    by_class: dict[str, list[dict]] = defaultdict(list)
    for e in core:
        by_class[e["class"]].append(e)

    lines: list[str] = []
    lines.append("# Java 测试方法对照（thymeleaf-tests-core → thymeleaf-test）\n")
    lines.append(
        "本表把上游 `tests/thymeleaf-tests-core` 测试类的 **413 个测试方法**逐方法映射到 "
        "thymeleaf-test 中的 Rust 覆盖证据：每个方法一行，记录处置、运行时 case 与证据。\n"
    )
    lines.append("数据源：`docs/migration/baseline/source_parity_inventory.json`（875 个源码入口方法级处置，`MISSING=0`）。")
    lines.append(f"Java 基线：`{upstream['baseline']}`（{upstream['repository']}）。")
    lines.append("本表由 `scripts/generate_java_test_method_matrix.py` 生成，修改前请先改脚本。\n")

    lines.append("## 1. 汇总\n")
    lines.append("| 维度 | 数量 |")
    lines.append("|:---|---:|")
    lines.append(f"| 核心测试方法（tests/thymeleaf-tests-core） | {len(core)} |")
    lines.append(f"| 运行时 case（核心） | {summary['core_runtime_cases']} |")
    lines.append(f"| 集成测试方法（spring5/6/security） | {len(integration)} |")
    lines.append(f"| 运行时 case（集成） | {summary['integration_runtime_cases']} |")
    lines.append(f"| 未处置（missing） | {summary['missing']} |")
    lines.append("")
    lines.append("核心处置分布：")
    lines.append("")
    lines.append("| 处置 | 方法数 | 含义 |")
    lines.append("|:---|---:|:---|")
    lines.append("| SPLIT | %d | 方法级断言拆入对应 Rust 对象合同（`thymeleaf/src/**` `#[cfg(test)]`）+ 共享端到端语料 |" % core_disp["SPLIT"])
    lines.append("| MERGED | %d | Java `TestExecutor` 外壳合并到数据驱动语料运行器（`thtest_upstream_plain_batch.rs`），输入/期望/异常直读固定上游 .thtest |" % core_disp["MERGED"])
    lines.append("| MAPPED | %d | Java 测试由同名 Rust 合同测试 + 固定 Java Golden 逐记录验证（`thymeleaf-test/tests/*_java_parity.rs`） |" % core_disp["MAPPED"])
    lines.append("| NOT_APPLICABLE | %d | 基准工作负载类，正确性由语料与端到端测试承担 |" % core_disp["NOT_APPLICABLE"])
    lines.append("")
    lines.append("集成模块（Spring 方言）全部为 `POLICY_DIFFERENCE`：")
    lines.append("")
    lines.append("| 模块 | 方法数 | 处置 |")
    lines.append("|:---|---:|:---|")
    by_module: dict[str, int] = Counter(e["module"] for e in integration)
    for module in sorted(by_module):
        lines.append(f"| `{module}` | {by_module[module]} | POLICY_DIFFERENCE |")
    lines.append("")

    lines.append("## 2. 方法级映射（按测试类）\n")
    for cls in sorted(by_class, key=str.lower):
        methods = by_class[cls]
        disp_counts = Counter(m["disposition"] for m in methods)
        badge = ", ".join(f"{d}={disp_counts[d]}" for d in DISPOSITION_ORDER if disp_counts[d] > 0)
        short = cls.split(".")[-1]
        lines.append(f"### `{short}`（{len(methods)} 方法；{badge}）\n")
        lines.append("| 方法 | 处置 | 运行时 case | Rust 覆盖证据 |")
        lines.append("|---|---|---|---|")
        lines.append(render_class_table(sorted(methods, key=lambda m: (m["line"], m["method"]))))
        lines.append("")

    lines.append("## 3. 证据图例\n")
    lines.append("- **RUST_TEST**：同名 parity 测试文件 `thymeleaf-test/tests/*_java_parity.rs` 中的 marker 测试（1:1 复刻 Java 断言）。")
    lines.append("- **RUST_LIB_CONTRACTS**：`thymeleaf/src/**` 内对象合同的 `#[cfg(test)]` 单测。")
    lines.append("- **UPSTREAM_THTEST**：`thymeleaf-test/tests/thtest_upstream_plain_batch.rs` 数据驱动运行器（2608 例 .thtest，`THYMELEAF_SCOPE=semantic_all`）。")
    lines.append("- **REPLACEMENT_TEST**：替代验证测试（工作负载类）。")

    OUTPUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {OUTPUT} ({len(lines)} lines)")
    print(f"core={len(core)} dispositions={dict(core_disp)}")


if __name__ == "__main__":
    main()
