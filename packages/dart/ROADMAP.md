# 路线图

这一份是给人看的：**为什么做、收益是什么、怎么算做完**。细节逐条见 [TODO.md](TODO.md)，体检数据见 [STATUS.md](STATUS.md)。

**它与 [Rust 库的路线图](../rust/ROADMAP.md) 是同一套计划的两次落地**——段、判据、靶子一一对应。两份可以同时开工，也可以放一起看：哪边先动，另一边同一轮跟上。

## 为什么做

Dart 库是**工作台（studio）要对齐的正本**，与 Rust 库（cli 的正本）是同一份领域模型的两种语言实现。它现在的问题与 Rust 侧**同位**：

- **判据被切开**：模型在 `criterion.dart`，读法（`criterionOf` / `readCriterion`）在 `workflow.dart`
- **`workflow.dart` 405 行**：装下 Step + Workflow + 校验 + 定义核对 + 节名 + 占位展开
- **尾部三件不属工作流的东西**（`Finding` / `looksLikeSection` / `expandPlaceholders`）寄在那里——Rust 侧同样的三件寄在同一个位置

两侧不是"各自有点乱"，是**同一处乱了两遍**。

## 改进的收益（六条，每条都能核）

### 一、重构有尺子，算不算过不由人判

| | |
|---|---|
| 现在 | 改 `workflow.dart` 任何一处，「行为没变」要靠人论证 |
| 改完 | `sh scripts/contract.sh` 一条命令：**12 份向量**，Dart 与 Rust 各跑一遍，两侧一致才算过 |
| 怎么核 | 跑这一条。**它现在是绿的**（重构的下限是保持绿） |

### 二、改判据不再翻 405 行

| | |
|---|---|
| 现在 | 改判据要在 `criterion.dart`（模型）与 `workflow.dart`（读法）之间来回 |
| 改完 | 判据全在 `criterion/`（模型 + 读法 + 翻成「要跑什么」） |
| 怎么核 | `grep -rn "criterionOf\|readCriterion" lib/src/` 只命中 `criterion/` |

studio 是它的消费者——这条收益直接落在工作台那一侧的改动成本上。

### 三、镜像要按「事」对，不按行数对（这条是这次体检新发现的）

| | |
|---|---|
| 现在 | 两侧文件同名同职责（好），但边界一样糊；而且**同一条「≤250 行」在两种语言里不等价**——同样内容的 `task`，Rust 292 行、Dart 216 行 |
| 改完 | 镜像按「一个文件讲几件事」对：Dart 的 `task.dart` **虽然 216 行没越界，也要照 Rust 的切分拆开**，否则两侧不同构，「照抄」失效 |
| 怎么核 | 两侧文件清单一一对照（名字 + 职责），不看行数 |

这条修正了我们原来的判据：**行数只是辅助信号，主判据是"一个文件讲几件事"**。若只按行数，Dart 侧会把 `task` 留着不动，镜像就悄悄破了。

### 四、补上 `AGENTS.md`（Dart 包现在没有）

| | |
|---|---|
| 现在 | Rust 包有 `AGENTS.md`（结构、事实源、提交约定、测试命令），**Dart 包没有** |
| 改完 | 补齐同构的一份；两个包各有一处权威入口 |
| 怎么核 | `packages/dart/AGENTS.md` 在；结构与 `lib/` 一致 |

没有这份文件，新会话（或 AI）接手时没有权威入口——它只能翻代码猜边界，而"猜边界"正是这轮要消灭的东西。

### 五、版本对齐：两侧错开已经真实伤害端侧

| | |
|---|---|
| 现在 | Dart `0.1.0-beta.5`、Rust `0.1.0-beta.4`——于是 **studio 引 `beta.5`、cli 引 `beta.4`**，而对表要求两侧算出同一个结果 |
| 改完 | 定规矩：**向量一致时两侧发同一个版本号**，并把端侧一起升到同一版本 |
| 怎么核 | `pubspec.yaml`、桶文件里的 `version`、Rust `Cargo.toml`、studio 的 `pubspec.yaml`、cli 的 `Cargo.toml` 五处一致 |

### 六、把「五语言」的真话说清（与 Rust 侧同一件，省钱的）

| | |
|---|---|
| 现在 | 五个语言包，只有 Rust 与 Dart 有实现（空壳 12~30 行），却各有发布线、三个版本号 |
| 改完 | 二选一：**甲**（建议）发布线收到两个真有实现的语言，空壳标未实现并摘出；**乙** 补齐三个空壳按同构实现 + 让五侧都跑向量 |
| 怎么核 | 发布线条数 = 有实现的包数；README 包表标注状态 |

## 靶子（交付哪些模块）

```
packages/dart/lib/
├── quanttide_work.dart   出口（桶文件）
└── src/
    ├── criterion/   聚合：判据——模型 + 读法 + itemsOf
    ├── task/        聚合：任务——模型 + 流水 + 运行上下文
    ├── workflow/    聚合：工作流——Step + Workflow + 整体校验
    ├── outcome.dart 结果信封（82 行，保留单文件）
    └── executor.dart 执行者常量（20 行，保留单文件——只有常量，立目录无收益）
```

与 Rust 侧 `packages/rust/src/` 的切分**同名同职责**。

## 六段怎么走

| 段 | 做什么 | 怎么算完 |
|---|---|---|
| 一 · 判据归位 | `criterionOf` / `readCriterion` 搬进 `criterion/` | 读法只命中 `criterion/`；尺子仍绿 |
| 二 · 拆 workflow | 405 → 每件 ≤250；尾部三件各归其主 | 无一件 >250；`workflow/` 只讲工作流 |
| 三 · 拆 task | 216 行但**照拆**（镜像按事对） | 两侧 `task/` 同名同职责；`done-voting` 向量绿 |
| 四 · 与 Rust 同批 | 两侧同一轮改完，不许单侧长期领先 | 两侧文件清单对得上；尺子绿 |
| 五 · 文档与约定 | 补 `AGENTS.md`、立约定 | 结构块与实际一致 |
| 六 · 版本与尺子 | 两侧版本对齐（含端侧）、`contract.sh` 进发布工作流 | 五处版本一致；两个发布工作流里有 `contract.sh` |

**每段做完都跑**（两条，缺一不算完）：

```bash
cd packages/dart && dart analyze lib/ test/ && dart test
cd ../.. && sh scripts/contract.sh
```

## 现在在哪

**段一~段五已完成**（2026-09-12，pi 执行 + Hermes 复核）：

- 判据归位：读法（`criterionOf` / `readCriterion`）进 `criterion/`，与模型、`itemsOf` 同处
- `workflow.dart` 405 行 → `workflow/{model,read,check}`；`task.dart` 216 行 → `task/{model,journal,context}`；**最长文件 169 行**（全部 ≤250）
- 三件寄居物各归其主：`Finding` + `looksLikeSection` → `workflow/check`，`expandPlaceholders` → `task/model`
- 与 Rust 侧**九个分件逐个对位**；桶文件仍是一处导入（`import 'package:quanttide_work/quanttide_work.dart'`）
- 新增两个扩展名：`TaskJournal`、`WorkflowCheck`（Dart 不能跨文件写实现，用扩展承接分件；调用写法不变）
- 补建了 `AGENTS.md`（原先没有）、约定立在 `lib/CONVENTIONS.md`（与 Rust 侧那份互为对照）
- 门禁：`dart analyze` 无问题、`dart test` 全绿；`sh scripts/contract.sh` **12 份向量、两侧一致**
- CHANGELOG 已记 `[Unreleased]`：桶文件导入方式不变，**端侧 studio 无感**

**剩下两件**：

- **段六 6.1 版本对齐——已对齐到 `0.1.0-beta.6`**：两侧清单与版本常量改齐（取两侧的下一号，规则是「向量一致时两侧发同一个号」），`scripts/contract.sh` 加了一条「两侧清单版本一致」的判据，错号即红。**包还没发布**——按发布纪律等创始人放行
- 段六 6.2 已做：`sh scripts/contract.sh` 已挂进两条发布工作流的门禁（两侧一起跑）

**后续（issue #2）**：按规范三轴补上「场所」——新增 `workspace/`（模型 + 定义核对 + 落点 + 流水判定，用扩展承接分件），`RunContext` 退出工具箱（位置不进模型，目录基准当参数传；桶文件不再导出 `RunContext`）。详见 `CHANGELOG.md`「Unreleased」与 [issue #2](https://github.com/quanttide/quanttide-work-toolkit/issues/2)。

## 边界外（越界但相关，需要你拍板）

1. **三个空壳语言怎么办**（收益第六条）：摘掉发布线，还是补齐实现
2. **工具箱的契约是裁剪版**：它只装"不因平台而变的核心逻辑"，没有 IO，**不需要端侧那套「服务 / 适配」分类**；它的规矩是「一个模型一个目录 + 一个文件一件事 + 一处一写 + 说法不进库」——这条要写进约定，免得有人拿端侧的整套规矩来量它
3. **两份 ROADMAP 最好一起执行**：单侧先做完，收益（照抄）反而是负的——所以建议同一轮把 Rust 与 Dart 一次推完
4. **校验态类型化（`Raw` / `Validated`）**：让「未校验」与「已校验」在类型上分开。Dart 没有 Rust 那样的零成本 phantom，做了要么只有 Rust 做（镜像破裂），要么 Dart 用两层类包装——除非出现「拿未校验值当已校验用」的真实事故，先不做（[issue #1](https://github.com/quanttide/quanttide-work-toolkit/issues/1)）
5. **`exists` 回调扩展成能核内容**：`check` 的 `exists` 只能答「在不在」。「判据怎么核」该先由规范定，不宜先扩接口再补规范（同上 issue）
6. **`RuleItem` 换成保留类型的结构**：现在是 `kind` + 位置 `args`，端侧要按顺序重新解释。等真需要更多语义时再动，避免提前抽象（同上 issue）
7. **路径改用路径类型**：判据里的路径现在是裸 `String`（`path` / `absent` / `file`）。审下来的结论是**不立**——加了类型不删一行代码，`expanded` / `check` 还得在字符串上认占位（描述字段也带占位，逃不掉）；真正出过事的是**错误出处**（次序写反害得一条分支走不到），那一处已经用 `Source` + `Position` 类型化了。路径的拼接与规范化归端侧——「平台管位置」。等出现「把落点当定义里的路径用」这类真实事故再回头立（同上 issue）
