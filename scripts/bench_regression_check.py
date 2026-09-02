#!/usr/bin/env python3
"""Criterion 基准回归检查（thymeleaf-rust 适配版）。

对比本次运行与上次 CI 缓存的基准，任一基准回归 >10% 即失败。
来源：参考 easydoc-rust scripts/bench_regression_check.py（cargo bench + Criterion
estimates.json），自适应本仓库三基准（render_simple_variable / render_each_100 /
render_full_document）。

用法（CI bench-regression 任务）：
    cargo bench -p thymeleaf --bench render_baseline -- --save-baseline cur
    # cargo bench 用 criterion 自带 --save-baseline 写入 target/criterion/
    python3 scripts/bench_regression_check.py

流程：
1. 遍历 target/criterion/render_baseline/<id>/cur/estimates.json 取 median；
2. 与同 id/prev/estimates.json 对比，回归 >10% → 失败；
3. 无 prev（首次基准建立）→ 通过并把 cur 提升为 prev；
4. 通过后把 cur 提升为 prev，供下一次 CI 比较（同 ubuntu-latest 硬件类）。
"""
from __future__ import annotations

import glob
import json
import os
import shutil
import sys

REGRESSION_LIMIT = 1.10  # 允许 10% 以内的波动（与 easydoc 一致）


def median_estimate(estimates_path: str) -> float:
    """读取 criterion estimates.json 的 median 点估计（单位与基准一致：纳秒）。"""
    with open(estimates_path, encoding="utf-8") as f:
        data = json.load(f)
    return float(data["median"]["point_estimate"])


def main() -> int:
    # 仅检查 render_baseline 三个基准（与 benches/render_baseline.rs 一一对应）
    pattern = "target/criterion/*/*/cur/estimates.json"
    entries = sorted(glob.glob(pattern))
    if not entries:
        print(f"未找到本次基准结果（{pattern}）")
        return 1

    failures = []
    for cur_path in entries:
        bench_dir = os.path.dirname(os.path.dirname(cur_path))
        bench_id = os.path.basename(bench_dir)
        prev_path = os.path.join(bench_dir, "prev", "estimates.json")

        cur = median_estimate(cur_path)
        if not os.path.exists(prev_path):
            print(f"[基准建立] {bench_id}: cur={cur:.3e} ns（无历史基准）")
            # 首次运行：把 cur 提升为 prev，供下次比较
            prev_dir = os.path.join(bench_dir, "prev")
            if os.path.isdir(prev_dir):
                shutil.rmtree(prev_dir)
            shutil.copytree(os.path.join(bench_dir, "cur"), prev_dir)
            continue

        prev = median_estimate(prev_path)
        ratio = cur / prev
        delta = (ratio - 1.0) * 100.0
        status = "OK" if ratio <= REGRESSION_LIMIT else "REGRESSION"
        print(f"[{status:>10}] {bench_id}: prev={prev:.3e} cur={cur:.3e} "
              f"Δ={delta:+.2f}%")

        if ratio > REGRESSION_LIMIT:
            failures.append(f"{bench_id}: Δ={delta:+.2f}% 超出 10% 回归阈值")

        # 通过后把 cur 提升为 prev（保证下次 CI 能 diff）
        prev_dir = os.path.join(bench_dir, "prev")
        if os.path.isdir(prev_dir):
            shutil.rmtree(prev_dir)
        shutil.copytree(os.path.join(bench_dir, "cur"), prev_dir)

    if failures:
        print("\n基准回归门禁失败：")
        for f in failures:
            print(f"  - {f}")
        return 1

    print("\n基准回归门禁通过（≤10%）。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
