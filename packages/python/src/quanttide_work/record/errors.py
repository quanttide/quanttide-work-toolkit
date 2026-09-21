"""读一行读不通时的错误。

只存第几笔与为什么；文件名不进错误，由端侧渲染时拼（f"{file} {error}"）。
"""

from dataclasses import dataclass


@dataclass(frozen=True)
class RecordError(Exception):
    """读不通的一笔：第几笔 + 为什么。"""

    ordinal: int
    reason: str

    def __str__(self) -> str:
        return f"第 {self.ordinal} 笔{self.reason}"
