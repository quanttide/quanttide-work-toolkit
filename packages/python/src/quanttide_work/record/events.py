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
    name: Literal["WorkRecorded"] = "WorkRecorded"
    # 什么时候；时钟归端侧，库不出声。
    created_at: str
    record_id: str

    @classmethod
    def create(cls, record: WorkRecord, *, created_at: str) -> "WorkRecorded":
        """从记录与上下文造一条：记录凭证从记录取，只有一个住所。"""
        return cls(created_at=created_at, record_id=record.id)

    def to_jsonl(self) -> str:
        """落形为 JSONL 行：单行紧凑 JSON，名称与时刻在前。"""
        return self.model_dump_json()
