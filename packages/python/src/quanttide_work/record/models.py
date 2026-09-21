"""工作记录：工单流水中的单条记录。

一条记录记一件事的发生：发生时刻（created_at）、所执行步骤的锚点（step_id）、
简要描述（description）与判定结果（is_succeeded）。跨边界引用以 id 为准，排序以
seq 为准。记录不独立落盘，内嵌于工单的 records 字段；只增不改，落笔后原样保留，
改结论的唯一方式是追加一条新记录。

读出即校验：read_line 读一行 JSONL，要么产出一条记录，要么抛
RecordError(第几笔, 为什么)。行级读不通（不是合法 JSON、不是映射）与模型级读不通
（字段表之外、必选缺失、类型不符）都在这一个入口报出。单条记录在整本流水中是否
有效——凭证唯一、页码连续、时序递增——属账本级约束，由账本侧对账执行。
"""

import json

from pydantic import BaseModel, ConfigDict, Field

from quanttide_work.record.errors import RecordError


class WorkRecord(BaseModel):
    """工作记录实体：凭证、页码、发生时刻、工单与步骤锚点、简要描述、判定结果。"""

    model_config = ConfigDict(extra="forbid", frozen=True)

    # 凭证号：追加方生成，落笔后永不改变；跨边界引用一律以它为准。
    id: str = Field(min_length=1)
    # 页码：账本方分配，自 1 起严格递增且连续；排序一律以它为准。
    seq: int = Field(ge=1, strict=True)
    # 步骤发生时刻，而非记录录入时刻——流水是证据链，答「何时发生」。
    created_at: str = Field(min_length=1)
    # 所属工单的凭证。
    order_id: str = Field(min_length=1)
    # 所执行步骤的机器锚点，与所引工作流中的步骤同一指向；落笔后不可失配。
    step_id: str = Field(min_length=1)
    # 对已发生事实的简要描述，记录中唯一的自由文本字段；默认为空。
    description: str = ""
    # 判定结果；缺省 False——未记录「通过」即视为未通过。
    is_succeeded: bool = Field(default=False, strict=True)

    def to_jsonl(self) -> str:
        """落形为 JSONL 行：单行紧凑 JSON，字段全写，不产生字段表之外的键。"""
        return self.model_dump_json()


# 字段表与模型同源：按声明顺序生成，抄错字段的硬伤在这里长不出来。
FIELDS = tuple(WorkRecord.model_fields)

# 必选的文本字段：缺失或取值为空白均视为未提供；读出时按这个顺序报缺字段。
REQUIRED_TEXT = ("id", "created_at", "order_id", "step_id")


def read_line(line: str, ordinal: int) -> WorkRecord:
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
