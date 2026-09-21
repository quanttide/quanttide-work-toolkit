import json

from quanttide_work.record.events import WorkRecorded
from quanttide_work.record.models import read_line

SAMPLE = (
    '{"id":"0f0e5b1a-9d0c-4c7e-8d1e-2b6a5f4e3d2c","seq":1,'
    '"created_at":"2026-02-11T10:00:00Z",'
    '"order_id":"3b8a2c1d-4e5f-4a6b-8c9d-0e1f2a3b4c5d",'
    '"step_id":"8c7d6e5f-4a3b-4c2d-9e1f-0a2b3c4d5e6f",'
    '"description":"草拟了提纲","is_succeeded":true}'
)


def recorded() -> WorkRecorded:
    return WorkRecorded.of(
        read_line(SAMPLE, 1),
        at="2026-02-11T10:00:01Z",
        workspace_id="ws-1",
        order_name="甲单",
    )


# 一行紧凑 JSON：公共三样在前（event / at / workspace_id），聚合字段随后。
def test_work_recorded_is_one_jsonl_line():
    got = recorded().to_jsonl()
    assert "\n" not in got
    payload = json.loads(got)
    assert list(payload) == [
        "event",
        "at",
        "workspace_id",
        "order_id",
        "order_name",
        "record_id",
        "seq",
        "step_id",
        "record",
    ]


# 凭证、页码与锚点只有记录一个住所：事件里的与记录里的一致。
def test_identity_comes_from_the_record():
    payload = json.loads(recorded().to_jsonl())
    record = read_line(SAMPLE, 1)
    assert payload["order_id"] == record.order_id
    assert payload["order_id"] == payload["record"]["order_id"]
    assert payload["record_id"] == record.id
    assert payload["seq"] == record.seq
    assert payload["step_id"] == record.step_id


# 记录全文走记录自己的落形，与记录行同源。
def test_full_record_comes_from_the_record():
    payload = json.loads(recorded().to_jsonl())
    assert payload["record"] == json.loads(read_line(SAMPLE, 1).to_jsonl())


# 只描述追加的那一条，不含全量流水——重建靠事件序列回放，不靠快照覆盖。
def test_carries_only_the_appended_record():
    payload = json.loads(recorded().to_jsonl())
    assert isinstance(payload["record"], dict)
    assert "records" not in payload
