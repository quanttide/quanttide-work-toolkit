import json

from quanttide_work.record.events import EVENT, recorded
from quanttide_work.record.models import read_line

SAMPLE = (
    '{"id":"0f0e5b1a-9d0c-4c7e-8d1e-2b6a5f4e3d2c","seq":1,'
    '"created_at":"2026-02-11T10:00:00Z",'
    '"order_id":"3b8a2c1d-4e5f-4a6b-8c9d-0e1f2a3b4c5d",'
    '"step_id":"8c7d6e5f-4a3b-4c2d-9e1f-0a2b3c4d5e6f",'
    '"description":"草拟了提纲","is_succeeded":true}'
)


def event_line() -> str:
    record = read_line(SAMPLE, 1)
    return recorded(
        record,
        at="2026-02-11T10:00:01Z",
        workspace_id="ws-1",
        order_id=record.order_id,
        order_name="甲单",
    )


# 一行紧凑 JSON：公共三样在前（event / at / workspace_id），聚合字段随后。
def test_recorded_is_one_jsonl_line():
    got = event_line()
    assert "\n" not in got
    payload = json.loads(got)
    assert EVENT == "WorkRecorded"
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
    assert payload["event"] == EVENT
    assert payload["at"] == "2026-02-11T10:00:01Z"
    assert payload["workspace_id"] == "ws-1"


# 负载带上该条的凭证、页码与锚点，正文原样带走：下游凭 step_id 直认，零回查。
def test_recorded_carries_identity_and_full_record():
    payload = json.loads(event_line())
    record = read_line(SAMPLE, 1)
    assert payload["record_id"] == record.id
    assert payload["seq"] == record.seq
    assert payload["step_id"] == record.step_id
    assert payload["record"] == json.loads(record.to_jsonl())


# 只描述追加的那一条，不含全量流水——重建靠事件序列回放，不靠快照覆盖。
def test_recorded_carries_only_the_appended_record():
    payload = json.loads(event_line())
    assert isinstance(payload["record"], dict)
    assert "records" not in payload
