#!/usr/bin/env python3
"""为缺失"对应 Java"来源注释的公共对象/方法生成诚实注释草稿。

- 从 docs/migration/对象级对照表.md 解析 Rust 文件 -> Java 类映射
- 从 docs/migration/baseline/java_api_inventory.json 索引 Java 类方法名
- 从 Rust 源文件读取公共项种类（fn/struct/enum/trait/union）
- 方法名 snake_case -> camelCase 与 Java 方法名核对：
    * 命中 -> "对应 Java: <Class>#<method>()。"
    * 未命中 -> "对应 Java 语义：<Class> 的 <method> 行为（Rust 侧辅助路径）。"
- 类型项 -> "对应 Java: <Class>。"（Rust 类型名与 Java 类名一致时）
dry-run 模式只输出统计与样本，不写文件。
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

KIND_RE = re.compile(
    r"(?:fn|struct|enum|trait|union)\s+([A-Za-z_][A-Za-z0-9_]*)"
)


def parse_object_table(path: Path) -> dict[str, dict]:
    """rust_file -> {"java": qname, "rust_type": name}"""
    mapping: dict[str, dict] = {}
    current_pkg = ""
    for line in path.read_text(encoding="utf-8").splitlines():
        m = re.match(r"###\s+`([^`]+)`", line)
        if m:
            current_pkg = m.group(1).rstrip(".")
            continue
        if not line.startswith("|") or "Java 主对象" in line:
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 6 or not cells[0].isdigit():
            continue
        java_name = cells[1].strip("`")
        rust_file = cells[4].strip("`")
        rust_type = cells[5].strip("`")
        mapping[rust_file] = {
            "java": f"{current_pkg}.{java_name}",
            "rust_type": rust_type,
        }
    return mapping


def load_java_methods(path: Path) -> tuple[dict[str, set[str]], set[str]]:
    inv = json.loads(path.read_text(encoding="utf-8"))
    result: dict[str, set[str]] = {}
    global_methods: set[str] = set()
    for obj in inv["objects"]:
        methods = {m["name"] for m in obj.get("methods", [])}
        result[obj["name"]] = methods
        global_methods.update(methods)
    return result, global_methods


def snake_to_camel(name: str) -> str:
    parts = name.split("_")
    return parts[0] + "".join(p.capitalize() for p in parts[1:])


def java_simple_name(qname: str) -> str:
    return qname.rsplit(".", 1)[-1]


def classify(path: Path, item: str) -> tuple[str, str] | None:
    """按审计报告的名称在全文件定位定义，返回 (kind, name)。

    以审计名称为准（杜绝邻近函数误配），行号仅用于读取源文件。
    """
    source = path.read_text(encoding="utf-8")
    for keyword in ("fn", "struct", "enum", "trait", "union"):
        m = re.search(rf"\b{keyword}\s+{re.escape(item)}\b", source)
        if m:
            return (keyword, item)
    return None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--findings", required=True)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--rust-root", default=str(ROOT / "thymeleaf"))
    args = ap.parse_args()

    table = parse_object_table(ROOT / "docs/migration/对象级对照表.md")
    java_methods, all_java_methods = load_java_methods(
        ROOT / "docs/migration/baseline/java_api_inventory.json"
    )
    print(
        f"mapping entries: {len(table)}, java classes: {len(java_methods)}",
        file=sys.stderr,
    )

    per_file: dict[str, list[tuple[int, str]]] = {}
    for line in Path(args.findings).read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        path, rest = line.split(":", 1)
        lineno, item = rest.split(" ", 1)
        per_file.setdefault(path, []).append((int(lineno), item.strip()))

    derivable, auxiliary, unclassed = 0, 0, 0
    planned: dict[str, list[tuple[int, str, str]]] = {}
    samples: list[str] = []
    for file, items in sorted(per_file.items()):
        rust_file = f"thymeleaf/{file}"
        entry = table.get(rust_file) or table.get(file)
        java_qname = entry["java"] if entry else None
        java_simple = java_simple_name(java_qname) if java_qname else None
        for lineno, item in items:
            kind_name = classify(Path(args.rust_root) / file, item)
            if kind_name is None:
                unclassed += 1
                comment = "/// 对应 Java 语义：公共项（具体对应见实现注释）。"
            else:
                kind, name = kind_name
                if kind in ("struct", "enum", "trait", "union"):
                    if java_simple and name == java_simple:
                        comment = f"/// 对应 Java: `{java_simple}`。"
                        derivable += 1
                    elif java_simple:
                        comment = (
                            f"/// 对应 Java 语义：`{java_simple}` 的 Rust 侧类型 "
                            f"`{name}`。"
                        )
                        auxiliary += 1
                    else:
                        comment = (
                            "/// 对应 Java 语义：Rust 侧内部类型"
                            "（Java 无直接对应对象）。"
                        )
                        unclassed += 1
                else:  # fn
                    camel = snake_to_camel(name)
                    if (
                        java_simple
                        and java_simple in java_methods
                        and camel in java_methods[java_simple]
                    ):
                        comment = f"/// 对应 Java: `{java_simple}#{camel}()`。"
                        derivable += 1
                    elif camel in all_java_methods:
                        comment = (
                            f"/// 对应 Java 语义：Java 接口/超类方法 `{camel}()` 的"
                            f" Rust 移植（`{java_simple}` 继承路径）。"
                        )
                        derivable += 1
                    elif java_simple:
                        comment = (
                            f"/// 对应 Java 语义：`{java_simple}` 的 `{name}` 行为"
                            "（Rust 侧辅助/私有路径）。"
                        )
                        auxiliary += 1
                    else:
                        comment = (
                            "/// 对应 Java 语义：Rust 侧辅助函数"
                            "（Java 无直接对应）。"
                        )
                        unclassed += 1
            planned.setdefault(file, []).append((lineno, item, comment))
            if len(samples) < 15:
                samples.append(f"{file}:{lineno} {item}\n    -> {comment}")

    print(f"derivable={derivable} auxiliary={auxiliary} unclassed={unclassed}")
    print("=== samples ===")
    for s in samples:
        print(s)

    if args.apply:
        applied = 0
        for file, entries in planned.items():
            path = Path(args.rust_root) / file
            src_lines = path.read_text(encoding="utf-8").splitlines()
            for lineno, _item, comment in sorted(entries, reverse=True):
                indent_match = re.match(r"(\s*)", src_lines[lineno - 1])
                indent = indent_match.group(1)
                src_lines.insert(lineno - 1, f"{indent}{comment}")
            path.write_text("\n".join(src_lines) + "\n", encoding="utf-8")
            applied += len(entries)
        print(f"applied {applied} comments")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
