# record：工作记录方案

把规格的工作记录封成工具箱正本：字段模型、记账纪律、事件负载、账本接口——不因平台而变的那部分。出处：`docs/specification/process/work-record.md`。

## 背景

工具箱现存的流水表达是 `task/journal.rs` 的 `JournalEvent`（`at` / `step` / `detail` / `ok`），出自旧规格 task 篇：没有凭证（`id`）、没有页码（`seq`）、没有机器锚点（`step_id`）、不认工单（`order_id`）——下游投影与跨端对账要的四样全缺。CLI 侧已按新规格落地八项流水与四条纪律，工具箱作为正本还停在旧表达，一处事实两处说法。目标格式已立空件：`src/record/{models,errors,events,repos,services}.rs`。

## 模块分工

- `models.rs`——账上的一笔：`WorkRecord` 七字段（`id` / `seq` / `created_at` / `order_id` / `step_id` / `description` / `is_succeeded`），字段表与模型同源（`FIELDS`，另有测试钉住七个字段）；读出即校验——`read_line(line, ordinal)` 认 JSONL 行，读不通给出 `RecordError`（第几笔 + 为什么）；`to_jsonl` 落形为单行 JSON。
- `errors.rs`——读不通时报什么：只有 `RecordError { ordinal, reason }`一对——第几笔与为什么；渲染时拼「第 n 笔{这一条}」，文件名由端侧加在前头（`format!("{file} {error}")`）。
- `repos.rs`——账本长什么样：只增接口形状——追加、全量读（按 `seq` 序）、凭页码取单条、按站名筛读。接口只认 `id` 与 `seq`，不认数组下标。两套实现：`local` 落本机盘，`remote` 落对象存储（主要是 S3）——两套账本，一本纪律，都过 `services` 的对账；S3 没有追加，remote 的记账语义（整本重写或一条一件）由实现内部定，接口形状不变。
- `services.rs`——记账的手：追加收**草稿**——`Draft` 只装提交者手里的（`order_id` / `step` / `description` / `is_succeeded`；`id` 是幂等键，自带、重放即拒），账本发的号（`seq` / `step_id` / `created_at`）在类型上不在提交者手里。追加时干三件事——分配 `seq`（末条加一）、按 `step` 查填 `step_id`、对账四纪律。
- `events.rs`——报信：`WorkRecorded` 负载（工单名、工作区 `id`、该条的 `id` / `seq` / `step_id`、记录全文），只描述追加的那一条，不含全量流水。

## 纪律落点

规格每条约束都有唯一的代码住址：

- 字段取值之外一律拒 → `models` 的读出（`read_line`）；
- 同 `id` 两条即拒（复制的账）→ `services` 对账，`repos` 读出全量时复验；
- `seq` 自 1 起严格递增不跳号（被抽走的账）→ `services` 分配与对账；
- `created_at` 不得早于末条（倒流的账不可信）→ `services` 对账；
- `is_succeeded` 缺省按 `false` 断 → `models` 的读出；
- 引用凭 `id`、排序凭 `seq`、下标不进语义 → `repos` 接口形状。

## 记账人与次序

四纪律管账合不合法，还得管谁在记：

- **记账须串行**：同一工单的追加不得并发交叉。本地单写者声明、文件锁是防线不是协议；remote 走条件写（If-Match / ETag），冲突即拒、重读重算——S3 整本重写的 last-writer-wins 丢的是整段账，比跳号严重一个量级。repos 接口带版本参数，开工前定。规格已立：`work-record.md`·约束「记账须串行」。
- **先账后信**：先追加、后发 `WorkRecorded`。本地同目录，账落信没落可对账补信；远端两处存储非原子，信凭记录 `id` 幂等重放——权威规则见规格 `work-record.md`·事件约束。

## 旧表达全面淘汰

`record` 实现的同一步里，`task/journal.rs` 的 `JournalEvent`（`at` / `step` / `detail` / `ok`）退役：`task` 流水换成 `WorkRecord`，`workspace/progress` 的走过判定换成读 `records`，`journal.rs` 撤除——规格里没有的旧说法不保留第二处事实源。盘上旧账不换算：journal 数据属 CLI v1 时代，CLI 已自管迁移；工具箱不做规格外字段的换算——**断层：新账新记，旧账归档只读**。

波及面与同步义务：

- 契约向量里踩着 journal 读法的（`done-voting` 等）随淘汰更新；`contract.sh` 要求 Rust 与 Dart 两侧一致，Dart 镜像的 `journal.dart` 同步退役，不留在后续轮次；
- CLI 的 `order/record.rs` 已是同款新纪律，属平台实现，不在淘汰范围；是否收回工具箱对齐，另行再议。

## 步骤

规格先行：权威关系与串行纪律已入 `work-record.md`，动工前提。

1. `errors.rs` 与 `models.rs`：`RecordError`（第几笔 + 为什么）；七字段、读出即校验（`read_line`）、`to_jsonl` 落形；
2. `repos.rs`：接口形状（带版本参数，供条件写）与 `local` 实现——落工单文档（规格原文「不独立落盘，内嵌 `records`」），账本只有一个化身；
3. `repos` 的 `remote` 实现（S3）：条件写保串行，测试对假 S3（容器或桩），不连真桶；
4. `services.rs`：追加收 `Draft`，四纪律加并发控制在把守；
5. `events.rs`：`WorkRecorded` 负载（全文是投影，权威规则见规格）；
6. `lib.rs` 注册 `pub mod record`，测试与契约向量跟上；
7. 淘汰旧表达：`task` 流水换 `WorkRecord`、`progress` 换读法、`journal.rs` 撤除并声明断层，契约向量两侧同步更新，Dart 镜像同轮退役。

## 验收判据

```bash
cd packages/rust
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --locked
cd ../.. && sh scripts/contract.sh      # 契约向量，Rust 与 Dart 两侧一致
```

新增 record 向量：合法一笔、同 `id` 拒、跳号拒、倒流拒、缺必选拒、并发双写（本地锁生效、remote 版本冲突拒）、断层（旧账不换算）——两侧共跑；Dart 镜像的落地节奏见 [ROADMAP](../../ROADMAP.md)。
