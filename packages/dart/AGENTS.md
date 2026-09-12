# AGENTS.md - quanttide-work Dart 库

## 项目结构

```
packages/dart/
├── AGENTS.md                # 本文件
├── CHANGELOG.md             # 版本变更记录
├── pubspec.yaml             # Dart 包配置
├── README.md                # 项目说明
├── lib/
│   ├── CONVENTIONS.md       # 代码约定（横切约束）
│   ├── quanttide_work.dart  # 出口（桶文件）：七个模型对外
│   └── src/
│       ├── executor.dart    # 执行者常量
│       ├── outcome.dart     # 结果信封
│       ├── artifact/        # 产物：名字 + 规格（验收判据）
│       ├── criterion/       # 判据：模型 + 读法 + 翻成要跑什么
│       ├── task/            # 任务：模型 + 流水
│       ├── workflow/        # 工作流：模型 + 语法校验
│       └── workspace/       # 工作区：模型 + 定义核对 + 落点 + 流水判定
└── test/
    ├── contract_test.dart   # 契约：跑共用向量
    └── package_test.dart    # 包测试
```

与 [Rust 侧](../rust/src) 的切分同名同职责：`artifact/` / `criterion/` / `task/` / `workflow/` / `workspace/`
目录内的分件按「事」一一对应（模型 / 读法 / 流水 / 定义核对 / 落点）。

## 事实源

领域模型与规格以 `quanttide-work/docs/specification` 为准；本库只做表达，不定义领域。

## 提交约定

Conventional Commits（`feat:` / `fix:` / `docs:` / `chore:`）；破坏性变更标 `!` 并在 body 说明迁移方式。

## 测试

```bash
dart analyze lib/ test/ && dart test
```

契约（与 Rust 侧共用向量，两侧结论必须一致）：

```bash
sh ../../scripts/contract.sh
```
