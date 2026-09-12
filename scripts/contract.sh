#!/bin/sh
# 契约：同一批用例向量（tests/contract/*.json），Rust 与 Dart 两侧各跑一遍，结论必须一样。
#
# 用法：sh scripts/contract.sh
# 结论只有一种：两侧都过。任一侧红，契约就破了。

set -e
here=$(cd "$(dirname "$0")/.." && pwd)

echo "— 两侧版本一致"
rust_version=$(grep -m1 '^version' "$here/packages/rust/Cargo.toml" | cut -d'"' -f2)
dart_version=$(awk '/^version:/{print $2; exit}' "$here/packages/dart/pubspec.yaml")
if [ "$rust_version" != "$dart_version" ]; then
  echo "两侧版本不一致：Rust $rust_version / Dart $dart_version" >&2
  exit 1
fi
echo "两侧版本：$rust_version"

echo "向量：$(ls "$here"/tests/contract/*.json | wc -l) 份"

echo "— Rust 侧"
(cd "$here/packages/rust" && cargo test --locked --test contract -- --nocapture)

echo "— Dart 侧"
(cd "$here/packages/dart" && dart test test/contract_test.dart)
