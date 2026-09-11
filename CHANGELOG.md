# Changelog

## [Unreleased]

- 初始化工具箱仓库：Python 包骨架（`packages/python`）与验证工作流。
- 初始化 Rust 库骨架（`packages/rust`）。
- 初始化 TypeScript 包（`packages/typescript`）、Dart 包（`packages/dart`）、Go 包（`packages/go`）骨架。
- 发布工作流改为按语言一个 `release-{语言}.yml`（python / rust / typescript / dart / go）：标签与清单版本、CHANGELOG 校验，测试与打包预检通过后发布；原 `verify-*.yml` 的检查并入，不再单独保留。
