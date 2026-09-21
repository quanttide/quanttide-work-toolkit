import json

import pytest

from quanttide_work.record.errors import RecordError
from quanttide_work.record.models import WorkRecord
from quanttide_work.record.repos import REQUIRED_TEXT, WorkRecordRepo

ORDINAL = 3
REPO = WorkRecordRepo()


def line(value: dict) -> str:
    """把 JSON 字面量装成一行紧凑 JSON。"""
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


# 字段表就是模型的字段，且仍是规格的七个——抄错一个、数错一个就红。
def test_field_table_matches_the_record():
    assert tuple(WorkRecord.model_fields) == (
        "id",
        "seq",
        "created_at",
        "order_id",
        "step_id",
        "description",
        "is_succeeded",
    )


# 合法行读出记录，取值逐一相符。
def test_valid_line_reads_a_record():
    record = REPO.read_line(line(sample()), ORDINAL)
    assert record.id == sample()["id"]
    assert record.seq == 1
    assert record.description == "草拟了提纲"
    assert record.is_succeeded is True


# 落形是单行紧凑 JSON，与样本逐字相同；往返不失真。
def test_dumped_line_is_compact():
    record = REPO.read_line(line(sample()), ORDINAL)
    got = record.model_dump_json()
    assert "\n" not in got
    assert got == line(sample())
    assert REPO.read_line(got, ORDINAL) == record


# 行级读不通：不是合法 JSON、不是映射，各报一句。
def test_line_level_reasons():
    with pytest.raises(RecordError) as caught:
        REPO.read_line("{oops", ORDINAL)
    assert str(caught.value) == "第 3 笔不是合法的 JSON 行"
    with pytest.raises(RecordError) as caught:
        REPO.read_line("[1, 2]", ORDINAL)
    assert str(caught.value) == "第 3 笔不是映射（工作记录是七个字段的账）"


# 推荐字段可省略：description 缺省为空串，is_succeeded 缺省为 False
# （规格约束：未记录「通过」即视为未通过）。
def test_optional_fields_fill_in():
    bare = sample()
    del bare["description"], bare["is_succeeded"]
    record = REPO.read_line(line(bare), ORDINAL)
    assert record.description == ""
    assert record.is_succeeded is False


# 字段表之外的字段被拒收，报错里列名并带上字段表。以 step 为反例：
# 站名不落账，记录仅以 step_id（机器锚点）指认步骤。
def test_unknown_fields_are_rejected():
    odd = sample() | {"step": "草拟"}
    with pytest.raises(RecordError) as caught:
        REPO.read_line(line(odd), ORDINAL)
    assert caught.value.reason == (
        f"有不认识的字段：step（只认 {'、'.join(WorkRecord.model_fields)}）"
    )


# 多个不认识的字段按名字排序，报错不随输入顺序摇摆。
def test_unknown_fields_are_sorted():
    odd = sample() | {"b": 1, "a": 2}
    with pytest.raises(RecordError) as caught:
        REPO.read_line(line(odd), ORDINAL)
    assert caught.value.reason.startswith("有不认识的字段：a、b（")


# 四个必选文本字段逐一缺失时，均报「少了 <字段>」。
def test_missing_required_is_rejected():
    for name in REQUIRED_TEXT:
        bare = sample()
        del bare[name]
        with pytest.raises(RecordError) as caught:
            REPO.read_line(line(bare), ORDINAL)
        assert caught.value.reason == f"少了 {name}"


# 类型不符逐项拒收：seq 为零、负数、字符串或布尔均属坏页码
# （页码须为自 1 起的整数）；description 非字符串、is_succeeded 非布尔
# 均属类型不符。
def test_bad_values_are_rejected():
    for seq in (0, -1, "1", True):
        with pytest.raises(RecordError) as caught:
            REPO.read_line(line(sample() | {"seq": seq}), ORDINAL)
        assert caught.value.reason == "的 seq 缺了，或不是自 1 起的整数"
    for name, bad in (("description", 3), ("is_succeeded", "yes")):
        with pytest.raises(RecordError) as caught:
            REPO.read_line(line(sample() | {name: bad}), ORDINAL)
        assert caught.value.reason == f"的 {name} 类型不对"


# 错误只装第几笔与为什么；文件名由端侧渲染时拼。
def test_error_carries_ordinal_and_reason_only():
    with pytest.raises(RecordError) as caught:
        REPO.read_line("{oops", 7)
    error = caught.value
    assert (error.ordinal, error.reason) == (7, "不是合法的 JSON 行")
    assert str(error) == "第 7 笔不是合法的 JSON 行"
    assert f"records.jsonl {error}" == "records.jsonl 第 7 笔不是合法的 JSON 行"
