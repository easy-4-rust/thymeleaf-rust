#!/usr/bin/env python3
"""生成固定 Thymeleaf 上游 `.thtest` 的确定性迁移清单。"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from collections import Counter
from pathlib import Path


DIRECTIVE_PATTERN = re.compile(
    r"^%([A-Z_]+)(?:\[([^\]]+)\])?(?:[ \t]+(.*))?$"
)
ALLOWED_DIRECTIVES = {
    "CACHE",
    "CONTEXT",
    "EXACT_MATCH",
    "EXCEPTION",
    "EXCEPTION_MESSAGE_PATTERN",
    "EXTENDS",
    "FRAGMENT",
    "INPUT",
    "MESSAGES",
    "NAME",
    "OUTPUT",
    "TEMPLATE_MODE",
}
UPSTREAM_RESOURCE_ROOT = Path(
    "tests/thymeleaf-tests-core/src/test/resources"
)


def parse_arguments() -> argparse.Namespace:
    """解析命令行参数。"""
    parser = argparse.ArgumentParser()
    parser.add_argument("--upstream", required=True, type=Path)
    parser.add_argument("--baseline", required=True)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument(
        "--check",
        action="store_true",
        help="只校验现有清单与固定上游一致，不改写文件",
    )
    return parser.parse_args()


def sha256(content: bytes) -> str:
    """返回内容的 SHA-256 十六进制摘要。"""
    return hashlib.sha256(content).hexdigest()


def resolve_extends(test_file: Path, value: str) -> Path:
    """按上游测试资源的相对路径规则解析 `%EXTENDS`。"""
    candidate = test_file.parent / value.strip()
    return candidate.resolve()


def main() -> None:
    """扫描全部 `.thtest`，校验引用并写出稳定 JSON。"""
    arguments = parse_arguments()
    upstream = arguments.upstream.resolve()
    resource_root = upstream / UPSTREAM_RESOURCE_ROOT
    if not resource_root.is_dir():
        raise SystemExit(f"test resource root does not exist: {resource_root}")

    test_files = sorted(resource_root.rglob("*.thtest"))
    directive_counts: Counter[str] = Counter()
    mode_counts: Counter[str] = Counter()
    directory_counts: Counter[str] = Counter()
    referenced_common_files: set[Path] = set()
    violations: list[str] = []
    tests: list[dict[str, object]] = []
    support_files = 0

    for test_file in test_files:
        content = test_file.read_bytes()
        text = content.decode("utf-8")
        relative_path = test_file.relative_to(upstream).as_posix()
        resource_relative_path = test_file.relative_to(resource_root).as_posix()
        top_directory = resource_relative_path.split("/", 1)[0]
        directory_counts[top_directory] += 1
        directives: list[dict[str, object]] = []

        for line_number, line in enumerate(text.splitlines(), start=1):
            if not line.startswith("%"):
                continue
            match = DIRECTIVE_PATTERN.fullmatch(line)
            if match is None:
                violations.append(
                    f"{relative_path}:{line_number}: malformed directive {line!r}"
                )
                continue
            name, qualifier, value = match.groups()
            if name not in ALLOWED_DIRECTIVES:
                violations.append(
                    f"{relative_path}:{line_number}: unknown directive %{name}"
                )
            directive_counts[name] += 1
            directive = {
                "name": name,
                "line": line_number,
                "qualifier": qualifier,
                "value": value,
            }
            directives.append(directive)

            if name == "TEMPLATE_MODE" and value:
                mode_counts[value.strip()] += 1
            if name == "EXTENDS" and value:
                extended_file = resolve_extends(test_file, value)
                if not extended_file.is_file():
                    violations.append(
                        f"{relative_path}:{line_number}: missing EXTENDS target {value}"
                    )
                else:
                    referenced_common_files.add(extended_file)
                    directive["resolved_path"] = extended_file.relative_to(
                        upstream
                    ).as_posix()

        directive_names = {directive["name"] for directive in directives}
        is_support_file = test_file.name.endswith(".common.thtest")
        if is_support_file:
            support_files += 1
        if (
            not is_support_file
            and "INPUT" not in directive_names
            and "EXTENDS" not in directive_names
        ):
            violations.append(f"{relative_path}: has neither INPUT nor EXTENDS")
        if (
            not is_support_file
            and not ({"OUTPUT", "EXCEPTION", "EXTENDS"} & directive_names)
        ):
            violations.append(
                f"{relative_path}: has neither OUTPUT, EXCEPTION nor EXTENDS"
            )

        tests.append(
            {
                "path": relative_path,
                "resource_path": resource_relative_path,
                "sha256": sha256(content),
                "bytes": len(content),
                "lines": len(text.splitlines()),
                "directives": directives,
                "kind": "SUPPORT" if is_support_file else "EXECUTABLE",
                "verification_status": (
                    "NOT_APPLICABLE" if is_support_file else "PENDING"
                ),
            }
        )

    common_files = []
    for common_file in sorted(referenced_common_files):
        content = common_file.read_bytes()
        common_files.append(
            {
                "path": common_file.relative_to(upstream).as_posix(),
                "sha256": sha256(content),
                "bytes": len(content),
            }
        )

    inventory = {
        "schema_version": 1,
        "upstream": {
            "repository": "thymeleaf/thymeleaf",
            "baseline": arguments.baseline,
            "resource_root": UPSTREAM_RESOURCE_ROOT.as_posix(),
        },
        "summary": {
            "thtest_files": len(tests),
            "executable_tests": len(tests) - support_files,
            "support_thtest_files": support_files,
            "referenced_common_files": len(common_files),
            "directive_counts": dict(sorted(directive_counts.items())),
            "template_mode_counts": dict(sorted(mode_counts.items())),
            "top_directory_counts": dict(sorted(directory_counts.items())),
            "violations": len(violations),
            "pending": len(tests) - support_files,
        },
        "violations": violations,
        "common_files": common_files,
        "tests": tests,
    }

    rendered_inventory = (
        json.dumps(inventory, ensure_ascii=False, indent=2) + "\n"
    )
    if arguments.check:
        if not arguments.output.is_file():
            raise SystemExit(f"inventory does not exist: {arguments.output}")
        current_inventory = arguments.output.read_text(encoding="utf-8")
        if current_inventory != rendered_inventory:
            raise SystemExit(
                f"inventory is stale; regenerate {arguments.output}"
            )
    else:
        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        arguments.output.write_text(rendered_inventory, encoding="utf-8")
    print(
        f"{'checked' if arguments.check else 'wrote'} {arguments.output}: "
        f"thtest={len(tests)} common={len(common_files)} "
        f"violations={len(violations)}"
    )
    if violations:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
