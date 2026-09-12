# TODO：把 Rust 库收到契约下

逐条列出要改的每一个细节。依据是 [STATUS.md](STATUS.md)；路线与收益见 [ROADMAP.md](ROADMAP.md)。

**每段做完的通用判据**（缺一不算完）：

```bash
cd packages/rust
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --locked
cd ../.. && sh scripts/contract.sh      # 11 份向量，Rust 与 Dart 两侧必须仍一致
```

---

## 段一 · 判据归位（一件事只写一处）

**1.1** 把判据的读法从 `workflow.rs` 搬进 `criterion/`

- 搬：`criterion_of`、`read_criterion`（YAML → `Criterion` 的翻译与校验）
- 留：`workflow.rs` 只管工作流的定义（`Step`、`Workflow`、整体语法校验）
- 判据：`grep -rn "fn criterion_of\|fn read_criterion" src/` 只命中 `criterion/`；`workflow/` 里不再有判据的字段解析
- 收益：改判据的取法只动一处，不必翻工作流文件

**1.2** `criterion` 立目录，`items_of`（判据翻成「要跑什么」）与模型同处

- 判据：`criterion/` 里同时有模型（`RuleKind` / `Criterion` / `RuleItem`）与读法；端侧只 `use criterion::` 就能用全

## 段二 · 拆 `workflow.rs`（516 → 每件 ≤250）

**2.1** 按「事」拆进 `workflow/`：定义的模型（`Step` / `Workflow`）与整体语法校验（`validate` / `DefinitionError`）分件，`mod.rs` 只留类型与出口

**2.2** 尾部三件不属工作流的东西归位：

- `looks_like_section`（节名）→ 与「报告两节」同源的模型处（工具箱里管节名的 `section-names` 向量在跑，落点要在同一处）
- `expand_placeholders`（占位展开）→ 与落点/占位同源处（`artifact-location` / `expand-placeholders` 两份向量管的事）
- `Finding`（定义核对的结果）→ 归定义核对（`workflow --check` 的模型侧）

- 判据：`workflow/` 的每件都只讲工作流；`wc -l` 无一件 >250

## 段三 · 拆 `task.rs`（292 → 每件 ≤250）

**3.1** `task/` 三分：任务模型（`Task`）、流水（`JournalEvent` + 「走过」的判定）、运行上下文（`RunContext`）

- 判据：`wc -l` 无一件 >250；两侧向量（`done-voting`）仍绿

## 段四 · 镜像同步 Dart 侧（与一、二、三同批）

**4.1** `packages/dart/lib/src/` 按 Rust 侧的新切分同构重排：`criterion/`、`workflow/`、`task/`，文件名与职责一一对应

- 判据：两侧同名同职责（列表对得上）；`dart analyze lib/ test/ && dart test` 全绿
- 理由：镜像破了，「照着另一侧抄」这件最省力的事就没了（见 ROADMAP 收益第三条）

## 段五 · 约定与文档

**5.1** 立 `packages/rust/CONVENTIONS.md`（与 cli 的同名文件一个路子），写死：一个模型一个目录、单文件 ≤250、一件事只写一处、说法与文案不进库、收进来的每一样要有规范出处

**5.2** `packages/rust/AGENTS.md` 的「项目结构」块改成实际结构（现在只写 `src/lib.rs`）

**5.3** Dart 侧同样立约定（或在工具箱 `AGENTS.md` 里写一份两侧共用的）

- 判据：结构块与新目录一致；约定文件里每条都能在新结构里指出落点

## 段六 · 版本与尺子（跨语言，与 Dart 侧同批）

**6.1** 两侧版本对齐：Rust `0.1.0-beta.4` 与 Dart `0.1.0-beta.5` 错开——定一条规矩并执行：**向量一致时两侧发同一个版本号**（端侧要同步升级，错开版本已经让 cli 与 studio 引了不同版本）

- 判据：`Cargo.toml` 与 `pubspec.yaml` 版本一致；两端（cli / studio）也引同一版本

**6.2** `scripts/contract.sh` 进 CI：把尺子挂到两侧的发布工作流上，向量不过就不许发

- 判据：`release-rust.yml` 与 `release-dart.yml` 里都有 `sh scripts/contract.sh`
