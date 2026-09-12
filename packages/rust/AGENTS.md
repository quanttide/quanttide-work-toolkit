# AGENTS.md - quanttide-work Rust 库

## 项目结构

```
packages/rust/
├── AGENTS.md          # 本文件
├── CHANGELOG.md       # 版本变更记录
├── CONTRIBUTING.md    # 贡献指南
├── Cargo.toml         # Rust 包配置
├── README.md          # 项目说明
├── src/
│   ├── lib.rs         # 出口：六个模型对外
│   ├── executor.rs    # 执行者常量
│   ├── outcome.rs     # 结果信封
│   ├── criterion/     # 判据：模型 + 读法 + 翻成要跑什么
│   ├── task/          # 任务：模型 + 流水
│   ├── workflow/      # 工作流：模型 + 语法校验
│   └── workspace/     # 工作区：模型 + 定义核对 + 落点 + 流水判定
└── tests/
    ├── contract.rs    # 契约：跑共用向量
    └── package.rs     # 集成测试
```

## 事实源

领域模型与规格以 `quanttide-work/docs/specification` 为准；本库只做表达，不定义领域。
