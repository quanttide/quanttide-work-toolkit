# Changelog

## [Unreleased]

- 仓库许可由 CC BY 4.0 改为 Apache-2.0：`LICENSE`（根与 Dart 包）与 Rust / TypeScript / Python 清单同步；已发布的旧版本仍按原许可。
- 补上「产物」（`artifact`）聚合，Rust 与 Dart 两侧同构：产物 = 名字 + 规格（验收判据），**不把报告 / 日志这类具体种类写进模型**；落点改由 `Workspace::place(任务, 产物)` 按名字算（原先 `artifact(task, kind)` 里的 `report` / `journal` / `log` 特例下线）
- 落点不再拼目录（只给相对工作区根的路径），占位展开收敛成 `Workspace::expanded`；不认识的占位算定义错误；`check` 去掉空转的 `base`
- 发布工作流补上 GitHub Release：`release-rust.yml` / `release-dart.yml` 在注册表发布成功后建 Release（自动生成变更说明，alpha / beta / rc 标预发布）；补齐历史 tag 缺的 Releases（rust `beta.1~4`、dart `beta.1~5`），alpha 系列一并改标预发布
- 新增工作区（`workspace`）聚合，Rust 与 Dart 两侧同构：按规范三轴补上「场所」，`RunContext` 退出工具箱（位置不进模型）
- 初始化工具箱仓库：Python 包骨架（`packages/python`）与验证工作流。
- 初始化 Rust 库骨架（`packages/rust`）。
- 初始化 TypeScript 包（`packages/typescript`）、Dart 包（`packages/dart`）、Go 包（`packages/go`）骨架。
- 发布工作流改为按语言一个 `release-{语言}.yml`（python / rust / typescript / dart / go）：标签与清单版本、CHANGELOG 校验，测试与打包预检通过后发布；原 `verify-*.yml` 的检查并入，不再单独保留。
