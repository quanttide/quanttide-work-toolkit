"""工作记录的账本：一行行读。

一行 JSONL 要么产出一条记录，要么报第几笔与为什么——行级读不通（不是合法 JSON、
不是映射）与模型级读不通（字段表之外、必选缺失、类型不符）都在这一个入口报出。
出处：docs/specification/process/work-record.md · 约束。
"""

import json

from quanttide_work.record.errors import RecordError
from quanttide_work.record.models import FIELDS, REQUIRED_TEXT, WorkRecord


class WorkRecordRepo:
    """账本。"""

    def read_line(self, line: str, ordinal: int) -> WorkRecord:
        """读一行 JSONL：读不通就抛 RecordError(ordinal, 为什么)。

        检查顺序即报错顺序——行是不是合法 JSON、是不是映射、字段表之外、
        必选缺失、页码、两个推荐字段的类型。文案里的「七个字段」由
        字段表测试守着。
        """
        try:
            value = json.loads(line)
        except json.JSONDecodeError as error:
            raise RecordError(ordinal, "不是合法的 JSON 行") from error
        if not isinstance(value, dict):
            raise RecordError(ordinal, "不是映射（工作记录是七个字段的账）")

        def text(key: str) -> str:
            # 取文本字段：去掉两侧空白，取值不是字符串时视为未提供。
            got = value.get(key)
            return got.strip() if isinstance(got, str) else ""

        unknown = sorted(key for key in value if key not in FIELDS)
        if unknown:
            raise RecordError(
                ordinal,
                f"有不认识的字段：{'、'.join(unknown)}（只认 {'、'.join(FIELDS)}）",
            )
        for name in REQUIRED_TEXT:
            if not text(name):
                raise RecordError(ordinal, f"少了 {name}")
        seq = value.get("seq")
        if not isinstance(seq, int) or isinstance(seq, bool) or seq < 1:
            raise RecordError(ordinal, "的 seq 缺了，或不是自 1 起的整数")
        if "description" in value and not isinstance(value["description"], str):
            raise RecordError(ordinal, "的 description 类型不对")
        if "is_succeeded" in value and not isinstance(value["is_succeeded"], bool):
            raise RecordError(ordinal, "的 is_succeeded 类型不对")
        return WorkRecord(
            id=text("id"),
            seq=seq,
            created_at=text("created_at"),
            order_id=text("order_id"),
            step_id=text("step_id"),
            description=text("description"),
            is_succeeded=value.get("is_succeeded", False),
        )
