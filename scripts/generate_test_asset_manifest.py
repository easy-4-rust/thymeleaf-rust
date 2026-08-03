#!/usr/bin/env python3
"""生成 acceptance.rs 编译期所需的 source-test-parity.json（TEST_CASE + TEST_ASSET）。

依据 rust-java-migration-testing 技能：SOURCE_PARITY 必须有 TEST_CASE 与
TEST_ASSET 两个部分，且每个资产按 SHA-256 校验字节一致。

- test_case.entries：2,608 个可执行 .thtest（排除 *.common.thtest 支持文件），
  字段 source_relative_path（"tests/" + assets 相对路径）与 asset_sha256
- test_asset.entries：2,686 个资产（2,609 thtest 镜像 + 77 golden fixtures），
  字段 target_path（仓库根相对）与 sha256
- upstream：固定基线 10f9dd2eb... 与版本 3.1.5.RELEASE（上游 pom.xml）

清单由 *.json 忽略规则排除入库，CI 在编译前调用本脚本生成。
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add"
UPSTREAM_VERSION = "3.1.5.RELEASE"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--output", required=True, type=Path)
    args = ap.parse_args()

    assets_root = ROOT / "thymeleaf-test" / "assets" / "thymeleaf-tests"
    golden_root = ROOT / "thymeleaf" / "tests" / "fixtures"

    cases: list[dict] = []
    assets: list[dict] = []
    for path in sorted(assets_root.rglob("*.thtest")):
        rel = path.relative_to(assets_root).as_posix()
        assets.append(
            {
                "target_path": f"thymeleaf-test/assets/thymeleaf-tests/{rel}",
                "sha256": sha256(path),
            }
        )
        if not path.name.endswith(".common.thtest"):
            cases.append(
                {
                    "source_relative_path": f"tests/{rel}",
                    "asset_sha256": sha256(path),
                }
            )
    for path in sorted(golden_root.glob("*.txt")):
        assets.append(
            {
                "target_path": f"thymeleaf/tests/fixtures/{path.name}",
                "sha256": sha256(path),
            }
        )

    manifest = {
        "schema_version": 1,
        "upstream": {"baseline": BASELINE, "version": UPSTREAM_VERSION},
        "test_case": {"entries": cases},
        "test_asset": {"entries": assets},
    }

    if len(cases) != 2_608:
        print(
            f"error: expected 2,608 executable test cases, got {len(cases)}",
            file=sys.stderr,
        )
        return 2
    if len(assets) != 2_686:
        print(
            f"error: expected 2,686 assets (2,609 thtest + 77 golden), got {len(assets)}",
            file=sys.stderr,
        )
        return 2

    args.output.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    print(
        f"wrote {args.output}: cases={len(cases)} assets={len(assets)}",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
