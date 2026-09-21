# record：工作记录

一行 JSONL 的格式，加上读它的一个函数。出处：`docs/specification/process/work-record.md`。

## 格式

七个字段：`id` / `seq` / `created_at` / `order_id` / `step_id` / `description` / `is_succeeded`。字段表与模型同源（`FIELDS`），另有测试钉住七个字段——抄错一个就红。

`WorkRecord::to_jsonl` 落形：单行紧凑 JSON，字段全写，不产生字段表之外的键。

## 读法

```rust
pub fn read_line(line: &str, ordinal: usize) -> Result<WorkRecord, RecordError>
```

读通产出一条记录；读不通给出 `RecordError { ordinal, reason }`——第几笔 + 为什么。文件名由端侧渲染时拼（`format!("{file} {error}")`），错误自己不携带半句话。

检查顺序即报错顺序，两层文案自然分开：

- 行级：`不是合法的 JSON 行`、`不是映射（工作记录是七个字段的账）`；
- 模型级：`有不认识的字段：…（只认 …）`、`少了 id`、`的 seq 缺了，或不是自 1 起的整数`、`的 is_succeeded 类型不对`。

Python 侧（`packages/python/src/quanttide_work/record/`）同一套文案，`to_jsonl` 输出逐字相同。

## 验收

```bash
cd packages/rust
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --locked
```

## 不在本稿

账本是另一件事——`repos` / `services` / `events`、local 与 remote（主要是 S3）两套实现、记账人与次序、`task/journal` 退役。等真要动它、真有了调用者时另立一稿，按用途推；按 ROADMAP 的规矩，没有真实需求先不立。属于不变量的部分已落规格：记录七字段与约束、记账须串行、事件是通知不是正本。
