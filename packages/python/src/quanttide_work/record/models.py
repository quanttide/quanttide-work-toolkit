"""工作记录：工单流水中的单条记录。

一条记录记一件事的发生：发生时刻（created_at）、所执行步骤的锚点（step_id）、
简要描述（description）与判定结果（is_succeeded）。跨边界引用以 id 为准，排序以
seq 为准。记录不独立落盘，内嵌于工单的 records 字段；只增不改，落笔后原样保留，
改结论的唯一方式是追加一条新记录。

读出即校验：from_jsonl 解析单行 JSON，语法校验与构造一次完成（字段表之外、必选
缺失、类型不符均抛出 Fault），读出的一笔必是语法合法的。to_jsonl 落形为单行
JSON。单条记录在整本流水中是否有效——凭证唯一、页码连续、时序递增——属账本级
约束，由账本侧对账执行。
"""

import json
from dataclasses import dataclass

from quanttide_work.record.errors import (
    RECORD_FIELDS,
    BadSeq,
    BadType,
    MissingField,
    NotJson,
    NotMapping,
    UnknownFields,
)

# 必选的文本字段：缺失或取值为空白均视为未提供。
REQUIRED_TEXT = ("id", "created_at", "order_id", "step_id")


@dataclass(frozen=True)
class WorkRecord:
    """工作记录实体：凭证、页码、发生时刻、工单与步骤锚点、简要描述、判定结果。"""

    # 凭证号：追加方生成，落笔后永不改变；跨边界引用一律以它为准。
    id: str
    # 页码：账本方分配，自 1 起严格递增且连续；排序一律以它为准。
    seq: int
    # 步骤发生时刻，而非记录录入时刻——流水是证据链，答「何时发生」。
    created_at: str
    # 所属工单的凭证。
    order_id: str
    # 所执行步骤的机器锚点，与所引工作流中的步骤同一指向；落笔后不可失配。
    step_id: str
    # 对已发生事实的简要描述，记录中唯一的自由文本字段；默认为空。
    description: str = ""
    # 判定结果；缺省 False——未记录「通过」即视为未通过。
    is_succeeded: bool = False

    @classmethod
    def from_jsonl(cls, line: str) -> "WorkRecord":
        """从 JSONL 行读出：解析单行 JSON、语法校验、构造一次完成。

        字段表之外、必选缺失、类型不符均抛出 Fault；行不是合法 JSON 时抛
        NotJson；is_succeeded 缺省为 False。
        """
        try:
            value = json.loads(line)
        except json.JSONDecodeError as error:
            raise NotJson() from error
        if not isinstance(value, dict):
            raise NotMapping()

        def text(key: str) -> str:
            # 取文本字段：去掉两侧空白，取值不是字符串时视为未提供。
            got = value.get(key)
            return got.strip() if isinstance(got, str) else ""

        unknown = tuple(key for key in value if key not in RECORD_FIELDS)
        if unknown:
            raise UnknownFields(unknown)
        for name in REQUIRED_TEXT:
            if not text(name):
                raise MissingField(name)
        seq = value.get("seq")
        if not isinstance(seq, int) or isinstance(seq, bool) or seq < 1:
            raise BadSeq()
        if "description" in value and not isinstance(value["description"], str):
            raise BadType("description")
        if "is_succeeded" in value and not isinstance(value["is_succeeded"], bool):
            raise BadType("is_succeeded")
        return cls(
            id=text("id"),
            seq=seq,
            created_at=text("created_at"),
            order_id=text("order_id"),
            step_id=text("step_id"),
            description=text("description"),
            is_succeeded=value.get("is_succeeded", False),
        )

    def to_jsonl(self) -> str:
        """落形为 JSONL 行：单行紧凑 JSON，七个字段全写，不产生字段表之外的键。"""
        return json.dumps(
            {
                "id": self.id,
                "seq": self.seq,
                "created_at": self.created_at,
                "order_id": self.order_id,
                "step_id": self.step_id,
                "description": self.description,
                "is_succeeded": self.is_succeeded,
            },
            ensure_ascii=False,
            separators=(",", ":"),
        )
