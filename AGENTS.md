# AGENTS.md - quanttide-work-toolkit

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
├── tests/contract/       # 契约向量（11 份，各语言共用一份正本）
├── AGENTS.md
├── CHANGELOG.md
├── CONTRIBUTING.md
└── README.md
```
