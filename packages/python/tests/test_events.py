import json

from quanttide_work.record.events import WorkRecorded
from quanttide_work.record.repos import Repo

REPO = Repo()

SAMPLE = (
    '{"id":"0f0e5b1a-9d0c-4c7e-8d1e-2b6a5f4e3d2c","seq":1,'
    '"created_at":"2026-02-11T10:00:00Z",'
    '"order_id":"3b8a2c1d-4e5f-4a6b-8c9d-0e1f2a3b4c5d",'
    '"step_id":"8c7d6e5f-4a3b-4c2d-9e1f-0a2b3c4d5e6f",'
    '"description":"草拟了提纲","is_succeeded":true}'
)


def recorded() -> WorkRecorded:
    return WorkRecorded.create(REPO.read_line(SAMPLE, 1), created_at="2026-02-11T10:00:01Z")


# 一行紧凑 JSON：名称与时刻在前，其后是这条记录的凭证。
def test_work_recorded_is_one_jsonl_line():
    got = recorded().model_dump_json()
    assert "\n" not in got
    payload = json.loads(got)
    assert list(payload) == ["name", "created_at", "record_id"]


# 记录凭证只有记录一个住所；事件名由类型钉住。
def test_identity_comes_from_the_record():
    payload = json.loads(recorded().model_dump_json())
    assert payload["name"] == "WorkRecorded"
    assert payload["created_at"] == "2026-02-11T10:00:01Z"
    assert payload["record_id"] == REPO.read_line(SAMPLE, 1).id


# 只指认那一条，不带全文：事件是通知，正本在工单的 records 字段里。
def test_carries_only_the_reference():
    payload = json.loads(recorded().model_dump_json())
    assert set(payload) == {"name", "created_at", "record_id"}
    assert "record" not in payload
    assert "records" not in payload
