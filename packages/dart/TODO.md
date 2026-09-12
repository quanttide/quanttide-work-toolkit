# TODO：把 Dart 库收到契约下

逐条列出要改的每一个细节。依据是 [STATUS.md](STATUS.md)；路线与收益见 [ROADMAP.md](ROADMAP.md)。

**红线：与 Rust 侧同批。** 两侧是镜像，Dart 独自先改完 = 镜像破裂 = 「照着另一侧抄」失效。每一段做完，必须与 Rust 侧同名同职责地一起绿。

**每段做完的通用判据**（缺一不算完）：

```bash
cd packages/dart
dart analyze lib/ test/ && dart test
cd ../.. && sh scripts/contract.sh      # 12 份向量，Dart 与 Rust 两侧必须仍一致
```

---

## 段一 · 判据归位（一件事只写一处）

**1.1** 把判据的读法从 `workflow.dart` 搬进 `criterion/`

- 搬：`criterionOf`（第 301 行）、`readCriterion`（第 320 行）——YAML → `Criterion` 的翻译与校验
- 留：`workflow.dart` 只管工作流定义（`Step`、`Workflow`、整体语法校验）
- 判据：`criterionOf` / `readCriterion` 的**定义**只在 `criterion/`（工作流里仍会**调用**它们——`Step` 怎么读判据是工作流的事，不算违规）

**1.2** `criterion` 立目录，与 `itemsOf`（判据翻成「要跑什么」）同处

- 判据：`criterion/` 里同时有模型、读法、`RuleItem` / `itemsOf`；端侧 `import` 一处即可用全

## 段二 · 拆 `workflow.dart`（405 → 每件 ≤250）

**2.1** 按「事」拆进 `workflow/`：定义的模型（`Step` / `Workflow`）与整体校验（`DefinitionError` / `fromValue` 的校验段）分件，桶文件只留类型与出口

**2.2** 尾部三件各归其主（与 Rust 侧同落点）：

- `looksLikeSection`（节名）→ 与「报告两节」同源处（`section-names` 向量管的事）
- `expandPlaceholders`（占位展开）→ 与落点/占位同源处（`expand-placeholders` / `artifact-location` 向量管的事）
- `Finding`（定义核对的结果）→ 归定义核对（`workflow --check` 的模型侧）

- 判据：`workflow/` 每件只讲工作流；无一件 >250

## 段三 · 拆 `task.dart`（216 行，未越界但**照拆**）

**3.1** `task/` 三分：任务模型（`Task`）、流水（`JournalEvent` 与「走过」的判定）、运行上下文（`RunContext`）——与 Rust 侧同名同职责

- 判据：两侧 `task/` 的文件名与职责一一对应；`done-voting` 向量仍绿
- **为什么 216 行也要拆**：镜像按「事」对，不按行数对。Rust 侧 `task` 要拆，Dart 侧留着单文件就不同构，照抄失效

## 段四 · 与 Rust 侧同批（这一段是纪律，不是活）

**4.1** 段一至段三与 Rust 侧**同一批**改完：一侧动、另一侧同一轮跟上，不允许长期单侧领先

- 判据：两侧文件清单一一对照（名字 + 职责）；`sh scripts/contract.sh` 绿

## 段五 · 文档与约定

**5.1** 补 `packages/dart/AGENTS.md`——**现在没有**（Rust 包有）。内容对齐 Rust 那份：项目结构、事实源（领域模型以 `quanttide-work/docs/specification` 为准，本库只做表达）、提交约定、测试命令

**5.2** 立约定（`packages/dart/CONVENTIONS.md`，或与 Rust 共用一份写在工具箱 `AGENTS.md`）：一个模型一个目录、**一个文件只讲一件事**（行数 ≤250 只作辅助信号，两语言不等价）、一件事只写一处、说法与文案不进库

- 判据：结构块与新目录一致；约定每条都能在新结构里指出落点

## 段六 · 版本与尺子（跨语言，与 Rust 侧同批）

**6.1** 版本对齐：Dart `0.1.0-beta.5` 与 Rust `0.1.0-beta.4` 错开——定规矩并执行：**向量一致时两侧发同一个版本号**；随后把端侧（studio 的 `pubspec.yaml`、cli 的 `Cargo.toml`）也升到同一版本

- 判据：`pubspec.yaml`、`lib/quanttide_work.dart` 里的 `version`、Rust `Cargo.toml` 三处一致；端侧引同一个版本

**6.2** `scripts/contract.sh` 进 CI：挂到 `release-dart.yml` 与 `release-rust.yml` 上，向量不过不许发

- 判据：两个发布工作流里都有 `sh scripts/contract.sh`
