#!/usr/bin/env bash
# scripts/publish-precheck.sh -- 发布前预检（参考 wxrust publish-order.sh）
#
# 验证 thymeleaf 单 crate 包清单可生成、metadata 完整、依赖无未发布 crates，
# 对应 wxrust 的 Layer 0/3 干跑。本仓只发 thymeleaf 一个 crate（thymeleaf-test
# publish=false 是测试包），单 crate 无内部依赖 → 完整 cargo publish --dry-run。
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

export PATH="$HOME/.cargo/bin:$PATH"

# 预检清单
echo "=== 1) workspace 元数据 ==="
cargo metadata --format-version 1 --no-deps > /dev/null
echo "  metadata OK"

echo "=== 2) 依赖树无未发布 workspace 内部 crate ==="
CARGO_TREE=$(cargo tree -p thymeleaf --depth 1 2>&1)
echo "$CARGO_TREE"
if echo "$CARGO_TREE" | grep -qE "thymeleaf-test \(.*path"; then
  echo "::error::thymeleaf 依赖 thymeleaf-test（path 形式）；thymeleaf-test publish=false，发布失败"
  exit 1
fi
echo "  无内部 path 依赖"

echo "=== 3) thymeleaf 单 crate 完整干跑 ==="
if cargo publish -p thymeleaf --dry-run --allow-dirty 2>&1 | tail -20 | tee /tmp/publish-dryrun.log | grep -qE "aborting upload due to dry run"; then
  echo "  PASS（dry-run 完成）"
else
  echo "::error::cargo publish --dry-run 异常，详见 /tmp/publish-dryrun.log"
  exit 1
fi

echo "=== 4) CHANGELOG 存在未发布条目检查 ==="
NEXT=$(cargo metadata --format-version 1 --no-deps | python3 -c "import json,sys; m=json.load(sys.stdin); print([p['version'] for p in m['packages'] if p['name']=='thymeleaf'][0])")
echo "  待发布版本: $NEXT"
if ! grep -q "^## \[$NEXT\]" CHANGELOG.md; then
  echo "::error::CHANGELOG.md 缺少 [${NEXT}] 条目（无版本时也算缺）"
  exit 1
fi
echo "  CHANGELOG 条目存在"

echo "=== 预检通过 ==="
