#!/usr/bin/env bash
# scripts/check_block_on.sh -- 任务卡 token 检查（参考 wxrust scripts/check_block_on.sh）
#
# CI 门禁：若 CHANGELOG 的任务卡（DoD）存在 "BLOCKED" 标记，CI 失败。
# 用于保护发布纪律——未完成的关键迁移项不允许发版。
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# 仅检查 [Unreleased] 区段
SECTION=$(awk '
  /^## \[Unreleased\]$/ {found=1; next}
  found && /^## \[/ && !/Unreleased/ {exit}
  found {print}
' CHANGELOG.md)

if echo "$SECTION" | grep -qiE "BLOCKED|未完成"; then
  echo "::error::CHANGELOG 的 [Unreleased] 区段含 'BLOCKED' 或 '未完成' 标记："
  echo "$SECTION" | grep -iE "BLOCKED|未完成" | head -3
  exit 1
fi
echo "BLOCK_ON 检查通过（[Unreleased] 无未完成项）"
