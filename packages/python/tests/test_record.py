import json

import pytest

from quanttide_work.record.errors import (
    RECORD_FIELDS,
    BadSeq,
    BadType,
    MissingField,
    NotJson,
    NotMapping,
    RecordError,
    UnknownFields,
)
from quanttide_work.record.models import REQUIRED_TEXT, WorkRecord


def line(value: dict) -> str:
    """把 JSON 字面量装成一行紧凑 JSON，供 WorkRecord.from_jsonl 测试使用。"""
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


def sample() -> dict:
    """合法记录样本：七个字段齐备，seq 为 1，判定为通过。"""
    return {
        "id": "0f0e5b1a-9d0c-4c7e-8d1e-2b6a5f4e3d2c",
        "seq": 1,
        "created_at": "2026-02-11T10:00:00Z",
        "order_id": "3b8a2c1d-4e5f-4a6b-8c9d-0e1f2a3b4c5d",
        "step_id": "8c7d6e5f-4a3b-4c2d-9e1f-0a2b3c4d5e6f",
        "description": "草拟了提纲",
        "is_succeeded": True,
    }


# 合法记录读出成功，取值逐一相符；且「to_jsonl 落形 → from_jsonl 读出」
# 往返后保持不变。
def test_valid_record_round_trips():
    record = WorkRecord.from_jsonl(line(sample()))
    assert record.seq == 1
    assert record.is_succeeded is True
    assert WorkRecord.from_jsonl(record.to_jsonl()) == record


# 落形是单行紧凑 JSON，字段顺序与字段表一致。
def test_to_jsonl_is_one_compact_line():
    record = WorkRecord.from_jsonl(line(sample()))
    got = record.to_jsonl()
    assert "\n" not in got
    assert list(json.loads(got)) == list(RECORD_FIELDS)
    assert got == line(sample())


# 非法 JSON 行抛 NotJson；合法 JSON 但不是映射（如数组）抛 NotMapping。
def test_malformed_lines_are_rejected():
    with pytest.raises(NotJson):
        WorkRecord.from_jsonl("{oops")
    with pytest.raises(NotMapping):
        WorkRecord.from_jsonl("[1, 2]")


# 推荐字段可省略：description 缺省为空串，is_succeeded 缺省为 False
# （规格约束：未记录「通过」即视为未通过）。
def test_optional_fields_fill_in():
    bare = sample()
    del bare["description"], bare["is_succeeded"]
    record = WorkRecord.from_jsonl(line(bare))
    assert record.description == ""
    assert record.is_succeeded is False


# 字段表之外的字段被拒收。以 step 为反例：站名不落账，
# 记录仅以 step_id（机器锚点）指认步骤。
def test_unknown_fields_are_rejected():
    odd = sample() | {"step": "草拟"}
    with pytest.raises(UnknownFields) as caught:
        WorkRecord.from_jsonl(line(odd))
    assert caught.value == UnknownFields(("step",))


# 四个必选文本字段逐一缺失时，均抛 MissingField 并指名缺失字段。
def test_missing_required_is_rejected():
    for name in REQUIRED_TEXT:
        bare = sample()
        del bare[name]
        with pytest.raises(MissingField) as caught:
            WorkRecord.from_jsonl(line(bare))
        assert caught.value.name == name


# 类型不符逐项拒收：seq 为零、负数、字符串或布尔均属 BadSeq
# （页码须为自 1 起的整数）；is_succeeded 非布尔、description 非字符串
# 均属 BadType。
def test_bad_values_are_rejected():
    for seq in (0, -1, "1", True):
        with pytest.raises(BadSeq):
            WorkRecord.from_jsonl(line(sample() | {"seq": seq}))
    with pytest.raises(BadType) as caught:
        WorkRecord.from_jsonl(line(sample() | {"is_succeeded": "yes"}))
    assert caught.value.name == "is_succeeded"
    with pytest.raises(BadType) as caught:
        WorkRecord.from_jsonl(line(sample() | {"description": 3}))
    assert caught.value.name == "description"


# 渲染时文件在前、第几笔居中、毛病在后；不带文件时只出后两段。
def test_error_message_prefixes_the_file_and_keeps_the_ordinal():
    error = RecordError(3, BadSeq())
    assert (
        error.message("records.jsonl")
        == "records.jsonl 第 3 笔的 seq 缺了，或不是自 1 起的整数"
    )
    assert str(error) == "第 3 笔的 seq 缺了，或不是自 1 起的整数"


# 不认识的字段逐一列名，并带上字段表，便于对照改账。
def test_error_lists_unknown_fields_with_the_table():
    error = RecordError(1, UnknownFields(("step",)))
    assert error.message("records.jsonl") == (
        "records.jsonl 第 1 笔有不认识的字段：step"
        "（只认 id、seq、created_at、order_id、step_id、description、is_succeeded）"
    )
