# record：工作记录方案

把规格的工作记录封成工具箱正本：字段模型、记账纪律、事件负载、账本接口——不因平台而变的那部分。出处：`docs/specification/process/work-record.md`。

## 背景

工具箱现存的流水表达是 `task/journal.rs` 的 `JournalEvent`（`at` / `step` / `detail` / `ok`），出自旧规格 task 篇：没有凭证（`id`）、没有页码（`seq`）、没有机器锚点（`step_id`）、不认工单（`order_id`）——下游投影与跨端对账要的四样全缺。CLI 侧已按新规格落地八项流水与四条纪律，工具箱作为正本还停在旧表达，一处事实两处说法。目标格式已立空件：`src/record/{models,events,repos,services}.rs`。

## 四件套分工

- `models.rs`——账上的一笔：`WorkRecord` 八字段（`id` / `seq` / `created_at` / `order_id` / `step` / `step_id` / `description` / `is_succeeded`），`of` 读出、`to_yaml` 落形、`validate` 把语法关（字段表之外拒、必选缺失拒、类型不符拒）。
- `repos.rs`——账本长什么样：流水的只增接口形状——追加、全量读（按 `seq` 序）、凭页码取单条、按站名筛读。接口只认 `id` 与 `seq`，不认数组下标；落盘归平台，这里只立形状。
- `services.rs`——记账的手：追加时账本方干的三件事——分配 `seq`（末条加一）、按 `step` 查填 `step_id`、对账四纪律。函数签名不收 `seq` 与 `step_id`：账本发的号，类型上就不让提交者带，而不是收了再扔。
- `events.rs`——报信：`WorkRecorded` 负载（工单名、工作区 `id`、该条的 `id` / `seq` / `step_id`、记录全文），只描述追加的那一条，不含全量流水。

## 纪律落点

规格每条约束都有唯一的代码住址：

- 字段取值之外一律拒 → `models::validate`；
- 同 `id` 两条即拒（复制的账）→ `services` 对账，`repos` 读出全量时复验；
- `seq` 自 1 起严格递增不跳号（被抽走的账）→ `services` 分配与对账；
- `created_at` 不得早于末条（倒流的账不可信）→ `services` 对账；
- `is_succeeded` 缺省按 `false` 断 → `models::of` 读法；
- 引用凭 `id`、排序凭 `seq`、下标不进语义 → `repos` 接口形状。

## 与既有件的关系

`task/journal.rs` 的 `JournalEvent` 是旧表达，本方案不动它——`record` 落地后两者并存，迁移（`task` 流水换 `WorkRecord`）另立事项再议。CLI 的 `order/record.rs` 已有同款纪律，属平台自己的实现；工具箱落地后是否收回对齐，等下一轮再定，本方案不背这个活。

## 步骤

1. `models.rs`：八字段、读法、`validate`；
2. `repos.rs`：接口形状（trait 或纯函数组，与库内既有风格取齐）；
3. `services.rs`：追加服务，四纪律全在这里把守；
4. `events.rs`：`WorkRecorded` 负载；
5. `lib.rs` 注册 `pub mod record`，测试与契约向量跟上。

## 验收判据

```bash
cd packages/rust
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --locked
cd ../.. && sh scripts/contract.sh      # 契约向量，Rust 与 Dart 两侧一致
```

新增 record 向量：合法一笔、同 `id` 拒、跳号拒、倒流拒、缺必选拒——两侧共跑；Dart 镜像的落地节奏见 [ROADMAP](../../ROADMAP.md)。
