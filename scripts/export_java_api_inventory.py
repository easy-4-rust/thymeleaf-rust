#!/usr/bin/env python3
"""从固定 CodeGraph 数据库导出 Thymeleaf Core 对象和方法级基线。"""

from __future__ import annotations

import argparse
import json
import re
import sqlite3
import subprocess
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


CORE_PREFIX = "lib/thymeleaf/src/main/java/org/thymeleaf/"


def parse_arguments() -> argparse.Namespace:
    """解析命令行参数。"""
    parser = argparse.ArgumentParser()
    parser.add_argument("--java-root", required=True, type=Path)
    parser.add_argument("--baseline", required=True)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def split_parameters(signature: str | None) -> list[str]:
    """按泛型和数组嵌套层级拆分 Java 参数。"""
    if not signature or "(" not in signature or ")" not in signature:
        return []
    body = signature[signature.find("(") + 1 : signature.rfind(")")].strip()
    if not body:
        return []

    parameters: list[str] = []
    current: list[str] = []
    depths = {"<": 0, "(": 0, "[": 0, "{": 0}
    closing = {">": "<", ")": "(", "]": "[", "}": "{"}
    for character in body:
        if character in depths:
            depths[character] += 1
        elif character in closing:
            opener = closing[character]
            depths[opener] = max(0, depths[opener] - 1)
        if character == "," and all(depth == 0 for depth in depths.values()):
            parameters.append("".join(current).strip())
            current.clear()
        else:
            current.append(character)
    parameters.append("".join(current).strip())
    return [parameter for parameter in parameters if parameter]


def parameter_record(position: int, declaration: str) -> dict[str, Any]:
    """从 CodeGraph 原始参数声明提取名称和类型文本。"""
    without_annotations = re.sub(r"@\w+(?:\([^)]*\))?\s*", "", declaration)
    without_modifiers = re.sub(r"\bfinal\s+", "", without_annotations).strip()
    match = re.search(r"([A-Za-z_$][A-Za-z0-9_$]*)\s*(?:\[\])?$", without_modifiers)
    name = match.group(1) if match else None
    type_text = (
        without_modifiers[: match.start(1)].strip() if match else without_modifiers
    )
    return {
        "position": position,
        "name": name,
        "type": type_text,
        "declaration": declaration,
        "varargs": "..." in declaration,
    }


def query_rows(connection: sqlite3.Connection, sql: str) -> list[sqlite3.Row]:
    """执行只读查询并返回行。"""
    return list(connection.execute(sql, (f"{CORE_PREFIX}%",)))


def main() -> None:
    """验证基线并导出完整清单。"""
    arguments = parse_arguments()
    java_root = arguments.java_root.resolve()
    output = arguments.output.resolve()

    actual_sha = subprocess.run(
        ["git", "-C", str(java_root), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    repository_url = subprocess.run(
        ["git", "-C", str(java_root), "remote", "get-url", "origin"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if actual_sha != arguments.baseline:
        raise SystemExit(
            f"expected Java baseline {arguments.baseline}, got {actual_sha}"
        )

    database = java_root / ".codegraph" / "codegraph.db"
    if not database.is_file():
        raise SystemExit(f"CodeGraph database not found: {database}")

    connection = sqlite3.connect(f"file:{database}?immutable=1", uri=True)
    connection.row_factory = sqlite3.Row

    type_rows = query_rows(
        connection,
        """
        SELECT kind, name, qualified_name, file_path, start_line, end_line,
               visibility, is_static, is_abstract, signature, return_type,
               decorators, type_parameters
        FROM nodes
        WHERE language = 'java'
          AND file_path LIKE ?
          AND kind IN ('class', 'interface', 'enum')
        ORDER BY file_path, start_line, start_column
        """,
    )
    method_rows = query_rows(
        connection,
        """
        SELECT kind, name, qualified_name, file_path, start_line, end_line,
               visibility, is_static, is_abstract, signature, return_type,
               decorators, type_parameters
        FROM nodes
        WHERE language = 'java'
          AND file_path LIKE ?
          AND kind = 'method'
        ORDER BY file_path, start_line, start_column
        """,
    )

    types_by_file: dict[str, list[sqlite3.Row]] = defaultdict(list)
    methods_by_file: dict[str, list[sqlite3.Row]] = defaultdict(list)
    for row in type_rows:
        types_by_file[row["file_path"]].append(row)
    for row in method_rows:
        methods_by_file[row["file_path"]].append(row)

    objects: list[dict[str, Any]] = []
    method_visibility = Counter()
    parameter_count = 0
    overloads = Counter()

    for file_path in sorted(types_by_file):
        primary_name = Path(file_path).stem
        type_candidates = types_by_file[file_path]
        primary = next(
            (row for row in type_candidates if row["name"] == primary_name),
            type_candidates[0],
        )
        nested = [row["name"] for row in type_candidates if row is not primary]
        methods: list[dict[str, Any]] = []

        for row in methods_by_file.get(file_path, []):
            declarations = split_parameters(row["signature"])
            parameters = [
                parameter_record(position, declaration)
                for position, declaration in enumerate(declarations)
            ]
            parameter_count += len(parameters)
            visibility = row["visibility"] or "package"
            method_visibility[visibility] += 1
            overloads[(row["qualified_name"], row["name"])] += 1
            methods.append(
                {
                    "name": row["name"],
                    "qualified_name": row["qualified_name"],
                    "signature": row["signature"],
                    "return_type": row["return_type"],
                    "visibility": visibility,
                    "static": bool(row["is_static"]),
                    "abstract": bool(row["is_abstract"]),
                    "start_line": row["start_line"],
                    "end_line": row["end_line"],
                    "parameters": parameters,
                }
            )

        objects.append(
            {
                "name": primary["name"],
                "qualified_name": primary["qualified_name"],
                "kind": primary["kind"],
                "visibility": primary["visibility"] or "package",
                "abstract": bool(primary["is_abstract"]),
                "source_file": file_path,
                "start_line": primary["start_line"],
                "end_line": primary["end_line"],
                "nested_types": nested,
                "methods": methods,
            }
        )

    overloaded_groups = sum(1 for count in overloads.values() if count > 1)
    payload = {
        "schema_version": 1,
        "java": {
            "repository": repository_url,
            "baseline": actual_sha,
            "scope": CORE_PREFIX,
            "codegraph_database": ".codegraph/codegraph.db",
        },
        "summary": {
            "primary_objects": len(objects),
            "nested_types": sum(len(item["nested_types"]) for item in objects),
            "methods": len(method_rows),
            "parameters": parameter_count,
            "overloaded_method_groups": overloaded_groups,
            "methods_by_visibility": dict(sorted(method_visibility.items())),
        },
        "objects": objects,
    }

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    print(
        f"objects={payload['summary']['primary_objects']} "
        f"methods={payload['summary']['methods']} "
        f"parameters={payload['summary']['parameters']} "
        f"output={output}"
    )


if __name__ == "__main__":
    main()
