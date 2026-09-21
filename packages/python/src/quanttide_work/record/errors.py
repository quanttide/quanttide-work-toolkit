"""记录读不通时的错误。

只存第几笔与种类（Fault）；文件名不进错误，由端侧在渲染时给——
RecordError.message 出 canonical 文案。
"""

from dataclasses import dataclass

# 一笔工作记录认得的字段：模型校验与报错文案共用这一份。
RECORD_FIELDS = (
    "id",
    "seq",
    "created_at",
    "order_id",
    "step_id",
    "description",
    "is_succeeded",
)


class Fault(Exception):
    """这一笔错在哪；读出不通就抛出，账本侧接住后补第几笔。"""

    def text(self) -> str:
        """第几笔之后的那一句（canonical 文案的主体）。"""
        raise NotImplementedError


@dataclass(frozen=True)
class NotJson(Fault):
    """JSONL 行不是合法 JSON。"""

    def text(self) -> str:
        return "不是合法的 JSON 行"


@dataclass(frozen=True)
class NotMapping(Fault):
    """记录不是映射结构。"""

    def text(self) -> str:
        return "不是映射（工作记录是七个字段的账）"


@dataclass(frozen=True)
class UnknownFields(Fault):
    """包含字段表之外的字段。"""

    unknown: tuple[str, ...]

    def text(self) -> str:
        return f"有不认识的字段：{'、'.join(self.unknown)}（只认 {'、'.join(RECORD_FIELDS)}）"


@dataclass(frozen=True)
class MissingField(Fault):
    """缺少必选字段，或取值为空白。"""

    name: str

    def text(self) -> str:
        return f"少了 {self.name}"


@dataclass(frozen=True)
class BadSeq(Fault):
    """seq 缺失，或不是自 1 起的整数。"""

    def text(self) -> str:
        return "的 seq 缺了，或不是自 1 起的整数"


@dataclass(frozen=True)
class BadType(Fault):
    """字段已声明，但取值类型不符。"""

    name: str

    def text(self) -> str:
        return f"的 {self.name} 类型不对"


@dataclass(frozen=True)
class RecordError(Exception):
    """一笔记录读不通：第几笔 + 哪一条不成立。"""

    ordinal: int
    fault: Fault

    def message(self, file: str) -> str:
        """canonical 报错文字：{file} 第 n 笔{这一条}；文件由端侧在渲染时给。"""
        return f"{file} 第 {self.ordinal} 笔{self.fault.text()}"

    def __str__(self) -> str:
        return f"第 {self.ordinal} 笔{self.fault.text()}"
