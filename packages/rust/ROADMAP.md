# 路线图

这一份是给人看的：**为什么做、收益是什么、怎么算做完**。细节逐条见 [TODO.md](TODO.md)，体检数据见 [STATUS.md](STATUS.md)。

## 为什么做

Rust 库是**所有端侧要对齐的正本**（cli 与 studio 都引它），所以它自己的结构乱了，代价会同时落在两边。现在它有三处硬伤：

- **判据被切开**：模型在 `criterion.rs`，读法（YAML → 判据）在 `workflow.rs`——改判据要在两个文件之间来回
- **`workflow.rs` 516 行**、`task.rs` 292 行：一个装下「Step + Workflow + 校验 + 定义核对 + 节名 + 占位展开」，另一个装下「任务 + 流水 + 运行上下文」
- **尾部三件不属工作流的东西**（`Finding` 定义核对结果、`looks_like_section` 节名、`expand_placeholders` 占位展开）寄在 `workflow.rs` 里

而且它与 Dart 侧是**镜像**（同名同职责）——这是工具箱最有价值的纪律，也是本次重构的硬约束：**只拆一侧，镜像就破了**。

## 改进的收益（五条，每条都能核）

### 一、重构有尺子，算不算过不由人判

| | |
|---|---|
| 现在 | 改 `workflow.rs` 任何一处，两侧只有"整体测试"，没有定向判据；「行为没变」要人自己论证 |
| 改完 | `sh scripts/contract.sh` 一条命令：**12 份向量**，Rust 与 Dart 各跑一遍，两侧一致才算过 |
| 怎么核 | 跑这一条命令。**它现在是绿的**（重构后的最低要求就是保持绿） |

这是最实在的收益：重构的风险不再靠"我检查过了"承担，而是靠一条可复现的命令承担。

### 二、改判据不再翻 516 行

| | |
|---|---|
| 现在 | 改判据的取法，要在 `criterion.rs`（模型）与 `workflow.rs`（读法）之间来回；`workflow.rs` 516 行，每次改动都得整篇读进去 |
| 改完 | 判据全在 `criterion/`（模型 + 读法 + 翻成「要跑什么」），工作流只讲工作流 |
| 怎么核 | `grep -rn "fn criterion_of\|fn read_criterion" src/` 只命中 `criterion/` |

这条直接对上「模块化是 AI 的弱项」：**结构定了，改动范围才定**——AI（和人）改判据时不必把工作流读一遍。

### 三、镜像保住"照抄"这件最省力的事

| | |
|---|---|
| 现在 | 两侧同名同职责（好），但两侧的 `workflow`（516 / 405 行）与 `task`（292 / 216 行）都长大了，边界一样糊 |
| 改完 | 两侧同构的新切分：新增一个模型、改一个判断，就是**在两侧同名文件里做同一件事** |
| 怎么核 | 两侧文件清单一一对照，名字与职责都对得上 |

**风险要说清**：只重构 Rust 一侧，这条收益立刻变负——镜像破了，「照抄另一侧」失效，以后每个改动都要在两种结构里各找一遍。所以段一至段三必须与 Dart 侧同批（TODO 段四）。

### 四、端侧的依赖从"跨层借"变成"按模型引"

| | |
|---|---|
| 现在 | 端侧引用跨层，例如 `quanttide_work::workflow::{Step, text_of}`——工具函数从工作流文件里被借出去，说明边界不清 |
| 改完 | 端侧只引它真正需要的模型文件；谁依赖谁一眼可见 |
| 怎么核 | 把 cli / studio 里所有 `quanttide_work::` 的引用按模型归类，不该出现的跨层引用为零 |

### 五、把「五语言」的真话说清（这条是省钱的）

| | |
|---|---|
| 现在 | 对外说五个语言包；实际**只有 Rust（1 190 行）与 Dart（~954 行）有实现**，go / python / typescript 是只有版本常量的空壳（12~30 行）——却**各有发布线**（`release-{语言}.yml` 五条），版本号已经三个值（rust `beta.4`、dart `beta.5`、其余 `0.1.0`） |
| 改完 | 二选一：**甲**（建议）发布线收到真有实现的两个语言，空壳标为未实现并摘出发布线——省三条线、三个版本号的维护，对外说法不再虚；**乙** 认可五语言目标，把三个空壳按同构补齐，代价是三种语言实现 + 契约向量要让五侧都跑 |
| 怎么核 | 发布线条数 = 有实现的包数；README 包表标注实现状态；两侧版本号一致 |

顺带一条：Rust 与 Dart 版本错开（`beta.4` / `beta.5`）已经**真实伤害了端侧**——cli 引的是 `beta.4`、studio 引的是 `beta.5`，而对表要求两边算出同一个结果。定规矩：**向量一致时两侧发同一个版本号**（TODO 6.1）。

## 靶子（交付哪些模块）

```
packages/rust/src/
├── criterion/    聚合：判据——模型（RuleKind / Criterion / RuleItem）+ 读法 + items_of
├── task/         聚合：任务——模型 + 流水（JournalEvent 与「走过」的判定）+ 运行上下文
├── workflow/     聚合：工作流——Step + Workflow + 整体语法校验
├── outcome.rs    结果信封（103 行，保留单文件）
├── executor.rs   执行者常量（19 行，保留单文件——只有常量，立目录无收益）
└── lib.rs        出口：五个模型对外
```

Dart 侧 `lib/src/` 按同样的切分同构重排。

## 六段怎么走

| 段 | 做什么 | 怎么算完 |
|---|---|---|
| 一 · 判据归位 | 读法从 `workflow.rs` 搬进 `criterion/` | 读法只命中 `criterion/`；尺子仍绿 |
| 二 · 拆 workflow | 516 → 每件 ≤250；尾部三件各归其主 | 无一件 >250；`workflow/` 只讲工作流 |
| 三 · 拆 task | 292 → 模型 / 流水 / 运行上下文 | 无一件 >250；`done-voting` 向量仍绿 |
| 四 · 镜像同步 | Dart 侧同构重排 | 两侧文件同名同职责；`dart test` 绿 |
| 五 · 约定与文档 | 立约定文件、`AGENTS.md` 结构块改对 | 结构块与新目录一致 |
| 六 · 版本与尺子 | 两侧版本对齐、`contract.sh` 进两侧发布工作流 | 两包版本一致；发布工作流里有 `contract.sh` |

**每段做完都跑**（两条，缺一不算完）：

```bash
cd packages/rust && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --locked
cd ../.. && sh scripts/contract.sh
```

## 现在在哪

**段一~段五已完成**（2026-09-12，pi 执行 + Hermes 复核）：

- 判据归位：读法（`criterion_of` / `read_criterion`）进 `criterion/`，与模型、`items_of` 同处
- `workflow.rs` 516 行 → `workflow/{model,read,check}`；`task.rs` 292 行 → `task/{model,journal,context}`；**最长文件 195 行**（全部 ≤250）
- 三件寄居物各归其主：`Finding` + `looks_like_section` → `workflow/check`，`expand_placeholders` → `task/model`
- 与 Dart 侧**九个分件逐个对位**（`criterion/{model,items,read}`、`task/{model,journal,context}`、`workflow/{model,read,check}`）
- 约定立在 `src/CONVENTIONS.md`；`AGENTS.md` 结构块已改对
- 门禁：`fmt` / `clippy` / `test` 全绿；`sh scripts/contract.sh` **12 份向量、两侧一致**
- CHANGELOG 已记 `[Unreleased]`：三条公共路径迁移属**破坏性变更**（下游 cli 与 studio 经查均未用到，无需改动）

**剩下两件**：

- **段六 6.1 版本对齐——等你拍板**：Rust `0.1.0-beta.4` 与 Dart `0.1.0-beta.5` 要合成一个号（端侧 cli / studio 也跟着引同一号）。**版本号由你定，我不擅自改**
- 段六 6.2 已做：`sh scripts/contract.sh` 已挂进 `release-rust.yml` 与 `release-dart.yml` 的门禁（两侧一起跑，一侧绿不算过）

**一处与原文的偏差（已改齐）**：约定文件落在 `src/CONVENTIONS.md`（与 cli 同路子），TODO 原先写的是包根

## 边界外（越界但相关，需要你拍板）

1. **三个空壳语言怎么办**（收益第五条）：摘掉发布线，还是补齐实现？
2. **`packages/rust/AGENTS.md` 的结构块**过时（只写 `src/lib.rs`），段五顺手改
3. **工具箱是否也要「服务 / 适配」两类**：本库只装"不因平台而变的核心逻辑"，没有 IO，**不需要那两类**——所以它的契约是裁剪版：**一个模型一个目录 + 单文件 ≤250 + 一处一写 + 说法不进库**。这条要写进约定，免得以后有人拿端侧的规矩来量它
4. **校验态类型化（`Raw` / `Validated`）**：让「未校验」与「已校验」在类型上分开。Dart 没有 Rust 那样的零成本 phantom，做了要么只有 Rust 做（镜像破裂），要么 Dart 用两层类包装——除非出现「拿未校验值当已校验用」的真实事故，先不做（[issue #1](https://github.com/quanttide/quanttide-work-toolkit/issues/1)）
5. **`exists` 回调扩展成能核内容**：`check` 的 `exists: Fn(&str) -> bool` 只能答「在不在」。「判据怎么核」该先由规范定，不宜先扩接口再补规范（同上 issue）
6. **`RuleItem` 换成保留类型的结构**：现在是 `kind` + 位置 `args`，端侧要按顺序重新解释。等真需要更多语义时再动，避免提前抽象（同上 issue）
