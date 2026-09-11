#!/bin/sh
# 契约：同一批用例向量（tests/contract/*.json），Rust 与 Dart 两侧各跑一遍，结论必须一样。
#
# 用法：sh scripts/contract.sh
# 结论只有一种：两侧都过。任一侧红，契约就破了。

set -e
here=$(cd "$(dirname "$0")/.." && pwd)

echo "向量：$(ls "$here"/tests/contract/*.json | wc -l) 份"

echo "— Rust 侧"
(cd "$here/packages/rust" && cargo test --locked --test contract -- --nocapture)

echo "— Dart 侧"
(cd "$here/packages/dart" && dart test test/contract_test.dart)
