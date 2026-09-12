# quanttide-work-toolkit

量潮知识工作工具箱 — 知识工作领域的共享库与工具集（独立仓库，挂载于 quanttide-work/packages）。

## 概述

承载知识工作领域的纯逻辑与共享能力，按语言拆分为独立包：

| 包 | 语言 | 包名 | 说明 |
|---|---|---|---|
| [`packages/python`](packages/python) | Python | `quanttide-work` | 知识工作 Python 包 |
| [`packages/rust`](packages/rust) | Rust | `quanttide-work` | 知识工作 Rust 库 |
| [`packages/typescript`](packages/typescript) | TypeScript | `quanttide-work` | 知识工作 TypeScript 包 |
| [`packages/dart`](packages/dart) | Dart | `quanttide_work` | 知识工作 Dart 包 |
| [`packages/go`](packages/go) | Go | `quanttide_work` | 知识工作 Go 包 |

新增语言包时在 `packages/{语言}/` 下独立发布，互不依赖。

**实现状态**：目前只有 **Rust 与 Dart 有实现**（七个模型齐全，两侧同一批契约向量对齐）；Go / Python / TypeScript 是空壳，只有领域名与版本常量——各包内 `STATUS.md` / `ROADMAP.md` 记着现状。

## 项目结构

```
quanttide-work-toolkit/
├── .github/workflows/    # 各语言发布工作流（release-{语言}.yml）
├── docs/user-guide/      # 给平台开发者：怎么把工具箱接进自己的端（按模型分篇）
├── packages/
│   ├── python/           # Python 包（quanttide-work）
│   ├── rust/             # Rust 库（quanttide-work）
│   ├── typescript/       # TypeScript 包（quanttide-work）
│   ├── dart/             # Dart 包（quanttide_work）
│   └── go/               # Go 包（quanttide_work）
├── scripts/              # contract.sh 两侧向量一致 / coverage.sh 覆盖率 / doc-tests.sh 文档示例
├── tests/contract/       # 契约向量（13 份，各语言共用一份正本）
├── AGENTS.md
├── CHANGELOG.md
├── CONTRIBUTING.md
└── README.md
```

## 文档

| 文档 | 给谁看 |
|---|---|
| [`docs/user-guide/`](docs/user-guide/index.md) | **平台开发者**：怎么把工具箱接进自己的端——各语言的接法统一在一篇里讲 |

## 发布自动化

一个语言包一个发布工作流，推 `{语言}/vX.Y.Z` 标签触发：先校验标签、清单版本与 CHANGELOG 三者一致，跑测试与打包预检，再发布到对应仓库，最后在仓库里补一条 GitHub Release（自动生成变更说明，`-alpha.` / `-beta.` / `-rc.` 标预发布）。

| 工作流 | 标签 | 发布目标 |
|---|---|---|
| [`release-python.yml`](.github/workflows/release-python.yml) | `python/vX.Y.Z` | PyPI |
| [`release-rust.yml`](.github/workflows/release-rust.yml) | `rust/vX.Y.Z` | crates.io |
| [`release-typescript.yml`](.github/workflows/release-typescript.yml) | `typescript/vX.Y.Z` | npm |
| [`release-dart.yml`](.github/workflows/release-dart.yml) | `dart/vX.Y.Z` | pub.dev |
| [`release-go.yml`](.github/workflows/release-go.yml) | `go/vX.Y.Z` | 生成模块别名标签 `packages/go/vX.Y.Z` |

发版前把包内 CHANGELOG 的 `[Unreleased]` 落成 `## [X.Y.Z]`，并同步清单版本（`pyproject.toml` / `Cargo.toml` / `package.json` / `pubspec.yaml`）。也可在 Actions 页面手动派发，指定标签重跑。

工作流只管「发布前验证 + 发布」，不在 push/PR 上单独跑 CI；需要 PR 门禁时，给对应工作流加 `pull_request` 路径触发即可。

## 许可

[CC BY 4.0](LICENSE)
