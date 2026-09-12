# Changelog

## [Unreleased]

### Added

- `Criterion.expanded(f)`：判据各字段里的占位一次换掉（Rust 侧 beta.1 已有，Dart 侧下一版带上）

## [0.1.0-beta.1] - 2026-09-12

### Changed

- 判据改密封类型：`path` / `absent` / `file`+`contains` / `run` 各一个子类型，`agent` / `human` 各一个；定义的字段形状不变
- `Workflow` / `Step` 改成不可变值：`Workflow.fromValue` 校验并建模型、`Workflow.of` 读已校验的定义、`toMap` 写回字段形状；`Step.criteria` 是 `List<Criterion>`
- `itemsOf` 吃 `Iterable<Criterion>`；`prompts` 的判据入参同步

### Added

- `schema.dart`：字段名、取值、判据种类与 `textOf` / `unknownFields` 单列；消掉 `definition.dart` 与 `tasklog.dart` 重复的 `textOf`（公开面不再需要 `hide`）

## [0.1.0-alpha.10] - 2026-09-12

### Fixed

- 发布改用与 quanttide-data-toolkit 同一个 action（k-paxian/dart-package-publisher），不再手写凭证落点


## [0.1.0-alpha.8] - 2026-09-12

### Fixed

- 凭证改走 pub 自己的 token 仓（以前把整份 credentials.json 写进 pub-cache，新版 pub 不读它）


## [0.1.0-alpha.7] - 2026-09-12

### Fixed

- 凭证到位后重发


## [0.1.0-alpha.6] - 2026-09-12

### Fixed

- 发布步不让 pub 挂等交互：验凭证形状、关 stdin、加 300 秒上限（alpha.5 那一步静默挂了十几分钟）


## [0.1.0-alpha.5] - 2026-09-12

### Fixed

- Dart 发布改用 `PUBDEV_CREDENTIAL_JSON` 落到 pub 缓存的 `credentials.json`（alpha.4 那一步取不到 token）


## [0.1.0-alpha.4] - 2026-09-12

### Fixed

- pub.dev 的 token 认几个常见名字（alpha.3 的 Dart 发布卡在 `PUB_DEV_TOKEN` 取不到）


## [0.1.0-alpha.3] - 2026-09-12

### Fixed

- 版本改动与 `Cargo.lock` 同步（alpha.2 的发布工作流卡在 `--locked`）


## [0.1.0-alpha.2] - 2026-09-12

### Fixed

- Rust 侧按 `cargo fmt` 格式化（alpha.1 的发布工作流卡在这一步）


## [0.1.0-alpha.1] - 2026-09-12

### Added

- 抽出命令行与 studio 两侧意义相同的领域模型：信封、工作流定义（字段表、校验、视图、核对）、机械判据、任务流水与「走过」的算法、给智能体的两段话
- 契约测试：`contract/` 下的用例向量两侧共用，`scripts/contract.sh` 跑 rust 与 dart 各一遍


- 初始化 Dart 包骨架：`quanttide_work` 库入口（领域名与版本常量）与测试。
