#!/usr/bin/env bash
# scripts/publish.sh -- 发布 thymeleaf crate 到 crates.io（参考 wxrust publish-013.sh）
#
# 重试最多 3 次（含 429 限流与"已存在版本"幂等成功）。
# 前置：CARGO_REGISTRY_TOKEN 在仓库 Settings -> Secrets -> Actions 配置。
set -u

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

export PATH="$HOME/.cargo/bin:$PATH"

# 前置：token 未配置立即失败（避免空 token 重试 21x3 次的假绿）
if [ -z "${CARGO_REGISTRY_TOKEN:-}" ]; then
  echo "::error::CARGO_REGISTRY_TOKEN secret 未配置（仓库 Settings -> Secrets -> Actions）" >&2
  exit 1
fi

# 单 crate 发布（thymeleaf-test publish=false 不发布）
CRATES=(
  thymeleaf
)

ok=0
for crate in "${CRATES[@]}"; do
  for attempt in 1 2 3; do
    out=$(cargo publish -p "$crate" --allow-dirty 2>&1) || true

    # 明确成功标志：cargo 同步输出 "Published ... at registry"
    if echo "$out" | grep -qE "^[[:space:]]*Published ${crate} v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)? at registry"; then
      echo "Successfully published $crate"
      ok=1
      break
    fi
    # 幂等重跑：版本已上传视为成功
    if echo "$out" | grep -qiE "already (uploaded|exists)"; then
      echo "$crate already published (version exists), skipping"
      ok=1
      break
    fi
    # 限流退避
    if echo "$out" | grep -qiE "429|too many requests|rate limit"; then
      echo "[WAIT] $crate attempt $attempt -- rate limit, sleeping 60s"
      sleep 60
      continue
    fi
    # 其它失败：dump + 重试
    echo "$out" | tail -5
    echo "[RETRY] $crate attempt $attempt failed, sleeping 15s before retry..."
    sleep 15
  done
  if [ "$ok" -ne 1 ]; then
    echo "::error::$crate 3 次重试后仍发布失败" >&2
    exit 1
  fi
done

echo "ALL DONE"
