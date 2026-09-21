"""工作记录的事件：一行 JSONL。

出处：docs/specification/process/work-record.md · 领域事件。
"""

from typing import Literal

from pydantic import BaseModel, ConfigDict

from quanttide_work.record.models import WorkRecord


class WorkRecorded(BaseModel):
    """工作记录已追加：只描述追加的那一条，不含全量流水。"""

    model_config = ConfigDict(extra="forbid", frozen=True)

    # 事件名——跨语言契约按它认行。
    event: Literal["WorkRecorded"] = "WorkRecorded"
    # 什么时候；时钟归端侧，库不出声。
    at: str
    # 工作区凭证。
    workspace_id: str
    # 工单凭证与工单名。
    order_id: str
    order_name: str
    # 这一条的凭证、页码与锚点；下游凭 step_id 直认，零回查。
    record_id: str
    seq: int
    step_id: str
    # 记录全文。
    record: WorkRecord

    @classmethod
    def of(
        cls,
        record: WorkRecord,
        *,
        at: str,
        workspace_id: str,
        order_name: str,
    ) -> "WorkRecorded":
        """从记录与上下文造一条：凭证、页码与锚点从记录取，只有一个住所。"""
        return cls(
            at=at,
            workspace_id=workspace_id,
            order_id=record.order_id,
            order_name=order_name,
            record_id=record.id,
            seq=record.seq,
            step_id=record.step_id,
            record=record,
        )

    def to_jsonl(self) -> str:
        """落形为 JSONL 行：单行紧凑 JSON，公共三样在前、聚合字段随后。"""
        return self.model_dump_json()
