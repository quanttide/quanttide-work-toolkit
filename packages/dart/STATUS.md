# 状态：quanttide-work Dart 库 对照契约

体检日 2026-09-12。这份只管 `packages/dart/`；与 Rust 侧的镜像、五语言的事见 [ROADMAP.md](ROADMAP.md)。

契约出处：结构契约与阶段条款在 [契约原型](../../../../../quanttide-code/data/insight/code-agent/contract.md)（code 域洞察）；本库的分界规矩在 `lib/quanttide_work.dart` 头注释。

## 规模

| 项 | 数 |
|---|---|
| `lib/src/` 文件 | 5（930 行）+ 桶文件 `lib/quanttide_work.dart` 24 行——**平铺**，无目录 |
| 各文件 | `workflow.dart` **405**、`criterion.dart` 207、`task.dart` 216、`outcome.dart` 82、`executor.dart` 20 |
| `test/` | `contract_test.dart`（跑共用向量）、`package_test.dart` |
| 契约向量 | `../../tests/contract/*.json` **11 份**，Dart 与 Rust 两侧共跑 |

## 尺子（重构的验收判据，现在就是绿的）

```bash
sh scripts/contract.sh     # 11 份向量，Dart 与 Rust 各跑一遍，两侧结论必须一样
```

当前基线：**两侧一致**（Dart「All tests passed!」；Rust「契约：11 份向量，两侧一致」）。重构只要让它保持绿，就算「行为不变」。

## 结构契约

| 条款 | 现状 | 判定 |
|---|---|---|
| 一个模型一个文件 | 五个模型各一文件 | ✓ |
| 单文件 ≤250 行 | `workflow.dart` 405 | ✗ 越界（`task.dart` 216 未越界，但见下） |
| 同一件事只写一处 | **判据被切开**：模型在 `criterion.dart`（`RuleKind`、各判据类、`itemsOf`），读法在 `workflow.dart`（`criterionOf` 第 301 行、`readCriterion` 第 320 行） | ✗ 待归位 |
| 一个文件一件事 | `workflow.dart` 尾部同样寄着三件不属工作流的事：`Finding`（定义核对结果）、`looksLikeSection`（节名）、`expandPlaceholders`（占位展开） | ✗ 待归位 |
| 不可变模型 | `Workflow.of` / `with*` 返回新值 | ✓ |

**与 Rust 侧是同位的问题**：Rust 的 `criterion_of`/`read_criterion` 也在 `workflow.rs`、尾部三件同名同处。两侧不是"各自有点乱"，是**同一处乱了两遍**。

## 与 Rust 侧的镜像

| 条款 | 现状 | 判定 |
|---|---|---|
| 两侧同名同职责 | `criterion` / `executor` / `outcome` / `task` / `workflow` + 桶文件（`quanttide_work.dart` / `lib.rs`） | ✓ 镜像成立 |
| 两侧同一处越界 | Dart `workflow.dart` 405 / Rust `workflow.rs` 516；判据读法都在 `workflow.*` | ✗ 同病 |
| 版本同步 | Dart `0.1.0-beta.5`、Rust `0.1.0-beta.4` | ✗ 错开 |
| 文档对称 | **Dart 包没有 `AGENTS.md`**（Rust 包有） | ✗ 缺 |

**一条要记下的观察**：同一条「≤250 行」在两种语言里**不等价**——同样内容的 `task`，Rust 292 行、Dart 216 行。所以行数只能是**辅助信号**，主判据该是「一个文件讲几件事」；镜像也不该按行数对，要按**事**对（见 ROADMAP 收益第二条）。

## 边界外（越界但相关，要拍板）

- 五语言里只有 Rust 与 Dart 有实现，go / python / typescript 是空壳（12~30 行），却各有发布线——见 ROADMAP「边界外」
- Dart 侧的消费者是 studio（`quanttide_work: ^0.1.0-beta.5`），Rust 侧的消费者是 cli（`quanttide-work 0.1.0-beta.4`）：**两侧版本错开，端侧就跟着错开**
