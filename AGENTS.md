# AGENTS.md - quanttide-work-toolkit

## 项目结构

```
quanttide-work-toolkit/
├── .github/workflows/    # 各语言发布工作流（release-{语言}.yml）
├── packages/
│   ├── python/           # Python 包（quanttide-work）
│   ├── rust/             # Rust 库（quanttide-work）
│   ├── typescript/       # TypeScript 包（quanttide-work）
│   ├── dart/             # Dart 包（quanttide_work）
│   └── go/               # Go 包（quanttide_work）
├── AGENTS.md
├── CHANGELOG.md
└── README.md
```

## 发布约定

- 一个语言包一个工作流：`release-{语言}.yml`，标签 `{语言}/vX.Y.Z`
- 工作流先跑质量门禁（标签 ↔ 清单版本 ↔ CHANGELOG 三方校验、测试、打包预检），再发布
- 发版前把包内 CHANGELOG 的 `[Unreleased]` 落成 `## [X.Y.Z]`，并同步清单版本，否则工作流在校验步失败

各语言检查命令（与工作流一致）：

```bash
cd packages/python     && uv sync --locked && uv run pytest && uv build
cd packages/rust       && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --locked
cd packages/typescript && npm ci && npm run typecheck && npm test
cd packages/dart       && dart analyze lib/ test/ && dart test
cd packages/go         && go vet ./... && go test ./...
```

## 提交约定

- 提交信息用中文祈使句，前缀 `feat:`、`fix:`、`docs:`、`chore:`、`refactor:`
- 提交后默认推送到远端
- 语言包内改动在语言包范围提交，仓库级文档改动单独提交

## 评判指标

简洁、生动：能少则少，用真实例子说话。
