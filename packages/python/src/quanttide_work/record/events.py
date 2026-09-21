"""工作记录的事件：一行 JSONL。

事件是通知，不是正本——流水正本在工单的 records 字段里，负载带全文是给跨区
下游的投影（零回查）；两处说法打架以正本为准，事件凭记录 id 幂等重放。
出处：docs/specification/process/work-record.md · 领域事件。
"""

import json
from dataclasses import asdict

from quanttide_work.record.models import WorkRecord

# 事件名。
EVENT = "WorkRecorded"


def recorded(
    record: WorkRecord,
    *,
    at: str,
    workspace_id: str,
    order_id: str,
    order_name: str,
) -> str:
    """工作记录已追加：拼一行 JSONL——公共三样在前，聚合字段随后。

    只描述追加的那一条，不含全量流水；下游按 record_id 幂等去重。
    """
    return json.dumps(
        {
            "event": EVENT,
            "at": at,
            "workspace_id": workspace_id,
            "order_id": order_id,
            "order_name": order_name,
            "record_id": record.id,
            "seq": record.seq,
            "step_id": record.step_id,
            "record": asdict(record),
        },
        ensure_ascii=False,
        separators=(",", ":"),
    )
