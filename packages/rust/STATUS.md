# 状态：quanttide-work Rust 库 对照契约

体检日 2026-09-12。这份只管 `packages/rust/`；Dart 镜像与五语言的事见 [ROADMAP.md](ROADMAP.md) 的「边界外」。

> **本表是整理前的体检**。段一~段五已完成（判据归位、两个大文件拆成目录），整理后的现状见 [ROADMAP.md](ROADMAP.md) 的「现在在哪」。

契约出处：结构契约与阶段条款在 [契约原型](../../../../../quanttide-code/data/insight/code-agent/contract.md)（code 域洞察）；本库自己的分界规矩在 `src/lib.rs` 头注释与 `../AGENTS.md`。

## 规模

| 项 | 数 |
|---|---|
| `src/` 文件 | 6（1 190 行）——**平铺**，无目录 |
| 各文件 | `criterion.rs` 237、`executor.rs` 19、`lib.rs` 23、`outcome.rs` 103、`task.rs` **292**、`workflow.rs` **516** |
| `tests/` | `contract.rs`（跑共用向量）、`package.rs` |
| 契约向量 | `../../tests/contract/*.json` **12 份**，Rust 与 Dart 两侧共跑 |

## 尺子（重构的验收判据，现在就是绿的）

```bash
sh scripts/contract.sh     # 12 份向量，Rust 与 Dart 各跑一遍，两侧结论必须一样
```

当前基线：**两侧一致**（Rust「契约：12 份向量，两侧一致」；Dart「All tests passed!」）。重构只要让它保持绿，就算「行为不变」——不需要人逐行看。

## 结构契约

| 条款 | 现状 | 判定 |
|---|---|---|
| 一个模型一个文件（`lib.rs` 明文） | 五个模型各一文件：`workflow` `task` `criterion` `executor` `outcome` | ✓ |
| 单文件 ≤250 行 | `workflow.rs` 516、`task.rs` 292 | ✗ 越界 |
| 同一件事只写一处 | **判据被切开**：模型在 `criterion.rs`（`RuleKind`、`Criterion`、`RuleItem`、`items_of`），而读法在 `workflow.rs`（`criterion_of`、`read_criterion`） | ✗ 待归位 |
| 一个文件一件事 | `workflow.rs` 尾部还装着三件不是工作流的事：`Finding`（定义核对的结果）、`looks_like_section`（节名）、`expand_placeholders`（占位展开） | ✗ 待归位 |
| 收进来的每一样，都能在规范里找到出处 | 各文件头都引了出处（如 `docs/specification/process/workflow.md`·语法） | ✓ |
| 说法与文案不进库 | 拼句、退出码、路径怎么显示、提示词都在端侧（`cli/help.rs`、`prompts.rs`、`paths.rs`、`cli/emit.rs`） | ✓ |
| 不可变模型 | 各模型 `with_*` 返回新值，改动由调用方落盘 | ✓ |

## 与 Dart 侧的镜像

| 条款 | 现状 | 判定 |
|---|---|---|
| 两侧同名同职责 | 文件名一一对上：`criterion` `executor` `outcome` `task` `workflow` + 桶文件（`lib.rs` / `quanttide_work.dart`） | ✓ 镜像成立 |
| 两侧都在长 | Rust `workflow.rs` 516 / Dart `workflow.dart` 405；Rust `task.rs` 292 / Dart `task.dart` 216 | ✗ 同一处越界 |
| 版本同步 | Rust `0.1.0-beta.4`、Dart `0.1.0-beta.5` | ✗ 错开 |

**镜像是一条硬约束**：只重构一侧，另一侧就不同构了，「照着另一侧抄」这件最省力的动作立刻失效。所以本库的整理必须与 Dart 侧同批走。

## 文档

- `packages/rust/AGENTS.md` 的「项目结构」块**过时**：只写了 `src/lib.rs` 一个文件，实际已有六个
- 没有 STATUS / TODO / ROADMAP 三件套（本文是头一份）
- `CHANGELOG.md` 与 `Cargo.toml` 版本一致（`0.1.0-beta.4`）✓

## 边界外（越界但相关，要拍板）

`packages/` 下五个语言包，**只有 Rust 与 Dart 有实现**：

| 包 | 实现 | 行数 |
|---|---|---|
| rust | 五个模型齐 | 1 190 |
| dart | 五个模型齐 | ~954 + 测试 |
| go | 只有 `Domain` / `Version` 两个常量 | 29 |
| python | 只有 `__version__` | 12 |
| typescript | 只有 `DOMAIN` / `VERSION` | 30 |

三个空壳的注释都写着「领域模型**就绪后**在此按子领域分文件」——**但目前 `.github/workflows/` 里有五条发布线**（`release-{语言}.yml`），空包也有发布线的位置。见 ROADMAP「边界外」。
