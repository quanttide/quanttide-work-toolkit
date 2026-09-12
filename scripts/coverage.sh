#!/usr/bin/env bash
# 覆盖率门禁：两侧（Rust / Dart）的行覆盖都要 ≥ 90%。
#
# 为什么是 90：这是纯逻辑库（无 IO、无网络、无界面），测试成本极低；
# 80% 只是通用下限。留 10% 给确实不该测的（Default 派生、Display/Error 样板等）。
#
# 用法（在工具箱根目录）：sh scripts/coverage.sh
# 退出码：0 = 两侧都达标；非 0 = 有侧不达标（两侧都会照跑，数字都打出来）
root="$(cd "$(dirname "$0")/.." && pwd)"
floor=90
failed=0

echo "— Rust 侧（cargo llvm-cov）"
if command -v cargo-llvm-cov >/dev/null 2>&1; then
  # CARGO_INCREMENTAL=0：llvm-cov 与增量编译打架（第一次会报收集不到目标文件）
  (cd "$root/packages/rust" && CARGO_INCREMENTAL=0 cargo llvm-cov --locked --fail-under-lines "$floor" --summary-only) \
    || failed=1
else
  echo "  缺 cargo-llvm-cov：cargo install cargo-llvm-cov" >&2
  failed=1
fi

echo
echo "— Dart 侧（dart test --coverage）"
if command -v dart >/dev/null 2>&1; then
  if (cd "$root/packages/dart" && rm -rf coverage \
      && dart test --coverage=coverage >/dev/null \
      && dart pub global run coverage:format_coverage --lcov --in=coverage --out=coverage/lcov.info --report-on=lib); then
    python3 - "$root/packages/dart/coverage/lcov.info" "$floor" <<'PY' || failed=1
import re, sys

path, floor = sys.argv[1], int(sys.argv[2])
agg = {}
for block in open(path, encoding="utf-8").read().split("end_of_record"):
    f = re.search(r"SF:(.+)", block)
    lf = re.search(r"LF:(\d+)", block)
    lh = re.search(r"LH:(\d+)", block)
    if f and lf and lh:
        here, total = int(lh.group(1)), int(lf.group(1))
        agg[f.group(1).strip().split("packages/dart/")[-1]] = (here, total)

here = sum(h for h, _ in agg.values())
total = sum(t for _, t in agg.values())
pct = here / total * 100 if total else 100.0
for rel, (h, t) in sorted(agg.items(), key=lambda kv: kv[1][0] / kv[1][1] if kv[1][1] else 1):
    print(f"  {(h / t * 100 if t else 100):6.2f}%  {h:4d}/{t:<4d}  {rel}")
print(f"  合计：{here}/{total} = {pct:.2f}%（门槛 {floor}%）")
sys.exit(0 if pct >= floor else 1)
PY
  else
    echo "  Dart 侧跑不起来（test 或 format_coverage 失败）" >&2
    failed=1
  fi
else
  echo "  缺 dart（把 flutter/bin 加进 PATH）" >&2
  failed=1
fi

echo
if [ "$failed" -eq 0 ]; then
  echo "两侧覆盖率都达标（≥ ${floor}%）"
else
  echo "有侧没达标：见上面的分文件数字" >&2
fi
exit "$failed"
